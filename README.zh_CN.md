# qubit-model-metadata

[English](README.md) | [用户指南](doc/user_guide.zh_CN.md)

`qubit-model-metadata` 是 Qubit Rust 模型的领域语义层。它以
`qubit-reflect` 为唯一 Rust 结构事实源，在此基础上提供模型角色、字段约束、
Property、稳定 `ModelId`、注册表和显式跨模型解析。

配套的 `qubit-model-derive` 为 `#[Entity]`、`#[Projection]`、`#[Model]`、
`#[Enum]`、`#[Value]` 和 `#[ModelProperties]` 声明生成 metadata。

```rust,ignore
use qubit_model_derive::Entity;
use qubit_model_metadata::{ModelDescriptorExt, TypeDescriptor, TypeMetadata};

#[Entity(id = "example.User")]
struct User {
    #[identifier]
    id: u64,
    #[unique(ignore_case = true)]
    email: String,
}

let metadata = TypeMetadata::of::<User>();
assert_eq!(metadata.model_id().unwrap().as_str(), "example.User");
assert!(std::ptr::eq(metadata.descriptor(), TypeDescriptor::of::<User>()));
assert!(metadata.descriptor().model_metadata().is_some());
```

主要边界：

- `TypeDescriptor`、`FieldDescriptor`、`TypeRef` 和动态值均来自
  `qubit-reflect`，本 crate 不维护平行反射系统。
- 静态 metadata 查询不会初始化全局模型注册表。
- 跨 crate ID、reference、Projection source 和 Query 只在显式
  `ModelResolver` 阶段校验。
- 当前版本的 validator 只保存声明 metadata，注册与执行留待后续设计。
- codec 直接使用 `qubit-codec` 及其 `ValueEncoder` / `ValueDecoder` 约束。

完整流程见[用户指南](doc/user_guide.zh_CN.md)，API 细节见
[docs.rs](https://docs.rs/qubit-model-metadata)。

本项目采用 Apache-2.0 许可证。
