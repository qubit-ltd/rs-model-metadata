# qubit-model-metadata

[![Rust CI](https://github.com/qubit-ltd/rs-model-metadata/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-model-metadata/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-model-metadata/coverage-badge.json)](https://qubit-ltd.github.io/rs-model-metadata/coverage/)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

`qubit-model-metadata` 为已由 `qubit-reflect` 描述结构的 Rust 类型补充稳定的领域语义。
它适合框架与应用开发者：通过生成的模型角色、字段语义和稳定 ID，在多个已链接的模型 crate
之间显式解析关系，同时避免另起一套反射系统。

## 安装

运行时 crate 需要 Rust 1.94，使用 edition 2024。Qubit 模型 crate 目前仅供内部使用且不发布
（`publish = false`），请使用检出路径，并按工作区布局调整：

```toml
[dependencies]
qubit-model-metadata = { version = "0.1", path = "../rs-model-metadata" }
qubit-model-derive = { version = "0.1", path = "../rs-model-metadata/derive" }
qubit-id = { version = "0.6", path = "../../rust-common/rs-id" }
```

`qubit-id` 提供 `Entity` 和 `Projection` 标识字段必须使用的 `Id` 类型。应用若要运行
`ModelResolver`，还需直接依赖 `qubit-validator` 与 `qubit-codec`；完整配置见用户指南。

## 快速开始

账户服务只需声明一次账户类型，就能在无需维护第二套模型注册流程的前提下读取模型元数据。派生宏生成
角色感知的元数据，`TypeMetadata` 则通过 `qubit-reflect` 采用的同一个 `TypeDescriptor` 暴露它。

```rust,ignore
use qubit_model_derive::Entity;
use qubit_id::Id;
use qubit_model_metadata::{ModelRegistry, TypeDescriptor, TypeMetadata};

#[Entity(id = "example.User")]
struct User {
    #[identifier]
    id: Id,
    #[unique(ignore_case = true)]
    email: String,
}

let metadata = TypeMetadata::of::<User>();
assert_eq!(metadata.model_id().unwrap().as_str(), "example.User");
assert!(std::ptr::eq(metadata.descriptor(), TypeDescriptor::of::<User>()));
let registry = ModelRegistry::try_global().expect("链接模型图有效");
assert!(registry.metadata_for(TypeDescriptor::of::<User>()).is_some());
```

得到的是 `User` 的静态元数据；`TypeMetadata::of` 不会初始化全局模型注册表。descriptor capability
和 Property 查询会使用冻结的 `ReflectRegistry` 快照，以便看见独立生成的模型 overlay。跨 crate
关系的解析流程请参阅用户指南。

## 为什么需要这个项目

反射可以回答类型有哪些字段、字段使用什么 Rust 类型等结构问题。领域模型还需要标识符、约束、
引用、Property、角色和可持久化的模型 ID 等信息。本 crate 将这些语义附着在反射 descriptor 上，
而不是重复维护一份反射模型。

## 核心能力

- `qubit-model-derive` 可为 `#[Entity]`、`#[Projection]`、`#[Model]`、`#[Enum]`、`#[Value]`
与 `#[ModelImpl]` 声明生成 metadata。
- `TypeMetadata` 为生成的类型提供静态的角色、Field、Property、泛型定义和可选 `ModelId` 信息。
- `ModelRegistry` 从冻结的 `ReflectRegistry` 快照投影具体模型及泛型定义；模型层不再维护自有 inventory。
- `ModelResolver` 在模型、validator 和 codec 注册表上执行显式解析。
- 解析成功时得到不可变的 `ResolvedModelGraph`；若引用、角色、Property、validator 或 codec
  无法解析，则按确定顺序汇总返回错误。
- 解析图提供引用、Projection source 与 producer、可执行 validator/codec 绑定、合并后的
  Property，以及由索引字段生成的查询 metadata。

本 crate 不会取代 `qubit-reflect`，静态元数据查询也不会隐式注册模型或解析跨模型关系。生成的
metadata 在穿过隐藏 model ABI v4 边界前，会校验 descriptor、Field、Property、角色和 codec 的不变量；
生成代码只依赖经过收窄的模型 facade 及其精确私有 ABI。

## 延伸阅读

- [English user guide](doc/user_guide.md)
- [简体中文用户指南](doc/user_guide.zh_CN.md)
- [`qubit-model-derive` 声明指南](derive/doc/user_guide.zh_CN.md)
- 配套的 derive crate 位于本仓库的 [`derive/`](derive/) workspace member 中。
- 本地 API 文档：运行 `cargo doc --open`
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

仓库地址：[https://github.com/qubit-ltd/rs-model-metadata](https://github.com/qubit-ltd/rs-model-metadata)
