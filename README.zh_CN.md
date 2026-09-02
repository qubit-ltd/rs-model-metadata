# qubit-model-derive

[![Rust CI](https://github.com/qubit-ltd/rs-model-derive/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-model-derive/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-model-derive/coverage-badge.json)](https://qubit-ltd.github.io/rs-model-derive/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-model-derive.svg?color=blue)](https://crates.io/crates/qubit-model-derive)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

`qubit-model-derive` 把 Rust 领域类型声明编译为 Qubit 模型元数据。它面向需要同时获得 Rust
结构反射和领域语义的应用、框架作者：身份、约束、关系、脱敏、序列化策略与安全属性都从同一份类型声明生成，无需手工维护另一套 schema。

## 安装

本 crate 使用 Rust 1.94 和 edition 2024，且当前未发布到 crates.io（`publish = false`）。
请在同时包含宏 crate 和 `qubit-model-metadata` 运行时门面的工作区中使用路径依赖；路径需按实际目录调整：

```toml
[dependencies]
qubit-model-derive = { version = "0.1", path = "../rs-model-derive" }
qubit-model-metadata = { version = "0.1", path = "../rs-model-metadata" }
```

生成代码会通过 `proc-macro-crate` 解析 `qubit-model-metadata` 的实际依赖名，因此支持重命名 runtime
依赖；业务 crate 不必直接依赖 `qubit-reflect`。

## 快速开始

以登录服务为例：用户必须有稳定身份，邮箱不能在日志中明文输出，框架还需要发现可写的 `email`
属性。只需声明一次模型：

```rust,ignore
use qubit_model_derive::{Entity, ModelImpl};
use qubit_model_metadata::{ModelDescriptorExt, TypeMetadata};

#[Entity(id = "example.User")]
pub struct User {
    #[identifier]
    id: u64,
    #[unique(ignore_case = true)]
    #[redact(level = "medium")]
    email: String,
}

#[ModelImpl]
impl User {
    pub fn email(&self) -> &str { &self.email }
    pub fn set_email(&mut self, value: String) { self.email = value; }
}

let metadata = TypeMetadata::of::<User>();
assert!(metadata.field("id").unwrap().is_identifier());
assert!(metadata.try_property("email").unwrap().unwrap().is_writable());
assert!(metadata.descriptor().model_metadata().is_some());
```

角色宏会委托 `qubit-reflect` 生成 Rust 结构描述符，再将唯一的 `TypeMetadata` 类型化能力
附加到同一个描述符上。生成的 `Debug`、`Display`、`Serialize` 会遵守脱敏策略，不会把邮箱按普通明文输出。

## 提供的能力

六个属性宏共用解析、规范化、校验和展开流程：

- `#[Entity]`：声明带持久化身份的模型。
- `#[Projection]`：声明实体的开放或固定视图。
- `#[Model]`：声明普通结构化数据。
- `#[Enum]`：声明领域枚举，并保留 Rust 名、canonical 名和 Serde 名。
- `#[Value]`：声明值对象；`transparent` 支持单字段包装类型。
- `#[ModelImpl]`：把公开固有方法中的 getter/setter 与字段合并为安全的属性元数据。

五种角色默认生成 `Clone`、遵守脱敏策略的 `Debug` / `Display` / `Serialize`，以及
`Deserialize`、`PartialEq`、`Eq`、`Hash`、`Redact`。可用对应的 `no_*` 参数关闭；`copy`、
`default`、`partial_ord`、`ord` 需要显式开启。全 unit Enum 默认实现 `Copy`，指定 `no_copy`
后例外。

角色 attribute 必须写在用户自定义 `#[derive(...)]` 前，使宏能够识别会重复生成实现或绕开脱敏输出的组合。

## 边界

通过 `TypeMetadata::of::<T>()` 或 `ModelDescriptorExt::model_metadata()` 查询静态元数据不会初始化全局注册表。只有在所有参与 crate 都已链接后，才使用 `ModelRegistry`、`ValidatorRegistry`、
`ValueCodecRegistry` 和 `ModelResolver` 解析稳定 ID、reference、Projection 来源、Query、validator 或 codec。
`ValueCodecRegistry` 需要应用直接依赖 `qubit-codec` 并启用 `features = ["registry"]`；该 feature 不在
默认 feature 集中。

小写 `#[validator(...)]` 生成已校验的 occurrence；resolver 会按稳定 ID 绑定 `qubit-validator`
注册项并解析可读依赖。Rust codec 会直接生成可执行 `ValueCodecDescriptor`，或按稳定 ID 绑定，且会校验
精确 value type。若多个原始 map key
脱敏后相同，序列化会失败，避免静默覆盖数据。

## 延伸阅读

- [English user guide](doc/user_guide.md)
- [中文用户指南](doc/user_guide.zh_CN.md)
- 本地 API 文档：在 crate 根目录运行 `cargo doc --open`
- [最终设计](doc/rs-model-derive-final-design.zh_CN.md)
- [English README](README.md)

## 测试

```bash
# 使用默认 feature 集运行测试
cargo test

# 使用项目声明的全部 feature 运行测试
cargo test --all-features

# 运行项目 CI 检查
./ci-check.sh

# 检查代码覆盖率
./coverage.sh
```

## 许可证

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

本项目基于 Apache License 2.0 授权。完整许可证文本请参阅
[LICENSE](LICENSE)。

## 贡献

欢迎贡献。请遵循 Rust API 指南，及时更新公共 API 文档与测试，并在提交
Pull Request 前运行 `./align-ci.sh`格式化代码，运行`./ci-check.sh`对齐CI要求。

## 作者

**Haixing Hu** - *Qubit Co. Ltd.*

仓库地址：[https://github.com/qubit-ltd/rs-model-derive](https://github.com/qubit-ltd/rs-model-derive)
