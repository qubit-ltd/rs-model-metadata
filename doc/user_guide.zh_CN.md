# qubit-model-derive 用户指南

[README](../README.zh_CN.md) | [English user guide](user_guide.md) | [最终设计](rs-model-derive-final-design.zh_CN.md)

## 手册目标与读者

本指南面向使用 `qubit-model-derive` 0.1.0 的领域模型作者和框架开发者，说明如何声明六个公开宏、查询生成的 metadata、解析跨模型关系，并理解对应诊断。当前 crate 使用 Rust 1.94 和 edition 2024。

## 概念模型

每个角色宏都会委托 `qubit-reflect` 生成 Rust 描述符，再将 `TypeMetadata` 作为类型化能力
附加到同一个描述符上。字段（field）是真实存储槽位；属性（property）是由字段、getter、setter 按名称合并而成的视图。

```text
角色声明 -> Reflect descriptor -> model metadata capability
                                -> 冻结注册表投影
泛型角色声明 ------------------> generic ModelRegistration
ModelImpl impl ----------> property capability
```

直接调用 `TypeMetadata::of` 不依赖全局模型注册表；descriptor capability 与 Property 查询会冻结反射快照，
确保独立 fragment 可见。生成 facade 为 model ABI v3，并只使用反射层 `codegen_v2`。稳定 ID、reference、
Projection 来源和 Query 则需要在完整链接的模型集合中解析。

## 安装与最小配置

宏 crate 当前未发布到 crates.io（`publish = false`）。请在同时检出两个仓库的环境中使用路径依赖，并按
实际工作区布局调整路径：

```toml
[dependencies]
qubit-model-derive = { version = "0.1", path = "../rs-model-derive" }
qubit-model-metadata = { version = "0.1", path = "../rs-model-metadata" }
qubit-id = { version = "0.6", path = "../../rust-common/rs-id" }
qubit-codec = { version = "0.14", features = ["registry"] }
serde_json = "1"
```

后文的模型图解析示例会使用 `ValueCodecRegistry`，因此业务 crate 必须直接依赖 `qubit-codec` 并启用
`registry` feature。生成代码可以自动识别重命名后的 `qubit-model-metadata` 依赖，业务 crate 无需直接
依赖 `qubit-reflect`。

## 贯穿场景：用户实体与登录请求

设想一个登录服务：邮箱写入日志时必须脱敏，用户记录需要稳定身份，登录请求还要引用对应用户。完成后应能
通过静态 metadata 找到 identifier 和可写邮箱属性，序列化输出不泄露原始邮箱，并能在所有业务 crate
完成链接后把登录请求中的 reference 解析到 `example.User`。

下面的声明包含透明值对象、实体、固定来源 Projection、带引用的模型，以及字段支持和计算属性：

```rust
use qubit_id::Id;
use qubit_model_derive::{Entity, Model, ModelImpl, Projection, Value};

#[Value(transparent)]
pub struct Email(
    #[redact(level = "medium")]
    String,
);

#[Entity(id = "example.User")]
pub struct User {
    #[identifier(assigned_by = application)]
    id: Id,
    #[unique(ignore_case = false)]
    #[redact(nested)]
    email: Email,
    #[serde(default)]
    aliases: Vec<String>,
}

#[Projection(id = "example.UserView", source = User)]
pub struct UserView {
    #[identifier]
    id: Id,
    #[redact(nested)]
    email: Email,
}

#[Model(id = "example.Login")]
pub struct Login {
    #[reference(entity_id = "example.User", property = id)]
    user_id: Id,
}

#[ModelImpl]
impl User {
    pub fn email(&self) -> &Email { &self.email }
    pub fn set_email(&mut self, value: Email) { self.email = value; }
    pub fn alias_slice(&self) -> &[String] { &self.aliases }
    pub fn view(&self) -> UserView {
        UserView { id: self.id, email: self.email.clone() }
    }
}
```

生成的序列化实现会执行 `nested` 脱敏规则，因此可观察到的 JSON 不会包含原始邮箱：

```rust,ignore
let user = User {
    id: Id::new(7),
    email: Email("alice@example.com".to_owned()),
    aliases: Vec::new(),
};
let json = serde_json::to_string(&user)?;
assert!(!json.contains("alice@example.com"));
```

## 核心工作流

### 查询静态元数据

类型声明完成后即可查询，这条路径不会初始化 `ModelRegistry`：

```rust,ignore
use qubit_model_metadata::{ModelDescriptorExt, TypeMetadata};

let user = TypeMetadata::of::<User>();
assert!(user.field("id").unwrap().is_identifier());
assert!(user.try_property("email").unwrap().unwrap().is_writable());
assert!(user.try_property("alias_slice").unwrap().unwrap().is_computed());
assert!(user.descriptor().model_metadata().is_some());
```

`TypeMetadata` 只是唯一 `TypeDescriptor` 的领域 overlay，不会建立第二棵结构图。若字段类型在泛型定义中是 opaque 或 symbolic，`descriptor()` 会返回 `None`；此时请用 `type_ref()` 检查实际的 `Resolved`、`Opaque` 或 `Symbolic` 形式。

### 解析完整模型图

所有业务 crate 链接完成后，再解析依赖其他模型的声明。缺失 target、错误角色、未知引用 Property 等问题会在这一步报告：

```rust,ignore
use qubit_codec::ValueCodecRegistry;
use qubit_model_metadata::{
    ModelRegistry, ModelResolver, PropertyPath, ResolveInputs, TypeMetadata,
};
use qubit_validator::ValidatorRegistry;

fn inspect_graph() -> Result<(), Box<dyn std::error::Error>> {
    let models = ModelRegistry::try_global()?;
    let validators = ValidatorRegistry::try_global()?;
    let codecs = ValueCodecRegistry::try_global()?;
    let graph = ModelResolver::new(ResolveInputs { models, validators, codecs })
        .resolve_all()?;

    let field = TypeMetadata::of::<Login>().field("user_id").unwrap();
    assert_eq!(
        graph.reference(field).unwrap().target().model_id().unwrap().as_str(),
        "example.User",
    );

    let user = TypeMetadata::of::<User>().as_entity().unwrap();
    let query = graph.query(user).unwrap();
    assert!(query.unique_keys().iter().any(|key| {
        key.path().is_some_and(|path| path == PropertyPath::new(&["email"]))
    }));
    assert_eq!(graph.projection_producers().len(), 1);
    Ok(())
}
```

解析成功后得到的模型图同时保存 Entity 查询视图和 Entity 到 Projection 的 producer edge。本地
`FieldMetadata` 只表达声明事实，不会提前猜测这些跨模型结果；任何注册项校验失败时也不会发布部分模型图。

## 进阶用法

### 角色与泛型规则

- `Entity`、`Projection` 只接受具名 struct，均要求一个 `#[identifier]`，且不支持泛型。
- 未指定来源的 `Projection` 是开放投影；`source = EntityType` 与 `source_id = "..."` 用于固定来源，二者互斥。
- `Model` 接受具名或 unit struct，不接受 tuple struct。
- `Enum` 只接受 enum，并保存每个 variant 的 Rust 名、canonical 名和 Serde 名。
- `Value` 接受具名字段 struct 或单字段 tuple struct。`transparent` 要求恰好一个字段，并使 `Serialize`、`Deserialize`、`Display` 使用内部表示。
- `Model`、`Enum`、`Value` 支持 type parameter、where clause 和反射支持的 primitive const generic，不支持 lifetime parameter。
- 带 ID 的泛型声明只注册 definition；具体单态类型没有 `ModelId`。可通过 `generic_definition()` 查看模板和其中的 symbolic 字段。

### 字段声明

常用声明包括：

- `#[identifier(assigned_by = application|database)]`、`#[indexed]`；
- `#[unique(respect_to(tenant_id), ignore_case = true)]`；
- `#[reference(entity = User, property = id)]` 或 `#[reference(entity_id = "example.User", property = id)]`；
- `#[key_part(order = 0)]`；
- `#[text(...)]`、`#[decimal(...)]`、`#[money(...)]`、`#[time(...)]`、`#[sequence(...)]`、`#[map(...)]`；
- 非递归的 `#[element(...)]`、`#[map_key(...)]`、`#[map_value(...)]` selector；
- `#[codec(MyCodec)]` 或 `#[codec(id = "example.codec")]`；
- `#[redact(level = "medium")]`、`skip`、`nested`、`map`、`keyed_by`、`json`；
- `#[validator(id = "example.rule", params(...), depends_on(...))]`。

identifier 的默认分配方是 `application`，只有 Entity 可以选择 `database`。`#[indexed]` 不接收参数，
表达的是逻辑过滤能力。`#[unique]` 默认忽略大小写，因此该默认值只适用于文本字段；
`respect_to(...)` 按源码顺序记录 scope 路径。reference 必须在 `entity = RustType` 与
`entity_id = "..."` 中选择一个；`property` 指定保存的目标 Property，`existing` 默认为 `true`，
`path` 用于绑定同一对象图中的另一处 reference。

#### 约束参数速查

- `text` 支持 `min_chars`、`max_chars`、`min_bytes`、`max_bytes`、`non_blank`、
  `allowed_chars` 和 `format`。字符集可选 `unicode`、`printable_unicode`、`ascii`、
  `printable_ascii`、`code`；格式可选 `email`、`cn_mobile`、`uri`、`uuid`。
- `decimal` 与 `money` 支持 `precision`、`scale`、字符串形式的 `min` / `max`、
  `min_inclusive`、`max_inclusive` 和 `rounding`。`money` 必须指定 `scale`；`decimal` 默认
  `half_even`，`money` 默认 `unnecessary`。
- `time` 必须指定
  `precision = second|millisecond|microsecond|nanosecond`。
- `sequence` 支持 `min_items`、`max_items`、`unique_items`；`map` 支持 `min_entries` 与
  `max_entries`。每个约束至少要有一个参数，且最小值不能大于最大值。

宏会在编译期检查约束是否适用于字段类型。标准叶子约束不会自动下沉到容器成员；应显式使用
`element`、`map_key` 或 `map_value`。selector 可以组合叶子约束、validator、一个 codec 和
`redact(level = "...")`，但不能继续嵌套 selector。

`key_part` 表达值语义上的逻辑键，而不是持久化身份。它只允许标在具名 `Model` 或具名 `Value`
的真实存储字段上。逻辑键可以选择字段子集，但所选 order 必须从零开始、连续且不重复。例如，一个
带命名空间的编码可把 `namespace` 与 `code` 作为键，而不把展示用 `label` 纳入键：

```rust
use qubit_model_derive::Value;

#[Value]
struct LocalizedCode {
    #[key_part(order = 0)]
    namespace: String,
    #[key_part(order = 1)]
    code: String,
    label: String,
}
```

Entity/Projection 的身份应使用 `#[identifier]`。`key_part` 不能用于这两种角色，也不能用于 `Enum`
或 tuple/newtype Value，因为这些形状不表示对多个具名存储字段的有序选择。

selector 上的脱敏只接受 `redact(level = "...")`。`element` 和 `map_value` 应用于容器 value；`map_key`
处理输出 key。若两个原始 key 脱敏后相同，Serde 序列化会返回错误，不会静默覆盖 value。

`#[opaque]` 会保留字段的外层 shape，但在叶子处终止 descriptor 遍历，适合无法实现反射、且由调用方
提供的外部类型。它不能与 `#[reference]` 同时使用，也不能用来隐藏已经注册的 Entity、Projection 或
Model 以绕过 resolver 检查。

`validator` 生成经过语法校验的 occurrence metadata；`ModelResolver` 会将其 ID 绑定到可执行的
`qubit-validator` descriptor，校验 value type，并解析 dependency Property。Rust codec 会在编译期校验
`Default + ValueEncoder<Value, Output = String> + ValueDecoder<str, Output = Value>`。

validator 参数只接受 bool、整数、字符串，以及元素类型一致且非空的字面量数组；同一字段上的多个
validator 会保留源码顺序。codec 可以通过 Rust 类型或稳定 ID 选择；只有 Value 可以通过
`#[Value(codec = CodecType)]` 声明 whole-value canonical codec。

### 默认接口、Serde 与脱敏

五种角色默认实现 `Clone`、`PartialEq`、`Eq`、`Hash`、`Redact`、`Debug`、`Display`、`Serialize`、`Deserialize`。输出接口经由 `qubit-redact` 以 fail-closed 方式处理；`Deserialize` 只处理输入。

可用 `no_clone`、`no_debug`、`no_display`、`no_partial_eq`、`no_eq`、`no_hash`、`no_redact`、`no_serialize`、`no_deserialize` 逐项关闭。`no_redact` 仅允许没有 field 或 selector 脱敏规则的类型；保留的输出接口会使用普通实现。`copy`、`default`、`partial_ord`、`ord` 需要显式开启；全 unit Enum 默认 `Copy`，但可用 `no_copy` 关闭。

角色 attribute 应位于用户 `#[derive(...)]` 之前。启用脱敏时，宏会拒绝可能绕过安全输出的已有 `Debug` 或 `Serialize` 实现；使用 `no_redact` 时，会复用兼容的已有实现而非重复派生。

具名 `Option` 和标准集合字段默认具有 Serde 行为：反序列化时可省略，序列化时会省略空值。`#[keep_serializing]` 可保留空值输出；显式 Serde 配置优先。

### ModelImpl

`#[ModelImpl]` 标记 inherent impl，并把其中 public、safe、sync、非泛型的方法转换为 Property fragment。
getter 使用 `&self`，返回值可为 `T`、`&T`、`&str`、`&[T]` 或 `Option<&T>`；metadata 会准确
区分 owned、borrowed、optional borrowed 与 borrowed slice。setter 使用 `&mut self, T` 并返回
`()`；若在调用用户代码前失败，错误会归还 replacement。重复 Property 或对同一类型重复标记 impl 都会
在编译期失败。

字段、getter、setter 按名称合并：

- 没有 accessor 的 field 是 `FieldBacked`；
- 没有 field 的 getter 是 `Computed`；
- 没有 field 的 setter 是 `Virtual`；
- 同名 accessor 会补充同一个 Property，不会产生重复项。

### 查询视图与自动 Projection

模型图解析成功后，可通过 `ResolvedModelGraph::query` 取得 Entity 的查询事实。identifier 和全局 unique
路径属于唯一查找键；scoped unique 与显式 indexed 路径可成为 filter。普通值对象可以递归展开，reference
最多展开一跳，平面查询名默认用 `_` 连接。若两个完整 `PropertyPath` 得到相同平面名，resolver 会报错，
不会任意选择其中一个。这些信息描述逻辑查询能力，不等同于物理数据库索引。

当 Entity 的可读 Property 返回固定来源的 Projection 时，resolver 会建立
`ResolvedProjectionProducer`。本场景中的 `User::view` 就会产生 `UserView`。调用 `project` 后，运行时
还会核对 Projection 与 Entity 的 `qubit_id::Id` 是否完全一致；若 getter 改变了 ID，则返回
`ProjectionExecutionError::IdentifierMismatch`。没有可执行 getter 时，Projection 仍可由 DAO 或反序列化
流程构造，但自动投影会返回 `MissingProjector`。

## 错误与诊断

- 宏会一次聚合彼此独立的声明 shape、option、字段 singleton、key order 错误。
- 找不到 runtime facade 时，请添加 `qubit-model-metadata`；允许依赖重命名。
- 跨模型 ID、角色、来源、reference 与 Property 只能在完整链接集合中判断，因此由 `ModelResolver` 报告。
- `ModelRegistry::try_global()`、`ValidatorRegistry::try_global()` 和
  `ValueCodecRegistry::try_global()` 会返回初始化错误；对应的 `global()` 快捷方法会在缓存的注册表无效时
  panic，因此应用启动阶段应优先使用可失败接口。
- `resolve_all()` 失败时返回顺序确定的 `ModelResolveErrors`，不会发布部分成功的图。排查时应逐项查看错误
  kind、model ID、Property 路径、期望/实际角色或类型，以及来源 identity。
- `TypeRef::Opaque` 与 `TypeRef::Symbolic` 没有 concrete descriptor。请检查 `type_ref()`，不要把 `descriptor() == None` 当作 metadata 缺失。
- codec bound 失败时，请实现准确的 `ValueEncoder` 与 `ValueDecoder` 契约；不要额外引入不存在的 codec contract ID 类型。

## 排障

| 症状 | 优先检查 |
| --- | --- |
| 角色宏报告已有 derive 冲突 | 把角色 attribute 放在 `#[derive(...)]` 前；删除不安全的重复实现，或使用允许的 `no_*` 参数。 |
| reference 无法解析 | 确认应用链接了全部模型 crate，再以 `ModelRegistry::try_global()?` 的结果运行 `ModelResolver`。 |
| 初始化全局注册表时 panic | 临时改用 `try_global()`，查看缓存的重复 ID、registration、validator 或 codec 错误。 |
| Property 缺失或被拒绝 | 检查方法是否为 public、safe、sync、非泛型 inherent getter/setter，且 receiver、返回类型或参数类型受支持。 |
| 模型图报告查询名冲突 | 找出展平后得到同一 `_` 连接名称的完整 `PropertyPath`，通过调整模型 Property 名消除歧义，不要添加物理索引参数。 |
| collection 字段未出现在 JSON 中 | 检查默认省略行为、`#[keep_serializing]` 与显式 Serde 属性。 |
| 脱敏后 map 序列化失败 | 这是数据模型问题；应使用仍能区分 key 的脱敏策略，或不要脱敏 map key。 |

## 限制与最佳实践

不要为普通静态查询初始化 `ModelRegistry`，也不要以 Rust `type_name()` 充当稳定 `ModelId`。跨模型检查应在
全部模型链接完成后集中执行一次。resolver 会把 validator occurrence 绑定到可执行 registration，但本 crate
不负责安排应用在何时执行校验。

`#[indexed]`、`#[unique]` 与 `#[key_part]` 只向 schema、查询和诊断消费者提供逻辑事实，不配置数据表、
列名、索引顺序或数据库专属能力。selector 不能递归嵌套 selector；需要复用的深层规则应放进具名
`Value`、`Model` 或 `Enum`。

默认生成的 `Eq` 与 `Hash` 使用全部字段的结构语义。若 Entity 的参与字段可能变化，不要把实体本身放入
`HashSet` 或作为 `HashMap` 的 key，应改用稳定 identifier。

## 延伸阅读

- [README](../README.zh_CN.md)
- [English user guide](user_guide.md)
- 本地 API 文档：在 crate 根目录运行 `cargo doc --open`
- [最终设计](rs-model-derive-final-design.zh_CN.md)
