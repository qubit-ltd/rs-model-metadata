# Qubit Model Metadata

[![Rust CI](https://github.com/qubit-ltd/rs-model-metadata/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-model-metadata/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-model-metadata/coverage-badge.json)](https://qubit-ltd.github.io/rs-model-metadata/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-model-metadata.svg?color=blue)](https://crates.io/crates/qubit-model-metadata)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

`qubit-model-metadata` 为 Rust 领域模型提供不可变的强类型元数据。校验、面向 schema 的工具与应用代码可以通过它查看模型字段、类型结构、约束、键和关系，而无需可变的运行时注册表，也无需基于字符串推断类型。

## 安装

添加 runtime crate；如果希望从模型声明自动生成元数据，再添加配套的 derive crate：

```toml
[dependencies]
qubit-model-metadata = "0.1"
qubit-model-derive = "0.1.0"
```

可选 Cargo feature 为外部标量类型提供结构支持：

| Feature | 支持的类型 |
|---|---|
| `chrono` | `chrono::NaiveDate`、`NaiveTime`、`NaiveDateTime` 与 `DateTime<Utc>` |
| `big-decimal` | `bigdecimal::BigDecimal` |

## 快速开始

以账户模型为例，只需 derive 一次静态元数据，即可通过 runtime API 查询：

```rust
use qubit_model_derive::Model;
use qubit_model_metadata::{TypeShape, metadata_of};

#[Model(id = "example.Account")]
struct Account {
    #[field(identifier)]
    id: i64,
    #[field(text(min_chars = 3, max_chars = 320), unique(ignore_case))]
    email: String,
}

let metadata = metadata_of::<Account>();
let email = metadata.field("email").expect("email metadata");

assert!(metadata.primary_key().expect("primary key").contains("id"));
assert!(matches!(email.field_type().shape(), TypeShape::Scalar(_)));
assert_eq!(email.text_constraint().and_then(|text| text.max_chars()), Some(320));
```

查询只读取静态切片与函数指针，不会在运行时分配元数据图。

`Model` 是属性宏：它生成默认的模型 trait 与静态元数据；字段元数据使用
`#[field(...)]` 声明。

## 为什么需要这个项目

领域模型的消费者不仅需要 Rust 内存表示，还需要语义稳定的字段结构与约束。独立注册表容易与源码声明漂移，解析 `type_name` 字符串又会在类型别名和依赖重命名时失效。本 crate 使用递归 trait 表达结构，使用进程/构建本地的 `TypeId` 表达运行时身份；类型名称只保留为诊断展示数据。`TypeId` 仅适合进程内元数据查询，不能作为持久化或跨进程稳定标识。

## 提供的能力

- 为受支持的标量、`Option<T>`、`Vec<T>`、Set、Map、固定数组、命名模型与显式 opaque 字段提供递归 `TypeShape` 元数据。
- 提供 derive 编译期校验所用的能力标志。Option 与 newtype 继承内部能力；数组同时暴露 `SEQUENCE` 与 `ARRAY`，因此可以表达元素唯一性，同时仍以类型中的固定长度为准。
- 提供静态模型、字段、enum、newtype、约束、键、索引与关系值对象，以及强类型 getter。
- 提供无分配的字段、属性、键、索引与嵌套字段路径查询。
- 提供 const-compatible 公共构造器，拒绝反向范围、超过 precision 的 decimal scale，以及字段集合为空的键类元数据。
- 通过显式 Cargo feature 提供可选的 `chrono` 与 `bigdecimal` 标量集成。

## 已知限制

- runtime crate 提供由进程中已链接的分布式注册项组成的不可变全局 `ModelRegistry`。未链接的模型 crate 会有意缺席；需要时调用方也可以从显式注册项集合构造注册表。
- 不定义数据库映射、校验错误文案、序列化格式、codec、generator 或脱敏实现。
- 跨模型图校验与关系环检查不属于本 crate 的本地元数据 API。
- 用户自定义类型必须实现 `HasTypeShape`；结构确实不可用时，配套宏提供显式 `#[field(opaque)]` 逃生口。它会保留可见的 `Option`、序列、Set、数组和 Map 外层结构，只隐藏叶子类型。

## 延伸阅读

- [用户手册](doc/user_guide.zh_CN.md)
- [English user guide](doc/user_guide.md)
- [中文用户指南](doc/user_guide.zh_CN.md)
- [API 文档](https://docs.rs/qubit-model-metadata)
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

仓库地址：[https://github.com/qubit-ltd/rs-model-metadata](https://github.com/qubit-ltd/rs-model-metadata)
