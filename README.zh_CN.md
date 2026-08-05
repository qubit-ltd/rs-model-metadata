# Qubit Model Derive

[![Rust CI](https://github.com/qubit-ltd/rs-model-derive/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-model-derive/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-model-derive/coverage-badge.json)](https://qubit-ltd.github.io/rs-model-derive/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-model-derive.svg?color=blue)](https://crates.io/crates/qubit-model-derive)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

`qubit-model-derive` 为 Rust 领域模型提供 `#[derive(ModelMetadata)]`。它将模型声明转换为 `qubit-model-metadata` 暴露的静态强类型元数据，使校验、面向 schema 的工具和应用代码无需运行时注册表或基于字符串的类型推断即可查询模型结构。

## 安装

请为 derive crate 和 runtime crate 使用匹配版本：

```toml
[dependencies]
qubit-model-derive = "0.1"
qubit-model-metadata = "0.1"
```

runtime crate 是必需依赖：若展开位置没有 `qubit-model-metadata` 依赖，宏会发出 `compile_error!`，说明缺少该依赖。

## 快速开始

以账户模型为例，只需 derive 一次，应用中任何需要查看模型的地方都可以查询元数据：

```rust
use qubit_model_derive::ModelMetadata;
use qubit_model_metadata::metadata_of;

#[derive(ModelMetadata)]
struct Account {
    #[model(identifier)]
    id: i64,
    #[model(unique(ignore_case), text(min_chars = 3, max_chars = 320))]
    email: String,
}

fn main() {
    let metadata = metadata_of::<Account>();
    assert!(metadata.primary_key().expect("primary key").contains("id"));
    assert!(!metadata.field("email").expect("email field").is_nullable());
}
```

derive 会生成不可变元数据和所需的 runtime trait。该查询可观察声明的主键与字段元数据，而不会在运行时解析 Rust 类型名称。

## 为什么需要这个项目

Rust 模型通常不仅需要 Rust 类型：校验、持久化和 schema 工具还需要字段结构与领域约束。把这些事实维护在独立注册表中容易与模型声明漂移；基于类型名字符串的推断又会在类型别名和依赖重命名时失效。本 crate 让模型声明保持为唯一事实来源，并由 Rust 在编译期解析实际类型。

## 提供的能力

- 为具名字段 struct、unit struct、单字段 tuple newtype 与 fieldless enum 生成静态 `HasTypeShape` 和 `HasTypeMetadata` 实现。
- 从受支持的 `#[model(...)]` 属性生成字段、类型、键、唯一性、索引、文本、集合、时间、decimal、reference、敏感信息、codec 与 generator 元数据。
- 按 Cargo 包名解析 runtime 依赖。若本地将 `qubit-model-metadata` 重命名，展开代码会使用该本地依赖名；即使存在同名本地模块可能造成遮蔽，也仍然适用：

  ```toml
  [dependencies]
  model_runtime = { package = "qubit-model-metadata", version = "0.1.0" }
  ```

- 要求未知外部字段类型通过 `#[model(opaque)]` 显式选择退出结构解析。opaque 字段会保留 Rust 类型名并暴露 `TypeShape::Opaque`，而不要求外部类型实现 `HasTypeShape`。

对确实不应进行结构解析的字段使用 opaque 标记：

```rust
struct ExternalToken;

#[derive(ModelMetadata)]
struct ImportRecord {
    #[model(opaque)]
    token: ExternalToken,
}
```

未使用 `opaque` 时，外部类型必须实现 `HasTypeShape`；`opaque` 不能与依赖结构形状的字段约束同时使用，例如 `text`、`sequence`、`map`、`time`、`decimal` 或 `money`。

## 已知限制

- 多字段 tuple struct、带数据的 enum variant、union 和泛型模型会被拒绝。
- 宏执行本地声明与能力校验。阶段 D 的关系能力不在本版本范围内：不提供显式模型集合，也不执行跨模型的目标字段、类型相容性或关系环校验。
- `same_as` 当前仅接受一个本地 Rust 字段；嵌套关系路径仍属于暂缓的模型集合校验工作。
- 不定义表/列映射、PostgreSQL 专属类型、JSON 导出格式，或 codec/generator 的策略 trait 实现。

## 延伸阅读

- [用户手册](doc/user_guide.zh_CN.md)
- [模型元数据与 derive 设计](doc/model-metadata-and-derive-design.md)
- [API 文档](https://docs.rs/qubit-model-derive)
- [English document](README.md)

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
