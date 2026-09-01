# qubit-model-derive 用户指南

[README](../README.zh_CN.md) | [English user guide](user_guide.md) | [最终设计](rs-model-derive-final-design.zh_CN.md)

## 手册目标与读者

本指南面向使用 `qubit-model-derive` 0.1.x 的领域模型作者和框架开发者，说明如何声明六个公开宏、查询生成的元数据、解析跨模型关系，并理解对应诊断。当前 crate 使用 Rust 1.94 和 edition 2024。

## 概念模型

每个角色宏都会委托 `qubit-reflect` 生成 Rust 描述符，再将 `TypeMetadata` 作为类型化能力
附加到同一个描述符上。字段（field）是真实存储槽位；属性（property）是由字段、getter、setter 按名称合并而成的视图。

```text
角色声明 -> Reflect descriptor -> model metadata capability
                                -> 可选 ModelRegistration
ModelImpl impl ----------> property capability
```

静态查询不依赖全局注册表。只有稳定 ID、reference、Projection 来源和 Query 需要在完整链接的模型集合中解析。

## 贯穿场景：用户实体与登录请求

设想登录服务需要脱敏的邮箱值、可持久化的用户记录，以及指向该用户的登录请求。宏 crate 当前未发布到 crates.io（`publish = false`）。在同时检出两个仓库的环境中，应使用路径依赖；下面的路径仅为示例，需按工作区目录调整：

```toml
[dependencies]
qubit-model-derive = { path = "../rs-model-derive" }
qubit-model-metadata = { path = "../rs-model-metadata" }
```

下面的声明包含透明值对象、实体、带引用的模型，以及字段支持和计算属性：

```rust,ignore
use qubit_model_derive::{Entity, Model, ModelImpl, Value};

#[Value(transparent)]
pub struct Email(
    #[redact(level = "medium")]
    String,
);

#[Entity(id = "example.User")]
pub struct User {
    #[identifier(assigned_by = application)]
    id: u64,
    #[unique(ignore_case = true)]
    #[redact(nested)]
    email: Email,
    #[serde(default)]
    aliases: Vec<String>,
}

#[Model(id = "example.Login")]
pub struct Login {
    #[reference(entity_id = "example.User", property = id)]
    user_id: u64,
}

#[ModelImpl]
impl User {
    pub fn email(&self) -> &Email { &self.email }
    pub fn set_email(&mut self, value: Email) { self.email = value; }
    pub fn alias_slice(&self) -> &[String] { &self.aliases }
}
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
use qubit_model_metadata::{ModelRegistry, ModelResolver, ResolveInputs, TypeMetadata};
use qubit_validator::ValidatorRegistry;

let models = ModelRegistry::try_global()?;
let validators = ValidatorRegistry::try_global()?;
let codecs = ValueCodecRegistry::try_global()?;
let graph = ModelResolver::new(ResolveInputs { models, validators, codecs }).resolve_all()?;
let field = TypeMetadata::of::<Login>().field("user_id").unwrap();
assert_eq!(
    graph.reference(field).unwrap().target().model_id().unwrap().as_str(),
    "example.User",
);
# Ok::<(), Box<dyn std::error::Error>>(())
```

## 角色与泛型规则

- `Entity`、`Projection` 只接受具名 struct，均要求一个 `#[identifier]`，且不支持泛型。
- 未指定来源的 `Projection` 是开放投影；`source = EntityType` 与 `source_id = "..."` 用于固定来源，二者互斥。
- `Model` 接受具名或 unit struct，不接受 tuple struct。
- `Enum` 只接受 enum，并保存每个 variant 的 Rust 名、canonical 名和 Serde 名。
- `Value` 接受具名字段 struct 或单字段 tuple struct。`transparent` 要求恰好一个字段，并使 `Serialize`、`Deserialize`、`Display` 使用内部表示。
- `Model`、`Enum`、`Value` 支持 type parameter、where clause 和反射支持的 primitive const generic，不支持 lifetime parameter。
- 带 ID 的泛型声明只注册 definition；具体单态类型没有 `ModelId`。可通过 `generic_definition()` 查看模板和其中的 symbolic 字段。

## 字段声明

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

selector 上的脱敏只接受 `redact(level = "...")`。`element` 和 `map_value` 应用于容器 value；`map_key`
处理输出 key。若两个原始 key 脱敏后相同，Serde 序列化会返回错误，不会静默覆盖 value。

`validator` 生成经过语法校验的 occurrence metadata；`ModelResolver` 会将其 ID 绑定到可执行的
`qubit-validator` descriptor，校验 value type，并解析 dependency Property。Rust codec 会在编译期校验
`Default + ValueEncoder<Value, Output = String> + ValueDecoder<str, Output = Value>`。

## 默认接口、Serde 与脱敏

五种角色默认实现 `Clone`、`PartialEq`、`Eq`、`Hash`、`Redact`、`Debug`、`Display`、`Serialize`、`Deserialize`。输出接口经由 `qubit-redact` 以 fail-closed 方式处理；`Deserialize` 只处理输入。

可用 `no_clone`、`no_debug`、`no_display`、`no_partial_eq`、`no_eq`、`no_hash`、`no_redact`、`no_serialize`、`no_deserialize` 逐项关闭。`no_redact` 仅允许没有 field 或 selector 脱敏规则的类型；保留的输出接口会使用普通实现。`copy`、`default`、`partial_ord`、`ord` 需要显式开启；全 unit Enum 默认 `Copy`，但可用 `no_copy` 关闭。

角色 attribute 应位于用户 `#[derive(...)]` 之前。启用脱敏时，宏会拒绝可能绕过安全输出的已有 `Debug` 或 `Serialize` 实现；使用 `no_redact` 时，会复用兼容的已有实现而非重复派生。

具名 `Option` 和标准集合字段默认具有 Serde 行为：反序列化时可省略，序列化时会省略空值。`#[keep_serializing]` 可保留空值输出；显式 Serde 配置优先。

## ModelImpl

`#[ModelImpl]` 只接受 public、safe、sync、非泛型的 inherent impl。getter 使用 `&self`，返回值可为 `T`、`&T`、`&str`、`&[T]` 或 `Option<&T>`；setter 使用 `&mut self, T` 并返回 `()`。生成的 getter 会保留借用关系；setter 在调用用户代码前失败时会归还 replacement。重复 Property 或重复标记的 impl 会在编译期失败。

字段、getter、setter 按名称合并：

- 没有 accessor 的 field 是 `FieldBacked`；
- 没有 field 的 getter 是 `Computed`；
- 没有 field 的 setter 是 `Virtual`；
- 同名 accessor 会补充同一个 Property，不会产生重复项。

## 错误与诊断

- 宏会一次聚合彼此独立的声明 shape、option、字段 singleton、key order 错误。
- 找不到 runtime facade 时，请添加 `qubit-model-metadata`；允许依赖重命名。
- 跨模型 ID、角色、来源、reference 与 Property 只能在完整链接集合中判断，因此由 `ModelResolver` 报告。
- `TypeRef::Opaque` 与 `TypeRef::Symbolic` 没有 concrete descriptor。请检查 `type_ref()`，不要把 `descriptor() == None` 当作 metadata 缺失。
- codec bound 失败时，请实现准确的 `ValueEncoder` 与 `ValueDecoder` 契约；不要额外引入不存在的 codec contract ID 类型。

## 排障

| 症状 | 优先检查 |
| --- | --- |
| 角色宏报告已有 derive 冲突 | 把角色 attribute 放在 `#[derive(...)]` 前；删除不安全的重复实现，或使用允许的 `no_*` 参数。 |
| reference 无法解析 | 确认应用链接了全部模型 crate，再以 `ModelRegistry::try_global()?` 的结果运行 `ModelResolver`。 |
| Property 缺失或被拒绝 | 检查方法是否为 public、safe、sync、非泛型 inherent getter/setter，且 receiver、返回类型或参数类型受支持。 |
| collection 字段未出现在 JSON 中 | 检查默认省略行为、`#[keep_serializing]` 与显式 Serde 属性。 |
| 脱敏后 map 序列化失败 | 这是数据模型问题；应使用仍能区分 key 的脱敏策略，或不要脱敏 map key。 |

## 限制与最佳实践

不要为普通静态查询初始化 `ModelRegistry`，也不要以 Rust `type_name()` 充当稳定 `ModelId`。跨模型检查应在全部模型链接完成后集中执行一次。validator 执行协议不属于当前 API；目前声明可用于 schema、文档和未来 resolver 输入。

## 延伸阅读

- [README](../README.zh_CN.md)
- [English user guide](user_guide.md)
- 本地 API 文档：在 crate 根目录运行 `cargo doc --open`
- [最终设计](rs-model-derive-final-design.zh_CN.md)
