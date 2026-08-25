# Qubit Model Metadata

[![Rust CI](https://github.com/qubit-ltd/rs-model-metadata/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-model-metadata/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-model-metadata/coverage-badge.json)](https://qubit-ltd.github.io/rs-model-metadata/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-model-metadata.svg?color=blue)](https://crates.io/crates/qubit-model-metadata)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

`qubit-model-metadata` 为校验逻辑、schema 工具和应用代码提供 Rust 领域模型的
不可变强类型视图：字段、类型形状、约束、键和关系都可以按类型查询。这些事实
通常由配套的 `qubit-model-derive` 从模型声明生成，因此不必再维护一份容易和源码
漂移的注册表，也不必靠解析 `type_name` 字符串来还原结构。

## 安装

本 crate 是运行时查询 API。若要从模型声明生成元数据，请同时加入配套 derive
crate 和 Serde。两个属性宏都要求消费方依赖 `serde`。最低 Rust 版本为 1.94。

```toml
[dependencies]
qubit-model-metadata = "0.1"
qubit-model-derive = "0.1"
serde = { version = "1", features = ["derive"] }
```

可选 Cargo feature 为外部标量类型提供形状支持：

| Feature | 支持的类型 |
|---|---|
| `chrono` | `chrono::NaiveDate`、`NaiveTime`、`NaiveDateTime` 与 `DateTime<Utc>` |
| `big-decimal` | `bigdecimal::BigDecimal` |

## 快速开始

注册服务要落账户记录。写库之前，schema 辅助代码需要知道主键是哪一列、邮箱最长
多少字符、唯一性比较是否忽略大小写。模型声明一次即可；查询只读取静态切片和函数
指针，不会在运行时分配元数据图。

```rust
use qubit_model_derive::Model;
use qubit_model_metadata::TypeShape;
use qubit_model_metadata::UniqueComparison;
use qubit_model_metadata::metadata_of;

#[Model(id = "example.Account")]
struct Account {
    #[field(identifier)]
    id: i64,
    #[field(text(min_chars = 3, max_chars = 320), unique(ignore_case))]
    email: String,
}

fn main() {
    let metadata = metadata_of::<Account>();
    let email = metadata.field("email").expect("email metadata");

    assert!(metadata.primary_key().expect("primary key").contains("id"));
    assert!(matches!(email.field_type().shape(), TypeShape::Scalar(_)));
    assert_eq!(email.text_constraint().and_then(|text| text.max_chars()), Some(320));
    assert_eq!(
        metadata
            .unique_constraints()
            .next()
            .and_then(|unique| unique.comparison_of("email")),
        Some(UniqueComparison::IgnoreCase)
    );
}
```

`Model` 是属性宏，不是 `#[derive(Model)]`。它生成运行时 trait 和进程内注册项；
字段元数据用 `#[field(...)]` 声明。`unique(ignore_case)` 会提升为模型级唯一约束。

## 为什么需要这个项目

领域模型的消费者要的不只是 Rust 内存布局，还要语义稳定的字段结构和约束。手工
维护的注册表很容易和源码声明不一致；解析 `type_name` 又会在类型别名、依赖重命名
时失效。

本 crate 用递归 trait 描述结构，用当前进程的 `TypeId` 识别类型。类型名称只保留
为诊断展示信息。`TypeId` 只适合进程内查找，不能持久化，也不能当作跨进程稳定标识。
可移植的标识是 `ModelId`。

## 提供的能力

- 为受支持的标量、`Option<T>`、`Vec<T>`、`HashSet`/`BTreeSet`、`HashMap`/`BTreeMap`、
  固定数组、具名模型和显式 opaque 字段提供递归 `TypeShape`。
- 提供属性校验所用的能力标志。`Option` 继承内部类型的能力；数组同时带有
  `SEQUENCE` 和 `ARRAY`，因此可以表达元素唯一性，但仍以类型上的固定长度为准。
- 提供模型、字段、enum、newtype、约束、键、索引和关系的静态值对象，以及强类型
  getter。
- 提供无堆分配的字段、属性、键、索引和嵌套字段路径查询。
- 提供可用于 `const` 的公共构造器，拒绝反向范围、大于 precision 的 decimal
  scale，以及字段集合为空的键类元数据。
- 基于已链接进进程的注册项提供不可变 `ModelRegistry`；完整模型集合就绪后，可调用
  `validate_graph()` 校验跨模型引用。
- 通过显式 Cargo feature 接入可选的 `chrono` 与 `bigdecimal` 标量。

本 crate 不负责数据库映射、校验文案、序列化格式，也不执行 codec、generator 或
脱敏。

## 已知限制

- 全局注册表只包含链接进当前进程的模型 crate。未链接的 crate 会有意缺席。需要
  封闭集合时，可调用 `ModelRegistry::from_registrations`。
- 构建注册表时会检查 ID 是否合法、注册项与元数据 ID 是否一致，以及是否出现重复
  ID 或类型身份。它不会遍历关系。链接完整模型集合后，再调用
  `ModelRegistry::validate_graph()`。
- 用户自定义字段类型必须实现 `HasTypeShape`。配套宏提供 `#[field(opaque)]`：
  可见的 `Option`、序列、集合、数组和 Map 外层会保留，只有叶子类型不被解释。
- `FieldMetadata::is_nullable()` 只看最外层是否为 `Option`。因此
  `Option<Vec<String>>` 可空，`Vec<Option<String>>` 不可空。

## 延伸阅读

- [用户手册](doc/user_guide.zh_CN.md)
- [English user guide](doc/user_guide.md)
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
