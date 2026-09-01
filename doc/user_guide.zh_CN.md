# qubit-model-metadata 用户指南

[English](user_guide.md) | [README](../README.zh_CN.md) | [API 文档](https://docs.rs/qubit-model-metadata)

适用于 `qubit-model-metadata` 0.1.x、Rust 1.94 和 edition 2024。

## 手册目标与读者

本指南面向使用 `qubit-model-derive` 声明领域模型的框架和应用开发者。它说明结构反射与领域语义的
边界，并以账户模型为例，展示如何从模型声明走到不可变的解析结果图。

## 概念模型

`qubit-reflect` 负责 Rust 类型的结构信息，`qubit-model-metadata` 在结构之上附加领域语义，
`qubit-model-derive` 根据同一份声明生成这两层信息。

```text
Rust 声明 -> TypeDescriptor -> TypeMetadata -> ModelRegistry -> ResolvedModelGraph
                |                  |
          FieldDescriptor      Field / Property 语义
```

`TypeRef` 可处于 `Resolved`、`Opaque` 或 `Symbolic` 状态。因此，
`FieldMetadata::descriptor()` 和 `PropertyMetadata::descriptor()` 返回 `Option`：
没有具体 descriptor 可能只是 opaque 或 symbolic 类型的正常表示，并非 metadata 缺失。

## 贯穿场景

一个账户服务需要立即检查自身的模型声明，并在应用链接完全部模型 crate 后，解析登录模型的引用。
完成标志有两个：不依赖全局注册表即可读取账户的 metadata；完整注册表能够将
`Login.account_id` 解析到 `Account` Entity 的 `id` Property。

## 安装与最小配置

在应用中加入运行时 crate 与派生宏 crate：

```toml
[dependencies]
qubit-model-metadata = "0.1"
qubit-model-derive = "0.1"
```

传给 `TypeMetadata::of` 的类型必须使用模型角色派生宏。该宏会生成所需的 metadata provider 与 trait
bound；仅派生结构反射的类型不能满足这一要求。

## 核心工作流

先声明 Entity，再声明引用它的 Model。`#[reference]` 使用稳定的 Entity ID 和公开的 Property 名称，
留待后续解析。

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

先对账户模型做静态查询；这一过程不会初始化全局模型注册表：

```rust,ignore
use qubit_model_metadata::TypeMetadata;

let account = TypeMetadata::of::<Account>();
assert!(account.field("id").unwrap().is_identifier());
assert!(account.property("email").unwrap().is_readable());
```

待所有模型 crate 都完成链接后，取得三个显式注册表并执行一次解析：

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

成功后会得到不可变的 `ResolvedModelGraph`。resolver 只会在全部关系解析成功后返回完整结果，不会逐步
发布半成品图。

## 进阶用法

Field 是反射层提供的真实存储槽位。Property 以一个公开名称组合 Field 与可选的 getter、setter。
getter adapter 会保留借用生命周期；若 setter 在执行前失败，`PropertySetFailure` 会保留 replacement，
调用方仍可继续处理该值。

带有模型 ID 的泛型声明只注册一个 `GenericModelMetadata` 模板。具体单态类型没有 `ModelId`，也不会
进入注册表；可通过 `generic_definition()` 回到该模板。模板字段可以使用 symbolic 类型，而具体实例的
字段类型可以是已解析的。

## 错误与诊断

`ModelRegistry::try_global()` 会以 `ModelRegistryError` 报告重复模型 ID 或反射注册表创建失败。
`resolve_all()` 返回 `ModelResolveErrors`，它会按确定顺序聚合错误，而不会发布带未解析关系的图。错误涵盖
模型 ID 缺失、角色不匹配、Property 缺失或不可读、Projection source 无效，以及 validator 或 codec
注册缺失、value type 不兼容等情况。

生成的 metadata 在发布前还会检查局部 ABI 不变量。若 panic 信息以 `QMM-ABI-` 开头，说明生成代码或手写的
隐藏 ABI metadata 违反了相应不变量，已被拒绝。

## 排障

- `TypeMetadata::of::<T>()` 无法编译时，确认 `T` 使用了模型角色宏，并满足生成的 trait bound。
- `descriptor()` 返回 `None` 时，先检查 `type_ref()`；opaque 和 symbolic 引用本来就没有具体 descriptor。
- `resolve_all()` 返回错误时，逐项检查 `ModelResolveError`，修正稳定 ID、目标角色、Property 名称或对应的
  validator/codec 注册，然后重新执行完整解析。
- 成功解析的 validator 会提供强类型注册项和可读的依赖 Property；成功解析的 codec 会提供可执行 descriptor，
  对于 ID 声明还会提供匹配的注册项。

## 限制与最佳实践

稳定链接的声明使用 `ModelId`，动态输入先交给 `ModelIdBuf::parse` 解析；不要把 Rust 诊断类型名用作持久化
ID。普通静态查询应与全局注册表初始化分离，只有在完整模型集合都链接后才进行解析。本 crate 不提供另一套
反射系统，也不会在静态查询时隐式绑定跨模型引用。

## 延伸阅读

- [README](../README.zh_CN.md)
- [English user guide](user_guide.md)
- [API 文档](https://docs.rs/qubit-model-metadata)
