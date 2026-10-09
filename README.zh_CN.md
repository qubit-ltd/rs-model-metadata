# qubit-model-metadata

[![Rust CI](https://github.com/qubit-ltd/rs-model-metadata/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-model-metadata/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-model-metadata/coverage-badge.json)](https://qubit-ltd.github.io/rs-model-metadata/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-model-metadata.svg?color=blue)](https://crates.io/crates/qubit-model-metadata)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

`qubit-model-metadata` 让程序读取领域对象上的业务约束。`qubit-reflect` 描述 Rust
结构，`qubit-model-derive` 用宏补充标识、唯一性、文本长度、引用、索引等语义，本 crate
则提供读取与跨模型解析能力。校验器、约束感知的测试数据生成器、REST 查询层都可以复用同一份模型声明。

## 安装

<!-- example: core -->
```toml
[dependencies]
qubit-model-metadata = { version = "0.2", path = "../rs-model-metadata", default-features = false }
qubit-model-derive = { version = "0.2", path = "../rs-model-metadata/derive" }
qubit-id = "0.7"
qubit-reflect = { version = "0.2", path = "../rs-reflect" }
```

示例假定应用 crate 位于 `rs-platform/app`，与 `rs-model-metadata`、`rs-reflect`
同级；在该应用目录运行完整示例。metadata、model-id 和 derive 的 manifest 均设置了
`publish = false`。版本字段标识包版本，实际来源由对应的检出路径确定。完整目录布局和
validation 安装方式见运行时指南。

默认不启用可选 feature。按需启用 `validation` 构建校验计划、`codec` 绑定编码器，或
`generic` 读取泛型定义元数据。只需传递稳定模型 ID 时，可直接依赖独立的
`qubit-model-id` 包：

<!-- example: core/model-id -->
```toml
[dependencies]
qubit-model-id = { version = "0.1", path = "../rs-model-metadata/model-id" }
```

## 示例：声明用户账号

<!-- example: core/quick-start -->
```rust
use qubit_id::Id;
use qubit_model_derive::Entity;
use qubit_model_metadata::metadata::TypeMetadata;

#[Entity(id = "guide.iam.User")]
struct User {
    #[identifier]
    id: Id,
    #[unique(ignore_case = true)]
    #[text(non_blank, min_chars = 3, max_chars = 64, allowed_chars = ascii)]
    username: String,
    #[indexed]
    #[text(max_chars = 128)]
    display_name: String,
}

fn main() {
    let user_metadata = TypeMetadata::of::<User>();
    let username_field = user_metadata.field("username").unwrap();
    assert!(username_field.is_unique());
    assert_eq!(username_field.text_constraint().unwrap().max_chars(), Some(64));
    assert!(user_metadata.field("display_name").unwrap().is_indexed());
}
```

标注记录的是规则，不会直接校验某个 `User` 实例。校验消费者可检查长度，并查询数据库确认唯一性；
测试数据生成器可据此生成满足约束的账号；查询层可依据 `indexed` 接受 `display_name`
过滤条件。数据库查找、生成策略和查询执行由应用提供。

## 运行时提供什么

| API | 用途 |
| --- | --- |
| `TypeMetadata`、`FieldMetadata` | 无需实例即可读取角色、字段、约束与标注。 |
| `ModelRegistry` | 在同一反射快照中按稳定 `ModelId` 或 Rust 类型查找模型。 |
| `StructureResolver`、`ModelGraph` | 跨模型检查并解析引用、Projection 来源、唯一性作用域与查询声明。 |
| `PropertyAccessPath` | 编译属性路径，再通过受支持的访问器读写实例。 |
| 可选的 `validation`、`codec` feature | 将声明绑定到显式提供的规则或编码器注册表。 |

`ModelId` 是稳定的外部模型身份；Rust `TypeId` 与 `FieldLocation` 只用于进程内结构定位。
`#[unique]` 声明唯一性规则，本身不查询数据库；`#[indexed]` 标记查询字段，本 crate
不创建物理索引或 SQL。

[运行时用户手册](doc/user_guide.zh_CN.md)详述注册、关系解析、字段和属性访问、校验及错误处理；
[宏声明手册](derive/doc/user_guide.zh_CN.md)可用于速查标注。

## 延伸阅读

- [English user guide](doc/user_guide.md)
- [简体中文用户指南](doc/user_guide.zh_CN.md)
- [Design](doc/design.md) · [设计文档](doc/design.zh_CN.md)
- [`qubit-model-derive` 声明指南](derive/doc/user_guide.zh_CN.md)
- [覆盖率测量与工具复现检查](doc/coverage_measurement.zh_CN.md)
- 配套的 derive crate 位于本仓库的 [`derive/`](derive/) workspace member 中。
- 本地 API 文档：运行 `cargo doc --open`
- [English README](README.md)

## 测试

```bash
# 使用默认 feature 集运行测试
cargo test --workspace --locked

# 使用项目声明的全部 feature 运行测试
cargo test --workspace --all-features --locked

# 运行项目 CI 检查
./.infra/bin/ci-check.sh

# 检查代码覆盖率
./.infra/bin/coverage.sh
```

## 许可证

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

本项目基于 Apache License 2.0 授权。完整许可证文本请参阅
[LICENSE](LICENSE)。

## 贡献

欢迎贡献。请遵循 Rust API 指南，及时更新公共 API 文档与测试。提交 Pull Request
前，运行 `./.infra/bin/style-check.sh` 检查项目 Rust 样式，运行
`./.infra/bin/align-ci.sh` 按项目固定的 nightly 工具链和 rustfmt 配置自动对齐，
并运行 `./.infra/bin/ci-check.sh` 执行完整 CI 门禁。这些 rs-infra 命令是项目的
权威门禁；普通 `cargo fmt --all -- --check` 使用当前工具链和默认 rustfmt 配置，
不能替代它们。Cargo workspace 和路径依赖布局也会影响各命令覆盖的源码文件；
应以 rs-infra 门禁实际报告的路径判断检查或格式化了哪些文件。

## 作者

**Haixing Hu** - *Qubit Co. Ltd.*

仓库地址：[https://github.com/qubit-ltd/rs-model-metadata](https://github.com/qubit-ltd/rs-model-metadata)
