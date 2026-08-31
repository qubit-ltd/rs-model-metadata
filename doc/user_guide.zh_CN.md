# qubit-model-derive 用户指南

[README](../README.zh_CN.md) | [Final design](2026-08-31-182016-rs-model-derive-final-api-and-implementation-design.md)

## 手册目标与适用版本

本指南面向使用 0.1.x 的领域模型作者和框架开发者，说明如何用六个公开宏声明模型、查询 metadata、
解析跨模型关系，以及理解默认能力和诊断边界。当前仓库使用 Rust 1.94、edition 2024。

## 概念模型

角色宏先生成 `qubit-reflect::TypeDescriptor`，再把 `TypeMetadata` 作为 typed capability 挂到同一个
descriptor 根上。Field 来自真实存储槽位；Property 是 field、getter、setter 的按名合并视图。

```text
角色声明 -> Reflect descriptor -> model metadata capability
                                -> 可选 ModelRegistration
ModelProperties impl ----------> property capability
```

静态查询不依赖全局注册表。只有跨 crate ID、reference、Projection source 和 Query 才需要显式 resolver。

## 贯穿场景：用户实体与登录请求

应用至少依赖宏 crate 和 runtime facade：

```toml
[dependencies]
qubit-model-derive = "0.1"
qubit-model-metadata = "0.1"
```

声明 Entity、Value 和引用 Entity 的 Model：

```rust,ignore
use qubit_model_derive::{Entity, Model, ModelProperties, Value};

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

#[ModelProperties]
impl User {
    pub fn email(&self) -> &Email { &self.email }
    pub fn set_email(&mut self, value: Email) { self.email = value; }
    pub fn alias_slice(&self) -> &[String] { &self.aliases }
}
```

随后可以直接查询静态 metadata：

```rust,ignore
use qubit_model_metadata::TypeMetadata;

let user = TypeMetadata::of::<User>();
assert!(user.field("id").unwrap().is_identifier());
assert!(user.property("email").unwrap().is_writable());
assert!(user.property("alias_slice").unwrap().is_computed());
```

所有业务 crate 链接完成后，再显式解析关系：

```rust,ignore
use qubit_model_metadata::{ModelRegistry, ModelResolver, ResolveInputs, TypeMetadata};

let registry = ModelRegistry::try_global()?;
let graph = ModelResolver::new(ResolveInputs { models: registry }).resolve_all()?;
let field = TypeMetadata::of::<Login>().field("user_id").unwrap();
assert_eq!(
    graph.reference(field).unwrap().target().model_id().unwrap().as_str(),
    "example.User",
);
# Ok::<(), Box<dyn std::error::Error>>(())
```

## 角色与泛型规则

- `Entity`、`Projection` 只接受具名 struct，且必须各有一个 `#[identifier]`；二者不支持泛型。
- `Projection` 省略来源时是开放投影；`source = EntityType` 或 `source_id = "..."` 声明固定来源，二者只能选一个。
- `Model` 接受具名或 unit struct，不接受 tuple struct。
- `Enum` 只接受 enum，并分别保存 Rust 名、canonical 名以及 Serde 双向名称。
- `Value` 接受具名字段或单字段 tuple struct；`transparent` 要求恰好一个字段，并让
  Serialize、Deserialize、Display 使用内部表示。
- `Model`、`Enum`、`Value` 支持 type parameter、where clause 和反射支持的 primitive const generic，
  不支持 lifetime parameter。
- 带 ID 的泛型声明只注册 definition；具体单态类型没有 `ModelId`，但可通过
  `generic_definition()` 导航到包含 symbolic 字段的模板。

## 字段语义

常用声明包括：

- `#[identifier(assigned_by = application|database)]`、`#[indexed]`；
- `#[unique(respect_to(tenant_id), ignore_case = true)]`；
- `#[reference(entity = User, property = id)]` 或
  `#[reference(entity_id = "example.User", property = id)]`；
- `#[key_part(order = 0)]`；
- `#[text(...)]`、`#[decimal(...)]`、`#[money(...)]`、`#[time(...)]`、
  `#[sequence(...)]`、`#[map(...)]`；
- 非递归 `#[element(...)]`、`#[map_key(...)]`、`#[map_value(...)]` selector；
- `#[codec(MyCodec)]` 或 `#[codec(id = "example.codec")]`；
- `#[redact(level = "medium")]`、`skip`、`nested`、`map`、`keyed_by`、`json`；
- `#[validator(id = "example.rule", params(...), depends_on(...))]`。

selector 上的脱敏当前只接受 `redact(level = "...")`。`element` 和 `map_value` 会递归应用到容器值；
`map_key` 会脱敏输出 key，并在两个原始 key 脱敏为同一个 key 时让 Serde 序列化失败，避免静默覆盖数据。

validator 只保留经过语法校验的 occurrence metadata，当前不注册、不解析也不执行。Rust codec 会在编译期检查
`Default + ValueEncoder<Value, Output = String> + ValueDecoder<str, Output = Value>`。

## 默认能力与 Serde / Redact

五种角色默认实现 `Clone`、`PartialEq`、`Eq`、`Hash`、`Redact`、`Debug`、`Display`、
`Serialize` 和 `Deserialize`。其中输出接口通过 `qubit-redact` fail closed；`Deserialize` 只处理输入。

可分别使用 `no_clone`、`no_debug`、`no_display`、`no_partial_eq`、`no_eq`、`no_hash`、
`no_redact`、`no_serialize`、`no_deserialize`。`no_redact` 只允许完全没有字段或 selector 脱敏规则的类型，
保留的输出接口会恢复普通实现。`copy`、`default`、`partial_ord`、`ord` 为 opt-in；全 unit Enum
默认 `Copy`，可以指定 `no_copy`。

角色 attribute 应写在用户自定义的 `#[derive(...)]` 之前。这样宏可以识别已有 `Debug`/`Serialize`：启用脱敏时拒绝
会绕过安全输出的实现，使用 `no_redact` 时则复用已有安全实现而不重复派生。

具名 `Option` 和标准集合字段默认 `#[serde(default)]`，序列化时省略空值；`#[keep_serializing]` 可保留空值输出。
显式 Serde 配置优先。

## ModelProperties

方法必须是 public、safe、sync、非 const、非泛型 inherent method。getter 采用 `&self -> T`、`&T`、
`&str`、`&[T]` 或 `Option<&T>`；setter 采用 `&mut self, T -> ()`。生成的 adapter 不延长借用生命周期，
setter 在执行前发生类型错误时会归还 replacement。重复 Property 或重复 `ModelProperties` impl 会编译失败。

## 错误与排障

- 宏会聚合当前声明中彼此独立的 shape、option、字段 singleton 和 key order 错误。
- runtime facade 缺失时，诊断会要求添加 `qubit-model-metadata`；dependency 重命名是支持的。
- reference 的 ID、角色或 Property 只能在完整链接集合上判断，因此由 `ModelResolver` 报告。
- `TypeRef::Opaque` 与 `TypeRef::Symbolic` 没有 concrete descriptor；应检查 `type_ref()`，不要把
  `descriptor() == None` 当成 metadata 缺失。
- 若 codec trait bound 失败，应为 codec 实现准确的 `ValueEncoder` / `ValueDecoder`，不要引入不存在的 codec contract ID 类型。

## 限制与最佳实践

不要在普通静态查询中初始化 `ModelRegistry`，也不要用 Rust `type_name()` 代替稳定 `ModelId`。
跨模型检查应在所有模型 crate 链接完成后集中执行一次。validator 执行协议仍是后续 TODO；当前只把声明用于 schema、
文档和未来 resolver 输入。

详细 API 与实现边界见
[`2026-08-31` 最终设计](2026-08-31-182016-rs-model-derive-final-api-and-implementation-design.md)。
