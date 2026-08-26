# Qubit Model Derive 用户手册

[English](user_guide.md) · [README](../README.zh_CN.md) · [API 文档](https://docs.rs/qubit-model-derive)

适用于 `qubit-model-derive` 0.1.0 与 `qubit-model-metadata` 0.1.0。

本 crate 对外提供两个属性宏：结构体用 `#[Model(...)]`，枚举用
`#[Enum(...)]`。没有 `#[derive(Model)]` 这种写法。

## 手册目标与读者

当 Rust 领域声明需要成为校验、schema 工具或应用代码的元数据事实来源时，请读
这本手册。宏生成的是 `qubit-model-metadata` 消费的静态实现和注册项，不会校验
实例数据。

具名字段结构体、空结构体、单字段元组 newtype 用 `#[Model]`；枚举用
`#[Enum]`。宏选错会直接编译失败。

## 概念模型

编译期由对应的宏读取类型、`#[Model(...)]` 上的模型级参数，以及字段上的独立辅
助属性（如 `#[identifier]`、`#[text(...)]`），再生成默认 trait、`Display`、Serde
命名规则和运行时元数据 trait。

```text
结构体 + #[identifier] / #[text(...)] / …  ──►  #[Model]  ──►  TypeKind::Struct | Newtype
unit / tuple / struct 枚举                   ──►  #[Enum]   ──►  TypeKind::Enum
                                                            │
                                                            ▼
                         HasTypeShape + HasTypeMetadata + ModelRegistry
                                                            │
                                                            ▼
                                                    metadata_of::<T>()
```

两个宏共用同一套 ID 规则、三个运行时 trait，以及向 `ModelRegistry` 的自动注
册。差别在于接受的形状、默认 trait、Serde 命名、`Display`，以及是否存在字段
约束。

`primary_key`、`index`、`key`、`ownership` 等模型级键写在 `#[Model(...)]` 参数
里。字段约束写成字段上的独立属性。已移除的 `#[field(...)]` 包装会触发编译错
误。

模型 ID 的模块段必须是 ASCII snake_case，最后一段必须是与 Rust 类型名一致的
ASCII UpperCamelCase，例如 `example.AccountStatus`。

## 贯穿场景：账户及其状态

应用要保存账户。每个账户有生成型标识、唯一邮箱，以及生命周期状态。状态是一
组封闭的名字，不是带字段的结构体。成功标准是：两个类型都能编译、完成注册，
并且能用类型化 API 查出元数据。

### 安装与最小配置

```toml
[dependencies]
qubit-model-derive = "0.1.0"
qubit-model-metadata = "0.1.0"
serde = { version = "1", features = ["derive"] }
```

两个宏都要求消费方依赖 `serde`，即使稍后用 `no_serialize` 关掉序列化也一样。
只有真正用到的外部标量才打开 runtime feature：

```toml
[dependencies]
qubit-model-metadata = { version = "0.1.0", features = ["chrono", "big-decimal"] }
chrono = { version = "0.4", default-features = false, features = ["std"] }
bigdecimal = "0.4"
```

声明需要脱敏时再加入 `qubit-redact`：

```toml
[dependencies]
qubit-redact = { version = "0.5", features = ["serde", "derive"] }
```

本地把 `qubit-model-metadata` 重命名后，展开仍按 Cargo 包名解析：

```toml
[dependencies]
model_runtime = { package = "qubit-model-metadata", version = "0.1.0" }
```

### 核心工作流

状态用 `#[Enum]`，账户用 `#[Model]`，然后查询规范化后的元数据：

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
    #[identifier(generated)]
    id: i64,
    #[unique(ignore_case)]
    #[text(min_chars = 3, max_chars = 320)]
    email: String,
    status: AccountStatus,
}

let account = metadata_of::<Account>();
let email = account.field("email").expect("declared field");
assert!(account.primary_key().expect("primary key").contains("id"));
assert_eq!(email.text_constraint().and_then(|value| value.max_chars()), Some(320));

let status = AccountStatus::Suspended;
assert_eq!(format!("{status}"), "SUSPENDED");
assert_eq!(status.name(), "SUSPENDED");
assert_eq!(AccountStatus::from_name("ACTIVE"), Some(AccountStatus::Active));
assert!(matches!(metadata_of::<AccountStatus>().kind(), TypeKind::Enum(_)));
```

`identifier(generated)` 会变成模型级主键，`unique(ignore_case)` 会变成模型级
唯一约束。调用方查询的是规范化结果，而不是宏输入的原始写法。

枚举这边同样可观察：`AccountStatus::Suspended` 显示为 `SUSPENDED`，序列化为
`"SUSPENDED"`，并能通过 `from_name` 还原。

## `#[Model]`：结构体能力

`#[Model]` 会改写结构体声明，并为其生成元数据。

### 接受的形状

| 形状 | 元数据种类 | 说明 |
| --- | --- | --- |
| 具名字段结构体 | `TypeKind::Struct` | 字段约束和模型级键写在这里。 |
| 空结构体 | `TypeKind::Struct` | 没有字段。 |
| 单字段元组 newtype | `TypeKind::Newtype` | 内层字段在元数据里叫 `"0"`。非 opaque 的 newtype 会继承内层类型的 `TypeCapabilities`。 |

泛型、多字段元组结构体、union 和枚举都会被拒绝。枚举必须改用 `#[Enum]`。

### 默认 trait 与命名

未关闭时，结构体会得到 `Clone`、`Debug`、`Eq`、`PartialEq`、`Hash`、
`Serialize`、`Deserialize`。宏还会实现 Debug 风格的 `Display`，具名字段结构
体打印成 `Account { email: "..." }`。Serde 字段名使用 `snake_case`。

结构体不会得到 `Copy`、`PartialOrd`、`Ord`。在结构体上写 `no_copy` 会编译失
败。

### Option 和集合字段的 Serde 省略规则

开启序列化时，`#[Model]` 会省略值为 `None` 的直接声明 `Option<T>` 字段；直接
声明的 `Vec`、`LinkedList`、`VecDeque`、`HashMap`、`BTreeMap`、`HashSet`、
`BTreeSet`、`BinaryHeap` 以及固定长度数组为空时也会省略。开启反序列化时，这些
集合字段会自动获得 `#[serde(default)]`，缺少字段就构造对应的空默认值。

```rust
use std::collections::HashMap;

use qubit_model_derive::Model;

#[Model(id = "example.SearchFilter", no_hash)]
struct SearchFilter {
    query: Option<String>,
    labels: Vec<String>,
    facets: HashMap<String, String>,
    #[keep_serializing]
    explicit_labels: Vec<String>,
}

let filter = SearchFilter {
    query: None,
    labels: Vec::new(),
    facets: HashMap::new(),
    explicit_labels: Vec::new(),
};
assert_eq!(
    serde_json::to_string(&filter).expect("serialize filter"),
    r#"{"explicit_labels":[]}"#
);
```

`#[keep_serializing]` 会让该字段不使用宏自动添加的两项规则：序列化时保留
`null` 或空值，并且不会自动附加 `serde(default)`。它不会删除字段上已经显式写
出的 `#[serde(...)]`。宏只识别直接写出的类型语法，不处理类型别名。固定长度数组
只有长度为零时才为空，长度非零的数组仍会输出。

### 模型级属性

这些属性写在 `#[Model(...)]` 参数里。除了 `id`，它们只允许出现在具名字段结构
体上。

| 属性 | 含义 |
| --- | --- |
| `id = "example.Account"` | 必填的稳定模型 ID。 |
| `textual` | 把具名字段结构体标成具备文本能力的值对象，以便 `text(format = mobile)` 这类约束能作用到它。 |
| `primary_key(fields(id), generated(id))` | 有序主键。生成字段必须属于该主键。 |
| `index(name = "created_at", fields(created_at))` | 有序索引。 |
| `key(name = "account", fields(org_id, username))` | 逻辑键。 |
| `ownership(owner = Organization)` | 拥有该模型的类型。`target = Type` 可作为 `owner` 的别名。 |

字段上的 `#[identifier]` 是单字段主键简写；`#[unique(...)]` 和 `#[indexed]` 是
对应的单字段简写。复合唯一性在参与字段之一上写
`#[unique(respectTo = [other_fields], ...)]`；`#[Model(...)]` 里没有模型级
`unique(...)` 参数。

### 独立字段属性

字段约束写成字段上的独立属性。可空性来自 `Option<T>`，没有 `nullable` 开关。

| 属性 | 用途 |
| --- | --- |
| `identifier`、`identifier(generated)` | 单字段主键。 |
| `unique`、`unique(ignore_case)`、`unique(ignoreCase = true)` | 单字段或复合唯一约束。复合键用 `respectTo = [fields]`，可选 `name = "..."`。 |
| `indexed` | 单字段索引。 |
| `text(...)` | 字符/字节范围、字符集、`non_blank` 与格式。 |
| `sequence(...)` | 元素个数范围和 `unique_items`。 |
| `map(...)` | 条目个数范围。 |
| `element(text(...))`、`element(decimal(...))` | 作用在每个序列元素上的约束。 |
| `time(...)` | 时间精度；`DateTime<Utc>` 由其类型形状表示为 instant。 |
| `decimal(...)`、`money(...)` | decimal 语义和范围；二者不能同时使用。 |
| `reference(...)` | 指向另一个模型 ID 和字段路径的直接引用。 |
| `lookup_relation(...)` | 按当前作用域里的目标类型做查找关系。 |
| `codec`、`generator` | 只保存策略名；宏不会执行策略。 |
| `opaque` | 隐藏外部类型的内部结构。 |
| `keep_serializing` | Serde 输出中保留 `None` 或支持集合的空值，并阻止宏为该字段自动添加 `serde(default)`。 |

`text` 支持 `min_chars`、`max_chars`、`min_bytes`、`max_bytes`、
`allowed_chars = unicode\|ascii`、`non_blank`，以及
`format = email\|mobile\|uri\|uuid`。`sequence` 支持 `min_items`、
`max_items`、`unique_items`。`map` 支持 `min_entries`、`max_entries`。`time`
使用 `precision = second\|millisecond\|microsecond\|nanosecond`。
`DateTime<Utc>` 由其类型形状表示为 `ScalarType::Instant`，而
`NaiveDateTime` 仍表示无时区时间。`decimal` 与 `money` 支持 `precision`、
`scale` 和 `rounding = half_up\|half_even\|down\|up`。`money` 必须写 `scale`。
`codec` 和 `generator` 可写成 `codec = "name"` 或
`codec(name = "name")`。

类型结构来自 `HasTypeShape`，不是解析类型名字符串。支持的形状包括标量、
`Option<T>`、`Vec<T>`、`LinkedList<T>`、`VecDeque<T>`、`HashSet<T>`、
`BTreeSet<T>`、`HashMap<K, V>`、`BTreeMap<K, V>`、`BinaryHeap<T>`、固定数组，
以及其他已经派生元数据的模型。`Option<Vec<String>>` 和 `Vec<Option<String>>`
不是同一种形状；只有最外层 `Option` 会让字段可空。

## `#[Enum]`：枚举能力

`#[Enum]` 会改写枚举，并生成变体及载荷字段元数据。

### 接受的形状

支持 unit、tuple、struct 及混合变体。`#[Enum]` 用在结构体或 union 上会被拒
绝；泛型枚举仍不支持。

### 默认 trait 与命名

未关闭时，枚举会得到 `Clone`、`Debug`、`Eq`、`PartialEq`、`PartialOrd`、
`Ord`、`Hash`、`Serialize`、`Deserialize`。只有全部变体都是 unit variant
时，才会默认得到 `Copy`。若声明上还没有 `#[must_use]`，宏会补上。Serde 变
体名使用 `SCREAMING_SNAKE_CASE`。

`Display` 输出的是规范序列化名，不是 Rust 标识符：`AccountStatus::Suspended`
显示为 `SUSPENDED`。tuple 和 struct 变体会追加 Debug 风格载荷，例如
`PROGRESS(42)` 和 `FAILED { message: "timeout" }`。除非关闭 `Display` 或由脱敏
实现安全格式化，载荷字段必须实现 `Debug`。

### 规范名

所有枚举都会生成：

```rust
pub const fn name(&self) -> &'static str;
```

仅当全部变体都是 unit variant 时，还会生成：

```rust
pub fn from_name(name: &str) -> Option<Self>;
```

这两个方法、`Display`、Serde 和 `EnumVariantMetadata` 共用同一套名字。默认
是把变体标识符转成 `SCREAMING_SNAKE_CASE`。变体可以自行覆盖：

```rust
use qubit_model_derive::Enum;

#[Enum(id = "example.SerializedStatus")]
enum SerializedStatus {
    #[serde(rename = "reviewing")]
    Reviewing,
    #[serde(rename(serialize = "invalid-state"))]
    Invalid,
}

assert_eq!(SerializedStatus::Reviewing.name(), "reviewing");
assert_eq!(
    SerializedStatus::from_name("invalid-state"),
    Some(SerializedStatus::Invalid)
);
```

空的或重复的序列化名会编译失败。载荷字段复用 `FieldMetadata`：tuple 字段名依
次为 `"0"`、`"1"`，struct 字段保留 Rust 字段名。支持局部约束、策略、
`opaque`、Serde 默认规则和脱敏。`identifier`、`unique`、`indexed`、
`reference`、`lookup_relation` 等记录级 helper 以及模型级键会被拒绝，因为变
体没有共同的字段集合。

## 进阶用法

### 关闭默认能力

两个宏都接受同一组 `no_*` 开关。无法识别的 `no_*` 名称会被拒绝。解析后还会
套用依赖规则：

- `no_partial_eq` 会同时关掉 `Eq`、`PartialOrd`、`Ord`
- `no_eq` 或 `no_partial_ord` 会同时关掉 `Ord`

`no_copy` 只允许写在 `#[Enum]` 上。写上 `no_debug` 并不会拿掉 `Display`：结构
体在没有 `Debug` 时，仍可用 Debug 风格的 `Display` 打印。

```rust
use qubit_model_derive::Model;

#[Model(id = "example.Relaxed", no_display, no_eq, no_hash, no_serialize)]
struct Relaxed {
    value: f64,
}
```

### 脱敏

`#[Model(..., redact)]` 或 `#[Enum(..., redact)]` 会显式启用脱敏 derive。结
构体上只要有字段写了 `#[redact(...)]`，也会自动启用。字段语义委托给
`qubit-redact`，本 crate 不再维护第二套实现。

```rust
use qubit_model_derive::Model;
use qubit_redact::Redactor;

#[Model(id = "example.Credential")]
struct Credential {
    username: String,
    #[opaque]
    #[redact(level = "secret")]
    password: String,
}

let value = Credential {
    username: "alice".to_owned(),
    password: "raw-secret".to_owned(),
};
let output = Redactor::standard().redact(&value);
assert!(!output.text().as_str().contains("raw-secret"));
assert!(!serde_json::to_string(&value).unwrap().contains("raw-secret"));
```

开启脱敏后，`Debug`、`Display`、`Serialize` 遵循 `qubit-redact` 的字段模式和
全局 disabled-policy。直接走 Serde 没有 summary 通道；需要完整性或审计原因
时，使用 `Redactor::redact`。除非写了 `no_deserialize`，反序列化仍然可用。

### 关系与图校验

直接引用声明的是稳定目标模型 ID。来源 crate 不必依赖那个目标：

```rust
use qubit_model_derive::Model;

#[Model(id = "example.Organization")]
struct Organization {
    #[identifier]
    id: i64,
}

#[Model(id = "example.Membership")]
struct Membership {
    #[reference(
        entity = "example.Organization",
        property = id,
        existing = true,
        path = "organization.id"
    )]
    organization_id: i64,
}
```

`lookup_relation(target = Organization, target_field = id)` 则要求目标类型已
在当前作用域中。`reference` 还可以用 `path` 指向本地字段路径。

宏只校验单个模型。链接完整模型集合后调用 `ModelRegistry::validate_graph()`，
才会检查目标是否存在、目标字段是否相容、`path`、查找关系、ownership、强
制引用环和 ownership 环。

### Opaque 字段与文本值对象

外部类型有意不提供 `HasTypeShape` 时，把字段标成 `opaque`：

```rust
use qubit_model_derive::Model;

struct ExternalToken;

#[Model(id = "example.ImportRecord")]
struct ImportRecord {
    #[opaque]
    token: ExternalToken,
}
```

opaque 字段会保留可识别的容器外层，叶子表现为 `TypeShape::Opaque`。它不能和
`text`、`sequence`、`map`、`time`、`decimal`、`money` 一起使用。

具名字段结构体若本身应具备文本能力，加上 `textual`。`String` 的 newtype 不
需要这个标记，会直接继承 `TypeCapabilities::TEXT`：

```rust
use qubit_model_derive::Model;

#[Model(id = "example.Phone", textual)]
struct Phone {
    country_area: Option<String>,
    city_area: Option<String>,
    number: String,
}

#[Model(id = "example.PhoneLoginParams")]
struct PhoneLoginParams {
    #[text(format = mobile)]
    mobile: Option<Phone>,
}
```

## 错误与诊断

失败会以编译错误标在出问题的语法上。常见原因如下：

| 诊断 | 检查什么 |
| --- | --- |
| 缺少 `qubit-model-metadata` 或 `serde` | 消费方 `[dependencies]`。 |
| 缺少 `qubit-redact` | 只有出现 `redact` 或 `#[redact(...)]` 时才需要。 |
| 把 `#[Model]` 用在枚举上 | 改成 `#[Enum]`。 |
| 把 `#[Enum]` 用在结构体上 | 改成 `#[Model]`。 |
| 缺少或重复的 `id` | 每个声明都要有一个 `id = "module.Type"`。 |
| ID 类型段不匹配 | 最后一段必须等于 Rust 类型名。 |
| 不支持的形状 | 不要写泛型、union 或多字段元组结构体。 |
| 属性作用域错误 | 模型级键只能写在 `#[Model(...)]` 里；字段辅助属性只能写在字段上。 |
| 仍在使用 `#[field(...)]` | 改用 `#[identifier]`、`#[text(...)]` 等独立属性。 |
| 类型能力不匹配 | `text` 需要具备文本能力的类型；`ignore_case` 同样如此。 |
| 枚举序列化名重复 | 两个变体收成了同一个 Serde 名。 |
| 结构体上写了 `no_copy` | 只允许出现在 `#[Enum]` 上。 |
| 在 `#[Model(...)]` 里写 `unique(...)` | 改用字段级 `#[unique(...)]`。 |

`nullable` 和 `computed` 会被明确拒绝：可空请用 `Option<T>`，不要用计算字段
代替真实字段。

decimal 的 `scale` 不能大于 `precision`。未知外部类型要么实现
`HasTypeShape`，要么显式选择 `opaque`。

## 排障

1. 确认消费方已经声明 `qubit-model-derive`、`qubit-model-metadata` 和
   `serde`。
2. 确认宏和形状对应：结构体用 `#[Model]`，枚举用 `#[Enum]`。
3. 字段约束失败时，看该类型的 `HasTypeShape` 能力，不要看类型名字符串。类型
   别名由 Rust 自己解析。
4. 关系在单个模型上看起来合法、但稍后失败时，对已链接集合调用
   `ModelRegistry::validate_graph()`。宏不会证明另一个 crate 里的目标 ID 一
   定存在。
5. 若 `name()` / `from_name()` 和手写映射对不上，检查 `#[serde(rename)]`，并
   记住默认是 `SCREAMING_SNAKE_CASE`，不是 Rust 标识符。

## 限制与最佳实践

让领域约束紧邻声明，通过 `qubit-model-metadata` 消费元数据。不要把本 crate
当成校验器、ORM 或 schema 导出器。

- `codec` 和 `generator` 只保存策略名。
- 敏感值处理用 `qubit-redact` 表达，没有 `sensitive` 字段属性。
- `ownership(owner = Type)` 写的是当前作用域中的 Rust 类型；`reference` 通过
  `entity = "module.Type"` 写稳定模型 ID 字符串。
- 单字段键优先用字段简写，复合主键和索引写到模型级参数。

## 延伸阅读

- [项目说明](../README.zh_CN.md)
- [runtime 元数据用户手册](../../rs-model-metadata/doc/user_guide.zh_CN.md)
- [脱敏运行时手册](https://github.com/qubit-ltd/rs-redact/blob/main/doc/user_guide.zh_CN.md)
- [API 文档](https://docs.rs/qubit-model-derive)
- [English user guide](user_guide.md)
