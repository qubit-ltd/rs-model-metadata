# Qubit Model Metadata 用户手册

[English](user_guide.md) · [README](../README.zh_CN.md) · [API 文档](https://docs.rs/qubit-model-metadata)

适用于 `qubit-model-metadata` 0.1.0。

## 手册目标与读者

本手册面向需要在运行时消费静态领域模型元数据的应用和工具作者：schema 辅助代码、
校验逻辑，以及必须查看字段、约束、键和关系、但又不能维护可变注册表的代码。

元数据通常由 `qubit-model-derive` 生成。本 crate 提供强类型查询 API。
`#[Model]` / `#[Enum]` 的完整属性语言见
[derive 用户手册](https://github.com/qubit-ltd/rs-model-derive/blob/main/doc/user_guide.zh_CN.md)。
宏本身不校验实例数据。

## 概念模型

`HasTypeShape` 描述递归的类型形状。具名模型还会实现 `HasTypeMetadata`，并暴露
`TypeMetadata`。`metadata_of::<T>()` 返回由静态切片和函数指针构成的 `&'static`
值。

```text
HasTypeShape ──► TypeRef ──► TypeShape
       │
HasTypeMetadata ──► TypeMetadata ──► FieldMetadata + 模型级属性
       │
HasModelRegistration ──► MODEL_REGISTRATIONS ──► ModelRegistry
```

具名模型同时拥有可移植的 `ModelId`（例如 `example.Account`）和进程本地的
`TypeIdentity`。`TypeIdentity` 用 Rust 的 `TypeId` 比较类型。类型名称只用于诊断
展示。不要持久化 `TypeId`，也不要把它当作跨进程标识。

`TypeKind` 把具名类型分成 `Struct`、`Enum` 和 `Newtype`。字段查询面向 struct；
enum 和 newtype 的 `struct_fields()` 为空切片。

## 贯穿场景

注册服务要保存账户。每条账户有生成的标识、唯一邮箱、可选标签，以及嵌套的联系人
记录。成功标准是服务能够：

1. 让这些声明通过编译；
2. 读出主键、邮箱约束，以及忽略大小写的唯一性；
3. 确认 `tags: Option<Vec<String>>` 只在外层可空；
4. 沿静态字段路径解析到 `contact.email`。

## 安装与最小配置

最低 Rust 版本为 1.94。加入运行时 crate。若要从声明生成元数据，再加入配套
derive crate 和 Serde。两个属性宏都要求消费方依赖 `serde`。

```toml
[dependencies]
qubit-model-metadata = "0.1.0"
qubit-model-derive = "0.1.0"
serde = { version = "1", features = ["derive"] }
```

只为自己实际用到的标量类型打开 feature：

```toml
[dependencies]
qubit-model-metadata = { version = "0.1.0", features = ["chrono", "big-decimal"] }
chrono = { version = "0.4", default-features = false, features = ["std"] }
bigdecimal = "0.4"
```

`chrono` 覆盖 `NaiveDate`、`NaiveTime`、`NaiveDateTime` 与 `DateTime<Utc>`。
`big-decimal` 覆盖 `BigDecimal`。

## 核心工作流

先声明嵌套的联系人，再声明账户。`Model` 是属性宏，不是 `#[derive(Model)]`。
`#[field(identifier)]` 会变成模型级主键；`unique(ignore_case)` 会变成模型级唯一
约束。

```rust
use qubit_model_derive::Model;
use qubit_model_metadata::FieldPath;
use qubit_model_metadata::TypeShape;
use qubit_model_metadata::UniqueComparison;
use qubit_model_metadata::metadata_of;

#[Model(id = "example.Contact")]
struct Contact {
    #[field(text(max_chars = 320))]
    email: String,
}

#[Model(id = "example.Account")]
struct Account {
    #[field(identifier)]
    id: i64,
    #[field(text(min_chars = 3, max_chars = 320), unique(ignore_case))]
    email: String,
    tags: Option<Vec<String>>,
    contact: Contact,
}

fn inspect_account() {
    let metadata = metadata_of::<Account>();
    let email = metadata.field("email").expect("declared field");

    assert!(metadata.primary_key().expect("primary key").contains("id"));
    assert_eq!(email.text_constraint().and_then(|value| value.max_chars()), Some(320));
    assert_eq!(
        metadata
            .unique_constraints()
            .next()
            .and_then(|unique| unique.comparison_of("email")),
        Some(UniqueComparison::IgnoreCase)
    );
    assert!(matches!(
        metadata.field("tags").expect("tags field").field_type().shape(),
        TypeShape::Optional(_)
    ));
    assert!(metadata.field("tags").expect("tags field").is_nullable());

    let nested = metadata
        .resolve_field_path(FieldPath::new(&["contact", "email"]))
        .expect("nested field");
    assert_eq!(nested.name(), "email");
}
```

`field` 返回 `Option`，因为查询的名称可能并未声明。应用代码应把“字段不存在”
当成配置问题处理，而不是假定它不可能发生。

`ModelId` 由 ASCII snake_case 的模块段和 ASCII UpperCamelCase 的末段组成，例如
`example.Account`。空段、把 Rust 关键字当作模块段、以及末段不符合
UpperCamelCase 的 ID 都会被拒绝。

## 进阶用法

### 类型形状与可空性

`TypeRef` 是可复制的小型句柄。`shape()` 返回递归的 `TypeShape`：标量、具名模型、
可选值、序列、集合、映射、固定数组或 `Opaque`。宏生成的 opaque 字段会保留可见的
标准容器外层，只把叶子标成 `Opaque`。最外层形状决定可空性和关系投影。

```rust
use qubit_model_metadata::TypeRef;
use qubit_model_metadata::TypeShape;

let shape = TypeRef::of::<Option<Vec<String>>>().shape();
assert!(matches!(shape, TypeShape::Optional(_)));
```

`FieldMetadata::is_nullable()` 只检查最外层 `Option`。因此 `Option<Vec<String>>`
可空，`Vec<Option<String>>` 不可空。数组在 `TypeShape::Array` 中保留 const 长度，
并同时带有 sequence 与 array 能力；数组上的 `min_items` / `max_items` 会被拒绝，
因为长度已经由类型固定。

`TypeRef::strip_optional()` 去掉一层外层 `Option`。若结果是带 resolver 的具名
struct，`named_metadata()` 可以继续解析它的元数据。

### 属性查询

`TypeMetadata` 为 `primary_key`、`unique_constraints`、`indexes`、`keys` 和
`ownership` 提供了强类型 getter。需要按 `AttributeKind` 做通用查询时，再导入
`AttributeQuery`，使用 `attribute` 和 `attributes_of`。字段在相应属性存在时提供
`text_constraint`、`sequence_constraint`、`map_constraint`、
`temporal_constraint`、`decimal_constraint`、`element_metadata`、`reference`、
`lookup_relation`、`codec` 和 `generator`。

```rust
use qubit_model_metadata::AttributeKind;
use qubit_model_metadata::AttributeQuery;
use qubit_model_metadata::metadata_of;

let metadata = metadata_of::<Account>();
assert_eq!(metadata.attributes_of(AttributeKind::Unique).count(), 1);
assert!(matches!(
    metadata.attribute(AttributeKind::PrimaryKey),
    Some(_)
));
```

`AttributeMetadata` 是非穷尽枚举。应优先使用强类型 getter，或为将来新增的
variant 预留处理分支，不要依赖穷尽匹配。

### 字段路径

`FieldPath` 保存静态路径段。`resolve_field_path` 会沿可解析的具名 struct 元数据
走到终点字段。中间具名字段若带一层外层 `Option`，解析时会剥掉这一层；中间值不是
struct、某一段不存在、或具名类型没有 metadata resolver，都会得到带类型的错误。

路径解析只诊断这一次局部遍历，不是全局模型图校验。链接完整模型集合后，再调用
`ModelRegistry::validate_graph()`。

### 模型注册表

已链接的模型 crate 会把 `ModelRegistration` 写入分布式切片
`MODEL_REGISTRATIONS`。`ModelRegistry::try_global()` 惰性构建不可变索引，且不会
panic；链接集合不合法时，`global()` 会 panic。

构建阶段检查：

- 注册项 ID 与元数据 ID 是否符合 `ModelId` 协议；
- 这两个 ID 是否一致；
- 是否有两个注册项共用同一个稳定 ID 或同一个 `TypeIdentity`。

普通查询不会为此分配元数据图，也不会遍历关系。`get(id)` 按稳定 ID 查找；
`resolve` 实现 `MetadataResolver`，按运行时类型身份查找。

需要封闭集合的工具可以不使用进程级集合，改为对显式切片调用
`ModelRegistry::from_registrations`。

### 手工构造

公共构造器可用于 `const`，因此高级用户可以不依赖 derive crate 组装静态元数据。
它们会强制本地不变量：字段顺序与能力是否匹配、键 / 唯一约束 / 索引的字段集合
非空、文本和序列范围单调、decimal scale 不大于 precision。输入不合法时会
panic。多数场景仍应使用 derive，让声明紧挨着类型。

`TypeRef::opaque::<T>()` 在保留 Rust 类型名的同时把类型标为不解释。
`TypeRef::opaque_with_shape` 供能够看见标准容器语法、但仍要把叶子留成 opaque
的生成器使用。

## 错误与诊断

| API | 失败形式 | 含义 |
|---|---|---|
| `metadata_of::<T>()` | 无法编译 | `T` 未实现 `HasTypeMetadata`。 |
| `TypeMetadata::field` | `None` | 没有该规范化名称的已声明字段。 |
| `TypeMetadata::resolve_field_path` | `FieldPathResolveError` | 空路径、缺少字段段、中间值不是 struct，或具名元数据无法解析。 |
| `ModelId::try_new` / `validate` | `ModelIdError` | ID 为空、含空段、模块段或类型段不合法，或把 Rust 关键字当作模块段。 |
| `ModelRegistry::from_registrations` / `try_global` | `ModelRegistryError` | ID 不合法、注册项与元数据 ID 不一致、重复 ID 或重复类型身份。 |
| `ModelRegistry::global` | Panic | 与 `try_global` 相同的失败；调用方不能中止时改用 `try_global`。 |
| `ModelRegistry::validate_graph` | `ModelGraphErrors` | 一条或多条 `ModelGraphError`：目标缺失、目标字段缺失、投影不兼容、`path` 无效、归属目标缺失、必填引用环或归属环。 |
| `TypeMetadata::new`、`FieldMetadata::new`、约束 / 键构造器 | Panic | 构造时违反了本地不变量。 |

`FieldPathResolveError` 的 variant：

- `EmptyPath`
- `FieldNotFound { segment }`
- `IntermediateNotStruct { segment }`
- `NamedMetadataUnavailable { segment }`

`ModelGraphErrors::errors()` 以确定性顺序返回所有独立发现的图问题。构建注册表
时故意不走这一步，好让尚未齐全的链接集合仍然可用。

## 排障

| 症状 | 检查项 |
|---|---|
| `metadata_of::<T>()` 无法编译 | 确认 `T` 实现了 `HasTypeMetadata`，通常通过 `#[Model]` 或 `#[Enum]`。 |
| Model 拒绝外部字段类型 | 启用 `chrono` 或 `big-decimal`、自行实现 `HasTypeShape`，或在叶子应保持不解释时使用 `#[field(opaque)]`。 |
| 字段意外可空 | 检查最外层 `TypeShape`；只有外层 `Option<T>` 可空。 |
| 路径解析失败 | 核对每一段、中间具名类型是否为带 resolver 的 struct，以及路径是否为空。 |
| 工具找不到模型 | 确认模型 crate 已链接并注册，或从显式注册项集合构造 `ModelRegistry`。 |
| 启动时 `global()` panic | 改用 `try_global()`，根据 `ModelRegistryError` 排查不合法或重复的 ID。 |
| 单个关系看起来正常，合在一起却失败 | 等所有相关模型 crate 都链接后再调用 `validate_graph()`。 |

## 限制与最佳实践

- 本 crate 负责保存和查询元数据。它不映射数据库、不执行 codec 或 generator、
  不做脱敏，也不生成校验错误文案。
- `Opaque` 表示叶子类型被有意保持不解释，不能用来替代消费者仍然需要的结构。
- 优先使用 derive 生成的元数据，让 ID、字段和属性紧挨着类型。手工 `const` 构造
  更适合不能依赖宏的工具和测试。
- 查询时使用强类型 getter。`AttributeMetadata` 以后还可能增加 variant。
- 需要跨进程重启仍然有效的标识时使用 `ModelId`。`TypeIdentity` 只用于当前二进制
  内部。
- 不要把 `validate_graph()` 当成普通 `metadata_of` 查询的一部分。等链接集合完整
  后再运行。

## 延伸阅读

- [derive 用户手册](https://github.com/qubit-ltd/rs-model-derive/blob/main/doc/user_guide.zh_CN.md)
- [项目说明](../README.zh_CN.md)
- [API 文档](https://docs.rs/qubit-model-metadata)
- [English user guide](user_guide.md)
