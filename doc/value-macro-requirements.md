# `#[Value]` 宏需求说明

本文记录 `qubit_model_derive::Value` 的产品语义、使用边界和验收要求。本文只定义需求，不规定 `rs-model-derive`、`rs-model-metadata` 或其他 crate 的具体重构与实现方式。

## 背景

平台中存在大量具有明确领域语义、但不具备独立模型身份的值类型，例如：

- 基于整数的修订号 `Revision(u64)`；
- 基于字符串的邮箱、邮政编码和规范化名称；
- 由多个字段组成的金额、地址或数值区间。

这些类型需要 `Debug`、`Display`、Serde、类型形状和字段约束等通用能力，也需要在 `#[Model]` 字段中被识别并递归查询。然而，它们不是独立业务模型，不应为了获得这些能力而声明稳定 `ModelId`、进入全局模型注册表或出现在模型迁移清单中。

`#[Value]` 用于统一声明此类值对象，减少每个类型手工实现通用 trait 和元数据能力的重复工作。

## 核心定义

`Value` 是具有领域语义和静态类型信息的可复用值类型。它可以描述自身结构、内容约束、处理策略和表示方式，但不具有独立 Model 身份。

`#[Value]` 声明的类型必须满足以下语义：

- 不要求或接受稳定 `ModelId`；
- 不加入全局 Model 注册表；
- 不出现在 Model 迁移清单或 Model 数量统计中；
- 能被 `#[Model]`、`#[Enum]` 和其他 `#[Value]` 字段识别为非 opaque 类型；
- 能公开自身静态类型形状；
- 多字段值对象能公开字段及其局部约束；
- 外层类型能够递归查询嵌套值对象的字段路径和约束；
- 不承载主键、索引、唯一性、所有权或跨 Model 关系语义。

## 第一版支持范围

第一版支持：

```rust
#[Value]
pub struct Revision(u64);

#[Value]
pub struct Money {
    #[money(precision = 18, scale = 2)]
    amount: BigDecimal,
    currency: Currency,
}
```

具体包括：

- 单字段 tuple newtype；
- 具名字段 struct；
- 值对象嵌套；
- `Option<T>`、集合、Map、数组及当前元数据系统支持的字段类型形状；
- 字段局部约束和策略元数据；
- 与 `#[Model]` 一致的默认 Serde、格式化和脱敏行为。

第一版不支持 enum。枚举继续使用 `#[Enum]`。第一版也不要求支持泛型值对象。

## 类型身份与递归查询

值对象只需要在当前 Rust 程序中通过类型身份进行识别，不需要业务级或跨版本稳定 ID。

当值对象作为其他类型的字段时，元数据系统必须能够区分以下情况：

- 已知标量值，例如 `Revision(u64)`；
- 已知多字段值，例如 `Address`；
- 显式声明为 `#[opaque]` 的外部类型；
- 未实现元数据能力、因而不能直接作为非 opaque 字段使用的类型。

多字段值对象的结构必须可递归查询。例如：

```rust
#[Value]
pub struct PostalCode {
    #[text(min_chars = 5, max_chars = 10, allowed_chars = ascii)]
    value: String,
}

#[Value]
pub struct Address {
    postal_code: PostalCode,
}

#[Model(id = "example.Organization")]
pub struct Organization {
    address: Address,
}
```

调用方应能从 `Organization` 的元数据沿 `address.postal_code.value` 解析到最终字段，并读取其 `text` 约束。值对象嵌套不应因为没有 `ModelId` 而中断字段路径解析。

## 默认 trait 与能力

`#[Value]` 默认能力应尽量与 `#[Model]` 保持一致，以避免同类声明产生不同的 Rust 行为。未通过开关关闭时，值对象默认获得：

- `Clone`；
- `Debug`；
- `Display`；
- `PartialEq` 和 `Eq`；
- `Hash`；
- `Serialize` 和 `Deserialize`。

`Display` 默认采用与 `#[Model]` struct 一致的 Debug-shaped 表示。例如：

```text
Revision(32)
Money { amount: 12.50, currency: CNY }
```

单字段 newtype 的默认 JSON 表示沿用其底层类型。例如 `Revision(32)` 序列化为 JSON 数字：

```json
32
```

`#[Value]` 不应擅自生成业务默认值，因此不默认实现 `Default`。调用方可以显式编写 `#[derive(Default)]` 或自定义实现。

宏应支持与 `#[Model]`、`#[Enum]` 一致的能力关闭开关，包括：

- `no_clone`；
- `no_debug`；
- `no_display`；
- `no_eq`；
- `no_partial_eq`；
- `no_hash`；
- `no_serialize`；
- `no_deserialize`。

如果统一重构后的公共能力集合还包含其他适用于 struct 的开关，`#[Value]` 应与 `#[Model]` 保持一致，除非值对象语义明确不适用。

## `copy` 开关

`#[Value]` 不根据字段类型名称猜测是否应该实现 `Copy`。需要值拷贝语义时，由声明者显式使用 `copy`：

```rust
#[Value(copy)]
pub struct Revision(u64);
```

`copy` 的要求是：

- 为值对象生成 `Copy`；
- 依赖 Rust 编译器验证全部字段确实支持 `Copy`；
- 字段不支持 `Copy` 时产生清晰的编译错误；
- 不改变 Serde、Display 或类型元数据语义。

## `transparent` 开关

`transparent` 仅适用于单字段 tuple newtype，表示该值允许与底层类型进行无校验的双向转换：

```rust
#[Value(copy, transparent)]
pub struct Revision(u64);
```

该声明应提供等价于以下转换能力：

```rust
impl From<u64> for Revision;
impl From<Revision> for u64;
```

`transparent` 的边界如下：

- 只能用于单字段 tuple newtype；
- 用于具名字段 struct、零字段类型或多字段 tuple struct 时必须编译失败；
- 表示转换不会执行校验；
- 不应自动用于所有 newtype；
- 有范围、格式或其他构造不变量的类型应保留显式构造函数，而不使用 `transparent`；
- 不改变默认 Serde 表示；
- 不把值对象变成 Model，也不产生稳定 ID。

## `textual` 能力

多字段值对象有时整体表现为一个文本值，例如拆分存储国家区号、地区号和本地号码的 `Phone`。此时允许使用：

```rust
#[Value(textual)]
pub struct Phone {
    country_area: Option<String>,
    city_area: Option<String>,
    number: String,
}
```

`textual` 表示该值对象整体具备文本能力，因此外层字段可以对它使用适用的文本约束。基于 `String` 的单字段 newtype 应从底层类型继承文本能力，不要求重复声明 `textual`。

## 属性分类与作用域

### 模型级属性

| 属性 | 分类 | `#[Value]` | 说明 |
| --- | --- | --- | --- |
| `id = "..."` | Model 身份 | 不支持 | Value 没有稳定 `ModelId` |
| `textual` | 值类型能力 | 支持 | 将具名字段值对象标记为文本型值 |
| `primary_key(...)` | 持久化约束 | 不支持 | Value 不声明主键 |
| `index(...)` | 持久化约束 | 不支持 | 索引由嵌入 Value 的 Model 决定 |
| `key(...)` | 持久化约束 | 不支持 | Value 不声明逻辑键 |
| `ownership(...)` | Model 关系 | 不支持 | Value 不参与所有权关系图 |

### 字段级属性

| 属性 | 分类 | `#[Value]` | 说明 |
| --- | --- | --- | --- |
| `#[text(...)]` | 值约束 | 支持 | 字符数、字节数、字符集、非空白和格式 |
| `#[decimal(...)]` | 值约束 | 支持 | 精度、小数位和舍入方式 |
| `#[money(...)]` | 值约束 | 支持 | 货币金额精度、小数位和舍入方式 |
| `#[sequence(...)]` | 值约束 | 支持 | 元素数量和元素唯一性 |
| `#[map(...)]` | 值约束 | 支持 | Map 条目数量 |
| `#[element(...)]` | 值约束 | 支持 | 序列元素的 `text` 或 `decimal` 约束 |
| `#[time(...)]` | 值约束 | 支持 | 时间精度 |
| `#[identifier]` | 持久化约束 | 不支持 | 单字段主键简写 |
| `#[unique(...)]` | 持久化约束 | 不支持 | 唯一性属于嵌入 Value 的 Model |
| `#[indexed]` | 持久化约束 | 不支持 | 索引属于嵌入 Value 的 Model |
| `#[reference(...)]` | Model 关系 | 不支持 | Value 不直接声明跨 Model 引用 |
| `#[lookup_relation(...)]` | Model 关系 | 不支持 | Value 不参与关系查找 |
| `#[codec = "..."]` | 值处理策略 | 支持 | 仅记录 codec 策略名称 |
| `#[generator = "..."]` | 值处理策略 | 支持 | 仅记录 generator 策略名称 |
| `#[opaque]` | 类型表示 | 支持 | 显式隐藏某个字段类型的内部结构 |
| `#[keep_serializing]` | Serde 行为 | 支持 | 保留空值的正常序列化表示 |
| `#[redact(...)]` | 安全表示 | 支持 | 使用现有脱敏规则 |
| `#[serde(...)]` | Serde 行为 | 支持 | 透传兼容的原生 Serde 配置 |

### 值约束参数

| 约束 | 当前支持的参数 |
| --- | --- |
| `text` | `min_chars`、`max_chars`、`min_bytes`、`max_bytes`、`allowed_chars = unicode\|ascii`、`non_blank`、`format = email\|mobile\|uri\|uuid` |
| `decimal` | `precision`、`scale`、`rounding = half_up\|half_even\|down\|up` |
| `money` | `precision`、必填的 `scale`、`rounding` |
| `sequence` | `min_items`、`max_items`、`unique_items` |
| `map` | `min_entries`、`max_entries` |
| `element` | `text(...)` 或 `decimal(...)` |
| `time` | `precision = second\|millisecond\|microsecond\|nanosecond` |
| `codec` | `codec = "name"` 或 `codec(name = "name")` |
| `generator` | `generator = "name"` 或 `generator(name = "name")` |

## Serde 与空值规则

`#[Value]` 应与 `#[Model]` 采用一致的 struct Serde 默认规则：

- 具名字段使用 `snake_case`；
- 可空性由最外层 `Option<T>` 表达；
- 不支持 `nullable` 属性；
- 对当前支持的 `Option<T>` 和空集合字段应用与 `#[Model]` 一致的缺省反序列化及空值省略规则；
- `#[keep_serializing]` 保留字段的正常序列化表示；
- 显式 `#[serde(...)]` 在不破坏宏约束的前提下继续生效；
- `transparent` 不改变 Serde 表示。

## 脱敏

值对象必须复用现有 `qubit-redact` 语义，不能维护第二套脱敏实现。类型级 `redact` 或字段级 `#[redact(...)]` 应影响 `Debug`、`Display` 和序列化，具体默认行为与 `#[Model]` 保持一致。

值对象嵌套在 Model 或另一个 Value 中时，不得因为外层自动生成格式化或序列化实现而泄露已标记的敏感字段。

## 编译期诊断

`#[Value]` 必须对以下错误给出明确、定位到声明位置的编译诊断：

- 用于 enum、union 或第一版不支持的类型形状；
- 声明 `id`；
- 使用主键、唯一性、索引、所有权或关系属性；
- 在非单字段 tuple newtype 上使用 `transparent`；
- 重复声明互斥或单例开关；
- 字段约束与字段类型能力不匹配；
- `text`、`decimal`、`money`、`sequence`、`map`、`element` 或 `time` 参数非法；
- 使用 `nullable`，并提示改用 `Option<T>`；
- 使用 `computed`，并提示声明真实 Rust 字段；
- 使用未知的 `no_*` 开关或未知属性。

宏不得静默忽略不支持的约束。

## 与 `#[Model]` 和 `#[Enum]` 的关系

三个宏的职责划分如下：

| 宏 | 主要用途 | 稳定 ID | 全局注册 | 结构与局部约束 |
| --- | --- | --- | --- | --- |
| `#[Value]` | 可复用值对象 | 无 | 否 | 是 |
| `#[Model]` | 独立业务模型 | 有 | 是 | 是 |
| `#[Enum]` | 枚举及变体元数据 | 沿用现有设计 | 沿用现有设计 | 是 |

`#[Model]` 和 `#[Enum]` 的字段可以直接使用 `#[Value]` 类型，不需要 `#[opaque]`。`#[Value]` 也可以嵌套其他 `#[Value]`、`#[Model]` 系统已知类型以及显式 opaque 类型，但值对象自身不得借此声明跨 Model 关系。

## 非目标

本需求不要求：

- 决定 runtime 元数据类型的具体拆分或命名；
- 决定现有 `TypeMetadata`、`ModelRegistration` 或注册表的重构步骤；
- 为 Value 生成稳定 ID 或伪 ID；
- 将 Value 加入模型迁移清单；
- 支持 enum；
- 第一版支持泛型值对象；
- 自动执行 codec、generator 或运行时业务校验；
- 自动为值对象生成 `Default`；
- 根据字段类型猜测 `Copy`；
- 为所有 newtype 自动生成底层类型转换。

## 验收标准

统一重构完成后，至少应满足以下可观察行为：

1. `Revision(u64)` 可以使用 `#[Value(copy, transparent)]` 声明，无需 `ModelId`。
2. `Revision` 的默认值可以由声明者显式实现为 `0`。
3. `Revision` 自动获得约定的 Debug、Display 和 Serde 行为，其中 JSON 表示为数字。
4. `transparent` 提供底层 `u64` 的双向 `From`，非 transparent newtype 不自动获得该转换。
5. 多字段 `Money` 或 `Address` 可以使用受支持的字段约束。
6. 外层 Model 能递归解析嵌套 Value 字段路径并读取最终字段约束。
7. Value 不出现在全局 Model 注册表、迁移清单或模型数量统计中。
8. Model 和 Enum 字段使用 Value 时不需要 `#[opaque]`。
9. Value 上的持久化约束和跨 Model 关系属性产生编译错误。
10. `copy`、`transparent`、Serde、脱敏、能力关闭开关和非法属性均有覆盖正常路径与错误路径的测试。
