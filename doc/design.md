# `qubit-model-derive` 最终设计

## 1. 目标与边界

`qubit-model-derive` 是 `qubit-model-metadata` 的过程宏前端。它把一份 Rust 类型声明编译为两层互不重复的静态信息：

- `qubit-reflect` 持有 Rust 的结构事实：类型、字段、可见性、字段类型、泛型实参和结构化访问；
- `qubit-model-metadata` 以同一个 `TypeDescriptor` 为根，附加领域语义：模型角色、稳定 ID、约束、关系、策略、脱敏和属性。

宏 crate 不维护第二套结构描述符，也不在编译期跨 crate 解析模型图。它只负责解析、局部校验、规范化和生成不可变声明 metadata；跨模型的 ID、关系、validator、codec 与查询在运行时显式解析。

依赖方向如下：

```text
业务模型 crate
  ├── qubit-model-derive（声明编译）
  └── qubit-model-metadata（运行时门面）
        ├── qubit-reflect（唯一的结构反射来源）
        ├── qubit-redact（安全表示）
        ├── qubit-validator（validator 注册表）
        └── qubit-codec（codec 注册表）
```

生成代码通过 `proc-macro-crate` 定位 `qubit-model-metadata`，因此业务 crate 可以重命名该依赖，无需直接依赖 `qubit-reflect`。

## 2. 声明模型

### 2.1 六个入口和共享流水线

`#[Entity]`、`#[Projection]`、`#[Model]`、`#[Enum]`、`#[Value]` 与 `#[ModelProperties]` 都经由同一入口分派。前五个角色宏使用同一条编译流水线：

```text
TokenStream → syn AST → 角色/字段 IR → 局部校验与规范化
            → 默认 trait、Serde、Redact 与 Reflect 属性 → metadata/capability/registration
```

这保证所有角色对 runtime crate 重命名、重复 `Reflect`、默认能力和字段属性拥有一致的诊断。`#[ModelProperties]` 复用入口和运行时 capability，但单独解析 inherent `impl` 中的方法契约。

### 2.2 角色、形状与专属语义

| 宏 | 可标注的 Rust 类型 | 角色 metadata | 局部约束 |
| --- | --- | --- | --- |
| `#[Entity]` | 具名字段 struct | `EntityMetadata`，包含 identifier | 必须有且仅有一个 `#[identifier]`；不可泛型 |
| `#[Projection]` | 具名字段 struct | `ProjectionMetadata`，包含 identifier 和可选 source | 必须有且仅有一个 `#[identifier]`；不可泛型 |
| `#[Model]` | 具名字段 struct 或 unit struct | 空的 `ModelMetadata` | 不支持 tuple struct；可类型/const 泛型 |
| `#[Enum]` | enum（含 unit、tuple、具名字段变体） | `EnumMetadata` 与 variant overlay | 可类型/const 泛型 |
| `#[Value]` | 非空具名字段 struct 或单字段 tuple struct | `ValueMetadata` | 可类型/const 泛型 |

所有角色都拒绝 union 和 lifetime 参数。const 泛型只接受 `bool`、`char` 和 Rust 基元整数类型。`Entity` 与 `Projection` 不接受任何泛型参数或 where 子句。

角色参数的共同部分为 `id = "namespace.Type"`，以及默认能力开关。`id` 必须是 ASCII 标识形式，且由 `ModelId` 校验。没有 `id` 的非泛型类型仍可取得静态 `TypeMetadata`，但不会生成全局注册项。

`Projection(source = EntityType)` 或 `Projection(source_id = "namespace.Entity")` 声明固定来源；不指定来源的 projection 是开放 projection。`open` 可显式声明开放 projection，且不能与固定来源并用。`Value(transparent)` 仅适用于单字段 value，令 Serde、Display 和 value metadata 把它作为包装的那个字段处理。`Value(codec = CodecType)` 声明整个 value 的 canonical codec。

### 2.3 默认能力与安全输出

除非被 `no_*` 参数关闭，角色宏生成 `Clone`、`PartialEq`、`Eq`、`Hash`、`Deserialize` 与 `Redact`；还会生成受 Redact 策略约束的 `Debug`、`Display` 和 `Serialize`。`copy`、`default`、`partial_ord`、`ord` 是显式 opt-in；所有变体均为 unit 的 enum 默认 `Copy`，`no_copy` 可关闭。

当没有 `no_redact` 时，用户不能预先 derive `Debug` 或 `Serialize`，以免绕过字段脱敏。`no_redact` 改为生成普通 Rust `Debug` / `Serialize`，并允许自定义这些实现。角色宏必须位于用户 `#[derive(...)]` 之前，且不得显式 derive `Reflect`，因为宏会生成并配置唯一的 `Reflect` 实现。

## 3. 唯一的 metadata 根

每个具体类型仅有一个 `qubit_reflect::TypeDescriptor`。角色宏在该 descriptor 上注册 `qubit.model.metadata.v1` capability；`#[ModelProperties]` 在同一 descriptor 上注册 `qubit.model.properties.v1` capability。二者均为内部 provider ABI，公开读取入口为：

```rust
use qubit_model_metadata::{ModelDescriptorExt, TypeMetadata};

let by_type = TypeMetadata::of::<User>();
let by_descriptor = by_type.descriptor().model_metadata();
assert!(std::ptr::eq(by_type, by_descriptor.unwrap()));
```

`TypeMetadata` 是不可变 overlay，保存 descriptor、可选 `ModelId`、按反射声明顺序排列的 `FieldMetadata`、角色 payload、合并后的 `PropertyMetadata` 和可选泛型定义。运行时在 capability 边界验证 descriptor 指针、Rust `TypeId`、字段顺序、字段语义和角色不变量，避免反射事实与领域 metadata 分离。

`FieldMetadata` 不复制结构信息：`reflect()` 返回底层 `FieldDescriptor`，`type_ref()` 返回其 resolved、opaque 或 symbolic 类型引用；`descriptor()` 仅在类型已解析时返回 descriptor。领域层只保存属性 occurrence、标准约束、validator occurrence 和有效 Serde 行为。

## 4. 字段领域语义

字段属性被保留为 source-order occurrence，同时生成便于消费的汇总访问器。单值语义（identifier、unique、reference、key part、codec、redact、Serde、opaque）不能重复；validator 可以重复且顺序保留。

### 4.1 身份、索引、唯一性与关系

- `#[identifier]` 或 `#[identifier(assigned_by = application | database)]` 标记模型身份。Entity 和 Projection 要求恰好一个；identifier 自动成为索引原因。
- `#[indexed]` 追加显式索引原因。
- `#[unique(respect_to(path, ...), ignore_case = bool)]` 声明唯一性。`respect_to` 是作用域字段路径；resolver 把它们构造成唯一查询键。
- `#[reference(entity = EntityType | entity_id = "...", property = path, existing = bool, same_as = path)]` 声明到 Entity 或其可读属性的引用。目标、属性存在性和类型相容性由 resolver 验证；`same_as` 也必须解析为可读 property。
- `#[key_part(order = n)]` 声明复合键分量。所有 order 必须从 0 开始、无重复且连续。

关系与索引只描述逻辑语义，不直接声明数据库物理索引。

### 4.2 标准约束和容器 selector

根字段支持以下约束：

- `#[text(min_chars, max_chars, min_bytes, max_bytes, allowed_chars, non_blank, format)]`；
- `#[decimal(precision, scale, rounding, min, max, min_inclusive, max_inclusive)]`；`#[money(...)]` 是带 money 语义的 decimal；
- `#[time(precision = ...)]`；
- `#[sequence(min_items, max_items, unique_items)]`；
- `#[map(min_entries, max_entries)]`。

`#[element(...)]` 只作用于 sequence 元素；`#[map_key(...)]` 和 `#[map_value(...)]` 只作用于 map 键和值。selector 内可嵌套与容器位置相容的约束、`validator`、`codec` 和 `redact`。约束和 selector 先被规范化为 `ConstraintMetadata`、`SequenceConstraint`、`MapConstraint` 和 `SelectorMetadata`，之后由 validator / 业务框架消费。

### 4.3 自定义策略、Serde 与脱敏

`#[validator(id = "...", depends_on(path, ...), params(name = literal, ...))]` 只声明一个稳定 validator ID、可读依赖和受限字面量参数；它不在宏展开时寻找注册项。`#[codec(CodecType)]` 直接引用 Rust codec 类型，`#[codec(id = "...")]` 声明延后绑定的 codec ID。`#[opaque]` 将字段标记为反射 opaque。

`#[redact(level = "...")]`、`skip`、`nested`、`map`、`keyed_by = "..."` 形成字段或 selector 的 Redact metadata，并同时驱动生成的安全 `Debug`、`Display` 和 `Serialize`。map key 的不同原始值若脱敏后冲突，序列化必须失败，不能静默覆盖。

用户的 `#[serde(...)]` 会记录为字段有效 Serde metadata。模型默认会省略 redacted 字段；`#[keep_serializing]` 只关闭该默认省略，不会关闭脱敏。宏会移除仅供模型解析的字段 helper，再把必要的 `reflect(opaque)` 和 Redact 属性交给下游 derive。

## 5. Property overlay

`#[ModelProperties]` 只能标注无泛型的 inherent `impl`。公开、同步、无方法泛型的方法按名称归类：

- `fn name(&self) -> T`、`&T`、`&str`、`&[T]`、`Option<&T>` 或 `Option<&str>` 是 getter；
- `fn set_name(&mut self, value: T)` 是 setter，property 名为 `name`。

宏为每个方法生成类型擦除但类型检查过的 adapter。字段 property、getter 和 setter 按 property 名合并：同时有字段和访问器的是 field-backed；仅 getter 是 computed；仅 setter 是 virtual。`PropertyMetadata` 通过 `ReflectedRef` / `ReflectedMut` / `ReflectedOwned` 访问，getter 区分 owned、borrowed、optional borrowed 和 borrowed slice，setter 会在调用前验证目标和输入类型。重复 getter 或 setter、私有方法、trait impl、异步方法、错误 receiver 或不支持的签名均是编译错误。

## 6. 泛型、注册与解析

泛型角色类型仍为每一个具体 monomorph 生成 descriptor 和 `TypeMetadata` capability，但具体实例没有 `ModelId`。宏为泛型定义生成一次 `GenericModelMetadata` 和 inventory registration；模板字段可包含 `TypeExpression::Parameter` 等 symbolic 类型。非泛型且声明 `id` 的模型则生成 concrete registration。

`ModelRegistry::try_global()` 先初始化反射注册表，再收集链接进进程的 inventory 项，按 `ModelId` 和 fragment identity 固定排序，拒绝重复 ID 或矛盾注册。静态 `TypeMetadata::of::<T>()` 与 `ModelDescriptorExt::model_metadata()` 不会初始化该全局注册表。

跨模型解析必须显式提供输入：

```rust
let graph = ModelResolver::new(ResolveInputs {
    models: ModelRegistry::try_global()?,
    validators: ValidatorRegistry::global(),
    codecs: ValueCodecRegistry::global(),
}).resolve_all()?;
```

`ModelResolver` 只在此阶段绑定 reference、固定 projection source、validator、codec，并为 Entity 构造 `QueryMetadata`（可过滤路径与唯一键）。它聚合确定性错误，而不是把未链接 crate、注册顺序或环境状态隐含进宏展开。

## 7. 诊断与不变量

宏在本地立即拒绝无效类型形状、角色字段规则、无效 ID、重复单值字段语义、错误的 selector 位置、互相冲突的默认能力和危险的现有 derive。无法从当前 crate 安全得到的信息——例如 model ID 是否全局唯一、reference 目标是否存在、validator / codec 是否已注册、跨模型类型是否相容——留给 registry 与 resolver 诊断。

最终设计始终保持以下不变量：

1. Rust 结构只有 `qubit-reflect` 的一个 descriptor 根；模型 metadata 只能作为该根的 capability / overlay 存在。
2. 静态 metadata 查询无全局初始化副作用；跨模型绑定显式且可重复。
3. `FieldMetadata` 与反射字段在顺序和归属上严格一一对应。
4. 安全输出默认遵守 Redact；绕过它需要明确 `no_redact`。
5. 声明 metadata 保留原始 occurrence 顺序，解析结果属于 `ResolvedModelGraph`，不回写静态声明。

## 8. 验证范围

本设计由三类测试共同固定：trybuild 覆盖角色形状和诊断；runtime fixture 覆盖依赖重命名、缺失 runtime 和跨 crate 链接；端到端测试覆盖五种角色、泛型模板、字段语义、脱敏、property adapter、registry 与 resolver。新增语义时必须先确定它属于本地编译期校验、静态 declaration metadata，还是显式运行时解析，不能把这三层混合。
