# qubit-model-metadata 用户指南

[English](user_guide.md) | [README](../README.zh_CN.md)

## 手册目标与概念模型

本指南面向使用 0.1.x 的框架和应用开发者。`qubit-reflect` 负责 Rust 结构，
`qubit-model-metadata` 叠加领域含义，`qubit-model-derive` 则根据同一份声明生成二者。

```text
Rust 声明 -> TypeDescriptor -> TypeMetadata -> ModelRegistry -> ResolvedModelGraph
                |                  |
          FieldDescriptor      Field / Property 语义
```

`TypeRef` 分为 `Resolved`、`Opaque` 和 `Symbolic`。因此
`FieldMetadata::descriptor()` 与 `PropertyMetadata::descriptor()` 返回 `Option`，
`None` 并不表示 metadata 丢失。

## 贯穿场景：检查并解析账户模型

应用同时依赖 runtime 与宏 crate。当前仓库使用 Rust 1.94 和 edition 2024。

```toml
[dependencies]
qubit-model-metadata = "0.1"
qubit-model-derive = "0.1"
```

先声明 Entity 和引用它的 Model：

```rust,ignore
use qubit_model_derive::{Entity, Model};

#[Entity(id = "example.Account")]
pub struct Account {
    #[identifier]
    pub id: u64,
    #[unique(ignore_case = true)]
    pub email: String,
}

#[Model(id = "example.Login")]
pub struct Login {
    #[reference(entity_id = "example.Account", property = id)]
    pub account_id: u64,
}
```

静态查询不会触碰全局注册表：

```rust,ignore
use qubit_model_metadata::TypeMetadata;

let account = TypeMetadata::of::<Account>();
assert!(account.field("id").unwrap().is_identifier());
assert!(account.property("email").unwrap().is_readable());
```

等所有模型 crate 链接完成后，再集中收集注册项并解析外部关系：

```rust,ignore
use qubit_codec::ValueCodecRegistry;
use qubit_model_metadata::{ModelRegistry, ModelResolver, ResolveInputs, TypeMetadata};
use qubit_validator::ValidatorRegistry;

let models = ModelRegistry::try_global()?;
let validators = ValidatorRegistry::try_global()?;
let codecs = ValueCodecRegistry::try_global()?;
let graph = ModelResolver::new(ResolveInputs { models, validators, codecs }).resolve_all()?;
let field = TypeMetadata::of::<Login>().field("account_id").unwrap();
assert_eq!(
    graph.reference(field).unwrap().target().model_id().unwrap().as_str(),
    "example.Account",
);
# Ok::<(), Box<dyn std::error::Error>>(())
```

成功后得到不可变的 `ResolvedModelGraph`。只要存在未解析关系，resolver 就不会发布部分图，
而是按确定顺序一次返回所有错误。

## Field、Property 与泛型

Field 是反射得到的真实存储槽位。Property 按名称合并 field、getter 和 setter。
getter adapter 保留借用生命周期；setter 若在执行前失败，会归还 replacement。

带 ID 的泛型声明只注册一个 `GenericModelMetadata` 模板。具体单态类型没有 `ModelId`，
也不会进入注册表；它通过 `generic_definition()` 指向模板。模板字段可以使用 symbolic
`TypeRef`，具体单态字段则可解析为 concrete descriptor。

## 错误、诊断与排障

- `ModelRegistryError` 表示重复 ID 或反射注册表初始化失败。
- `ModelResolveErrors` 聚合缺失 ID、角色错误、Property 缺失或不可读、Projection source 无效，以及
  validator/codec 注册缺失或 value type 不兼容等问题。
- `TypeMetadata::of::<T>()` 无法编译时，检查 `T` 是否由角色宏生成，以及字段是否满足自动实现所需 trait bound。
- `descriptor()` 返回 `None` 时应继续检查 `type_ref()`；opaque 和 symbolic 类型本来就没有 concrete descriptor。
- 成功解析的 validator 会公开强类型注册项及可读 dependency Property；成功解析的 codec 会公开可执行
  descriptor，ID 声明还会保留对应注册项。
- `QMM-ABI-*` panic 表示生成代码或手写隐藏 ABI metadata 违反局部不变量，且已在发布前被拒绝。

## 限制与最佳实践

稳定链接声明使用 `ModelId`，动态字符串先经过 `ModelIdBuf::parse`。不要把 Rust 诊断类型名当作持久化 ID。
普通 metadata 查询与全局注册表初始化应保持分离，并在完整模型集合链接后显式执行一次 resolver。

更多信息见 [README](../README.zh_CN.md) 与
[API 文档](https://docs.rs/qubit-model-metadata)。
