# Qubit Model Metadata 用户手册

[English](user_guide.md) · [README](../README.zh_CN.md) · [API 文档](https://docs.rs/qubit-model-metadata)

适用于 `qubit-model-metadata` 0.1.0。

## 手册目标与读者

`qubit-model-metadata` 是静态 Rust 领域模型元数据的 runtime 表示与查询 API。通常由配套 derive crate 生成元数据；本手册说明应用和工具如何消费它，而不是完整讲解 derive 属性语言。

## 概念模型

`HasTypeShape` 描述递归结构形状。命名模型还实现 `HasTypeMetadata` 并暴露 `TypeMetadata`。`metadata_of` 获取由静态切片和函数指针构成的 `&'static` 值。

```text
HasTypeShape ──► TypeRef ──► TypeShape
       │
HasTypeMetadata ──► TypeMetadata ──► FieldMetadata + 模型级属性
```

`TypeIdentity` 在当前进程内通过 `TypeId` 比较类型。类型名称仅用于诊断展示；不要持久化 `TypeId`，也不要把它当作跨进程稳定标识。

## 贯穿场景：查询账户

安装 runtime 和 derive crate：

```toml
[dependencies]
qubit-model-metadata = "0.1.0"
qubit-model-derive = "0.1.0"
```

用强类型 API 查询生成的元数据：

```rust
use qubit_model_derive::Model;
use qubit_model_metadata::{AttributeQuery, TypeShape, metadata_of};

#[Model(id = "example.Account")]
struct Account {
    #[field(identifier)]
    id: i64,
    #[field(text(max_chars = 320), unique(ignore_case))]
    email: String,
    tags: Option<Vec<String>>,
}

let metadata = metadata_of::<Account>();
let email = metadata.field("email").expect("declared field");
assert!(metadata.primary_key().expect("primary key").contains("id"));
assert_eq!(email.text_constraint().and_then(|value| value.max_chars()), Some(320));
assert!(matches!(
    metadata.field("tags").expect("tags field").field_type().shape(),
    TypeShape::Optional(_)
));
```

`field` 返回 `Option`，因为查询的名称可能没有声明。应用代码应显式处理不存在的情况。

## 类型形状与可空性

`TypeRef` 是可复制的小型句柄。`shape()` 返回递归 `TypeShape`：标量、命名模型、可选值、序列、Set、Map、固定数组或 `Opaque`。宏生成的 opaque 字段会保留可见的标准容器外层，只将叶子标为 `Opaque`；最外层形状决定能力、可空性和关系投影。

```rust
use qubit_model_metadata::{TypeRef, TypeShape};

let shape = TypeRef::of::<Option<Vec<String>>>().shape();
assert!(matches!(shape, TypeShape::Optional(_)));
```

`FieldMetadata::is_nullable()` 只检查外层 `Option`。所以 `Option<Vec<String>>` 可空，`Vec<Option<String>>` 不可空。数组在 `TypeShape::Array` 中保留 const 长度，并同时暴露 sequence 与 array 能力。启用 `chrono` 可支持 `NaiveDate`、`NaiveTime`、`NaiveDateTime` 与 `DateTime<Utc>`；启用 `big-decimal` 可支持 `BigDecimal`。

## 查询属性和路径

导入 `AttributeQuery` 后可使用强类型便捷方法。模型级查询包括 `primary_key`、`unique_constraints`、`indexes`、`attributes_of` 和 `attribute`；字段在适用时提供 `text_constraint`、`reference`、`lookup_relation`、`sensitive`、`codec` 与 `generator`。

```rust
use qubit_model_metadata::{AttributeKind, AttributeQuery, metadata_of};

let metadata = metadata_of::<Account>();
assert!(metadata.indexes().all(|index| !index.fields().is_empty()));
assert_eq!(metadata.attributes_of(AttributeKind::Index).count(), 0);
```

`AttributeMetadata` 是非穷尽枚举。应优先使用强类型 getter，或者安全处理未来可能新增的 enum variant，而不能依赖穷尽匹配。

`FieldPath` 保存静态路径段。`resolve_field_path` 会沿可解析的命名 struct 元数据查找终点字段：

```rust
use qubit_model_metadata::{FieldPath, metadata_of};

let path = FieldPath::new(&["contact", "email"]);
let result = metadata_of::<Account>().resolve_field_path(path);
```

结果可能报告字段段不存在、中间值不是 struct，或命名类型的元数据无法解析。应将其作为集成或配置诊断，而不是全局模型图校验。

## 构造与边界

公共构造器支持 const，因此高级用户可以手工构造静态元数据。它们会校验字段顺序、非空键集合和路径、递增范围，以及 decimal scale 不大于 precision 等本地不变量。对大多数场景而言，derive 更安全，因为它让声明紧邻模型。

本 crate 会从进程中已链接的分布式注册项惰性构建不可变全局 `ModelRegistry`。只有已链接的模型 crate 会参与；需要受控模型集合的工具仍可从显式注册项集合构造注册表。注册表构建会重新校验稳定 ID、注册项一致性和重复 ID；调用 `ModelRegistry::validate_graph()` 可校验 Reference、LookupRelation、Ownership、关系投影和相应的环。它不负责数据库映射、codec/generator/脱敏执行或校验文案。`Opaque` 表示叶子类型有意不被解释，不能用来替代消费者所需的结构。

## 排障

| 症状 | 检查项 |
| --- | --- |
| `metadata_of::<T>()` 无法编译 | 确认 `T` 实现 `HasTypeMetadata`，通常通过 `Model` 完成。 |
| Model 拒绝外部字段类型 | 启用所需 feature、实现 `HasTypeShape`，或有意使用 `#[field(opaque)]`。 |
| 字段意外可空 | 检查最外层 `TypeShape`；只有外层 `Option<T>` 可空。 |
| 路径解析失败 | 验证每个字段段、中间具名 struct 及其元数据 resolver。 |
| 工具找不到模型 | 确认模型 crate 已链接且已注册，或从工具的显式注册项集合构造 `ModelRegistry`。 |

## 延伸阅读

- [derive 用户手册](../../rs-model-derive/doc/user_guide.zh_CN.md)
- [项目说明](../README.zh_CN.md)
- [README](../README.zh_CN.md)
- [English user guide](user_guide.md)
