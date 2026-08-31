# qubit-model-derive

[English](README.md) | [用户指南](doc/user_guide.zh_CN.md)

`qubit-model-derive` 提供 Qubit Rust 模型的最终 attribute macro API。六个宏共用同一套解析、
校验、规范化与展开流水线：

- `#[Entity]`：有持久化身份的领域实体；
- `#[Projection]`：Entity 的 open 或 fixed 视图；
- `#[Model]`：普通结构化数据；
- `#[Enum]`：领域枚举；
- `#[Value]`：值对象与 transparent wrapper；
- `#[ModelProperties]`：基于安全 getter/setter 的 Property。

```rust,ignore
use qubit_model_derive::{Entity, ModelProperties};
use qubit_model_metadata::TypeMetadata;

#[Entity(id = "example.User")]
pub struct User {
    #[identifier]
    id: u64,
    #[unique(ignore_case = true)]
    #[redact(level = "medium")]
    email: String,
}

#[ModelProperties]
impl User {
    pub fn email(&self) -> &str { &self.email }
    pub fn set_email(&mut self, value: String) { self.email = value; }
}

let metadata = TypeMetadata::of::<User>();
assert!(metadata.property("email").unwrap().is_writable());
```

生成代码只引用解析后的 `qubit-model-metadata` facade，由 `qubit-reflect` 提供唯一 Rust 结构，
模型层只生成领域语义 overlay；只有带稳定 ID 的声明才注册。runtime dependency 可以重命名。

五种角色默认提供 `Clone`、经过脱敏的 `Debug` / `Display` / `Serialize`、`Deserialize`、
`PartialEq`、`Eq`、`Hash` 和 `Redact`。可以使用对应 `no_*` 参数关闭接口；`copy`、
`default`、`partial_ord`、`ord` 为显式启用。全 unit Enum 默认 `Copy`，可用 `no_copy` 关闭。

小写 `#[validator(...)]` 当前只生成声明 metadata，不包含 validator 注册和执行。Rust codec 类型必须满足
`qubit-codec` 的 `ValueEncoder` 与 `ValueDecoder` 约束。

如果还要添加用户自定义 `#[derive(...)]`，请把角色 attribute 写在它前面，以便模型宏检测并复用或拒绝已有输出实现。

完整用法见[用户指南](doc/user_guide.zh_CN.md)和
[`2026-08-31` 最终设计](doc/2026-08-31-182016-rs-model-derive-final-api-and-implementation-design.md)。

本项目采用 Apache-2.0 许可证。
