# `qubit-model-derive` 目标 API 用户手册与参考

- 面向读者：使用 Qubit Rust 模型系统的领域模型开发者、框架开发者和代码生成器作者
- 文档状态：目标 API 审核稿
- 适用范围：角色化重构完成后的 `qubit-model-derive`、`qubit-model-metadata`，以及配套的
  `rs-validator`、`rs-codec`、`qubit-redact`
- 当前实现状态：本文描述最终目标契约；仓库当前代码尚未完整实现，不能把本文示例当作当前版本均可编译的保证
- 待确认项：所有尚未闭合的 runtime metadata 公共 API，见
  [待确认清单](2026-08-28-rs-model-derive-requirements-open-questions.md)

## 1. 这个库解决什么问题

Rust 类型能表达字段和枚举，却不会自动告诉框架：

- 哪个类型是有持久化身份的 Entity；
- 哪个字段是 identifier、唯一键、查询路径或另一 Entity 的引用；
- 字符串、金额、时间和容器必须满足什么约束；
- 一个类型是请求 DTO、Entity 的 Projection，还是按内容相等的 Value；
- 如何在运行时查询这些事实，并把它们交给 schema、validation、随机对象生成、DAO 测试或接口文档工具。

`qubit-model-derive` 提供五个角色宏和一组字段辅助属性，在编译期解析并验证声明；
`qubit-model-metadata` 保存宏产生的强类型静态 metadata，并提供静态查询与链接期注册表。

```text
领域类型
  │
  ├─ #[Entity] / #[Projection] / #[Model] / #[Enum] / #[Value]
  │       编译期解析、规范化、诊断、生成实现
  ▼
TypeMetadata + TypeDescriptor + FieldMetadata + PropertyMetadata
  │
  ├─ schema / API 文档
  ├─ 实例 validation
  ├─ 随机对象和 DAO 契约测试
  ├─ 查询条件生成
  └─ 跨 crate ModelRegistry 动态发现
```

宏不会执行数据库查询、网络调用或业务 service，也不会直接生成 SQL。它描述稳定领域语义，具体消费者各自决定
如何使用这些语义。

## 2. 五分钟理解：一个完整账号模型

下面的例子覆盖五种角色。先关注它们各自表达什么，后续章节会解释每个参数。

```rust
use qubit_id::Id;
use qubit_model_derive::{Entity, Enum, Model, ModelProperties, Projection, Value};

#[Value(transparent, id = "qubit.platform.iam.EmailAddress")]
pub struct EmailAddress(
    #[text(
        min_chars = 3,
        max_chars = 320,
        max_bytes = 320,
        non_blank,
        format = email,
    )]
    #[redact(level = "medium")]
    String,
);

#[Enum(id = "qubit.platform.iam.UserState")]
pub enum UserState {
    Pending,
    Active,
    #[variant(name = "LOCKED")]
    Locked,
}

#[Projection(
    id = "qubit.platform.iam.UserInfo",
    source = User,
)]
pub struct UserInfo {
    #[identifier]
    pub id: Id,
    pub username: String,
    #[redact(nested)]
    pub email: EmailAddress,
    pub state: UserState,
}

#[Entity(id = "qubit.platform.iam.User")]
pub struct User {
    #[identifier]
    pub id: Id,

    #[unique]
    #[text(min_chars = 3, max_chars = 32, allowed_chars = code)]
    pub username: String,

    #[redact(nested)]
    pub email: EmailAddress,

    #[redact(level = "high")]
    #[text(format = cn_mobile)]
    pub phone: Option<String>,

    #[indexed]
    pub state: UserState,

    #[indexed]
    pub created_at: DateTime,

    #[redact(skip)]
    pub password_hash: String,

    #[redact(skip)]
    pub internal_notes: Option<String>,
}

#[ModelProperties]
impl User {
    pub fn info(&self) -> UserInfo {
        UserInfo {
            id: self.id,
            username: self.username.clone(),
            email: self.email.clone(),
            state: self.state,
        }
    }
}

#[Model(id = "qubit.platform.iam.FindUserRequest")]
pub struct FindUserRequest {
    #[reference(entity = User, property = id)]
    pub user_id: Id,
}
```

这组声明表达以下事实：

- `User` 是唯一具有独立持久化身份的对象；`User.id` 是实例 identifier。
- `User` 还保存 phone、created_at、password_hash、internal_notes 等持久化或内部字段；`UserInfo` 刻意只暴露
  `id`、`username`、`email`、`state` 四个摘要字段。
- `UserInfo` 是由 `User` 派生的 Projection，借用同一个实例 ID，不产生独立持久化记录；Projection 不是 Entity
  的字段复制，而是面向特定读场景的派生表示。
- `FindUserRequest` 是数据契约，可以引用 Entity，但自身没有 identifier。
- `UserState` 是封闭值域。
- `EmailAddress` 是透明 Value，仍保留独立 Rust/领域类型身份，但外部表示与内部字符串一致。
- `username` 的 unique 默认忽略大小写，并隐含查询能力。
- `info` 没有同名存储字段，因此 metadata 自动把它识别为 computed property；不需要 `#[computed]`。

## 3. 先选择正确的角色

| 角色宏 | 领域语义 | 支持的 Rust 形状 | identifier | relation | `id` 参数与注册 |
| --- | --- | --- | --- | --- | --- |
| `#[Entity]` | 有独立身份和持久化生命周期 | 非泛型具名 struct | 恰好一个 | 可以 | `id` 必填，始终注册 |
| `#[Projection]` | Entity 实例的派生表示 | 非泛型具名 struct | 恰好一个 | 可以 | `id` 可选，有 id 才注册 |
| `#[Model]` | DTO、命令、配置、组合对象 | 具名或 unit struct；支持泛型 | 禁止 | 可以 | `id` 可选，有 id 才注册 |
| `#[Enum]` | 封闭值域或代数和 | unit/tuple/struct/mixed enum；支持泛型 | 禁止 | payload 禁止 direct relation | `id` 可选，有 id 才注册 |
| `#[Value]` | 按内容相等、无身份的值对象 | 具名 struct 或单字段 newtype；支持泛型 | 禁止 | 禁止 | `id` 可选，有 id 才注册 |

`id` 表示类型的稳定 `ModelId`；字段上的 `#[identifier]` 表示 Entity 实例身份。二者不是同一概念。

### 3.1 何时使用 Entity

对象拥有自己的 repository/DAO 生命周期、能被其他对象引用、并且在持久化中有独立记录时使用 Entity。

```rust
#[Entity(id = "qubit.platform.order.Order")]
pub struct Order {
    #[identifier(assigned_by = database)]
    pub id: Id,
    #[indexed]
    pub state: OrderState,
}
```

Entity 只支持具名字段 struct，不支持 tuple struct、unit struct、enum、泛型、lifetime 或 union。

### 3.2 何时使用 Projection

类型表示某个 Entity 实例的摘要、公开视图或查询投影，并与来源实例共享 identifier 时使用 Projection。

```rust
#[Projection(source = Order)]
pub struct OrderSummary {
    #[identifier]
    pub id: Id,
    pub total: Money,
}
```

省略 `source` 是开放 Projection；`source = Type` 或 `source_id = "..."` 是固定来源。source 只表达血缘与约束，
不会自动产生转换函数。一个 Entity 可以产生多个 Projection，因此 Entity 宏没有单数 `projection` 参数。

### 3.3 何时使用 Model

类型是请求、响应、命令、配置、查询条件、分页容器或对象图根节点，但没有持久化身份时使用 Model。

```rust
#[Model]
pub struct SearchUsers {
    pub query: Option<String>,
    #[reference(entity = Tenant, property = id)]
    pub tenant_id: Id,
}

#[Model]
pub struct RefreshCache;
```

Model 支持类型参数、`const N: usize` 和 where 子句，但不支持 lifetime：

```rust
#[Model(id = "qubit.commons.Page")]
pub struct Page<T>
where
    T: HasTypeDescriptor + 'static,
{
    pub items: Vec<T>,
    pub total: u64,
}
```

### 3.4 何时使用 Enum

```rust
#[Enum(id = "qubit.platform.payment.PaymentResult")]
pub enum PaymentResult<T> {
    Pending,
    Success(T),
    Failure {
        code: String,
        message: String,
    },
}
```

每个 variant 有 Rust 名、模型 canonical name 和 Serde wire name；三者可以不同。Enum payload 拥有完整字段
descriptor，但不能直接保存 Entity/Projection relation。

### 3.5 何时使用 Value

可复用、按内容相等、没有独立生命周期的领域值使用 Value，例如 email、phone、money、coordinate、revision。

```rust
#[Value(transparent, copy, ord)]
pub struct Revision(u64);

#[Value]
pub struct Coordinate {
    #[decimal(precision = 9, scale = 6)]
    pub latitude: Decimal,
    #[decimal(precision = 9, scale = 6)]
    pub longitude: Decimal,
}
```

Value 不能直接或间接包含 Entity、Projection、Model 或 reference；否则它不再是纯内容值。

## 4. 类型宏公共参数

### 4.1 稳定类型 ID

```rust
#[Entity(id = "qubit.platform.iam.User")]
struct User { /* ... */ }

#[Model(id = "FindUserRequest")]
struct FindUserRequest { /* ... */ }
```

`ModelId` 语法为：

```text
ModelId := Segment ("." Segment)*
Segment := [A-Za-z][A-Za-z0-9_]*
```

单段合法，不允许空段、前导点或尾随点。它是稳定协议标识，不要求末段与 Rust 类型名相同。

### 4.2 默认实现的能力

五种角色默认实现：

```text
Clone, Debug, Display, PartialEq, Eq, Hash,
Redact, Serialize, Deserialize
```

仅所有 variant 都是 unit 的 Enum 默认实现 `Copy`。默认不实现 `Default`、`PartialOrd`、`Ord`。

关闭默认能力：

```rust
#[Model(
    no_clone,
    no_debug,
    no_display,
    no_partial_eq,
    no_eq,
    no_hash,
    no_redact,
    no_serialize,
    no_deserialize,
)]
struct CustomContract { /* ... */ }
```

开启非默认能力：

```rust
#[Value(transparent, copy, default, ord)]
struct SequenceNumber(u64);
```

依赖规则：

- `Copy` 依赖 `Clone`；`copy + no_clone` 编译错误。
- `no_partial_eq` 同时移除 Eq、Hash、PartialOrd、Ord。
- `no_eq` 保留 PartialEq，但移除默认 Hash，并禁止 Ord。
- `ord` 同时启用 PartialEq、Eq、PartialOrd、Ord。
- `no_redact` 只允许完全没有字段脱敏规则的类型。
- `default` 只保证 Rust 默认值可构造，不保证值满足业务约束。
- 用户显式 `#[derive(...)]` 已含同一 trait 时，角色宏不得生成重复实现。

### 4.3 `Value(transparent)`

透明 Value 必须恰好有一个真实存储字段：

```rust
#[Value(transparent)]
struct EmailAddress(
    #[text(format = email)]
    #[redact(level = "medium")]
    String,
);
```

它仍是独立名义类型，但：

- Serialize/Deserialize 和 Display 使用内部值的形状；
- Debug 保留 `EmailAddress(...)` 类型名；
- Redact 应用唯一字段的策略；
- metadata 保留 Value 及其字段约束；
- 不自动实现 Deref、From、Into 或 TryFrom，因为构造内部值不一定满足约束。

## 5. Field 与 Property

Field 是 struct 中真实存在的存储字段。Property 是同名 field、getter、setter 合并后的可访问属性。

```rust
#[Model]
pub struct PersonName {
    first_name: String,
    last_name: String,
    display_name: String,
}

#[ModelProperties]
impl PersonName {
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    pub fn set_alias(&mut self, value: String) {
        self.display_name = value;
    }
}
```

| Property | field | getter | setter | readable | writable | storage kind |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| `first_name` | 是 | 否 | 否 | 是 | 是 | FieldBacked |
| `display_name` | 是 | 是 | 否 | 是 | 是 | FieldBacked |
| `full_name` | 否 | 是 | 否 | 是 | 否 | Computed |
| `alias` | 否 | 否 | 是 | 否 | 是 | Virtual |

规则为：

```text
is_readable = is_field || is_getter
is_writable = is_field || is_setter
```

getter 形状是 `pub fn name(&self) -> T`；setter 是 `pub fn set_name(&mut self, value: T)`。两者必须同步、safe、
非泛型。显式方法优先于宏生成的 field accessor。不存在 `#[computed]`：没有同名 field 的 getter 自动是 computed。

借用返回和 owned field 至少支持以下兼容关系：

```text
T          ↔ &T
String     ↔ str / &str
Vec<T>     ↔ [T] / &[T]
Option<T>  ↔ Option<&T>
```

## 6. 身份、查询与关系属性

### 6.1 `#[identifier]`

```rust
#[identifier]
pub id: Id,

#[identifier(assigned_by = database)]
pub id: Id,
```

Entity 和 Projection 各必须有且仅有一个直接 `Id` 字段。`Option<Id>`、容器、类型别名伪装或嵌套路径均非法。
`assigned_by` 默认 application；database 仅允许 Entity。identifier 隐含 indexed 查询能力，但根对象自己的
identifier 不进入 list filter，而由唯一查找 API 使用。

### 6.2 `#[indexed]`

```rust
#[indexed]
pub state: UserState,
```

它表示字段路径可形成查询过滤条件，不表示数据库必须创建单列物理索引。只支持无参数形式。
identifier、unique、reference 已隐含 indexed，再显式添加属于冗余错误。

复杂字段只沿内部有效 indexed 成员递归：

```rust
#[Model]
struct Category {
    #[indexed]
    code: String,
    title: String,
}

#[Entity(id = "example.Product")]
struct Product {
    #[identifier]
    id: Id,
    #[indexed]
    category: Category, // 形成 category.code，不形成 category.title
}
```

reference 图最多展开一跳。根对象引用 City 时可产生 `city.id/code/name` 条件，但不继续进入 City 引用的 Province。
结构化路径是规范身份，生成的平面名默认以 `_` 拼接；名称冲突必须编译报错。

### 6.3 `#[unique(...)]`

```rust
#[unique]
pub email: String, // 默认 ignore_case = true

#[unique(respect_to(tenant_id), ignore_case = false)]
pub employee_no: String,
```

`respect_to` 指定 scope 字段；省略表示全局唯一。ignore_case 只对 text-capable 字段有意义。unique 隐含 indexed。
全局 unique 不进入根对象 list filter；scoped unique 单字段仍可用于 list filter，完整字段组形成专用唯一查找键。

### 6.4 `#[reference(...)]`

```rust
#[reference(entity = User)]
pub owner: User,

#[reference(entity = User, property = id)]
pub owner_id: Id,

#[reference(entity_id = "qubit.platform.iam.User", property = info)]
pub owner_info: UserInfo,

#[reference(entity = User, existing = false)]
pub new_owner: User,

#[reference(entity = User, path = "owner")]
pub approver: User,
```

- `entity` 与 `entity_id` 必须二选一；前者编译期类型检查，后者在完整 registry 解析。
- `property` 省略时保存完整 Entity；它可以选择 field-backed、computed 或 virtual 以外的任意可读 property。
- `existing` 默认 true；false 表示目标不必预先持久化。
- `path` 复用当前对象图其他 reference 绑定的 Entity；它与 property 是不同维度。
- reference 隐含 indexed；Map 不能作为 reference 的直接保存形状。

### 6.5 `#[key_part(order = n)]`

```rust
#[Value]
struct Owner {
    #[key_part(order = 0)]
    kind: String,
    #[key_part(order = 1)]
    id: Id,
    label: Option<String>, // 不参与键投影
}
```

`key_part` 定义复杂 Model/Value 的部分字段如何稳定投影为 key components。order 必须从 0 连续、无重复、无缺号。
它服务于复杂 unique、respect_to、随机去重和重复键诊断，不创建数据库索引，也不是通用序列化协议。

## 7. 值约束属性

### 7.1 `#[text(...)]`

```rust
#[text(
    min_chars = 3,
    max_chars = 32,
    max_bytes = 64,
    non_blank,
    allowed_chars = code,
)]
pub username: String,

#[text(format = email)]
pub email: String,
```

参数：

| 参数 | 含义 |
| --- | --- |
| `min_chars/max_chars` | Unicode scalar value 数量上下界 |
| `min_bytes/max_bytes` | UTF-8 字节数上下界 |
| `non_blank` | 拒绝空串及全 Unicode whitespace |
| `format` | `email`、`cn_mobile`、`uri`、`uuid` |
| `allowed_chars` | `unicode`、`printable_unicode`、`ascii`、`printable_ascii`、`code` |

默认 allowed_chars 为 unicode。`code` 表示 `[A-Za-z0-9_-]`，不限制首字符也不隐含 non_blank。
完全无参数的 `#[text]` 非法；显式 `allowed_chars = unicode` 合法。

### 7.2 `#[decimal(...)]`

```rust
#[decimal(
    precision = 8,
    scale = 4,
    min = "0",
    max = "1",
    rounding = half_even,
)]
pub ratio: Decimal,
```

仅适用于精确 decimal 类型，不适用于 `f32/f64`。min/max 用字符串避免浮点损失；边界默认包含。
rounding 可为 up、down、ceiling、floor、half_up、half_down、half_even、unnecessary。validator 不修改值；
codec/解析器可在构造对象前规范化。

### 7.3 `#[money(...)]`

```rust
#[money(
    precision = 12,
    scale = 2,
    min = "0",
    rounding = unnecessary,
)]
pub amount: Decimal,
```

money 与 decimal 参数体系一致，但 scale 必填，metadata semantic 为 Money，默认 rounding 为 unnecessary。
它不包含 currency、货币符号或分组显示；这些是独立领域数据或输出表示。money 与 decimal 不能同时出现。

### 7.4 `#[time(...)]`

```rust
#[time(precision = millisecond)]
pub created_at: DateTime,
```

precision 必填，可为 second、millisecond、microsecond、nanosecond。值必须能被该精度准确表示。该属性不表达时区、
过去/未来或跨字段先后关系。

### 7.5 `#[sequence(...)]` 与 `#[element(...)]`

```rust
#[sequence(min_items = 1, max_items = 10, unique_items)]
#[element(
    text(max_chars = 32),
    validator(id = "qubit.tag.syntax"),
    redact(level = "low"),
)]
pub tags: Vec<String>,
```

sequence 约束容器大小和元素相等性唯一；element 约束第一层元素。Set 天然唯一，不能再写 unique_items；固定数组长度
由 Rust 类型决定，不能写 min/max。element 不继续嵌套 element/map，深层结构应定义具名 Value。

### 7.6 `#[map(...)]`、`#[map_key(...)]`、`#[map_value(...)]`

```rust
#[map(min_entries = 1, max_entries = 20)]
#[map_key(text(allowed_chars = code, max_chars = 32))]
#[map_value(
    text(max_chars = 256),
    validator(id = "qubit.attribute.value"),
    redact(level = "medium"),
)]
pub attributes: HashMap<String, String>,
```

map 只约束 entry 数；key 唯一由 Map 类型保证。map_key/map_value 分别选择第一层 key/value，并可组合 text、decimal、
money、time、validator、codec、redact。每个 selector 最多一个 codec 和 redact，但可有多个不同 validator。

## 8. 自定义策略

### 8.1 字段 validator

validator 只能检查由当前对象值本身确定的语法、格式、结构与字段间一致性。它不能访问数据库、repository、网络、
权限或库存，不能判断 username 是否已占用。

定义 validator：

```rust
#[Validator(id = "qubit.identity.card", value = str)]
pub struct IdentityCardValidator;

impl IdentityCardValidator {
    fn validate(
        &self,
        value: &str,
        context: &ValidationContext<'_>,
    ) -> ValidationResult {
        // 检查号码结构、校验位，以及显式依赖字段的一致性。
        todo!()
    }
}
```

使用 validator：

```rust
#[validator(
    id = "qubit.identity.card",
    depends_on(gender, birthday),
    params(strict = true, regions = ["44", "45"]),
)]
pub identity_card: String,
```

`params` 只支持 bool、整数、字符串及同类型数组；精确 decimal、时间等用字符串表示。多个 validator 按源码顺序产生
violation。具体 trait、registry 和大写宏属于 `rs-validator`；derive crate 只保存 occurrence metadata。

### 8.2 Value codec

定义文本 codec：

```rust
#[ValueCodec(
    id = "qubit.contact.phone",
    value = Phone,
    error = PhoneCodecError,
)]
pub struct PhoneCodec;

impl PhoneCodec {
    fn encode(&mut self, value: &Phone) -> Result<String, PhoneCodecError> {
        todo!()
    }

    fn decode(&mut self, input: &str) -> Result<Phone, PhoneCodecError> {
        todo!()
    }
}
```

声明 canonical codec 或字段覆盖：

```rust
#[Value(transparent, codec = PhoneCodec)]
pub struct Phone(String);

#[Model]
pub struct Contact {
    pub phone: Phone,
    #[codec(id = "qubit.contact.phone.international")]
    pub international_phone: Phone,
}
```

优先级为字段显式 codec、类型 canonical codec、无 codec。字段不能带 params；需要不同配置时定义不同 codec 类型和 ID。
具体宏、trait 和 ValueCodecRegistry 属于 `rs-codec`。

### 8.3 `#[opaque]`

```rust
struct ExternalKeyMaterial;

#[opaque]
pub material: Option<ExternalKeyMaterial>,
```

opaque 保留 Option/容器等外层结构，但停止叶子 descriptor 递归；默认 validator 不进入叶子，默认生成器也不会构造它。
调用方必须供值或提供独立生成 adapter。opaque 不能与 identifier/reference 组合，也不能隐藏领域角色来绕过检查。

最终 API 不提供字段级 `#[generator]`。

## 9. 脱敏和序列化

### 9.1 `#[redact(...)]`

字段必须且只能选择一种模式：

```rust
#[redact(level = "high")] pub phone: String,
#[redact(skip)] pub internal_token: String,
#[redact(nested)] pub email: EmailAddress,
#[redact(map)] pub dynamic_values: HashMap<String, String>,
#[redact(keyed_by = credential_type)] pub credential: String,
#[redact(json)] pub raw_payload: String,
```

字段级 redact 可穿透 Option、Box/Rc/Arc、sequence、set 和 array。Map 字段级策略默认只进入 value；修改 key 必须
显式使用 map_key selector。`redact(skip)` 只能省略整个字段，不能用于 element/map_key/map_value。

### 9.2 Serde 和 `#[keep_serializing]`

标准 `#[serde(...)]` 完整保留，显式配置优先。角色宏默认对具名字段省略 `Option::None` 和空标准集合，反序列化缺失
时补默认值。

```rust
#[keep_serializing]
pub aliases: Vec<String>, // 空 Vec 仍输出 []

#[keep_serializing]
pub nickname: Option<String>, // None 仍输出 null
```

`keep_serializing` 只关闭宏自动添加的 skip_serializing_if，不关闭反序列化缺失默认，也不覆盖用户显式 serde skip。
固定数组、newtype、tuple payload 不自动省略位置。

默认 Debug、Display、Serialize 执行字段脱敏；Deserialize 只负责输入，不脱敏。

## 10. Enum variant API

```rust
#[Enum]
pub enum ReviewState {
    InReview,
    #[variant(name = "APPROVED")]
    #[serde(rename = "accepted")]
    Approved,
}
```

对于 `Approved`：

```text
rust_name       = "Approved"
canonical name  = "APPROVED"
serialized_name = "accepted"
```

`#[variant]` 只支持 name。默认 canonical name 是 SCREAMING_SNAKE_CASE；同一 Enum 中不能重复或为空。

## 11. 递归与作用位置

`Option`、`Box`、`Rc`、`Arc` 是透明包装；sequence、set、array、map 是有语义的容器。

| 写法 | 作用位置 |
| --- | --- |
| `#[validator(...)] Vec<T>` | 整个 `Vec<T>` |
| `#[element(validator(...))] Vec<T>` | 每个 T |
| `#[codec(...)] Option<T>` | Some 内的 T；None 跳过 |
| `#[text(...)] Vec<String>` | 非法，不自动逐元素 |
| `#[element(text(...))] Vec<String>` | 每个 String |
| `#[redact(...)] Vec<String>` | 自动穿透至元素 |
| `#[redact(...)] HashMap<K, V>` | 默认处理 value |
| `#[map_key(redact(...))] HashMap<K, V>` | 显式处理 key |

未 opaque 的命名 Model/Value/Enum 无论位于字段、Option、元素还是 Map key/value，都按自身 descriptor 递归。

## 12. 运行时 metadata API

本章是运行时 metadata 的目标公共 API 参考。它只描述重构完成后的查询接口，不代表当前代码已经实现这些接口。

为避免把尚未决定的接口伪装成正式承诺，本章采用两种标记：

- **已确认**：讨论记录中已经明确确认，可以作为目标 API；
- **待确认占位符**：只确定了类型的职责或使用位置，精确接口尚未确认。每个占位符都对应
  [待确认清单](2026-08-28-rs-model-derive-requirements-open-questions.md) 中的唯一编号。

### 12.1 API 全景

运行时 metadata 按职责分为六层：

| 层次 | 主要公共类型 | 回答的问题 |
| --- | --- | --- |
| 类型描述 | `TypeDescriptor`、`TypeMetadata` | 这是哪一个 Rust 类型？它的结构是什么？它是否是五种模型角色之一？ |
| 成员描述 | `FieldMetadata`、`PropertyMetadata`、`GetterMetadata`、`SetterMetadata` | 类型有哪些存储字段和逻辑 Property？它们是否可读、可写？ |
| 角色描述 | `EntityMetadata`、`ProjectionMetadata`、`ModelMetadata`、`EnumMetadata`、`ValueMetadata` | 只有某个模型角色才具有哪些语义？ |
| 字段语义 | identifier、index、unique、reference、constraint、validator、codec、redact metadata | 一个字段具有哪些查询、约束、策略和安全输出语义？ |
| 泛型描述 | `GenericTypeMetadata` 及泛型参数 metadata | concrete 泛型实例来自哪个定义？类型实参如何代入字段描述？ |
| 动态发现 | `ModelRegistry`、resolver、相关错误类型 | 只知道稳定 ID 时如何找到模型？如何完成跨 crate 引用校验？ |

它们的主要导航关系如下：

```text
TypeDescriptor
  └─ metadata() ───────────────> TypeMetadata（仅五种角色类型存在）
                                   ├─ fields() ─────> FieldMetadata
                                   ├─ properties() ─> PropertyMetadata
                                   ├─ role_metadata() -> RoleMetadata
                                   └─ generic_definition() -> GenericTypeMetadata

FieldMetadata
  ├─ descriptor() ─────────────> TypeDescriptor
  ├─ identifier()/unique()/reference()
  └─ constraints()/validators()/codec()/redact()

PropertyMetadata
  ├─ descriptor() ─────────────> TypeDescriptor
  ├─ field() ──────────────────> FieldMetadata
  ├─ getter() ─────────────────> GetterMetadata
  └─ setter() ─────────────────> SetterMetadata

ModelRegistry / Resolver
  └─ stable ID ────────────────> TypeMetadata 或策略 metadata
```

公开 API 又分成两类：

- 普通开发者使用本章列出的只读查询 API；
- 派生宏生成代码所需的构造和注册入口放在 `qubit_model_metadata::__private` 等隐藏生产接口中。隐藏接口不是普通
  用户 API，不应由业务代码手写调用。

### 12.2 两个静态查询入口（已确认）

#### 12.2.1 已知五种角色类型：`TypeMetadata::of`

```rust
impl TypeMetadata {
    pub fn of<T>() -> &'static TypeMetadata
    where
        T: HasTypeMetadata + 'static;
}
```

这个入口只接受由 Entity、Projection、Model、Enum、Value 声明的领域类型：

```rust
let entity = TypeMetadata::of::<User>();
let projection = TypeMetadata::of::<UserInfo>();
let request = TypeMetadata::of::<LoginRequest>();
let state = TypeMetadata::of::<ReviewState>();
let value = TypeMetadata::of::<EmailAddress>();
```

以下调用必须编译失败，因为 `String` 不是五种角色类型：

```rust,compile_fail
let metadata = TypeMetadata::of::<String>();
```

静态查询不依赖类型是否注册，因此没有 `ModelId` 的局部 Model 仍然可以查询：

```rust
#[Model]
struct LocalRequest {
    username: String,
}

let metadata = TypeMetadata::of::<LocalRequest>();
assert_eq!(metadata.model_id(), None);
assert!(!metadata.is_registered());
```

系统不再提供同义的 `metadata_of::<T>()` 自由函数，也不向 `User` 注入 `User::metadata()` 固有方法。

#### 12.2.2 已知任意可描述 Rust 类型：`TypeDescriptor::of`

```rust
impl TypeDescriptor {
    pub fn of<T>() -> &'static TypeDescriptor
    where
        T: HasTypeDescriptor + 'static;

    pub fn metadata(&self) -> Option<&'static TypeMetadata>;
}
```

`TypeDescriptor` 的范围大于 `TypeMetadata`：

```rust
let scalar = TypeDescriptor::of::<String>();
let optional = TypeDescriptor::of::<Option<EmailAddress>>();
let collection = TypeDescriptor::of::<Vec<UserInfo>>();
let generic = TypeDescriptor::of::<Page<UserInfo>>();

assert!(scalar.metadata().is_none());
assert_eq!(
    TypeDescriptor::of::<User>().metadata().unwrap().role(),
    ModelRole::Entity,
);
```

它必须能够描述 scalar、透明包装、sequence、set、array、map、tuple、`Box`/`Rc`/`Arc`、五种角色、opaque、
泛型参数和 concrete 泛型实例。

> **待确认占位符 `META-API-TODO-001`：** `TypeDescriptor` 除 `of()` 和 `metadata()` 之外的完整结构查询接口尚未
> 定型，包括公开结构枚举、容器导航、类型身份、能力查询及 opaque 查询。这里暂不猜测方法名称和返回类型。

> **待确认占位符 `META-API-TODO-002`：** `TypeCapabilities` 的完整 flag 集合、查询入口以及“Rust 实现能力”和
> “字段约束 capability”是否使用同一类型尚未定型。

### 12.3 `TypeMetadata` 完整公共接口（已确认部分）

`TypeMetadata` 的公共查询接口必须集中列出，不能把字段和 Property 导航拆散到其他小节后假定读者能够自行补齐：

```rust
use std::any::TypeId;

impl TypeMetadata {
    // 静态入口
    pub fn of<T>() -> &'static TypeMetadata
    where
        T: HasTypeMetadata + 'static;

    // 当前程序内的 Rust 类型身份
    pub fn type_id(&self) -> TypeId;
    pub fn type_name(&self) -> &'static str;

    // 跨程序稳定身份和注册状态
    pub fn model_id(&self) -> Option<ModelId>;
    pub fn generic_definition(&self) -> Option<&'static GenericTypeMetadata>;
    pub fn is_registered(&self) -> bool;

    // 存储字段
    pub fn fields(&self) -> &[FieldMetadata];
    pub fn field(&self, name: &str) -> Option<&FieldMetadata>;
    pub fn field_at(&self, index: usize) -> Option<&FieldMetadata>;

    // 逻辑 Property
    pub fn properties(&self) -> &[PropertyMetadata];
    pub fn property(&self, name: &str) -> Option<&PropertyMetadata>;

    // 角色判断和角色专属数据
    pub fn role(&self) -> ModelRole;
    pub fn role_metadata(&self) -> &RoleMetadata;
    pub fn as_entity(&self) -> Option<&EntityMetadata>;
    pub fn as_projection(&self) -> Option<&ProjectionMetadata>;
    pub fn as_model(&self) -> Option<&ModelMetadata>;
    pub fn as_enum(&self) -> Option<&EnumMetadata>;
    pub fn as_value(&self) -> Option<&ValueMetadata>;
}
```

#### 12.3.1 身份的区别

```rust
let metadata = TypeMetadata::of::<User>();

assert_eq!(metadata.type_id(), std::any::TypeId::of::<User>());
assert!(metadata.type_name().ends_with("::User"));
assert_eq!(
    metadata.model_id().unwrap().as_str(),
    "qubit.platform.iam.User",
);
```

- `type_id()` 只在当前编译产物内标识 concrete Rust 类型；
- `type_name()` 只用于日志和诊断，字符串格式不是稳定协议；
- `model_id()` 才是跨进程、跨语言和跨版本进行动态发现时使用的稳定身份。

#### 12.3.2 字段集合

```rust
let metadata = TypeMetadata::of::<User>();

let all_fields: &[FieldMetadata] = metadata.fields();
let username = metadata.field("username").unwrap();
assert_eq!(metadata.field_at(username.index()).unwrap().name(), Some("username"));
assert!(metadata.field("missing").is_none());
```

字段规则：

- Entity、Projection、具名 Model 和具名 Value 返回所有真实存储字段，包括 private 字段；
- unit Model 返回空切片；
- 单字段 tuple Value 返回一个 `index() == 0`、`name() == None` 的字段；
- Enum 顶层 `fields()` 返回空切片，variant payload 字段从 `EnumVariantMetadata` 获取；
- `field(name)` 只查询具名字段；`field_at(index)` 按 Rust 声明顺序查询。

#### 12.3.3 Property 集合

```rust
let metadata = TypeMetadata::of::<User>();

let properties: &[PropertyMetadata] = metadata.properties();
let info = metadata.property("info").unwrap();

assert_eq!(info.name(), "info");
assert!(metadata.property("missing").is_none());
```

每个字段都会形成同名 field-backed Property；`#[ModelProperties]` 中识别出的 getter/setter 再按名称并入该集合。

### 12.4 公共角色导航（已确认）

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelRole {
    Entity,
    Projection,
    Model,
    Enum,
    Value,
}

pub enum RoleMetadata {
    Entity(EntityMetadata),
    Projection(ProjectionMetadata),
    Model(ModelMetadata),
    Enum(EnumMetadata),
    Value(ValueMetadata),
}
```

三种访问方式分别适合不同场景：

```rust
let metadata = TypeMetadata::of::<User>();

// 只需要分支判断
assert_eq!(metadata.role(), ModelRole::Entity);

// 需要穷举处理角色
match metadata.role_metadata() {
    RoleMetadata::Entity(entity) => {
        // 使用 Entity 专属 metadata
    }
    RoleMetadata::Projection(_) => {}
    RoleMetadata::Model(_) => {}
    RoleMetadata::Enum(_) => {}
    RoleMetadata::Value(_) => {}
}

// 调用方已经预期某个角色
assert!(metadata.as_entity().is_some());
assert!(metadata.as_projection().is_none());
```

角色不匹配必须返回 `None`；系统不提供 `unwrap_entity()` 一类 panic 型便利接口。

### 12.5 `FieldMetadata` 完整基础接口（已确认）

```rust
impl FieldMetadata {
    pub fn index(&self) -> usize;
    pub fn name(&self) -> Option<&'static str>;
    pub fn descriptor(&self) -> &'static TypeDescriptor;
    pub fn visibility(&self) -> FieldVisibility;
    pub fn attributes(&self) -> &[FieldAttributeMetadata];

    pub fn identifier(&self) -> Option<&IdentifierMetadata>;
    pub fn is_identifier(&self) -> bool;

    pub fn is_indexed(&self) -> bool;
    pub fn indexing_reasons(&self) -> IndexingReasons;

    pub fn unique(&self) -> Option<&UniqueMetadata>;
    pub fn is_unique(&self) -> bool;

    pub fn reference(&self) -> Option<&ReferenceMetadata>;
    pub fn is_reference(&self) -> bool;

    pub fn constraints(&self) -> &[ConstraintMetadata];
    pub fn validators(&self) -> &[ValidatorMetadata];
    pub fn codec(&self) -> Option<&CodecMetadata>;
    pub fn redact(&self) -> Option<&RedactMetadata>;
}
```

#### 12.5.1 字段位置、名称和类型

```rust,ignore
let field = TypeMetadata::of::<User>().field("username").unwrap();

assert_eq!(field.name(), Some("username"));
assert_eq!(
    field.descriptor().type_id(), // 此方法名尚受 META-API-TODO-001 约束
    std::any::TypeId::of::<String>(),
);
```

上例最后一条断言表达所需能力，但 `TypeDescriptor` 的 `type_id()` 是否采用该名称尚未确认；在
`META-API-TODO-001` 关闭前，该行不是最终可编译承诺。

#### 12.5.2 便利判断的严格等价关系

```rust
assert_eq!(field.is_identifier(), field.identifier().is_some());
assert_eq!(field.is_unique(), field.unique().is_some());
assert_eq!(field.is_reference(), field.reference().is_some());
```

#### 12.5.3 索引原因

```rust
bitflags! {
    pub struct IndexingReasons: u8 {
        const EXPLICIT   = 0b0001;
        const IDENTIFIER = 0b0010;
        const UNIQUE     = 0b0100;
        const REFERENCE  = 0b1000;
    }
}
```

一个字段可以同时因为多个隐含语义而可查询：

```rust
let reasons = field.indexing_reasons();

if reasons.contains(IndexingReasons::UNIQUE | IndexingReasons::REFERENCE) {
    // 同时具有 unique 和 reference 语义
}

assert_eq!(field.is_indexed(), !reasons.is_empty());
```

合法声明中，显式 `#[indexed]` 不得与 identifier、unique、reference 所隐含的 indexed 重复；多个隐含原因可以并存。

#### 12.5.4 字段可见性

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldVisibility {
    Public,
    Crate,
    Super,
    Path(&'static str),
    Private,
}
```

| Rust 声明 | metadata |
| --- | --- |
| `pub field: T` | `Public` |
| `pub(crate) field: T` 或 `pub(in crate) field: T` | `Crate` |
| `pub(super) field: T` 或 `pub(in super) field: T` | `Super` |
| `pub(in crate::internal) field: T` | `Path("crate::internal")` |
| `pub(self) field: T`、`pub(in self) field: T` 或无 `pub` | `Private` |

可见性只记录源码声明，不限制 metadata 查询，也不直接决定 field-backed Property 是否可读或可写。

> **待确认占位符 `META-API-TODO-005`：** `attributes() -> &[FieldAttributeMetadata]` 已经确认保留；
> `FieldAttributeMetadata` 自身是归一化枚举还是其他只读视图、包含哪些 variant，以及如何导航到强类型 metadata 尚未定型。

> **待确认占位符 `META-API-TODO-006`～`META-API-TODO-012`：** `IdentifierMetadata`、`UniqueMetadata`、
> `ReferenceMetadata`、`ConstraintMetadata`、`ValidatorMetadata`、`CodecMetadata`、`RedactMetadata` 的职责已经在前文
> 确定，但各类型完整 getter、关联 ID 类型、参数视图和解析后状态尚未定型。

### 12.6 `PropertyMetadata` 完整基础接口（已确认）

```rust
impl PropertyMetadata {
    pub fn name(&self) -> &'static str;
    pub fn descriptor(&self) -> &'static TypeDescriptor;

    pub fn field(&self) -> Option<&FieldMetadata>;
    pub fn getter(&self) -> Option<&GetterMetadata>;
    pub fn setter(&self) -> Option<&SetterMetadata>;

    pub fn is_field(&self) -> bool;
    pub fn is_getter(&self) -> bool;
    pub fn is_setter(&self) -> bool;

    pub fn is_readable(&self) -> bool;
    pub fn is_writable(&self) -> bool;
    pub fn is_computed(&self) -> bool;

    pub fn storage_kind(&self) -> PropertyStorageKind;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropertyStorageKind {
    FieldBacked,
    Computed,
    Virtual,
}
```

#### 12.6.1 Field-backed Property

```rust
let username = TypeMetadata::of::<User>()
    .property("username")
    .unwrap();

assert!(username.is_field());
assert!(username.is_readable());
assert!(username.is_writable());
assert_eq!(username.storage_kind(), PropertyStorageKind::FieldBacked);
```

只有 field、没有显式 getter/setter 时：

```text
is_field()    = true
is_getter()   = false
is_setter()   = false
is_readable() = true
is_writable() = true
```

#### 12.6.2 Computed Property

```rust
#[ModelProperties]
impl User {
    pub fn display_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}
```

```rust
let display_name = TypeMetadata::of::<User>()
    .property("display_name")
    .unwrap();

assert!(!display_name.is_field());
assert!(display_name.is_getter());
assert!(display_name.is_readable());
assert!(!display_name.is_writable());
assert!(display_name.is_computed());
assert_eq!(display_name.storage_kind(), PropertyStorageKind::Computed);
```

#### 12.6.3 只有 setter 的 Virtual Property

```rust
#[ModelProperties]
impl User {
    pub fn set_password(&mut self, password: String) {
        // 转换后写入其他存储字段
    }
}
```

```rust
let password = TypeMetadata::of::<User>()
    .property("password")
    .unwrap();

assert!(!password.is_field());
assert!(!password.is_getter());
assert!(password.is_setter());
assert!(!password.is_readable());
assert!(password.is_writable());
assert_eq!(password.storage_kind(), PropertyStorageKind::Virtual);
```

便利方法必须满足：

```rust
assert_eq!(property.is_field(), property.field().is_some());
assert_eq!(property.is_getter(), property.getter().is_some());
assert_eq!(property.is_setter(), property.setter().is_some());
```

一个 Property 可以同时具有 field、getter 和 setter；三者不是互斥 variant。

> **待确认占位符 `META-API-TODO-004`：** `GetterMetadata` 与 `SetterMetadata` 的类型信息、方法来源信息以及类型擦除
> 读写入口尚未定型。尤其需要单独确定 borrowed/owned getter、setter 入参所有权、失败类型和线程安全边界。

### 12.7 五种角色的专属 metadata（待确认）

本节只列出上一轮讨论形成的候选接口，全部受 `META-API-TODO-013` 约束，尚不是最终承诺。

```rust,ignore
impl EntityMetadata {
    pub fn model_id(&self) -> ModelId;
    pub fn identifier(&self) -> &'static FieldMetadata;
}

pub enum ProjectionSource {
    Type(&'static TypeMetadata),
    Id(ModelId),
}

impl ProjectionMetadata {
    pub fn identifier(&self) -> &'static FieldMetadata;
    pub fn source(&self) -> Option<&ProjectionSource>;
    pub fn is_open(&self) -> bool;
    pub fn is_fixed(&self) -> bool;
}

pub struct ModelMetadata {
    // 候选设计：首版没有角色专属公开属性。
}

impl EnumMetadata {
    pub fn variants(&self) -> &[EnumVariantMetadata];
    pub fn variant(&self, name: &str) -> Option<&EnumVariantMetadata>;
    pub fn variant_by_rust_name(&self, rust_name: &str) -> Option<&EnumVariantMetadata>;
    pub fn variant_by_serialized_name(&self, serialized_name: &str)
        -> Option<&EnumVariantMetadata>;
}

impl EnumVariantMetadata {
    pub fn index(&self) -> usize;
    pub fn rust_name(&self) -> &'static str;
    pub fn name(&self) -> &'static str;
    pub fn serialized_name(&self) -> &'static str;
    pub fn kind(&self) -> EnumVariantKind;
    pub fn fields(&self) -> &[FieldMetadata];
    pub fn field(&self, name: &str) -> Option<&FieldMetadata>;
    pub fn field_at(&self, index: usize) -> Option<&FieldMetadata>;
    pub fn is_default(&self) -> bool;
}

pub enum EnumVariantKind {
    Unit,
    Tuple,
    Struct,
}

impl ValueMetadata {
    pub fn is_transparent(&self) -> bool;
    pub fn transparent_field(&self) -> Option<&'static FieldMetadata>;
}
```

候选语义示例：

```rust,ignore
let entity = TypeMetadata::of::<User>().as_entity().unwrap();
assert_eq!(entity.identifier().name(), Some("id"));

let projection = TypeMetadata::of::<UserInfo>()
    .as_projection()
    .unwrap();
assert_eq!(projection.is_open(), projection.source().is_none());
assert_eq!(projection.is_fixed(), projection.source().is_some());

let value = TypeMetadata::of::<EmailAddress>().as_value().unwrap();
assert_eq!(value.is_transparent(), value.transparent_field().is_some());
```

`ProjectionMetadata::source()` 只返回声明事实，不得隐式读取全局注册表。把 `ProjectionSource::Id` 解析成 Entity 必须显式
调用 resolver。

### 12.8 字段语义 metadata（职责已确定，接口待确认）

这些类型不是实现细节，因为它们直接从 `FieldMetadata` 的公共方法返回。最终 API 必须为每个类型列出完整接口；在接口
确认前，本节只记录已经确定的职责，不能据此猜测方法名。

| TODO | 类型 | 已确定的职责 | 取得入口 |
| --- | --- | --- | --- |
| `META-API-TODO-006` | `IdentifierMetadata` | 标识 Entity/Projection 的唯一直接 `Id` 字段 | `FieldMetadata::identifier()` |
| `META-API-TODO-007` | `UniqueMetadata` | 描述字段级唯一语义，不承载物理数据库索引参数 | `FieldMetadata::unique()` |
| `META-API-TODO-008` | `ReferenceMetadata` | 描述目标 Entity、selection 和 binding 等关联事实 | `FieldMetadata::reference()` |
| `META-API-TODO-009` | `ConstraintMetadata` | 以强类型方式表达 text、decimal、money、time、sequence、element、map 等约束 | `FieldMetadata::constraints()` |
| `META-API-TODO-010` | `ValidatorMetadata` | 保存 validator 稳定 ID、静态参数和显式依赖 | `FieldMetadata::validators()` |
| `META-API-TODO-011` | `CodecMetadata` | 保存 Value codec 稳定 ID、静态参数和方向 | `FieldMetadata::codec()` |
| `META-API-TODO-012` | `RedactMetadata` | 保存脱敏策略及 selector 作用位置 | `FieldMetadata::redact()` |

例如，调用方最终应能沿强类型接口完成下列任务，但方框内的具体 getter 仍是占位符：

```rust,ignore
let field = TypeMetadata::of::<User>().field("username").unwrap();

if let Some(unique) = field.unique() {
    // [META-API-TODO-007] 读取唯一比较方式等已声明事实。
}

for constraint in field.constraints() {
    // [META-API-TODO-009] 模式匹配具体约束类型。
}

for validator in field.validators() {
    // [META-API-TODO-010] 读取稳定 StrategyId、params 和 depends_on。
}
```

`QueryMetadata` 用于汇总可查询路径，而不是塞入 `EntityMetadata`。其拥有者、查询入口和路径条目 API 受
`META-API-TODO-014` 约束。

### 12.9 泛型 metadata（行为已确认，接口待确认）

泛型 Model、Enum、Value 的稳定 ID 标识泛型定义。链接期只注册定义模板，不枚举所有 concrete 类型：

```rust
#[Model(id = "qubit.commons.Page")]
struct Page<T> {
    items: Vec<T>,
    total: u64,
}

let page = TypeMetadata::of::<Page<UserInfo>>();

assert_eq!(page.model_id(), None);
assert!(!page.is_registered());

let definition = page.generic_definition().unwrap();
```

`Page<UserInfo>` 是可静态查询的 concrete metadata，但不是注册项，也不会获得拼接生成的新 `ModelId`。模板可以有
`ModelId("qubit.commons.Page")`。

> **待确认占位符 `META-API-TODO-015`：** `GenericTypeMetadata`、类型参数、const 参数、where 约束、concrete 实参
> 列表、模板实例化和 registry 枚举的完整公开接口尚未定型。讨论中“首版是否支持 const generic”也存在需要统一的旧结论。

### 12.10 `ModelRegistry` 与 resolver（行为已确认，接口待确认）

只有具有 `ModelId` 的类型才注册：Entity 必有并始终注册；Projection、Model、Enum、Value 只有显式声明 ID 才注册。
已知 Rust 类型的静态查询不依赖注册表。

已经确认的调用职责是：

```rust,ignore
// 已知 Rust 类型：不访问 registry。
let local = TypeMetadata::of::<LocalRequest>();

// 只知道稳定 ModelId：访问 registry。
let by_id = ModelRegistry::global()
    .get(/* META-API-TODO-016：精确 key 参数类型待确认 */)
    .expect("User must be linked");
```

全局注册表必须同时提供：

- 可处理初始化错误的入口；
- 在注册无效时 panic 的便利入口；
- 按稳定 ID 查询 metadata；
- 确定性枚举注册项和泛型模板；
- 完成后不可变、可安全并发读取的快照。

> **待确认占位符 `META-API-TODO-016`：** `ModelRegistry` 的完整查询接口、`get()` 接受 `ModelId`、`&ModelId` 还是
> `&str`、注册项迭代器、按 `TypeId` 查询以及泛型模板枚举接口尚未定型。

resolver 必须显式完成跨 crate 的 `entity_id`、Projection `source_id`、validator ID 和 codec ID 解析，并验证目标角色、
字段/Property descriptor 兼容性和策略值类型兼容性。

> **待确认占位符 `META-API-TODO-017`：** resolver 是 trait、构建器还是 registry 方法集合，其输入、输出、解析后视图
> 及错误返回类型尚未定型。

> **待确认占位符 `META-API-TODO-018`：** registry/resolver 的公开错误类型、稳定错误分类、路径和源码位置访问接口
> 尚未定型。

### 12.11 公共查询 trait 与隐藏生产接口

`HasTypeMetadata` 和 `HasTypeDescriptor` 必须保持为公共 trait，使泛型代码可以表达静态约束：

```rust,ignore
fn inspect_model<T>()
where
    T: HasTypeMetadata + 'static,
{
    let metadata = TypeMetadata::of::<T>();
    // ...
}

fn inspect_value<T>()
where
    T: HasTypeDescriptor + 'static,
{
    let descriptor = TypeDescriptor::of::<T>();
    // ...
}
```

> **待确认占位符 `META-API-TODO-003`：** 两个公共 trait 的继承关系、关联项和是否允许受支持的用户手工实现尚未定型。
> `TypeMetadata::of()` 和 `TypeDescriptor::of()` 的调用形式已经确认，不因这个占位符改变。

派生宏展开代码需要调用公开可达的底层构造和注册 API，但这些入口不属于普通用户 API。目标结构为：

```rust,ignore
// 名称仅表示边界，不承诺内部条目的具体接口。
#[doc(hidden)]
pub mod __private {
    // derive expansion only
}
```

业务代码不应手工调用 `__private`。这层接口的准确构造 ABI、版本兼容范围和校验职责受
`META-API-TODO-019` 约束。

### 12.12 当前 API 完整度索引

| 类型或接口组 | 当前状态 | TODO |
| --- | --- | --- |
| `TypeMetadata` 身份、字段、Property、角色导航 | 已确认并完整列出 | — |
| `TypeDescriptor::of()`、`metadata()` | 已确认 | — |
| `TypeDescriptor` 结构导航 | 待确认占位 | `META-API-TODO-001` |
| `TypeCapabilities` | 待确认占位 | `META-API-TODO-002` |
| `HasTypeMetadata`、`HasTypeDescriptor` 关联项 | 待确认占位 | `META-API-TODO-003` |
| `FieldMetadata`、`PropertyMetadata` 基础接口 | 已确认并完整列出 | — |
| `GetterMetadata`、`SetterMetadata` | 待确认占位 | `META-API-TODO-004` |
| `FieldAttributeMetadata` | 待确认占位 | `META-API-TODO-005` |
| identifier/unique/reference/constraint/validator/codec/redact metadata | 待确认占位 | `META-API-TODO-006`～`012` |
| 五种角色专属 metadata | 候选接口，待确认 | `META-API-TODO-013` |
| `QueryMetadata` | 待确认占位 | `META-API-TODO-014` |
| 泛型 metadata | 待确认占位 | `META-API-TODO-015` |
| `ModelRegistry` | 待确认占位 | `META-API-TODO-016` |
| resolver | 待确认占位 | `META-API-TODO-017` |
| registry/resolver 错误 API | 待确认占位 | `META-API-TODO-018` |
| 派生宏隐藏生产 ABI | 边界已确认，精确接口待定 | `META-API-TODO-019` |

## 13. 编译诊断与完整注册表错误

能在当前声明中判断的问题必须成为编译错误：

- 角色与 Rust shape 不匹配；
- identifier 缺失、重复或不是直接 Id；
- 重复参数、互斥参数或无效范围；
- 标准约束与类型 capability 不匹配；
- `#[indexed]` 与隐含 indexed 重复；
- property getter/setter 形状错误或类型不兼容；
- 平面查询字段名冲突；
- key_part order 重复、缺号；
- selector 嵌套或作用位置非法。

只能跨 crate 判断的问题由完整 registry/resolver 返回结构化错误：

- 重复 ModelId；
- `entity_id` 或 `source_id` 不存在；
- ID 指向的角色不是 Entity；
- validator/codec ID 未注册或值类型不兼容；
- reference property 不存在、不可读或返回类型不匹配；
- fixed Projection source 与 producer 不一致；
- Map key 脱敏后发生输出键冲突。

错误 API 应包含稳定错误类别、完整字段/property 路径、相关 ModelId/策略 ID 和源码位置；展示文案由上层本地化。

## 14. 明确不提供的能力

最终 API 不提供：

- `lookup_relation`、`ownership`；
- `#[computed]` 及 computed depends_on；
- 字段级 `#[generator]`；
- 字段级 `modified`、`unmodified`；
- 通用 `#[exclude]`；
- `#[key_index]`，使用 `#[key_part(order = ...)]`；
- indexed/unique/reference 的 `name` 参数；
- 字段宏里的物理数据库索引或 SQL 参数；
- 自动 Deref/From/Into 绕过 Value 约束的构造能力。

## 15. 选择与使用建议

- 先确定角色，再添加字段属性；不要根据“它是 struct”就默认使用 Model。
- 只将跨程序稳定发现确有价值的类型赋予 ModelId；匿名局部 DTO 仍可通过 Rust 类型取得 metadata。
- 使用 Value 封装可复用约束，避免在 selector 中构造任意深度 DSL。
- unique/indexed/reference 描述领域查询能力；物理数据库索引留给持久化层。
- validator 保持纯函数；外部事实校验放在 service/repository 层。
- 使用 `opaque` 时明确谁负责供值、生成和持久化 adapter。
- 将敏感字段的默认 Debug/Display/Serialize 行为纳入测试，尤其注意 Map key 冲突。

## 16. 相关文档

- [语义讨论原始记录](2026-08-28-discuss-session.md)
- [语义确认工作记录](2026-08-27-rs-model-derive-semantics-confirmation.md)
- [目标 API 重构蓝图](2026-08-28-rs-model-derive-refactoring-blueprint.md)
- [仍待确认的问题](2026-08-28-rs-model-derive-requirements-open-questions.md)
