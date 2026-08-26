# Qubit Model Derive

[![Rust CI](https://github.com/qubit-ltd/rs-model-derive/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-model-derive/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-model-derive/coverage-badge.json)](https://qubit-ltd.github.io/rs-model-derive/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-model-derive.svg?color=blue)](https://crates.io/crates/qubit-model-derive)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

领域对象除了 Rust 类型本身，校验、持久化和 schema 工具还需要字段结构、约束和
稳定标识。这些事实如果另存一份，很快就会和代码分叉。`qubit-model-derive`
提供两个属性宏：`#[Model]` 和 `#[Enum]`。声明写在类型旁边，通过
`#[identifier]`、`#[text(...)]` 等独立字段属性表达约束，编译期生成
`qubit-model-metadata` 在运行时查询的静态实现和注册项。

## 安装

derive crate、runtime crate 与 Serde 使用匹配版本。两个宏都要求消费方依赖
`serde`：

```toml
[dependencies]
qubit-model-derive = "0.1"
qubit-model-metadata = "0.1"
serde = { version = "1", features = ["derive"] }
```

缺少 `qubit-model-metadata` 或 `serde` 时，展开会发出 `compile_error!`，指出缺
少哪个依赖。只有模型或枚举需要脱敏时，才额外加入 `qubit-redact`。

请写成 `#[Model(...)]` 和 `#[Enum(...)]`。没有 `#[derive(Model)]` 这类别名。

## 快速开始

账户记录和它的生命周期状态是两种形状：结构体和枚举。分别用对
应的宏声明，在字段上写独立属性，再查询生成的元数据：

```rust
use qubit_model_derive::Enum;
use qubit_model_derive::Model;
use qubit_model_metadata::AttributeQuery;
use qubit_model_metadata::TypeKind;
use qubit_model_metadata::metadata_of;

#[Enum(id = "example.AccountStatus")]
enum AccountStatus {
    Active,
    Suspended,
}

#[Model(id = "example.Account")]
struct Account {
    #[identifier]
    id: i64,
    #[unique(ignore_case)]
    #[text(min_chars = 3, max_chars = 320)]
    email: String,
    status: AccountStatus,
}

fn main() {
    let account = metadata_of::<Account>();
    assert!(account.primary_key().expect("primary key").contains("id"));
    assert!(!account.field("email").expect("email field").is_nullable());

    let status = AccountStatus::Suspended;
    assert_eq!(status.name(), "SUSPENDED");
    assert_eq!(AccountStatus::from_name("ACTIVE"), Some(AccountStatus::Active));
    assert!(matches!(metadata_of::<AccountStatus>().kind(), TypeKind::Enum(_)));
}
```

`Account` 会注册结构体元数据，并带上主键和邮箱唯一约束。`AccountStatus` 会注册
枚举元数据，同时提供 `Display`、Serde、`name` 和 `from_name` 共用的规范名。这些
查询都不会在运行时解析 Rust 类型名字符串。

## 为什么需要这个项目

领域类型不只是内存布局。下游还需要键、唯一性、文本和数值边界，以及一份不随
crate 重命名失效的稳定 ID。靠类型名字符串去猜，会在类型别名和依赖重命名时失
败。本 crate 把这些事实留在声明旁边，由 Rust 在编译期解析真实类型。

## 提供的能力

两个宏都必须写稳定 ID：`id = "module.Type"`，最后一段要和 Rust 类型名一致。每次
展开都会实现 `HasTypeShape`、`HasTypeMetadata` 和 `HasModelRegistration`，并向
不可变的全局 `ModelRegistry` 贡献一条注册。

runtime 依赖按 Cargo 包名解析。本地重命名仍然有效：

```toml
[dependencies]
model_runtime = { package = "qubit-model-metadata", version = "0.1.0" }
```

### `#[Model]`

`#[Model]` 接受具名字段结构体、空结构体和单字段元组 newtype。用在枚举上会编
译失败，应改用 `#[Enum]`。

`primary_key`、`index`、`key`、`ownership` 等模型级键写在 `#[Model(...)]` 参数
里。字段约束用独立字段属性，例如 `#[identifier]`、`#[unique(...)]`、
`#[text(...)]`、`#[reference(...)]`。已移除的 `#[field(...)]` 包装会触发编译
错误。

对结构体，它会生成：

- 默认 trait：`Clone`、`Debug`、`Eq`、`PartialEq`、`Hash`、`Serialize`、
  `Deserialize`
- Debug 风格的 `Display`
- `#[serde(rename_all = "snake_case")]`
- Serde 省略规则：值为 `None` 的 `Option<T>` 与直接声明的空标准集合不输出；
  集合字段在反序列化缺失时自动使用 `#[serde(default)]`
- 静态的 `TypeKind::Struct` 或 `TypeKind::Newtype` 元数据
- 来自独立字段属性和模型级参数的字段、键、唯一性、索引、文本、集合、时
  间、decimal、引用、codec 与 generator 元数据

结构体上不能写 `no_copy`。`#[Model(..., redact)]` 或任意字段 `#[redact(...)]`
会把格式化和序列化交给 `qubit-redact`。

未知外部字段类型必须显式加上 `#[opaque]`。opaque 字段会保留可见的
`Option`、序列、Set、数组和 Map 外层，叶子则暴露为 `TypeShape::Opaque`。不加
`opaque` 时，字段类型必须实现 `HasTypeShape`。`opaque` 不能和依赖形状的约束
一起用，例如 `text`、`sequence`、`map`、`time`、`decimal`、`money`。

集合省略规则只识别 `Option`、`Vec`、`LinkedList`、`VecDeque`、`HashMap`、
`BTreeMap`、`HashSet`、`BTreeSet`、`BinaryHeap` 和固定长度数组的无限定预导入
写法及显式标准库路径；类型别名和末段名称相同的限定路径自定义类型不在识别范围内。
对 `Option` 或上述集合字段加 `#[keep_serializing]`，即可不使用宏自动添加的省略和
默认值规则，序列化时保留 `null` 或空值。

字段上的 `#[unique(...)]` 简写会规范化为模型级唯一约束。复合唯一性在标注字
段上使用 `respectTo = [other_fields]`。

### `#[Enum]`

`#[Enum]` 接受 unit、tuple、struct 以及混合枚举。用在结构体上会编译失败；泛
型枚举仍不支持。

对枚举，它会生成：

- 默认 trait：`Clone`、`Debug`、`Eq`、`PartialEq`、`PartialOrd`、`Ord`、
  `Hash`、`Serialize`、`Deserialize`；全部为 unit variant 时还会生成 `Copy`
- 若声明上还没有 `#[must_use]`，则自动补上
- `#[serde(rename_all = "SCREAMING_SNAKE_CASE")]`
- 输出规范序列化名及 Debug 风格 tuple/struct 载荷的 `Display`
- `name(&self) -> &'static str`；全部为 unit variant 时还会生成
  `from_name(&str) -> Option<Self>`
- 静态的 `TypeKind::Enum` 元数据；变体会暴露 `Unit`、`Tuple` 或 `Struct` 形
  状，并复用 `FieldMetadata` 描述载荷字段

变体上的 `#[serde(rename = "...")]` 或 `#[serde(rename(serialize = "..."))]`
会覆盖规范名，并同时作用于元数据、`Display`、`name`，以及存在时的
`from_name`。重复的序列化名会被拒绝。

载荷字段可以使用 `text`、`sequence`、`map`、`time`、decimal、元素约束、策
略、`opaque` 和脱敏等局部规则。`identifier`、`unique`、`indexed`、
`reference`、`lookup_relation` 以及模型级键会被拒绝，因为不同变体没有共同的
记录级字段集合。tuple 载荷元数据字段名依次为 `"0"`、`"1"`。`no_copy` 对所
有枚举仍然有效。tuple 的自动 Serde 省略会保持位置：对于至少有两个字段的变体，只有
最后一个可选字段或空集合字段会被省略；缺失的尾部字段在反序列化时通过 `default` 补齐。
单字段 newtype 载荷受 Serde 表示限制，会保留 `null` 或空集合。

### 明确不提供的能力

宏不会校验实例数据，不定义表/列映射或 PostgreSQL 专属类型，不导出 JSON
schema，也不执行 codec/generator 策略。目标是否存在、投影是否相容、ownership
是否成环，要在链接完整模型集合后调用 `ModelRegistry::validate_graph()`。

## 已知限制

- 泛型模型、多字段元组结构体和 union 会被拒绝。
- `primary_key`、`index`、`key`、`ownership` 这类模型级约束只适用于具名字段
  结构体。
- `reference(entity = "module.Type", ...)` 使用稳定目标 ID，不要求对目标模型
  建立 Cargo 依赖。

## 延伸阅读

- [用户手册](doc/user_guide.zh_CN.md)
- [runtime 元数据用户手册](../rs-model-metadata/doc/user_guide.zh_CN.md)
- [脱敏运行时手册](https://github.com/qubit-ltd/rs-redact/blob/main/doc/user_guide.zh_CN.md)
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
