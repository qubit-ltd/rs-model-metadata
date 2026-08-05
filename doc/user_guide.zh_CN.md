# Qubit Model Derive 用户手册

[English](user_guide.md) · [README](../README.zh_CN.md) · [API 文档](https://docs.rs/qubit-model-derive)

适用于 `qubit-model-derive` 0.1.0 与 `qubit-model-metadata` 0.1.0。

## 手册目标与读者

当 Rust 领域模型声明需要成为校验、面向 schema 的工具或应用代码的元数据事实来源时，请使用本 crate。`#[derive(ModelMetadata)]` 会生成 `qubit-model-metadata` 所消费的静态实现；它既不会全局注册模型，也不执行数据校验。

## 概念模型

derive 宏在编译期读取受支持的声明及其 `#[model(...)]` 属性，并生成 `HasTypeShape` 与 `HasTypeMetadata`；runtime crate 以强类型查询公开最终的不可变元数据。

```text
Rust 模型 + #[model(...)]
            │ 编译期
            ▼
ModelMetadata derive ──► 静态 runtime 元数据 ──► metadata_of::<T>()
```

## 贯穿场景：描述账户

安装版本匹配的 runtime 和 derive crate：

```toml
[dependencies]
qubit-model-derive = "0.1.0"
qubit-model-metadata = "0.1.0"
```

让元数据声明紧邻模型，再查询其规范化结果：

```rust
use qubit_model_derive::ModelMetadata;
use qubit_model_metadata::{AttributeQuery, metadata_of};

#[derive(ModelMetadata)]
struct Account {
    #[model(identifier(generated))]
    id: i64,
    #[model(unique(ignore_case), text(min_chars = 3, max_chars = 320))]
    email: String,
}

let metadata = metadata_of::<Account>();
let email = metadata.field("email").expect("declared field");
assert!(metadata.primary_key().expect("primary key").contains("id"));
assert_eq!(email.text_constraint().and_then(|value| value.max_chars()), Some(320));
```

`identifier(generated)` 会变为模型级主键，字段级 `unique(ignore_case)` 会变为模型级唯一约束。消费者查询规范化后的元数据，而不是依赖宏输入的写法。

## 支持的声明与形状

宏支持具名字段 struct、unit struct、单字段 tuple newtype 与 fieldless enum；会拒绝泛型模型、多字段 tuple struct、union 和携带数据的 enum variant。

类型结构来自 `HasTypeShape`，而不是解析类型名称字符串。支持的形状包括标量、`Option<T>`、`Vec<T>`、`HashSet<T>`、`BTreeSet<T>`、`HashMap<K, V>`、`BTreeMap<K, V>`、固定数组和 derive 元数据的模型。`Option<Vec<String>>` 与 `Vec<Option<String>>` 保持不同；只有最外层 `Option` 使字段可空。

按需启用外部标量支持：

```toml
[dependencies]
qubit-model-metadata = { version = "0.1.0", features = ["chrono", "big-decimal"] }
chrono = { version = "0.4", default-features = false, features = ["std"] }
bigdecimal = "0.4"
```

## 属性参考

模型级属性描述整个模型：

| 属性 | 含义 |
| --- | --- |
| `primary_key(fields(id), generated(id))` | 有序主键；生成字段必须属于该主键。 |
| `unique(name = "account", fields(org_id, username), ignore_case(username))` | 按字段保存的唯一性比较规则。 |
| `index(name = "created_at", fields(created_at))` | 有序索引字段。 |
| `key(name = "account", fields(org_id, username))` | 逻辑键。 |
| `ownership(owner = Organization)` | 拥有该模型的模型。 |

字段级属性包括：

| 属性 | 用途 |
| --- | --- |
| `identifier`、`unique`、`index` | 单字段键、唯一和索引简写。 |
| `text(...)` | 字符/字节范围、`repertoire`、`non_blank` 与 `format`。 |
| `sequence(...)`、`map(...)` | 容器大小范围与 sequence 的 `unique_items`。 |
| `time(...)`、`decimal(...)`、`money(...)` | 时间或 decimal 语义及范围。 |
| `reference(...)`、`lookup_relation(...)` | 目标模型和目标字段关系元数据。 |
| `sensitive(...)`、`codec`、`generator` | 仅保存处理方式或策略名称元数据。 |
| `opaque` | 显式隐藏外部类型的结构。 |

`text` 支持 `min_chars`、`max_chars`、`min_bytes`、`max_bytes`、`repertoire = unicode|ascii`、`non_blank` 与 `format = email|uri|uuid`。`sequence` 支持 `min_items`、`max_items` 与 `unique_items`；`map` 支持 `min_entries` 与 `max_entries`。`time` 使用 `precision = second|millisecond|microsecond|nanosecond` 和 `normalization = preserve|utc`。`decimal` 与 `money` 支持 `precision`、`scale` 和 `rounding = half_up|half_even|down|up`，二者不可并用。

宏会拒绝错误作用域、重复或冲突声明、无效范围、不可用类型能力和无效本地字段引用。

## 关系与 Opaque 字段

关系要同时声明目标类型和目标字段：

```rust
use qubit_model_derive::ModelMetadata;

#[derive(ModelMetadata)]
struct Organization { #[model(identifier)] id: i64 }

#[derive(ModelMetadata)]
struct Membership {
    #[model(reference(target = Organization, target_field = id, must_exist = true))]
    organization_id: i64,
}
```

这只是本地声明：本版本不校验完整模型图中的目标字段类型相容性或关系环。

对有意不提供结构元数据的外部类型，请使用 `opaque`：

```rust
use qubit_model_derive::ModelMetadata;

struct ExternalToken;
#[derive(ModelMetadata)]
struct ImportRecord { #[model(opaque)] token: ExternalToken }
```

否则，外部字段类型必须实现 `HasTypeShape`。opaque 字段表现为 `TypeShape::Opaque`，并且不能配合依赖形状的 `text`、`sequence`、`map`、`time`、`decimal` 或 `money` 约束。

## 错误与诊断

编译错误会指向无效声明。请检查 runtime crate 是否存在、模型形状是否支持、属性拼写和作用域是否正确、字段引用是否存在，以及 decimal `scale` 是否不大于 `precision`。对未知外部类型，实现 `HasTypeShape`，或明确选择 `opaque`。runtime 包可以在 Cargo 中重命名；宏会按包名解析它。

## 限制与最佳实践

让领域约束紧邻模型，并通过 runtime crate 消费生成的元数据。本宏不提供表/列映射、JSON 格式、校验文案、codec/generator 执行或全局发现。

## 延伸阅读

- [runtime 元数据用户手册](../../rs-model-metadata/doc/user_guide.zh_CN.md)
- [模型元数据与 derive 设计](model-metadata-and-derive-design.md)
- [README](../README.zh_CN.md)
- [English user guide](user_guide.md)
