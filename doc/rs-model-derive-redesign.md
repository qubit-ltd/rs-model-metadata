# `rs-model-derive` 全量重构设计

- 日期：2026-08-27
- 状态：待审核
- 兼容性：不保留旧 API、旧元数据布局或旧注册协议
- 语义基线：[模型类型体系：五种类型的精确定义](model-type-definitions.md)
- 影响范围：`rs-model-derive`、`rs-model-metadata`，以及后续校验器、随机生成器和 DAO 测试框架的公共契约

本文以五种类型的基础语义为唯一前提，重新设计 `#[Entity]`、`#[Projection]`、`#[Model]`、`#[Enum]`
和 `#[Value]`。旧设计文档只作为问题样本，不构成兼容性约束。

本文既定义过程宏的公共 API，也定义宏必须生成的 runtime 元数据和可执行访问协议。实际数据库、校验器、随机
生成器和 DAO 实现不属于本次宏 crate，但其所需的信息与操作必须由本设计完整提供，不能再次依赖字段名猜测、
特殊字符串或 Java 式运行时反射。

## 1. 结论摘要

本设计作出以下核心决策：

1. 对外提供五个互斥的角色宏：`Entity`、`Projection`、`Model`、`Enum`、`Value`。
2. 五种类型都有 `TypeDescriptor`；只有前四种生成全局注册项。
3. 公共结构元数据不再包含 `ModelId`；`ModelId` 只存在于 Entity 角色元数据和 Entity 索引中。
4. Entity 与 Projection 必须各有且仅有一个 `Id` identifier；其他三类禁止 identifier。
5. 注册表以 `TypeIdentity` 为主索引，并额外维护 `ModelId -> Entity` 的唯一索引。
6. `TypeMetadata` 使用带角色载荷的和类型表达合法状态，不使用一组互相依赖的 `Option` 字段。
7. 字段的局部约束留在公共字段元数据中；identifier、持久化和关系进入对应角色的专属元数据。
8. `reference.property` 被拆成四种显式选择：Entity 本体、identifier、reference key、命名 Projection。
9. `existing + path` 被一个互斥的绑定来源取代：`preexisting`、`inline`、`same_as(path)`。
10. 命名 Projection 必须由 Entity 正式注册可执行 projector；彻底删除 `property = info` 特判。
11. 引用路径保存并解析“实际目标 Entity 的来源关系”，而不是另一个字段当前保存的投影值。
12. 宏生成借用访问、构造和角色专属操作，使校验器与生成器不需要 Rust 反射或 Serde 绕行。
13. Value 不注册，但其 `TypeDescriptor` 通过字段类型静态解析，因此嵌套约束可以自动递归校验。
14. 角色宏不再偷偷生成 `Clone`、Serde、`Display`、Eq、Hash 或脱敏实现；这些使用原生 `derive` 或专用 crate。
15. 每个 reference occurrence 都自动要求一个单路径索引；不得在同一路径重复声明单字段 index，但仍允许包含该
    reference 的复合 index。
16. 删除语义不清或没有真实消费者的旧能力：`primary_key`、`key`、旧 `ownership`、`lookup_relation`、裸
    `reference`、`nullable`、`computed` 及所有兼容别名。

## 2. 现状问题与真实需求

### 2.1 当前 Rust 实现的结构性问题

当前实现有以下根本限制：

- `TypeMetadata` 强制包含 `ModelId`，导致 Value、Projection、Model 和 Enum 无法按新语义自然表达；
- `ModelRegistration` 以 `ModelId` 为主键，错误地把“参与注册”等同于“拥有稳定协议身份”；
- `#[Model]` 和 `#[Enum]` 共用一个要求 `id` 的 `ModelIr`，角色约束只能通过事后猜测补丁实现；
- `#[identifier]` 被无条件归一化为数据库主键，无法表达 Projection 借用来源 Entity 身份的语义；
- `#[reference]` 虽然会生成索引并禁止同字段 `#[indexed]`，但现有元数据没有区分用户显式索引与关系派生索引，
  也无法为嵌套 Model 中的具体 reference occurrence 形成稳定路径；
- `property = info` 在注册表校验中被特殊跳过，既不能证明 projector 存在，也不能提供可执行的取值函数；
- 当前 `path` 校验主要围绕已保存字段值，无法完整表达父节点和隐藏目标 Entity 来源；
- 注册表图以 `ModelId` 标识所有源节点，无法容纳没有 `ModelId` 但可以声明关系的 Model 和 Projection；
- `ownership(owner = Type)` 与 Java 最新的 `OwnedBy(path, domain)` 语义不同；
- `key` 被设计成模型逻辑键，而 Java `KeyIndex` 的真实用途主要是嵌套值编码顺序；
- `lookup_relation`、旧 `ownership` 在现有生产 Rust 模型中没有实际使用，却显著扩大了元数据和图校验复杂度；
- 元数据只能描述字段，缺少值访问、构造和 projector 操作，后续生成器仍无法像 Java 反射那样工作。

### 2.2 Java 实现证明的实际场景

Java 版本只作为需求证据。真正需要保留的是以下场景，而不是它的注解形式：

- validator 根据字段的文本、数值、容器和时间约束校验对象；
- random 根据相同约束生成合法字段值；
- DAO 测试框架从一个根对象递归生成完整对象图；
- 引用字段可能保存目标 Entity 的 ID、唯一业务字段、Projection 或完整 Entity；
- 多数引用要求目标 Entity 先存在，部分引用只生成内联 Entity 而不独立持久化；
- 一个引用可以复用对象图中另一处引用背后的同一个 Entity，包括 `..` 父对象路径；
- 路径复用依赖“字段背后的 Entity 来源”，即使字段本身只保存 ID 或 Projection；
- 引用字段之间需要拓扑排序；必需依赖环必须报告，Optional 或空集合可以切断生成依赖；
- 内联 Entity 虽不独立持久化，其内部 `preexisting` 引用仍必须满足；
- 通用 `Info` 可由多种 Entity 产生，具体 Entity 类型来自引用上下文，而不是 `Info` 本身；
- `OwnedBy`、`ReferenceBy` 在给定生产模型与集成测试中没有实际使用，不能据此设计推测性 DSL。

### 2.3 设计目标

本次重构必须达到：

- 五种角色在宏输入、生成 trait、元数据和注册表中保持同一套严格边界；
- 非 Entity 无需伪造稳定 ID；
- Value 的递归查询、校验和生成不依赖全局注册；
- 跨 crate 引用只依赖 Entity 的稳定 `ModelId` 字符串，避免领域 crate 循环依赖；
- 完整程序链接后，可以验证目标 Entity、选择器、reference key、Projection 类型和路径；
- 后续校验器与生成器只消费统一 descriptor，不自行解析 Rust 类型名或维护特殊字段名单；
- 所有失败都有确定、可聚合、可定位的诊断；
- 类型结构尽量排除非法状态，而不是只靠文档约定。

### 2.4 非目标

本文不设计：

- ORM、SQL DDL 或具体数据库列展开规则；
- repository/DAO 的具体 trait；
- 权限系统的 owner domain DSL；
- `ReferenceBy` 式任意条件查询语言；
- 完整自定义校验器或随机策略注册中心；
- 泛型模型和泛型 Value；
- 跨集合索引的 `same_as` 路径；
- 旧宏语法的兼容层或自动迁移期双写。

## 3. 总体方案比较

### 3.1 方案 A：保留统一元数据，只把 `ModelId` 改为可选

做法是继续保留单一 `TypeMetadata`、`ModelRegistration` 和大部分现有代码，把 `id: ModelId` 改为
`id: Option<ModelId>`，再增加 `role: ModelRole`。

优点：

- 修改量最小；
- 现有查询 API 容易迁移；
- 可以快速加入五个入口宏。

缺点：

- `Entity` 无 ID、Value 被注册、Model 带 identifier 等非法状态都能被结构表示；
- 注册表每个查询都需要解释 `role + Option<ModelId>` 组合；
- 关系、持久化、identifier 仍会继续塞进公共属性袋；
- 宏内部仍是一条充满角色条件分支的单体流水线。

本设计不采用该方案。

### 3.2 方案 B：五套完全独立的元数据、注册表和宏实现

做法是为五种角色分别设计 `EntityMetadata`、`ProjectionMetadata` 等完整结构，并分别实现字段、约束、形状、
查询和展开。

优点：

- 角色隔离最强；
- 单个角色内部几乎不能构造非法状态；
- 每个宏入口容易独立理解。

缺点：

- 字段、enum payload、Value 嵌套、约束和容器形状会出现五套重复实现；
- validator、schema 和 generator 必须对五种描述做重复分派；
- 修复一个公共约束或容器 bug 时容易产生角色行为漂移。

本设计不采用该方案。

### 3.3 方案 C：共享结构描述，角色语义采用和类型分层

所有类型共享结构、字段、约束和访问协议；角色专属语义存放在 `TypeRoleMetadata` 的不同 variant 中。注册是
descriptor 之上的独立层，Entity 再拥有独立的 `ModelId` 索引。

优点：

- Value 与注册彻底解耦；
- 公共查询、校验和生成只实现一次；
- Entity、Projection 等角色专属元数据由 variant payload 限制；
- 注册表不再假定每个条目都有 `ModelId`；
- 宏内部可以共享 parser/shape/constraint 管线，同时保留角色专属验证器。

代价：

- `rs-model-metadata` 需要破坏性重构；
- 下游必须从“所有注册项按 ModelId 查询”迁移到“类型主索引 + Entity ID 次级索引”；
- 需要正式设计可执行访问操作，而不能只保存静态字符串。

本设计选择方案 C。

| 维度 | 方案 A | 方案 B | 方案 C |
| --- | --- | --- | --- |
| 非法状态隔离 | 弱 | 最强 | 强 |
| 公共能力复用 | 强 | 弱 | 强 |
| Value 不注册 | 勉强支持 | 支持 | 自然支持 |
| 下游查询复杂度 | 高 | 高 | 中 |
| 重构成本 | 低 | 最高 | 中高 |
| 长期可维护性 | 低 | 中 | 高 |

## 4. 公共宏 API

### 4.1 五个互斥角色宏

| 宏 | 接受形状 | 注册 | `ModelId` | identifier |
| --- | --- | --- | --- | --- |
| `#[Entity]` | 具名字段 struct | 是 | 必填 | 恰好一个 `Id` |
| `#[Projection]` | 具名字段 struct | 是 | 禁止 | 恰好一个 `Id` |
| `#[Model]` | 具名字段或 unit struct | 是 | 禁止 | 禁止 |
| `#[Enum]` | unit、tuple、struct 或混合 enum | 是 | 禁止 | 禁止 |
| `#[Value]` | 具名字段 struct 或单字段 tuple newtype | 否 | 禁止 | 禁止 |

同一声明只能使用其中一个角色宏。五种宏第一版都拒绝：

- 泛型参数和 lifetime 参数；
- `where` 子句；
- union；
- 多字段 tuple struct；
- 与角色不匹配的数据形状。

拒绝泛型的原因不是类型系统无法表达，而是注册、静态 descriptor、类型擦除构造和单态化集合尚无明确需求。
以后如需支持，必须单独设计“注册具体单态实例”协议，不能静默注册开放泛型定义。

字段组合还必须遵守以下规则：

| 所属角色 | 可直接嵌套 | 需要 `#[reference]` 才能出现 | 禁止作为字段叶子 |
| --- | --- | --- | --- |
| Entity | scalar、Value、Enum、Model、opaque | Entity、Projection | 无其他角色 |
| Projection | scalar、Value、Enum、Model、opaque | Entity、Projection | 无其他角色 |
| Model | scalar、Value、Enum、Model、opaque | Entity、Projection | 无其他角色 |
| Enum payload | scalar、Value、Enum、Model、opaque | 不允许直接关系 | Entity、Projection |
| Value | scalar、Value、Enum、opaque | 不允许关系 | Entity、Projection、Model |

表中的规则递归穿过 Option、Sequence、Set、Map、数组和 Box 等容器。Entity、Projection 或 Model 的字段叶子
如果是 Entity/Projection 却没有 `#[reference]`，完整 schema 校验必须拒绝，不能把领域身份对象当普通内嵌值。

### 4.2 Entity

```rust
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[Entity(
    model_id = "qubit.platform.person.Person",
    unique(
        name = "person_name_in_tenant",
        fields(tenant, name),
        ignore_case(name),
    ),
    index(name = "person_create_time", fields(create_time)),
    projection(
        name = "info",
        output = PersonInfo,
        using = Person::info,
    ),
)]
pub struct Person {
    #[identifier]
    pub id: Id,

    #[reference(
        entity = "qubit.platform.tenant.Tenant",
        select = projection("info"),
        bind = preexisting,
    )]
    pub tenant: Info,

    #[unique(ignore_case)]
    #[reference_key]
    #[text(min_chars = 1, max_chars = 128)]
    pub username: String,

    #[indexed]
    pub create_time: DateTime<Utc>,
}
```

`model_id` 是 Entity 宏唯一必填的模型级参数。`unique`、`index` 和 `projection` 可以重复。

Entity 必须实现 `Clone`。这不是领域身份定义，而是 Rust 自动对象图生成中完整 Entity 选择和隐藏来源保存所需的
运行时能力。宏不自动派生 `Clone`，而是生成明确的 trait 断言；声明者使用原生 `#[derive(Clone)]` 或手工实现。

### 4.3 Projection

通用 Projection：

```rust
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[Projection]
pub struct Info {
    #[identifier]
    pub id: Id,
    pub code: String,
    pub name: String,
}
```

固定来源 Projection：

```rust
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[Projection(source = "qubit.platform.person.Person")]
pub struct PersonInfo {
    #[identifier]
    pub id: Id,
    pub name: String,
}
```

`source` 是可选的 Entity `ModelId`：

- 省略表示通用 Projection；
- 指定后，完整注册表必须确认该 Entity 存在；
- 任何输出该 Projection 的 projector 都必须属于该来源 Entity；
- 固定来源 Projection 在完整模型集合中至少应有一个对应 projector；
- `source` 不是 Projection 的 ID，也不用于注册 Projection。

Projection 可以声明 Entity 关系，但不能声明任何持久化属性。

### 4.4 Model

```rust
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[Model]
pub struct RegisterUserRequest {
    #[text(min_chars = 1, max_chars = 64)]
    pub username: String,

    #[reference(
        entity = "qubit.platform.tenant.Organization",
        select = identifier,
        bind = preexisting,
    )]
    pub organization_id: Option<Id>,
}
```

Model 没有宏参数形式的 ID。它可以被注册、作为生成和校验根节点，并声明 Entity 关系，但不能出现 identifier、
unique、index 或其他持久化属性。

如果 DTO 字段表示“在 User 表中尚未被使用的 username”，这不是 Model 自身的唯一约束。它应使用明确的自定义
validator/generator 策略，或在服务层声明目标 Entity 语义，不能继续滥用 `#[unique]`。

### 4.5 Enum

```rust
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[Enum]
pub enum TaskState {
    Pending,
    Running {
        #[text(min_chars = 1, max_chars = 128)]
        worker: String,
    },
    #[variant(name = "DONE")]
    Completed,
}
```

Enum 的规范变体名默认使用 `SCREAMING_SNAKE_CASE`，可以用 `#[variant(name = "...")]` 覆盖。它与 Serde
配置是两项独立声明：宏不生成 Serde 实现，也不偷偷修改 `#[serde]`。若二者用于同一外部协议，声明者必须令它们
保持一致。

所有 Enum 生成 `name(&self) -> &'static str`；只有全部为 unit variant 时生成
`from_name(&str) -> Option<Self>`。这两项属于 Enum 元数据协议，不代表 Enum 拥有 `ModelId`。

Enum payload 只允许局部约束和策略，不允许 identifier、持久化属性或直接关系。

### 4.6 Value

```rust
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
#[Value(transparent)]
pub struct Revision(u64);
```

```rust
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[Value(textual)]
pub struct Phone {
    #[text(min_chars = 1, max_chars = 8, allowed_chars = ascii)]
    pub country_area: Option<String>,

    #[text(min_chars = 1, max_chars = 32, allowed_chars = ascii)]
    pub number: String,
}
```

Value 支持两个角色相关开关：

- `transparent`：仅允许单字段 tuple newtype，生成与内层类型的双向 `From`；转换不执行校验；
- `textual`：把具名 Value 的整体标记为文本能力；newtype 默认继承内层类型能力。

`Copy`、Serde、Debug、Display、Eq 和 Hash 都使用原生 derive，不由 `#[Value]` 生成。

Value 字段可以直接包含标量、Enum、其他 Value、这些类型的容器以及显式 opaque 叶子。Value 不能直接包含
Entity、Projection 或 Model；否则它会把身份、可发现根节点或关系语义伪装成纯值。带关系的 `Address`、
`Contact` 等对象应声明为 Model，而不是 Value。

### 4.7 通用 Rust 行为不再由角色宏生成

五个角色宏只负责：

- 角色语义；
- 静态 descriptor；
- 类型访问与构造适配器；
- 角色 trait；
- 需要的全局注册项；
- Enum 规范变体名与 Value `transparent` 等角色专属能力。

角色宏不再负责：

- `Clone`、`Copy`、Debug、Display、Eq、Ord、Hash；
- Serialize、Deserialize 或 Serde 空值省略；
- Default；
- 脱敏实现。

因此删除全部 `no_*` 开关、自动 Debug-shaped Display、自动 Serde rename/default/skip 和自动 redaction 集成。
声明者使用标准 `#[derive(...)]`、`#[serde(...)]` 和 `qubit-redact`。这样可以避免模型宏意外改变 wire format、
打印敏感值或因为无关 trait bound 阻塞元数据生成。

宏只移除自己认识的模型 helper 属性，所有 Rust、Serde、redact 和第三方属性原样保留。

## 5. 宏内部架构

### 5.1 统一流水线，角色化 IR

五个入口复用同一条结构流水线，但绝不再把所有角色压成一个要求 `id` 的 `ModelIr`：

```text
角色入口
  -> 解析 Rust 声明与角色参数
  -> 解析字段/变体 helper 属性
  -> 公共结构 IR + RoleIr
  -> 公共约束归一化
  -> 角色专属语义验证
  -> Rust 类型断言
  -> descriptor / operations / registration 展开
```

核心 IR 采用以下形态：

```rust
struct TypeIr {
    ident: Ident,
    shape: TypeShapeIr,
    fields_or_variants: StructuralIr,
    role: RoleIr,
    source: SourceSpans,
}

enum RoleIr {
    Entity(EntityIr),
    Projection(ProjectionIr),
    Model(ModelIr),
    Enum(EnumIr),
    Value(ValueIr),
}
```

公共 `TypeIr` 中没有 `ModelId`、identifier、unique、index 或 reference 字段。它们只能存在于允许它们的
`RoleIr` payload 中。解析器发现错误作用域时仍保留 span 并聚合诊断，但不会把非法属性塞进公共 IR。

### 5.2 Role policy

每种角色有一份集中、可测试的 policy：

```rust
struct RolePolicy {
    accepted_shapes: ShapeSet,
    registration: RegistrationPolicy,
    model_id: Cardinality,
    identifier: Cardinality,
    persistence: Permission,
    relations: Permission,
    allowed_compositions: CompositionPolicy,
}
```

policy 是 parser/validator 的规则源，不用于 runtime 元数据。任何新增 helper 属性必须先进入属性分类表，再明确
列出五种角色的作用域；禁止通过默认分支让新属性悄悄落入所有宏。

### 5.3 建议源码边界

```text
src/
├── entry/
│   ├── entity.rs
│   ├── projection.rs
│   ├── model.rs
│   ├── enum_type.rs
│   └── value.rs
├── parse/
│   ├── declaration.rs
│   ├── role_args.rs
│   ├── field_attrs.rs
│   └── paths.rs
├── ir/
│   ├── structural.rs
│   ├── role.rs
│   ├── constraints.rs
│   ├── persistence.rs
│   └── relations.rs
├── normalize/
├── validate/
│   ├── common.rs
│   ├── entity.rs
│   ├── projection.rs
│   ├── model.rs
│   ├── enum_type.rs
│   └── value.rs
├── expand/
│   ├── descriptor.rs
│   ├── access.rs
│   ├── registration.rs
│   ├── projection.rs
│   └── role_traits.rs
└── diagnostic.rs
```

模块名可以在实现时微调，但 parser、IR、validation 和 expansion 的边界不得再次合并成单个巨型文件。

## 6. Runtime 元数据分层

### 6.1 `TypeMetadata` 不再拥有 `ModelId`

所有五种类型共享：

```rust
pub struct TypeMetadata {
    identity: TypeIdentity,
    kind: TypeKind,
    capabilities: TypeCapabilities,
    role: TypeRoleMetadata,
}

pub enum TypeRoleMetadata {
    Entity(EntityMetadata),
    Projection(ProjectionMetadata),
    Model(ModelMetadata),
    Enum(EnumRoleMetadata),
    Value(ValueMetadata),
}
```

`TypeKind` 只表达 Rust 结构：Struct、Newtype、Enum。`TypeRoleMetadata` 表达领域角色。构造器必须验证合法组合：

- Entity、Projection 只能对应具名 Struct；
- Model 只能对应具名或 unit Struct；
- Enum 只能对应 Enum；
- Value 只能对应具名 Struct 或单字段 Newtype。

### 6.2 角色 payload

```rust
pub struct EntityMetadata {
    model_id: ModelId,
    identifier: IdentifierMetadata,
    persistence: EntityPersistenceMetadata,
    relations: RelationSet,
    projections: &'static [ProjectionFactory],
}

pub struct ProjectionMetadata {
    identifier: IdentifierMetadata,
    source: Option<ModelId>,
    relations: RelationSet,
}

pub struct ModelMetadata {
    relations: RelationSet,
}

pub struct EnumRoleMetadata;

pub struct ValueMetadata {
    transparent: bool,
    textual: bool,
}
```

具体字段类型可调整，但以下不变量不可改变：

- `ModelId` 只能从 `EntityMetadata` 获得；
- identifier 只能从 Entity 或 Projection payload 获得；
- persistence 只存在于 Entity payload；
- relations 只存在于 Entity、Projection、Model payload；
- Enum 与 Value 的结构不可能直接携带关系集合。

### 6.3 公共字段元数据只保存局部事实

```rust
pub struct FieldMetadata {
    ordinal: usize,
    name: &'static str,
    field_type: TypeRef,
    constraints: FieldConstraints,
    strategies: FieldStrategies,
}
```

identifier、unique、index、reference 不再伪装成 `FieldMetadata.attributes`。字段 helper 只是声明语法，归一化后
分别进入角色 payload，并用字段 ordinal/path 关联回公共字段。

`FieldConstraints` 使用有类型的可选项或小型和类型，而不是无类型 `AttributeMetadata` 列表。例如文本、十进制、
时间、序列、Map 和 element 约束都有强类型 getter。重复单例约束在宏阶段失败。

`FieldMetadata` 本身保持纯描述。与每个字段 ordinal 对齐的 `FieldAccess` 数组属于下一节的 `TypeAccess`，避免
同一操作同时出现在 metadata 和 descriptor 两处。

### 6.4 `TypeDescriptor` 同时提供描述与安全访问

只保存字段名称不足以支持实际校验和生成。本设计引入：

```rust
pub struct TypeDescriptor {
    metadata: &'static TypeMetadata,
    access: TypeAccess,
}
```

`TypeAccess` 是与 `TypeKind` 对齐的函数指针表，概念上提供：

- 从 `&dyn Any` 借用 struct/newtype/enum payload 字段；
- 从按 ordinal 提供的已类型检查值构造 struct 或 newtype；
- 根据 enum variant ordinal 构造对应 variant；
- 查看现有 enum 值当前处于哪个 variant；
- 对标准 Option、序列、Set、Map、数组和 `Box<T>` 进行只读结构遍历；
- 在 identifier、reference key 和 Projection 场景执行角色专属提取。

Struct/Newtype/Enum access 表中的字段和 variant 操作都按 metadata ordinal 一一对齐。构造
`TypeDescriptor` 时必须验证数量、ordinal 和期望 `TypeIdentity` 全部一致。

概念签名如下：

```rust
type BorrowField = for<'a> fn(&'a dyn Any) -> Result<&'a dyn Any, AccessError>;
type ConstructValue = fn(&mut dyn ValueSource) -> Result<Box<dyn Any>, AccessError>;
```

生成的 adapter 在进入用户字段前先 downcast 到宏声明时已知的 Rust 类型；类型不匹配返回 `AccessError`，不得
unchecked cast 或 panic。descriptor 中的期望 `TypeIdentity` 与 adapter 必须在构造时一致。

`TypeAccess` 只提供结构访问，不实现数据库、校验规则或随机策略。这使不同消费者可以共享安全反射层，而无需通过
Serde JSON 中转。Serde 可能 rename、skip 或拒绝反序列化，不能充当模型反射协议。

### 6.5 静态解析，不依赖注册表递归

```rust
pub trait HasTypeDescriptor: HasTypeShape + 'static {
    fn type_descriptor() -> &'static TypeDescriptor;
}
```

五种宏都实现 `HasTypeDescriptor`。`TypeRef::Named` 保存惰性的 descriptor resolver 函数指针。查询嵌套类型时：

```text
外层字段 TypeRef
  -> NamedTypeRef resolver
  -> 嵌套类型 TypeDescriptor
```

该过程不访问全局注册表，所以 Value 即使不注册，也能被 Model、Enum payload 或其他 Value 递归查询和处理。
resolver 必须保持惰性，避免递归类型在静态初始化时无限展开。

### 6.6 角色 trait

推荐公共 trait 层次：

```rust
pub trait ModelType: HasTypeDescriptor {}

pub trait RegisteredModel: ModelType {
    fn registration() -> &'static ModelRegistration;
}

pub trait IdentifiedModel: ModelType {
    const IDENTIFIER_FIELD: FieldOrdinal;
    fn identifier(&self) -> Id;
}

pub trait EntityModel: RegisteredModel + IdentifiedModel + Clone {
    const MODEL_ID: ModelId;
}

pub trait ProjectionModel: RegisteredModel + IdentifiedModel {
    const SOURCE: Option<ModelId>;
}

pub trait PlainModel: RegisteredModel {}
pub trait EnumModel: RegisteredModel {}
pub trait ValueType: ModelType {}

pub trait ValueComponent: HasTypeShape {}
```

最终命名可以为避免 crate API 冲突做机械调整，但能力边界必须保持。宏通过返回 `Id` 的方法让 Rust 编译器证明
identifier 字段类型确实是 `Id`，而不是依赖类型名字符串判断。

这些角色 trait 是宏生成的类型约束便利层，不是运行时语义的唯一事实来源。跨 crate 过程宏必须在用户 crate
中生成 impl，因此传统的私有 supertrait 无法在“允许过程宏实现”的同时真正阻止用户手工实现；本设计不虚构一层
并不存在的 sealed 保证。正确性边界如下：

- 五种宏只生成与自身角色一致的 impl，同一声明使用多个角色宏会因宏校验或冲突 impl 编译失败；
- 手工实现角色 trait 属于不受支持的用法；所有框架消费者仍以 `TypeRoleMetadata` 为运行时事实来源；
- metadata 的安全公共构造器重复校验角色与结构组合，类型擦除访问不得只凭 marker trait 执行 unchecked 操作；
- `ValueComponent` 的官方 impl 只覆盖 scalar、Enum、Value、opaque adapter 及其合法容器；每个 Value 字段生成
  `ValueComponent` 断言，使正常宏用法在跨 crate 场景也能于编译期拒绝 Entity、Projection 和 Model；
- 即使有人手工伪造 trait impl，checked descriptor 构造和完整 schema 校验仍必须拒绝非法角色组合。

## 7. 注册协议

### 7.1 注册项不再要求稳定 ID

```rust
pub struct ModelRegistration {
    descriptor: &'static TypeDescriptor,
    rust_type_name: &'static str,
    rust_module_path: &'static str,
    source: SourceLocation,
}
```

Entity、Projection、Model 和 Enum 生成 `linkme` 分布式注册项；Value 不生成注册项，也不实现
`RegisteredModel`。

注册项的主身份是 `descriptor.metadata.identity`。Rust 类型名只用于确定性排序和诊断，不作为持久化或跨进程键。

### 7.2 注册表索引

```text
ModelRegistry
├── TypeIdentity -> ModelRegistration       // 四种已注册角色
├── ModelId -> Entity ModelRegistration     // 仅 Entity
├── role -> [ModelRegistration]
└── (Entity ModelId, selector name) -> ProjectionFactory
```

建议 API：

```rust
registry.registration(TypeIdentity)
registry.descriptor_of::<T>()
registry.entity(ModelId)
registry.entities()
registry.projections()
registry.models()
registry.enums()
registry.projection_factory(ModelId, ProjectionSelector)
```

不提供按普通类型显示名称查找的 API，也不提供对 Projection、Model 或 Enum 的伪 `ModelId` 查找。

### 7.3 `ModelId` 规则

宏在编译期验证 `model_id` 和 reference 目标字符串的词法格式；完整注册表验证唯一性与目标存在性。

`ModelId` 不再强制最后一段等于 Rust 类型名。稳定协议身份必须能承受 Rust 类型或 module 重命名。推荐格式仍为：

```text
lower_snake_case namespace segments + UpperCamelCase final segment
```

同一完整注册表中两个 Entity 使用相同 `ModelId` 是确定性错误。非 Entity 声明 `model_id` 是编译错误。

### 7.4 构建与完整校验分离

注册表支持两步：

1. `collect`：收集已链接条目，验证重复 `TypeIdentity`、重复 Entity `ModelId` 和条目内部一致性；
2. `validate_complete`：在调用方确认模型集合完整后，验证跨 crate Entity 目标、Projection source、projector、
   reference key、关系选择和 `same_as` 路径。

局部工具仍可以从显式注册项切片构造部分注册表，但不能声称已经完成跨模型验证。

所有聚合错误按角色、来源类型、字段路径、目标 `ModelId` 和错误码确定性排序。

## 8. Identifier、持久化与 reference key

### 8.1 Identifier

`#[identifier]` 不接受 `generated`、`ignore_case` 或其他选项：

```rust
#[identifier]
pub id: Id,
```

Entity identifier 同时是领域实例身份和独立记录主键。Projection identifier 借用来源 Entity 身份，但不产生
Projection 主键。

identifier 始终是所属 struct 的直接字段，不能是 Option、容器或嵌套字段。Entity 的 `Id` 是构造 Entity 时就应
存在的应用层身份；宏不保存“数据库插入后生成”策略。需要数据库生成值的适配器必须先预分配/插入取得 `Id`，再
进入对象图构造协议，不能让一个尚无身份的 Entity 参与 reference 或 provenance。

删除 `primary_key(...)`。系统已经明确只支持一个 `Id` identifier，继续保留复合主键 DSL 会制造相互冲突的
身份模型。如果数据库中存在复合自然键，应使用 unique constraint；如果外部旧表确实没有 `Id`，应在适配层处理，
而不是削弱统一领域身份。

### 8.2 Entity 持久化约束

Entity 宏支持可重复的模型级约束：

```rust
#[Entity(
    model_id = "qubit.platform.tenant.Employee",
    unique(
        name = "employee_code_in_org",
        fields(organization, code),
        ignore_case(code),
    ),
    index(
        name = "employee_contact_mobile",
        fields(contact.mobile),
    ),
)]
```

字段简写：

```rust
#[unique]
#[unique(ignore_case)]
#[indexed]
```

规则如下：

- 字段简写只允许 Entity 的直接字段；
- 复合或嵌套约束必须写在 Entity 宏参数中；
- `fields(...)` 使用规范 Rust 字段路径，允许穿过具名类型和外层 `Option`；
- 路径不能穿过序列、Set、Map 或数组；
- 路径一旦遇到 direct reference 就必须在该字段终止，不能继续读取其当前保存的 ID、key、Projection 或 Entity
  子字段；
- unique 的最终字段必须是可比较的单值叶子；
- `ignore_case(...)` 中的每个路径必须属于 `fields(...)` 且具有文本能力；
- index 的最终路径必须是持久化消费者可识别的单值或 relation 叶子；
- 每个 reference occurrence 自动产生一个单路径 effective index；直接字段上的 `#[reference]` 与 `#[indexed]`
  重复声明是编译错误；
- Entity 模型级单字段 `index(fields(...))` 若与某个 reference occurrence 的派生路径相同，也是重复声明错误；
  包含该 reference 路径的复合 index 仍然允许；
- unique 与 index 可以同时声明，是否由数据库合并物理索引属于 ORM/DDL 消费者。

Projection 或共享 Model 不能声明独立的持久化索引。它们声明的 reference 仍携带“必须索引”的关系契约，并在
自身 descriptor 的逻辑 effective index view 中可见；这不等于为该类型建立独立数据表。Model 被嵌入 Entity 时，
Entity 的 effective persistence view 还会为每个具体 occurrence 派生完整索引路径。因此同一个共享 Model 被嵌入
两个 Entity 或同一 Entity 的两个字段时，会形成彼此独立的持久化 effective index。Projection 不是普通持久化
内嵌值，其自身的派生索引只供对象图、校验和查询规划等消费者读取，不自动归属到 source Entity 的 DDL。

`EntityPersistenceMetadata` 只保存用户显式声明的 unique/index；关系派生索引不复制进该列表。descriptor 可从
本类型的 direct relation 生成逻辑 effective index；完整 schema 还会在 Entity 根下遍历可达 Model descriptor，
按具体 reference occurrence 生成持久化 effective index 集合：

```rust
pub enum IndexOrigin {
    Explicit,
    ReferenceDerived {
        relation_owner: TypeIdentity,
        relation_field: FieldOrdinal,
    },
}

pub struct EffectiveIndexMetadata {
    path: PersistencePath,
    origin: IndexOrigin,
}
```

派生索引使用 occurrence 的规范字段路径作为逻辑身份，不要求过程宏虚构数据库索引名。ORM/DDL 消费者可以按既定
命名规则生成物理名称。显式单路径 index 与派生路径完全相同即为重复；复合 index 因路径集合不同，不算重复。

持久化约束中以 reference 结尾的路径，逻辑上表示目标 Entity 实例，而不是 selection 恰好保存的表示值。因此
`unique(fields(tenant, name))` 不会因为 `tenant` 从 Info 改成 identifier 而改变领域唯一性。ORM 如何把该逻辑关系
映射为物理列或索引不属于本设计；需要索引 Projection 中某个物化值时，应把它建模为 Entity 的独立普通字段。

### 8.3 Reference key

除 identifier 外，只有显式暴露的全局唯一字段才能成为跨模型引用键：

```rust
#[unique(ignore_case)]
#[reference_key(name = "username")]
pub username: String,
```

reference key 必须满足：

- 位于 Entity 的直接字段；
- 非 Optional、非容器、非 opaque；
- 被一个单字段全局 unique constraint 覆盖，不能是 scoped/composite unique 的一部分；
- 字段值实现 `Clone + 'static`，宏为其生成类型擦除提取器；
- key 名为非空 ASCII snake_case，在当前 Entity 内唯一；
- 比较语义继承对应 unique constraint，包括是否忽略大小写。

`#[unique]` 不自动把字段暴露为 reference key。可变 email 等唯一字段不应在无意中成为外部关系协议。

### 8.4 删除或移出核心的旧属性

- `key(...)`：删除。struct 字段声明顺序已经能表达嵌套值的规范顺序；特殊编码交给 codec。
- `ownership(owner = Type)`：删除。权限归属需要 `path + domain` 等独立设计，且目前没有生产用例。
- `lookup_relation(...)`：删除。规则查询关系需要单独的查询表达式设计，不能混入直接引用。
- `computed`：删除。计算值应是真实方法、Projection projector 或独立查询层能力。
- `nullable`：删除。可空性只由最外层 `Option<T>` 表达。

## 9. `#[reference]` 全新协议

### 9.1 完整语法

每个 direct reference 必须显式写出三个正交事实：目标 Entity、保存何种选择值、目标 Entity 从哪里取得。

```rust
#[reference(
    entity = "qubit.platform.person.Person",
    select = identifier,
    bind = preexisting,
)]
pub person_id: Id,
```

```rust
#[reference(
    entity = "qubit.platform.person.Person",
    select = key("username"),
    bind = preexisting,
)]
pub username: String,
```

```rust
#[reference(
    entity = "qubit.platform.person.Person",
    select = projection("info"),
    bind = preexisting,
)]
pub person: PersonInfo,
```

```rust
#[reference(
    entity = "qubit.platform.file.Attachment",
    select = entity,
    bind = inline,
)]
pub attachments: Vec<Attachment>,
```

```rust
#[reference(
    entity = "qubit.platform.order.Order",
    select = identifier,
    bind = same_as(".."),
)]
pub order_id: Id,
```

`select` 和 `bind` 都没有默认值。关系与生命周期是高影响语义，省略后猜测字段类型会重新引入旧设计的问题。

### 9.2 元数据和类型

```rust
pub struct ReferenceMetadata {
    field: FieldOrdinal,
    entity: ModelId,
    selection: EntitySelection,
    binding: ReferenceBinding,
}

pub enum EntitySelection {
    Entity,
    Identifier,
    Key(ReferenceKeyName),
    Projection(ProjectionSelector),
}

pub enum ReferenceBinding {
    Preexisting,
    Inline,
    SameAs(ObjectGraphPath),
}
```

旧 `ReferenceTarget::Property(FieldPath)` 不保留。普通字段、reference key 和命名 Projection 的合法性完全不同，
不能继续由一个任意字符串路径表示。

### 9.3 Selection 语义

| selection | 当前字段保存内容 | 注册表验证 | 运行时取值 |
| --- | --- | --- | --- |
| `entity` | 完整目标 Entity | 叶子类型等于目标 Entity | Clone 目标 Entity |
| `identifier` | 目标 Entity 的 `Id` | 叶子类型为 `Id` | 调用 `EntityModel::identifier` |
| `key("name")` | 显式 reference key 值 | key 存在且叶子类型一致 | 调用 key 提取器 |
| `projection("name")` | 命名 Projection | projector 存在且输出类型一致 | 调用 projector |

关系字段允许以下外层包装：

- 单值 `T`；
- `Option<T>`；
- Sequence/Set/固定数组及其外层 Optional 组合。

类型兼容检查逐层剥离这些关系容器并比较唯一叶子类型。Map 关系第一版拒绝，因为 key/value 哪一侧代表关系以及
路径复用语义都不明确。

关系叶子不能标记为 opaque。四种 selection 都需要验证和执行准确的目标类型，隐藏叶子结构会让兼容性检查失去
意义。

`inline` 只允许 `select = entity` 或 `select = projection(...)`。生成一个不持久化 Entity 后只保存其 ID 或 key
会制造悬空引用，因此在 schema 阶段直接拒绝。

### 9.4 Binding 语义

#### `preexisting`

目标 Entity 必须在产生当前字段值之前可用。生成器可以：

1. 从 repository 选择符合条件的已存在 Entity；或
2. 递归生成一个目标 Entity，满足其依赖并先持久化；
3. 再执行 selection 填写当前字段。

#### `inline`

生成器创建完整目标 Entity，但本引用不要求、也不允许把它作为独立记录预先持久化。目标 Entity 自己的
`preexisting` 关系仍必须递归满足。随后执行 whole-entity 或 Projection selection，把结果作为当前对象的内联值。

#### `same_as(path)`

不创建也不查询新 Entity，而是复用对象图中另一处已经绑定的实际目标 Entity，再执行当前字段自己的 selection。
两处字段可以保存不同表示，例如一处保存 `Info`，另一处保存 ID，只要它们指向同一个 Entity `ModelId`。

`same_as` 不复制被指向字段的保存值，也不要求两处 selection 或 binding 相同。

### 9.5 Projection projector

Entity 在宏参数中注册命名 projector：

```rust
#[Entity(
    model_id = "qubit.platform.metadata.Category",
    projection(
        name = "info",
        output = Info,
        using = Category::info,
    ),
    projection(
        name = "stateful_info",
        output = StatefulInfo,
        using = Category::stateful_info,
    ),
)]
pub struct Category {
    // ...
}
```

`using` 必须能强制转换为：

```rust
fn(&Category) -> OutputProjection
```

这是“完整 Entity 输入”形式。为了支持 Rust 不可变 struct 构造期间的父对象回指，projector 还支持显式的
“已就绪字段输入”形式：

```rust
#[Entity(
    model_id = "qubit.platform.metadata.Category",
    projection(
        name = "info",
        output = Info,
        inputs(id, code, name),
        using = Info::from_category_fields,
    ),
)]
pub struct Category {
    #[identifier]
    id: Id,
    code: String,
    name: String,
    // ...
}

impl Info {
    fn from_category_fields(id: &Id, code: &String, name: &String) -> Self {
        // ...
    }
}
```

省略 `inputs(...)` 时，`using` 的签名固定为 `fn(&Entity) -> OutputProjection`；声明 `inputs(...)` 时，签名固定为
按声明顺序接收各直接字段引用的 `fn(&Field1, &Field2, ...) -> OutputProjection`。第一版只允许不重复的直接字段，
不接受嵌套路径或空 `inputs()`。宏生成函数指针强制转换，让 Rust 编译器验证全部参数和输出类型。

宏生成类型擦除 `ProjectionFactory`，包含：

- Entity `ModelId`；
- selector 名；
- 输入 Entity `TypeIdentity`；
- 输出 Projection `TypeIdentity`；
- 输入模式：完整 Entity，或有序的输入字段 ordinal；
- 与输入模式对应的可执行类型擦除 adapter；
- 声明源位置。

概念结构如下：

```rust
pub enum ProjectionInput {
    CompleteEntity(ProjectFromEntity),
    ReadyFields {
        fields: &'static [FieldOrdinal],
        project: ProjectFromFields,
    },
}

type ProjectFromEntity =
    fn(&dyn Any) -> Result<Box<dyn Any>, ProjectionError>;
type ProjectFromFields =
    fn(&[&dyn Any]) -> Result<Box<dyn Any>, ProjectionError>;
```

类型擦除 adapter 必须逐个 downcast 字段，并把错误转成带 selector 和 ordinal 的 `ProjectionError`，不得 panic。

完整注册表验证：

- Entity `ModelId` 指向当前 Entity 注册项；
- input identity 与 Entity descriptor 一致；
- output 已注册且角色为 Projection；
- fixed-source Projection 的 `source` 与 Entity 一致；
- 同一 Entity 内 selector 唯一；
- ReadyFields 的 ordinal 存在、不重复，且 adapter 声明的期望类型与字段 descriptor 一致；
- reference 字段叶子类型与 output identity 一致。

projector 每次执行后必须比较输出 Projection identifier 与输入 Entity identifier。不同则返回
`ProjectionIdentifierMismatch`，不能把错误投影写入对象图。

projector 的返回值是完整 Projection。生成器不得再按 Projection descriptor 逐字段补写或覆盖，而应对返回值执行
普通递归值校验。Projection 自己的 relation metadata 继续用于 schema 校验，以及调用方显式提供关系上下文时的
引用存在性校验；纯 projector 返回值不能凭 identifier、key 或嵌套 Projection 值反推出其内部 relation 的实际
Entity provenance。因此第一版的 `same_as` 不能把 projector 输出内部的 reference 字段当作 provenance 来源。
需要复用关系时，路径必须经过来源 Entity 已登记的 reference provenance；系统不得猜测目标或临时查询后补全。

这套协议完全替代 `info` 特判。`info` 只是普通 selector 名，不享受任何保留行为。

### 9.6 `same_as` 路径与 Entity provenance

`ObjectGraphPath` 使用 `/` 分隔：

- `street/district`：从当前对象开始进入字段；
- `../organization`：进入父对象后访问字段；
- `..`：父对象本身；
- `/order/customer`：从当前生成根开始；
- `.`、空段和超出根的 `..` 非法。

字段约束路径继续使用 `contact.mobile` 点号语法。两种路径不是同一概念，不能混用。

生成器必须维护隐藏 provenance：

```text
(object graph node, reference field, optional element position)
    -> actual target Entity handle
```

例如字段 `app` 只保存 App 的 `Info`，`same_as("app/tenant")` 仍先从 `app` 的 provenance 找到实际 App Entity，
再沿 App Entity 的 `tenant` reference provenance 找到实际 Tenant，而不是尝试从 `Info` 值中读取 `tenant`。

路径解析规则：

- 可以穿过具名 Model/Value 的普通单值字段；
- 遇到 reference 字段时，下一段在其实际目标 Entity descriptor 上继续；
- 可以从集合元素上下文使用 `..` 回到包含对象；
- 第一版不允许显式穿过集合、Set、Map 或数组字段，也不支持下标选择；
- `same_as` 本身只允许单值或 Optional 单值 reference 字段，不允许集合字段；
- 终点必须是目标 Entity 节点或带同一 Entity `ModelId` 的 reference provenance；
- 完整 schema 下路径不能解析时直接报错，不采用“路径失败就创建新 Entity”的回退行为。

如果运行时路径经过的 Optional 普通字段或 reference provenance 为 None，则当前 same_as 字段为 Optional 时结果也是
None；当前字段为必需值时返回 `MissingReferenceProvenance`。集合字段仍不允许 same_as，因此不存在用空集合隐式选择
某个目标的规则。

路径验证分两层进行：注册表可以完整验证不离开当前类型上下文的相对子路径；包含 `..` 或根前缀 `/` 的路径依赖
“该 Model 当前被嵌在哪个根对象的哪个字段”这一具体 occurrence，只能由 generator 的
`GenerationPlanner::plan(root_descriptor)` 在生成前完整解析。规划失败仍是确定性 schema/plan 错误，不能延迟到
随机生成一半后再回退。可复用 Model 因此可以合法声明 parent path，而注册表不会假造一个不存在的固定父类型。

### 9.7 间接关系无需标记

删除 Java 式裸 `#[reference]`。如果 Entity 的 `contact: ContactModel` 中包含 direct reference，生成器和校验器
通过 `ContactModel` 的 descriptor 自然递归发现。要求额外标记“这里内部可能有关系”既重复又容易漏写。

Value 禁止关系，因此递归进入 Value 时无需构建关系分支；Model 可以有关系，并且已经注册。

### 9.8 Reference 自动要求索引

reference 同时建立对象关系语义和最低索引契约：每个 direct reference 都出现在所属 descriptor 的逻辑 effective
index 集合中；当它直接属于 Entity 或经 Model 嵌入 Entity 时，每个具体 occurrence 还必须出现在该 Entity 的
持久化 effective index 集合中，且派生的是该 occurrence 的单路径 index。该规则不随 `select` 是 Entity、
identifier、reference key 还是 Projection 而变化，也不随 `bind` 是 preexisting、inline 还是 same_as 而变化。

用户不需要、也不允许再为同一关系路径声明单字段 index：直接字段上的 `#[reference] + #[indexed]` 在过程宏阶段
报错；模型级或嵌套路径的等价重复在完整 schema 阶段报错。包含 reference 路径的复合 index 仍是独立访问策略，
因此合法。Projection、Model 自身没有 persistence view，但仍有可查询的逻辑 effective index；Model 中的关系在
某个 Entity 根下形成具体 occurrence 时才额外物化为持久化 effective index，避免为非持久化角色伪造独立表索引。

## 10. 字段约束与策略

### 10.1 局部约束

以下约束继续作为公共结构能力，并可用于所有角色允许的普通字段或 Enum payload：

| 属性 | 语义 |
| --- | --- |
| `#[text(...)]` | 字符数、字节数、字符集、非空白、格式 |
| `#[decimal(...)]` | 精度、小数位、范围和舍入 |
| `#[money(...)]` | 货币金额的 decimal 语义 |
| `#[time(...)]` | 时间精度和允许范围 |
| `#[sequence(...)]` | 元素数量和 `unique_items` |
| `#[map(...)]` | entry 数量 |
| `#[element(...)]` | 第一层序列/数组元素约束 |
| `#[opaque]` | 保留外层容器形状，但停止解析叶子内部结构 |
| `#[validator(name = "...")]` | 命名自定义校验策略 |
| `#[generator(name = "...")]` | 命名自定义生成策略 |
| `#[codec(name = "...")]` | 命名编码策略 |

参数名称只保留 Rust `snake_case`。删除 `ignoreCase`、`respectTo` 等兼容别名；未知参数一律报错。

`text`、`decimal`、`money`、`time`、`sequence`、`map` 和 `element` 的现有合法参数集合可以保留，但实现必须
改为独立的 typed constraint IR，不经过通用 attribute bag。

### 10.2 可空与容器语义

- 只有最外层 `Option<T>` 表示字段可空；
- 对 Optional 字段，值为 None 时跳过内层值约束；值为 Some 时，约束作用于解包后的准确形状；
- text/decimal/money/time 作用于 Optional 解包后的值叶子，sequence/map 作用于解包后的容器；
- element 作用于每个实际元素；元素自身为 Option 时，None 跳过内层 element 约束；
- `Option<Vec<T>>` 与 `Vec<Option<T>>` 必须保持不同形状；
- 序列、Set、Map、数组和 `Box<T>` 都应向校验器暴露递归访问；
- 固定数组长度来自 Rust 类型，不接受与长度冲突的 min/max items；
- Map 的 key 和 value 都递归校验，但 reference 不允许直接使用 Map 形状；
- `unique_items` 是集合内容约束，不等同于 Entity 数据库 unique。

### 10.3 Opaque

opaque 叶子不需要实现 `HasTypeDescriptor`，但：

- 不会递归校验内部字段；
- 自动生成必须有匹配的 generator 策略或调用方提供值；
- 标准约束只有在 opaque 类型仍显式提供相应值能力 adapter 时才允许；
- opaque 不能用来隐藏一个 Entity、Projection 或 Model 以绕过角色/关系检查。

### 10.4 自定义策略

策略元数据只保存经过格式验证的 `StrategyId` 和静态参数，不在宏展开期间执行。`StrategyId` 采用点号分隔的
ASCII lower_snake_case 段，例如 `qubit.email`；空段、未知参数和重复单例声明直接报错。实际 validator、
generator、codec 由消费方注册并负责类型匹配。

同一字段可以同时有 validator、generator 和 codec；重复同类策略是否允许由该策略类别明确规定，不能静默覆盖。
自定义策略错误必须保留完整字段路径和策略 ID。

## 11. 自动校验协议

### 11.1 三种校验不得混为一体

1. **宏局部校验**：语法、作用域、重复参数、类型形状和本类型字段路径；
2. **完整 schema 校验**：跨 crate Entity、Projection projector、reference key、`same_as` 路径；
3. **实例值校验**：运行时对象的文本、数值、容器、自定义规则和递归嵌套值。

数据库中目标是否真实存在、unique 是否与已有记录冲突，需要带 repository 的持久化校验上下文，不属于纯实例值
校验。

### 11.2 Typed 入口与动态入口

后续 validator 应提供等价能力：

```rust
validator.validate::<T>(&value)
validator.validate_erased(T::type_descriptor(), &value_as_any)
```

typed 入口直接从 `T: HasTypeDescriptor` 开始。动态入口要求调用方显式提供 descriptor。二者都不要求目标类型在
全局注册表中，因此 Value 可以独立或嵌套校验。

### 11.3 递归流程

```text
根 TypeDescriptor
  -> TypeAccess 借用当前字段值
  -> 根据 TypeRef 展开 Option / 容器 / Named / Opaque
  -> 执行当前形状和字段局部约束
  -> NamedTypeRef resolver 获取嵌套 descriptor
  -> 继续递归
```

示例：

```rust
#[Value]
pub struct Address {
    #[text(min_chars = 1, max_chars = 100)]
    pub city: String,
}

#[Model]
pub struct CreateOrganization {
    pub addresses: Vec<Address>,
}
```

第三个地址的 city 为空时，错误路径应为：

```text
addresses[2].city
```

Value 没有注册项不会中断任何一步，因为 `Vec<Address>` 的元素 `TypeRef` 静态携带 Address descriptor resolver。

### 11.4 错误模型

```rust
pub struct ValidationIssue {
    path: ValuePath,
    code: ValidationCode,
    constraint: ConstraintRef,
    message_args: MessageArgs,
}
```

validator 返回全部独立问题，而不是遇到第一个字段即停止。错误默认按规范路径、错误码和约束声明顺序排序。

错误消息本地化不写入宏生成代码；元数据保存稳定 code 和参数，展示层选择语言。

### 11.5 递归保护

descriptor 可以惰性指回自身。schema 路径查询使用 `(TypeIdentity, field ordinal)` visited set；实例校验同时使用最大
深度或对象地址 visited set，避免 `Box`/共享指针形成无限递归。遇到递归上限必须返回明确错误，不能栈溢出。

### 11.6 无类型输入边界

只有字符串 `"Address"` 和一段 JSON 时，系统不会从全局注册表找到 Value。调用方必须给出 Rust 目标类型或
显式 descriptor。如果业务要求按名称动态发现 Address，它应被定义为 Model。

## 12. 随机生成与 DAO 测试协议

### 12.1 Descriptor 必须支持构造

宏为 Struct/Newtype/Enum variant 生成类型擦除构造 adapter。生成器按 field ordinal 提供已经过类型检查的
`Box<dyn Any>`，adapter downcast 后使用原始 struct/variant 语法构造值。

这样即使类型没有 Default 或 Deserialize，生成器仍可工作；私有字段也不会阻塞，因为 adapter 与声明在同一宏
展开作用域内。

生成器只保证声明在 descriptor 中的约束和策略。无法由元数据表达的跨字段业务不变量必须由自定义 generator 或
构造后的 validator 处理。

### 12.2 普通字段生成

1. 根据 `TypeRef` 选择 scalar、Option、序列、Set、Map、数组、Box、Named 或 Opaque 生成器；
2. 合并字段 constraint 与生成 profile；
3. 递归生成 Value、Model 或 Enum payload；
4. 执行命名 generator 策略；
5. 构造当前值；
6. 用 validator 做生成后校验，失败时按受限重试策略处理。

Opaque 没有 generator 时返回 `MissingGeneratorStrategy`，不能填零值或跳过。

Projection 不能作为普通字段值或无上下文根值被随机拼装。Projection 字段按第 4.1 节必须是 reference，由
projector 产生。显式请求生成 Projection 时，调用方必须同时给出来源 Entity `ModelId` 与 selector；通用
Projection 因可能有多个来源，不能仅凭 Projection `TypeIdentity` 猜测来源。Projection 的 construct adapter
服务于查询映射、测试和其他已明确掌握来源的消费者，不改变其“由 Entity 派生”的角色语义。

### 12.3 分阶段对象构造与 Entity handle

Java 可以先 new 空 Bean、逐项 set，再让子对象读取尚未完成的父对象；Rust struct 通常必须一次性拿到全部字段才能
构造。为了支持实际存在的 `same_as("..")` 父对象 ID/Info 场景，生成器不能依赖 Default、setter、Serde，也不能
对 `MaybeUninit` 做不安全的部分初始化。

规划器为每个具体对象 occurrence 建立内部构造状态：

```text
ObjectOccurrence
├── descriptor、root、parent、parent field/element
├── field slots: Unplanned | Ready(Box<dyn Any>) | Consumed
├── reference provenance slots
└── Entity only: model_id、identifier slot、Complete(Box<dyn Any>)
```

该状态不是一个可借用的“半成品 `&Entity`”。运行规则是：

1. Entity identifier slot 最先生成，使父/祖先 identifier selection 不必等待整个 Entity；
2. 普通字段、reference 字段和嵌套对象分别进入 Ready slot；
3. reference key 在对应字段 Ready 后可提取；
4. ReadyFields projector 在列出的全部字段 Ready 后可执行；
5. whole Entity selection 和 CompleteEntity projector 必须等待 Entity 构造完成；
6. 全部字段 Ready 后，construct adapter 消费 slots 构造真实值；完成后字段读取改由 TypeAccess 从真实值借用；
7. reference provenance 独立保留，不随保存值被消费，因此后续路径仍能找到实际目标 Entity。

这样，子对象引用父 Entity identifier，或引用只依赖父 `id/code/name` 的 Info，都可在父对象最终构造前完成。若
子对象请求完整父 Entity，或请求一个依赖该子字段的 Projection，则字段依赖图会形成真实环并确定失败。

### 12.4 Reference 生成

对每个 reference 叶子：

1. 根据 binding 获取实际目标 Entity；
2. 若为 `preexisting`，先复用或生成并持久化目标；
3. 若为 `inline`，生成目标但不独立持久化，并递归满足其内部依赖；
4. 若为 `same_as`，从 provenance 按路径取得同一个目标；
5. 根据 selection 提取 Entity、identifier、reference key 或执行 projector；
6. 校验提取值的 `TypeIdentity` 和 Projection identifier；
7. 把保存值交给当前字段构造，同时登记实际 Entity provenance；
8. 当前根为 Entity 时，在所有必需依赖满足后再持久化当前 Entity。

repository 操作由生成上下文提供，metadata crate 不依赖具体 DAO。

### 12.5 两类图必须分开

#### Entity 获取/持久化图

`preexisting` 对必需 reference 产生“目标先于来源”的边。required 判断：

- 外层 `Option`：非必需；
- Sequence/Set：只有 `min_items > 0` 且元素必需时才必需；
- Map：reference 形状已被禁止，不进入关系依赖图；
- 固定数组：长度大于 0 且元素必需时必需；
- Scalar/Named/Opaque 单值：必需。

`inline` 不产生独立持久化边；`same_as` 不获取新 Entity。

#### 对象字段赋值图

该图以具体 object occurrence 的字段 slot、Entity completion 和 projector 执行为节点，不能用 Entity `ModelId`
图代替。依赖边至少包括：

- construct 依赖全部必需字段 Ready；
- identifier selection 依赖目标 identifier slot；
- reference key selection 依赖目标 key 字段 slot；
- ReadyFields Projection selection 依赖其 input slots；
- whole Entity 和 CompleteEntity Projection selection 依赖目标 Entity completion；
- `same_as` 先依赖路径上的 provenance，再依赖当前 selection 所需的上述就绪条件。

两类图分别拓扑排序并分别报告错误。把它们合并会误报父子对象合法回指为 Entity 创建环。

### 12.6 环处理

- 必需 `preexisting` 环：返回包含每个 Entity、字段路径和 binding 的 `RequiredEntityCycle`；
- Optional 或允许空的集合边：生成策略可以选 None/空集合切断环；
- `same_as` 赋值环：返回 `ReferenceBindingCycle`；
- parent `same_as("..")` 不构成新 Entity 获取边，但 selection 若等待完整 parent 仍可能构成字段赋值环；
- identifier、已就绪 reference key 或 ReadyFields projector 可以合法切断这种父子构造依赖；
- 不得靠递归深度耗尽或随机选择碰巧逃离环。

生成依赖规划属于后续 generator crate。`rs-model-metadata` 提供稳定的关系投影与路径解析 API，注册表只负责完整
schema 合法性，不在构建时隐式执行 repository 相关规划。

### 12.7 Unique 生成

Entity unique constraint 为生成器提供：

- 有序字段路径；
- ignore-case 字段集合；
- constraint 名；
- scope 值读取顺序。

生成器先生成 scope 字段，再生成依赖它们的唯一字段，并通过 repository/本批次集合检查冲突。DTO/Model 上的
“值必须未被使用”不复用 Entity unique metadata，应使用显式业务策略。

## 13. Enum 专项设计

### 13.1 Variant 元数据

```rust
pub struct EnumVariantMetadata {
    ordinal: usize,
    rust_name: &'static str,
    canonical_name: &'static str,
    kind: EnumVariantKind,
}

pub enum EnumVariantKind {
    Unit,
    Tuple(&'static [FieldMetadata]),
    Struct(&'static [FieldMetadata]),
}
```

tuple payload 字段名使用 ordinal 字符串，struct payload 使用 Rust 字段名。不同 variant 的字段不组成共同字段
集合，因此不存在 enum 级 field path、unique、index 或 relation。

variant 的读取与构造函数位于 `TypeDescriptor::access` 中的 `EnumAccess`，按 variant ordinal 与上述纯元数据
对齐。

### 13.2 规范名

- 默认从 Rust variant 转成 `SCREAMING_SNAKE_CASE`；
- `#[variant(name = "...")]` 可以覆盖；
- 名称不能为空且在 Enum 内唯一；
- 所有 variant 可调用 `name()`；
- 只有全 unit Enum 生成 `from_name()`；
- canonical name 不是 `ModelId`，也不是 Enum 的注册键。

### 13.3 Payload 递归

payload 可以包含 Value、Enum、Model 或其他具有 descriptor 的类型。Enum 本体和 payload 字段禁止直接
`#[reference]`；如果 payload 中的嵌套 Model 自己带关系，生成器进入该 Model descriptor 后按其角色语义处理。

## 14. Value 专项设计

### 14.1 组合边界

Value 的直接或容器叶子允许：

- 内建 scalar；
- Enum；
- Value；
- 显式 opaque 外部值。

禁止 Entity、Projection 和 Model 叶子。宏为每个字段生成 `ValueComponent` trait 断言，因此同 crate 与跨 crate
组合都在编译期检查；公开手工 metadata 构造器仍需重复验证该不变量。

### 14.2 Newtype

单字段 tuple Value：

- 元数据 kind 为 Newtype；
- 默认继承内层 `TypeCapabilities`；
- 字段约束可以作用在 newtype 内层字段；
- `transparent` 显式生成双向 `From`；
- 非 transparent 不生成转换；
- transparent 不自动调用 validator。

### 14.3 注册与递归

Value 生成静态 descriptor、access、construct adapter 和 `ValueType` 实现，但不生成 linkme 项。其 descriptor
只通过 `Value::type_descriptor()` 或外层字段 resolver 到达。

任何校验器、生成器或 path resolver 因 Value 不在注册表而跳过它，都是实现错误。

## 15. 编译与完整注册表诊断

### 15.1 过程宏阶段

宏必须聚合能够独立发现的错误，并定位到最小相关 span：

- 角色与 Rust 形状不匹配；
- 泛型、union、多字段 tuple struct；
- Entity 缺失/重复/非法 `model_id`；
- 非 Entity 声明 `model_id`；
- Entity/Projection identifier 数量或类型错误；
- Model/Enum/Value 使用 identifier；
- 角色不允许的 persistence/relation helper；
- 重复 constraint、参数冲突、范围反转；
- 字段路径语法错误或本类型字段不存在；
- `reference` 缺少 entity/select/bind；
- 同一直接字段同时声明 `reference` 和 `indexed`；
- inline 与 identifier/key selection 冲突；
- Value 非法嵌套已知角色；
- unknown helper、unknown `no_*`、camelCase 兼容参数；
- Entity projector 函数签名无法满足 `fn(&Entity) -> Projection`。

用户输入不得触发 proc macro panic。内部 `expect` 只能用于前一验证阶段已建立且由单元测试覆盖的不变量。

### 15.2 Rust 类型系统阶段

生成代码负责证明：

- identifier 确实可以作为 `Id` 返回；
- Entity 实现 Clone；
- 字段类型实现所需 shape/descriptor 或被标记 opaque；
- 约束与 `TypeCapabilities` 匹配；
- reference key 值实现 Clone；
- projector input/output 实现正确角色 trait；
- TypeAccess 构造和借用函数使用准确字段类型。

### 15.3 完整注册表阶段

聚合验证：

- 重复 TypeIdentity；
- 重复 Entity `ModelId`；
- fixed Projection source 缺失或角色错误；
- orphan fixed Projection；
- projector 重名、input/output/source 不一致；
- reference 目标缺失或不是 Entity；
- reference key 不存在或类型不兼容；
- Projection selector 不存在或输出类型不兼容；
- whole Entity/identifier selection 类型不兼容；
- 可在当前类型内解析的 `same_as` 子路径不存在、穿过集合或终点 Entity 不一致；
- `same_as` 的 `..`/根路径语法非法；其 occurrence 相关的完整路径由 GenerationPlanner 验证；
- Value 组合中出现禁止角色；
- Entity persistence path 无法解析或终点形状不合法；
- Entity 显式单路径 index 与 reference 派生索引路径重复。

错误对象必须同时包含源 Rust 类型、角色、字段/路径、目标 `ModelId`、选择/binding 和声明源位置。

## 16. 测试策略

### 16.1 `rs-model-derive` 单元与 compile tests

- 五种入口的 parser、RolePolicy、IR 和 normalize 单元测试；
- 每种角色允许/禁止属性矩阵；
- identifier 恰好一个与 `Id` 类型断言；
- Entity `model_id` 与其他角色无 ID；
- Value named/newtype、transparent、textual 和非法组合；
- Enum 三种 variant、规范名和 payload 约束；
- reference 四种 selection、三种 binding 和全部冲突；
- reference 自动派生 index、同字段 `indexed` 重复诊断；
- Entity unique/index/reference key/projector；
- unknown 属性、camelCase 旧参数和全部旧语法 compile-fail；
- renamed `qubit-model-metadata` dependency fixture。

trybuild stderr 只断言稳定、面向用户的诊断，不暴露内部 IR 名称。

### 16.2 `rs-model-metadata` runtime tests

- `TypeMetadata` 不含公共 `ModelId`；
- 五种 `TypeRoleMetadata` 构造合法性；
- TypeDescriptor/TypeAccess 的借用、构造和错误 downcast；
- Value descriptor 静态递归，不经 registry；
- 注册表 TypeIdentity 主索引和 Entity ModelId 次级索引；
- 四类注册、Value 不注册；
- projector 查找、执行和 identifier 一致性；
- reference key 提取；
- 完整关系与 persistence path 校验；
- Entity 根下嵌套 Model relation 的 occurrence index 派生、显式单路径重复诊断与复合 index 共存；
- 确定性聚合错误顺序。

### 16.3 跨 crate fixtures

至少建立以下独立 crate：

- `core-projection`：定义通用 Info Projection；
- `entity-a`：定义 Entity 与 projector；
- `entity-b`：仅通过 `ModelId` 字符串引用 entity-a，不依赖其 Rust crate；
- `model-root`：无 ModelId 的 Model 声明关系；
- `value-lib`：未注册 Value 被多层嵌套；
- `collector`：链接完整集合并验证 registry。

覆盖重复 Entity ID、缺失目标、缺失 projector、错误固定 source、错误 key 类型，以及 reference 源 crate 不直接
依赖目标 Entity crate 的情况。

### 16.4 消费者契约测试

在实现 validator/generator 前先用小型测试消费者验证 descriptor 是否足够：

- `Model -> Value -> Value -> String` 约束错误路径；
- Option、Vec、Map、数组、Box 的递归校验；
- 构造无 Default/Deserialize 的 Model；
- preexisting 先持久化目标；
- inline 不持久化目标但满足其内部依赖；
- same_as 复用实际 Entity，并允许两处使用不同 selection；
- `app/tenant` 沿隐藏 Entity provenance 而不是 Projection 值遍历；
- `..` 在父 Entity 未完成时复用其 identifier、reference key 或 ReadyFields Projection；
- 请求完整父 Entity 或依赖子字段的父 Projection 时报告字段赋值环；
- required cycle、optional cycle、same_as cycle；
- Projection identifier 不一致；
- scoped unique 的生成顺序和冲突重试。

## 17. 破坏性迁移顺序

本项目不提供兼容层，但实施仍应按依赖方向分阶段，保证每一步有明确验收面。

### 阶段 1：重构 `rs-model-metadata` 核心

1. 引入 `TypeRoleMetadata`、`TypeDescriptor`、`TypeAccess`；
2. 从公共 `TypeMetadata` 删除 `ModelId`；
3. 重构 NamedTypeRef/TypeRef 为惰性 descriptor resolver；
4. 增加 Box 和容器值访问协议；
5. 重构注册项和 TypeIdentity/Entity ModelId 双索引；
6. 删除旧 attribute bag 中的角色专属元数据。

### 阶段 2：重写五个过程宏

1. 建立共享结构 parser 和 RoleIr；
2. 实现 Entity、Projection、Model、Enum、Value policy；
3. 生成 descriptor、access、construct 和角色 trait；
4. 只为四种角色生成注册项；
5. 实现新的 unique/index/reference key/projector；
6. 删除自动 derives、Serde/redact 处理和 `no_*`。

### 阶段 3：实现新关系协议

1. 加入 `EntitySelection` 与 `ReferenceBinding`；
2. 实现 `select`/`bind` 新语法；
3. 实现 projector factory 和注册表校验；
4. 实现 Entity provenance path resolver；
5. 实现 reference occurrence 的 effective index 派生和显式单路径重复校验；
6. 删除 `property`、`existing`、旧 `path` 和 `info` 特判。

### 阶段 4：迁移平台模型

1. 将有独立表和 identifier 的旧 Model 分类为 Entity；
2. 将 Info、PersonInfo 等分类为 Projection；
3. 将 DTO、配置、组合结果分类为 Model；
4. 将领域 newtype/纯值 struct 分类为 Value；
5. Enum 删除 ID；
6. 仅为 Entity 保留或重新审定稳定 `ModelId`；
7. 把 Projection/Model 的 index/unique 移到实际 Entity 路径或业务策略；
8. 为每个实际命名投影注册 projector；
9. 把所有 reference 改为显式 select/bind；
10. 为 username/code 等真实外部键增加 reference key；
11. 用原生 derive/Serde/redact 属性补齐需要的 Rust 行为。

### 阶段 5：迁移完整平台校验与清单

- 迁移清单只把 Entity `ModelId` 视为稳定持久化协议；
- Projection、Model、Enum 报告按角色和 TypeIdentity/Rust 来源展示，不再伪造迁移 ID；
- 完整校验使用 `validate_complete`；
- 删除按所有注册项 `ModelId` 覆盖率计算的旧逻辑。

### 阶段 6：接入 validator 与 generator

- 先实现 descriptor 驱动的递归 validator；
- 再实现普通字段 generator/constructor；
- 最后实现 repository、reference binding、provenance 和依赖规划；
- 每个阶段使用第 16.4 节消费者契约测试验收。

## 18. 明确删除的旧公共行为

以下行为不会 deprecated，而是直接删除：

- `#[Model(id = "...")]`；
- `#[Enum(id = "...")]`；
- 用 `#[Model] + #[identifier]` 猜测 Entity 或 Projection；
- `TypeMetadata::id()` 对所有类型可用；
- 注册表按任意模型 `ModelId` 查询；
- `#[identifier(generated)]`；
- `primary_key(...)`；
- `key(...)`；
- `ownership(owner = Type)`；
- `lookup_relation(...)`；
- 裸 `#[reference]`；
- `reference(property = ...)`；
- `reference(existing = ...)`；
- 旧 `reference(path = ...)`；
- `info` 选择器特殊跳过；
- `nullable` 和 `computed`；
- camelCase 参数别名；
- 自动 Clone/Debug/Display/Eq/Hash/Serde/redact；
- 所有 `no_*` 开关；
- 用 Serde JSON 充当通用运行时反射或构造协议。

## 19. 完整验收标准

### 19.1 五种角色

1. 五种宏的形状、注册、ModelId、identifier 和关系权限与基础语义文档完全一致。
2. 只有 Entity 可以声明 `model_id`，且完整注册表按它唯一索引 Entity。
3. Projection、Model、Enum 无 ModelId 仍能参与全局注册。
4. Value 有完整 descriptor，但不出现在任何全局注册或 Entity 清单中。
5. 非法角色状态和字段组合无法通过公共构造器静默产生；Entity/Projection 叶子若没有 reference 也会失败。

### 19.2 元数据与访问

6. 任意已知类型可通过 `HasTypeDescriptor` 查询结构和角色。
7. NamedTypeRef 可惰性解析嵌套 Value，不需要 registry。
8. FieldAccess 能借用私有字段值并检查 downcast。
9. Construct adapter 能构造无 Default/Deserialize 的 struct、newtype 和 Enum payload。
10. Option、序列、Set、Map、数组和 Box 的结构与值访问一致。

### 19.3 Identifier 与持久化

11. Entity/Projection 缺少、重复或错误类型 identifier 时编译失败。
12. Projection identifier 不产生主键或独立持久化元数据。
13. Entity identifier 自动成为唯一主键身份，不存在第二套 primary key DSL。
14. unique/index 可引用合法 Entity 字段路径，其他角色声明时编译失败。
15. 每个 direct reference 自动进入所属 descriptor 的逻辑 effective index；Entity 根下每个可达 Model relation
    occurrence 还进入该 Entity 的持久化 effective index。同一路径的显式单字段 index 报错，包含该路径的复合
    index 可以共存。
16. reference key 必须显式、全局唯一、非空且类型可提取。

### 19.4 Projection 与 reference

17. 每个 projector 有可执行 adapter 和明确的 CompleteEntity/ReadyFields 就绪条件，并按
    `(Entity ModelId, selector)` 唯一解析。
18. 通用 Projection 可由多个 Entity 输出，fixed Projection 只能由声明来源输出。
19. `info` 与任意其他 selector 使用完全相同的验证和执行路径。
20. Entity、identifier、key、Projection 四种 selection 均验证 leaf 类型。
21. preexisting、inline、same_as 三种 binding 的生成语义互斥且无隐式默认。
22. inline + identifier/key 在 schema 阶段失败。
23. same_as 复用实际 Entity provenance，可以从同一 Entity 产生不同 selection。
24. 父路径、根路径、嵌套 Entity 路径能正确解析；Optional 缺失按目标可空性处理；集合穿越和越界确定失败。
25. 不存在 Java 式裸间接 reference 标记。

### 19.5 校验与生成

26. 嵌套 Value 约束自动校验并返回完整路径。
27. Typed Value 校验不依赖全局注册。
28. 无类型字符串不能动态发现未注册 Value。
29. generator 可以根据 descriptor 构造 Entity、Model、Value 和 Enum variant；Projection 只有在显式来源
    Entity 与 selector 上下文中通过 projector 产生。
30. preexisting 目标先于来源持久化；inline 目标不独立持久化但满足内部依赖。
31. 对象 occurrence 使用安全 field slots 分阶段构造；父 identifier 和 ReadyFields Projection 不要求完整父值。
32. same_as 字段按 provenance 与 selection 就绪条件拓扑排序。
33. required Entity cycle、字段赋值 cycle 与 same_as cycle 分别报告；Optional/空集合可由策略切断。
34. projector 输出 identifier 不一致时生成失败。
35. opaque 缺少 generator 时明确失败，不生成伪零值。
36. projector 输出被视为完整 Projection，生成器不会覆盖字段，也不会从投影保存值猜测内部 Entity provenance。

### 19.6 诊断与工程质量

37. proc macro 对用户错误不 panic，并尽量聚合独立诊断。
38. 完整注册表错误顺序稳定，包含角色、源类型、字段路径和目标 Entity。
39. 五种角色属性矩阵、跨 crate 注册、Value 递归、projector 和三种 binding 均有自动测试。
40. 旧语法有明确 compile-fail 覆盖，不存在静默兼容分支。
41. `rs-model-derive` 不再要求消费方仅为了宏默认行为而依赖 Serde 或 qubit-redact。
42. validator 和 generator 的小型契约实现能够只依赖 descriptor/registry 完成第 16.4 节场景，不再解析 Rust
    类型名或硬编码 `info`。

## 20. 审核时应重点确认的设计决策

本文没有保留待实现时再决定的语义空白。审核时最值得重点确认的是以下已作出的选择：

1. 公共 `TypeMetadata` 完全移除 `ModelId`，Entity payload 单独持有它；
2. Model/Projection/Enum 只按进程内 `TypeIdentity` 注册；
3. Model 禁止 persistence，但允许关系；Value 进一步禁止注册、关系以及嵌套身份类型；
4. Entity 必须实现 Clone，以支持完整 Entity selection 与来源保存；
5. 标准 Rust derives、Serde、Display 和 redaction 全部从角色宏移除；
6. identifier 固定为单个 `Id`，删除复合主键和 generated 选项；
7. 普通业务字段只有显式 `reference_key` 才能成为引用选择；
8. reference 的 select/bind 均强制显式，不提供默认值；
9. inline 禁止只保存 identifier/key；
10. `same_as` 指向隐藏 Entity provenance，失败时绝不回退为新建对象；
11. projector 在 Entity 宏中声明，并显式选择 CompleteEntity 或 ReadyFields 输入，不允许未登记的方法名选择；
12. 删除旧 ownership、key 和 lookup relation，等待真实需求后单独设计。

这些选择一旦批准，应直接作为实施计划与测试矩阵的输入，不在编码阶段重新解释。
