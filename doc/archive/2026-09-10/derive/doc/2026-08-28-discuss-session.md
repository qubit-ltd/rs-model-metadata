# 2026-08-28 设计讨论归档

> 本文是按时间保留的讨论记录，不是现行规范。最终行为以
> [最终需求规范](rs-model-derive-requirements.zh_CN.md)、
> [最终设计](rs-model-derive-final-design.zh_CN.md) 与测试为准。

顺序调整为“字段语义类别 → 各字段宏 → 类型角色及其允许的类别”。我会先把现有 Rust 宏、平台实际用法和 Java 注解完整归类，再给出第一批类别定义供确认；不会继续确认五种角色。

确认顺序已调整，并记录到：

[rs-model-derive 语义确认记录](rs-model-derive/doc/2026-08-27-rs-model-derive-semantics-confirmation.md)

我根据 Rust 现有字段标注、平台实际用法和 Java 注解场景，初步归纳为以下五类。不同类别相互正交，同一字段可以组合使用多类标注。

### C001 身份、持久化与对象装配

包含：

- `#[identifier]`
- `#[indexed]`
- `#[unique(...)]`
- `#[reference(...)]`
- 如果保留，`computed`、lookup relation、ownership 等也归入此类

共同用途：

- 描述领域身份、数据库字段、索引、唯一性和 Entity 关系；
- 指导 DAO/schema；
- 指导自动化测试生成 ID、生成唯一值、确定 Entity 创建顺序、复用已有对象和装配关系字段。

### C002 声明式值约束

包含：

- `#[text(...)]`
- `#[decimal(...)]`
- `#[money(...)]`
- `#[time(...)]`
- `#[sequence(...)]`
- `#[map(...)]`
- `#[element(...)]`

共同用途：

- 描述字段值或容器内容必须满足的不变量；
- 可用于数据库/schema 约束；
- 可用于实例 validation；
- 可用于生成满足约束的随机值；
- 可用于接口和字段文档。

它们不赋予字段身份、持久化生命周期或关系语义。

### C003 自定义处理策略

包含：

- `#[validator(...)]`
- `#[generator(...)]`
- `#[codec(...)]`

共同用途：

- 补充标准声明式元数据无法表达的行为；
- 分别指定自定义校验、随机生成和编解码策略；
- 宏只记录稳定策略标识和参数，由对应消费方执行。

### C004 结构解析与递归边界

目前主要包含：

- `#[opaque]`

它不描述值是否合法，也不描述持久化或输出策略，而是告诉 metadata、validator、generator 和 schema：

- 保留可见的 `Option`、容器等外层结构；
- 不继续解析叶子内部字段；
- 对该叶子的校验、生成和编解码需要显式策略或调用方提供能力。

### C005 输出表示与敏感信息保护

包含：

- `#[redact(...)]`
- `#[serde(...)]`
- 模型宏需要识别的 `#[keep_serializing]`

共同用途：

- 控制 `Debug`、`Display`、`Serialize` 等对外表示；
- 控制嵌套对象、集合和 Map 的脱敏；
- 控制 `None`、空集合以及字段名称等序列化行为；
- 不改变字段的领域身份、数据库关系或值合法性。

其中 `redact` 和 `serde` 分别由专用 crate 定义，模型宏负责默认派生时的正确联动。

请确认 C001～C005。可以整体同意，也可以逐项修改。

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

C001 同意，

#[computed(...)] 是需要的，其他的 lookup relation、ownership 不再需要。
#[computed(...)]  表示这个属性是“计算出来的”而不是存储在对象的数据结构内部，所以它应该是标注在 getter 方法上的。比如 User.info() -> UserInfo，这个getter返回的是一个 UserInfo对象，它就是 computed的，它应该是根据对象的其他属性计算出了这些属性，computed(...)括号内的参数应该是所有参与这个属性计算的其他属性，当然有时候也可以忽略。
但是 computed 是否归结于 C001 还需要你再次评估下，它到底放哪里。

C002 同意
C003 同意
C004 同意
C005 同意

你可以设置一个 C006 把computed单独放入

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

C006 已记录为独立的“派生属性与依赖关系”类别。确认记录已更新：

[rs-model-derive 语义确认记录](/tmp/superpowers-rs-model-semantics.W9rNcX/2026-08-27-rs-model-derive-semantics-confirmation.md)

下一批请确认 C001 中的四个字段宏。

### F001 `#[identifier]`

语义：

- 标记 Entity 实例的领域标识字段；
- 同时作为该 Entity 持久化记录的主键；
- 自动生成对象时优先生成或取得 identifier，以便其他对象引用它。

候选参数：

- `#[identifier]`：构造对象时由调用方提供 ID；
- `#[identifier(generated)]`：ID 由 DAO、数据库或 ID 生成策略产生；
- `generated` 默认 `false`。

限制：

- 必须是直接字段；
- 准确类型必须为 `Id`；
- 不能是 `Option<Id>` 或容器；
- 允许哪些角色使用及数量规则稍后随类型宏确认。

### F002 `#[indexed]`

语义：

- 表示该字段对应的持久化值需要建立单字段索引；
- 供数据库 schema 和查询规划使用；
- 不改变字段值的合法性或随机生成分布。

参数：

- 不接受参数。

规则：

- `#[reference(...)]` 自带等价的单字段索引；
- reference 字段不需要也不允许再写 `#[indexed]`；
- 重复标注时编译报错。

### F003 `#[unique(...)]`

语义：

- 声明当前字段在全局或指定 scope 内唯一；
- 数据库/schema 据此建立唯一约束；
- 带 repository 的 validation 据此检查已有数据；
- 随机对象生成器据此避免数据库和当前生成批次中的冲突。

候选参数：

```rust
#[unique(
    name = "employee_code_in_org",
    respect_to(organization),
    ignore_case = false,
)]
```

- `name = "..."`：可选稳定约束名；
- `respect_to(field, ...)`：可选 scope 字段列表；省略表示全局唯一；
- `ignore_case = bool`：文本比较是否忽略大小写，默认 `true`。

因此一般文本字段只写 `#[unique]`；需要区分大小写时才写：

```rust
#[unique(ignore_case = false)]
```

### F004 `#[reference(...)]`

语义：

- 当前字段保存目标 Entity 本身，或者目标 Entity 的普通属性、identifier、computed property；
- 数据库/schema 使用它表达关联；
- 自动生成器使用它取得目标 Entity、读取 `property` 并装配字段；
- 自动等价于 `#[indexed]`。

候选参数：

```rust
#[reference(
    entity = User,
    property = info,
    existing = true,
    path = "../user",
)]
```

或者：

```rust
#[reference(
    entity_id = "qubit.platform.iam.User",
    property = id,
)]
```

- `entity = RustType` 与 `entity_id = "ModelId"` 必须二选一，效果等价；
- `property = property_path` 可选：
  - 省略表示完整 Entity；
  - `property = id` 表示 identifier；
  - `property = info` 可以选择 `#[computed(...)]` getter；
- `existing = bool` 默认 `true`：
  - `true` 表示目标 Entity 必须预先存在；
  - `false` 表示不要求预先持久化；
- `path = "object/graph/path"` 可选，表示复用对象图中另一处 reference 已经绑定的同一个 Entity。

不再提供 `select`、`bind` 或 `reference_key`。

请确认或修改 F001～F004。

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

F002 做个调整：
indexed 不是指“单字段索引”。
准确说，indexed表示这个字段参与了索引，注意这个字段甚至可以是一个复杂对象。但对复杂对象而言，其内部字段有些有indexed有些没有。

这个字段的准确作用是：
我们可以根据一个对象的indexed字段，自动程序化生成查询这个对象时的查询过滤器，比如 UserFilter ，是传递给 UserApi.list 这个RESTful接口的一个参数，或者是传递给 UserDao.list 的一个参数，它里面可以有哪些值呢？因为User.gender 标记了 indexed, User.create_time 标记了 indexed；那么 UserFilter 就可以有 UserFilter.gender : Option<Gender>, UserFilter.create_time : Option<DateTime<Utc>>；当用户传递进来的查询参数设置了这两个字段值时，程序就可以根据这两个参数的条件过滤User对象。

所以indexed准确地说不是数据库的索引而是可选择的查询条件。当然，通常如果你能根据这个字段查询，应该就把这个字段对应的数据库字段设置索引。但这个索引也可能是组合索引。（这块目前还没想到好办法用indexed表示组合索引）。

如果indexed标注在一个对象上，比如
#[indexed]
User.category: Info
而
#[identifier]
Info.id
#[indexed]
Info.name
那么就说明，UserFilter里面可以设置一个category_id, category_name，他们也可以作为过滤的条件。

注意，identifier, reference, unique 都暗示着该字段已经indexed

另外，你可以看看如何能设计 indexed让他表示组合索引（或组合查询参数）。如果做不到或太复杂，可以先搁置。当然你也要判断下根据我们对indexed的用途是否有必要实现组合索引。

F003: 整体同意，但indexed和unique,reference，都不需要name 参数，没有多大用。

F004：很准确，同意

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

你的修正是对的：`indexed` 的第一语义应该是“可作为查询过滤条件”，数据库索引只是由此推导出的性能建议，不能把两者混为一谈。

Java 版 `@Indexed` 的 Javadoc 仍然定义成“数据库字段会被索引”，但它只是一个无参数 marker，也没有表达组合索引的能力。新设计不应照搬这个旧定义。

我已经把下面 4 个决策写入确认记录，等待你逐项确认。

### F002-A：核心语义与隐含规则

建议定义：

- `#[indexed]` 表示当前字段路径可以参与对象的查询过滤。
- API、DAO 或代码生成器可以根据 metadata 自动生成 `UserFilter` 一类过滤器。
- 对普通叶子字段，基础形式是：

```rust
pub gender: Option<Gender>,
pub create_time: Option<DateTime<Utc>>,
```

- `#[indexed]` 不等价于数据库单列索引。
- `#[indexed]` 不支持任何参数，也不支持 `name`。
- `identifier`、`unique`、`reference` 都隐含 indexed。
- 同一个字段已经标有 `identifier`、`unique` 或 `reference` 时，再标 `#[indexed]` 属于重复语义，建议直接产生编译错误。

`Option` 字段、容器字段以及范围查询等操作数的表示，放到后面的字段形状规则中单独确认。

### F002-B：复杂对象的递归展开

建议采用“逐层授权”的递归规则：

```rust
#[Projection]
pub struct Info {
    #[identifier]
    pub id: Id,

    #[indexed]
    pub name: String,

    pub description: String,
}

#[Entity(id = "example.User")]
pub struct User {
    #[identifier]
    pub id: Id,

    #[indexed]
    pub category: Info,
}
```

得到的过滤路径是：

```rust
pub struct UserFilter {
    pub category_id: Option<Id>,
    pub category_name: Option<String>,
}
```

但没有 `category_description`。

具体规则：

- 外层 `category` 必须具有 indexed 语义，才能进入 `Info`。
- 进入 `Info` 后，只继续展开内部具有有效 indexed 语义的成员。
- 有效 indexed 包括显式 `indexed`，以及 `identifier`、`unique`、`reference` 隐含的 indexed。
- 未 indexed 的中间节点会切断整条路径。
- metadata 保存结构化路径，例如 `["category", "name"]`，而不是只保存拼接后的字符串。
- 生成平面过滤器时，路径默认转成 `category_name`。
- 两条不同路径如果生成相同名称，派生时报错，不增加 `name` 参数解决。
- 复杂字段递归后没有任何查询叶子时，派生时报错。

### F002-C：reference 的递归边界

这里需要专门限制，否则完整 Entity 之间的 reference 会使过滤路径无限展开。

建议：

- reference 保存完整 Entity 时，只生成目标 Entity identifier 对应的查询条件。

```rust
#[reference(entity = Department)]
pub department: Department,
```

对应：

```rust
pub department_id: Option<Id>,
```

不会继续生成 `department_name`、`department_create_time` 等目标 Entity 的所有查询条件。

- reference 选择标量属性时，该字段直接成为叶子条件。

```rust
#[reference(entity = User, property = id)]
pub user_id: Id,
```

对应 `user_id: Option<Id>`。

- reference 选择 Projection、Value 等复杂属性时，按照 F002-B 展开其内部有效 indexed 字段。

这个边界既保留复杂值查询，又避免 Entity 关系图递归和过滤器无限膨胀。

### F002-D：组合查询与数据库组合索引

我的判断是：暂时不需要给 `#[indexed]` 增加组合语法。

原因是同时填写多个过滤字段，本身就是组合查询：

```rust
UserFilter {
    gender: Some(Gender::Male),
    create_time: Some(time),
    ..Default::default()
}
```

默认表示：

```text
gender = ? AND create_time = ?
```

因此“多个字段共同参与查询”已经自然成立，不需要再声明组合查询参数。

数据库组合索引则是另一层问题。它还需要表达：

- 字段顺序；
- 索引前缀；
- 升序或降序；
- 部分索引条件；
- 数据库方言；
- 查询负载和执行计划。

这些都是持久化实现和性能优化，不属于领域模型的可查询性。因此建议：

- `rs-model-derive` 当前不表达物理组合索引。
- 不给 `#[indexed]` 增加组合索引参数。
- 将来确实需要自动生成 DDL 时，在持久化层设计独立的类型级配置。

请确认或分别修订 F002-A、F002-B、F002-C、F002-D。确认后我会立即更新记录，再继续下一批。

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

reference 的递归边界：我建议约定，递归最多1层。比如

pub struct Address {
   #[identifier]
   pub id: Id,

   #[unique]
   pub code: String,

   #[indexed]
   pub name: String,

    #[reference(..)]
    pub city: City,
}

pub struct City {
   #[identifier]
   pub id: Id,

   #[unique]
   pub code: String,

   #[indexed]
   pub name: String,

    #[reference(..)]
    pub province: Province,
}

这样对 Address生成的filter，最多递归到 City 自己内部的 identifier, indexed, unique字段，但不会递归到 City的reference字段，具体而言
pub struct AddressFilter {
   pub name: Option<String>,
   pub city_id: Option<Id>,
   pub city_code: Option<String>,
   pub city_name: Option<String>,
}

注意Address自己的id, code没有出现在filter中，因为它的id，code可以“唯一”确定这个Address，不需要再放在filter中，否则filter其他字段都没有意义了。但是 City的id，code 虽然是唯一确定 City，但却应该出现在Address的filter中，因为同一个City可能对应多个Address

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

这个边界更合理。关键不是简单按字段类型递归一层，而是：

> 根对象自己的唯一查询键不进入 list filter；reference 目标的查询键可以筛选多个根对象，因此需要进入，但 reference 图最多展开一跳。

我已据此调整确认记录。现在需要确认以下 5 点。

### F002-A：区分查询能力和 list filter 投影

`identifier`、`unique`、`reference` 仍然隐含 indexed，表示它们具有查询能力。

但“具有 indexed 查询能力”不代表一定出现在当前类型的 list filter 中。框架需要记录 indexed 的来源：

- 显式 `#[indexed]`
- `identifier` 隐含
- `unique` 隐含
- `reference` 隐含

生成 list filter 时，再根据来源和字段所在层级决定是否纳入。

重复标注仍然报错，例如：

```rust
#[identifier]
#[indexed] // 编译错误：identifier 已经隐含 indexed
pub id: Id,
```

### F002-B：根对象唯一键不进入 list filter

生成 `AddressFilter` 时：

- 根对象显式 `#[indexed]` 的字段进入。
- 根对象的 reference 按 F002-D 展开。
- 根对象的 identifier 不进入。
- 根对象的全局 unique 字段不进入。

因为 identifier 和全局 unique 已经可以唯一确定 Address，应当通过专用查找接口查询，而不是混入 list filter。

例如框架在语义上会区分：

```rust
// 唯一查找
Address::find_by_id(id)
Address::find_by_code(code)

// 列表过滤
Address::list(AddressFilter {
    name: Some(name),
    city_id: Some(city_id),
})
```

对于带 `respect_to(...)` 的 scoped unique，我建议采用稍有区别的规则：

```rust
#[unique(respect_to(tenant_id))]
pub code: String,
```

`code` 单独不能唯一确定根对象，所以仍然进入 list filter；同时 `(tenant_id, code)` 作为完整的唯一查找键。

`respect_to` 中的 `tenant_id` 是否单独进入 list filter，由 `tenant_id` 自己的标注决定。

### F002-C：普通复杂字段的展开

对于非 reference 的复杂字段，仍采用逐层 indexed 展开：

```rust
#[indexed]
pub category: Info,
```

只展开 `Info` 内部具有 indexed 查询能力的路径。

这里同样应用“根层级”规则：

- `User.id`、`User.code` 是当前过滤对象的唯一键，因此排除。
- `User.category.id`、`User.category.code` 唯一确定的是 category，不唯一确定 User，因此保留。

所以之前的结果仍成立：

```rust
pub struct UserFilter {
    pub category_id: Option<Id>,
    pub category_code: Option<String>,
    pub category_name: Option<String>,
}
```

### F002-D：reference 图最多展开一跳

按照你的例子，结果精确定义为：

```rust
pub struct AddressFilter {
    pub name: Option<String>,
    pub city_id: Option<Id>,
    pub city_code: Option<String>,
    pub city_name: Option<String>,
}
```

规则是：

- 排除 Address 自己的 `id` 和全局唯一的 `code`。
- 包含 Address 自己显式 indexed 的 `name`。
- `Address.city` 是第一跳 reference，因此进入 City。
- 包含 City 直接声明的 identifier、indexed、unique 字段。
- 不进入 City 的 reference 字段，所以不生成任何 `city_province_*` 条件。

我建议“一跳限制”只针对 reference 图。City 的普通复杂值字段如果显式 indexed，仍可以按照 F002-C 展开；但只要遇到 `City.province` 这样的 reference 就立即停止。

`reference(property = ...)` 采用相同原则：

- property 是标量：直接形成一个过滤字段。
- property 是复杂对象：展开其中的 identifier、indexed、unique。
- 遇到该复杂对象内部的 reference：停止。

### F002-E：仍不表达物理组合索引

同时设置多个字段本身就表示组合过滤：

```rust
AddressFilter {
    city_id: Some(city_id),
    name: Some(name),
}
```

语义为：

```text
city_id = ? AND name = ?
```

所以我仍建议：

- `#[indexed]` 不增加组合查询参数。
- `#[indexed]` 不表达数据库组合索引。
- 物理组合索引将来如有需要，放到持久化层单独设计。

请确认或修订 F002-A～F002-E，特别是两点：

1. scoped unique 字段是否仍进入根对象 list filter；
2. reference 的“一层”是否只限制 reference 跳数，还是所有结构递归都只能一层。

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

完全同意你的意见

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

F002-A～F002-E 已全部记为“已确认”。

下一批确认以下 4 项。这里对旧设计做了一些主动收紧。

### F001：`#[identifier]`

建议最终定义：

```rust
#[identifier]
pub id: Id,
```

- 表示对象的实例身份。
- 对 Entity，它同时是领域身份和持久化主键。
- 对 Projection，它表示来源 Entity 的身份，不是 Projection 自己的主键。
- 只允许无参数形式。
- 删除 `generated` 和 `ignore_case`。
- 字段必须是直接字段，准确类型为 `Id`，不能是 `Option<Id>` 或容器。
- identifier 必须在对象进入构造、装配流程时已经存在。
- 数据库自增、序列等 ID 分配方式属于 DAO/repository 策略，不进入模型 metadata。
- identifier 隐含 indexed 查询能力，但不进入根对象自己的 list filter。
- identifier 的角色许可和基数稍后确认：预计 Entity、Projection 必须恰好一个，其他角色禁止。

删除 `generated` 的原因是：一个还没有身份的 Entity 无法安全参与 reference 装配。即便 ID 最终来自数据库，也应由持久化适配器先取得 ID，再构造完整 Entity。

### F010：`#[text(...)]`

```rust
#[text(
    min_chars = 1,
    max_chars = 128,
    min_bytes = 1,
    max_bytes = 256,
    allowed_chars = ascii,
    non_blank,
    format = email,
)]
pub email: String,
```

支持参数：

- `min_chars`、`max_chars`：Unicode scalar value 数量范围。
- `min_bytes`、`max_bytes`：UTF-8 字节数范围。
- `allowed_chars = unicode | ascii`：默认 `unicode`。
- `non_blank`：禁止空字符串和全 Unicode 空白字符串。
- `format = email | cn_mobile | uri | uuid`：内置格式。

规则：

- `ascii` 只表示字符码点必须位于 ASCII 范围，不等于“仅可打印 ASCII”。
- 原来的 `mobile` 改成语义明确的 `cn_mobile`。
- 不执行 trim、大小写转换等值修改。
- 字符数和字节数是两套独立约束。
- min 不能大于对应的 max。
- 参数不能重复。
- 只能用于具有文本能力的类型。
- 空的 `#[text]` 没有意义，建议报错。
- 只写默认值 `allowed_chars = unicode` 同样没有实际约束，建议报错。

这些 metadata 同时用于 validation、随机生成、数据库/schema 和接口文档。

### F011：`#[decimal(...)]`

建议补足数值范围能力：

```rust
#[decimal(
    precision = 18,
    scale = 6,
    min = "-90",
    max = "90",
    rounding = half_even,
)]
pub latitude: Decimal,
```

支持参数：

- `precision = u16`：定点存储允许的总位数。
- `scale = u16`：允许的小数位数。
- `min = "decimal"`、`max = "decimal"`：精确十进制范围，必须使用字符串，避免浮点字面量损失。
- `min_inclusive = bool`、`max_inclusive = bool`：默认 `true`。
- `rounding` 支持：

```text
up
down
ceiling
floor
half_up
half_down
half_even
unnecessary
```

默认 `rounding = unnecessary`，即禁止默认静默损失精度。

当 precision 和 scale 同时存在时，采用数据库定点数语义：

```text
最大整数位数 = precision - scale
最大小数位数 = scale
```

其他规则：

- `scale <= precision`。
- `min <= max`。
- 相同的 min/max 不能同时设置为排他边界。
- 只允许精确十进制类型，不允许 `f32`、`f64`。
- 至少提供一个有效参数。
- 不能和 `#[money]` 同时使用。
- validator 不修改对象，只检查值是否合法。
- codec、输入解析器和生成器可以根据 rounding 进行规范化。
- `unnecessary` 表示一旦需要舍入就返回错误。

### F012：`#[money(...)]`

```rust
#[money(
    precision = 18,
    scale = 2,
    min = "0",
    rounding = half_even,
)]
pub amount: Decimal,
```

`money` 表示这个十进制字段具有“货币金额”语义，而不仅是普通数值。

- 参数和 `decimal` 相同。
- `scale` 必须显式提供，因为不同币种、积分和数字资产不存在统一小数位数。
- 默认 `rounding = unnecessary`。
- metadata 的数值语义为 `Money`，消费者可以采用货币专用的 schema、显示、序列化和生成策略。
- 不支持 `currency` 参数；币种应当是独立领域数据。
- 不支持分组显示和货币符号；这些属于输出表示。
- 不能与 `#[decimal]` 同时使用。
- 只允许精确十进制类型。

请确认或修订 F001、F010、F011、F012。

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

F001，
不同意删除 generated，这个依然保留，默认false，如果为true表示这个值在插入数据库时由数据库自动生成，不需要手动赋值，或者说手动赋值的值在插入数据库后可能会被修改。但这个名字可能不准确，你看看是否还有更准确的名称。
ignore_case 要删除，id必须是Id类型，不存在 ignore_case
【数据库自增、序列等 ID 分配方式属于 DAO/repository 策略，不进入模型 metadata】metadata不需要记录数据库到底怎么自动分配这个id，但是要记录这个id是否由数据库自动分配，还是我们的代码要手动分配。这个和前面generated的意见是一致的。
【identifier对于Entity、Projection 必须恰好一个，其他角色禁止】这个确认

其他都确认

F010:

allowed_chars 的可选项，建议再增加几个：
unicode, printable_unicode, ascii, printable_ascii, 以及 identifier 【这个名字可能要换一个】
最后的 identifier 【暂定的名称】的意思是，这个字符串，只能是可以构成合法标志符的字符，比如大小写字母、数字、连字符下划线；不应该有问好、逗号、句号等标点符号。
但这块需要好好设计一下。
通常有这个要求的是用在code，username之类的上面。
当然，我觉得把这个约束放在allowed_chars中可能太复杂了，是不是应该在加个validator或者format之类的参数？
allow_chars 其实最早的目的，是有些字段在数据库设计时就把字段charset设置为ascii，有的则是utf8mb4；对于前者，如果插入一个非ascii值，数据库会报错，所以我们希望在生成这样的字段值做自动测试时，生成ascii值；同时我们也希望前端返回这个字段值时，可以自动做个校验。

F011:
这个同意，但 rounding 默认应该用 half_even

F012:
这个同意

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

已记录：

- F011 已确认，`rounding` 默认改为 `half_even`。
- F012 已确认，按上一版约定，Money 的 `rounding` 默认仍是 `unnecessary`。
- F001 的角色基数、`Id` 类型限制以及删除 `ignore_case` 已确认。

剩余建议这样确定。

### F001-B：使用 `assigned_by` 表示 ID 分配方

我建议不要继续使用含义模糊的 `generated`，改为：

```rust
// 默认由应用程序分配
#[identifier]
pub id: Id,

// 插入时由数据库分配
#[identifier(assigned_by = database)]
pub id: Id,
```

完整取值是：

```rust
assigned_by = application // 默认
assigned_by = database
```

对应公开 metadata：

```rust
pub enum IdentifierAssignment {
    Application,
    Database,
}
```

语义如下：

- `application`：调用插入操作前，代码必须提供最终 ID；数据库不能擅自替换。
- `database`：调用方不需要提供权威 ID；即使提供，数据库也允许覆盖或修改；DAO 必须返回或回填数据库最终确定的 ID。
- metadata 只记录分配责任方，不记录自增、sequence、trigger 等具体机制。
- `assigned_by = database` 只允许 Entity。
- Projection 的 ID 来自来源 Entity，因此只能使用默认的 `application`，不能声明数据库分配。
- 已持久化的 Entity 和所有 Projection 始终必须包含最终有效的 `Id`。

`assigned_by` 比 `generated` 更准确，因为它表达的是“谁对最终 ID 负责”，也覆盖了数据库修改调用方暂时提供值的情况。

### F010-B：`allowed_chars` 的五种字符集合

建议支持：

```rust
allowed_chars = unicode
allowed_chars = printable_unicode
allowed_chars = ascii
allowed_chars = printable_ascii
allowed_chars = code
```

精确定义如下：

| 取值 | 精确定义 |
|---|---|
| `unicode` | 允许所有 Unicode scalar value，包括控制字符；对应 Unicode/UTF-8 存储能力。 |
| `printable_unicode` | 允许 Unicode Letter、Mark、Number、Punctuation、Symbol 和 Space Separator；排除控制字符、格式字符、私用区、未分配字符以及行/段分隔符。 |
| `ascii` | 只允许 `U+0000..U+007F`，包括 ASCII 控制字符。 |
| `printable_ascii` | 只允许 `U+0020..U+007E`，即空格到 `~`。 |
| `code` | 只允许 ASCII 大小写字母、数字、下划线和连字符，即 `[A-Za-z0-9_-]`。 |

默认仍为 `unicode`。

这里需要注意，`printable_unicode` 会排除零宽连接符等 Unicode 格式字符，因此某些复杂 emoji 序列不合法。需要完整 Unicode 内容时应使用 `unicode`。

### F010-C：使用 `code`，不使用 `identifier`

我建议暂定名称最终就叫 `code`：

```rust
#[text(
    min_chars = 1,
    max_chars = 64,
    allowed_chars = code,
)]
pub username: String,
```

不叫 `identifier`，因为：

- 连字符 `-` 不是 Rust、Java 等语言标识符的合法组成字符。
- `identifier` 容易与 `#[identifier]` 领域身份宏混淆。
- 这个字符集主要服务于业务 code、username、外部编号等字符串，`code` 更贴近用途。

它应当属于 `allowed_chars`，不属于 `format`：

- `allowed_chars` 只回答“每个字符是否允许”。
- `format` 描述完整语法，例如 email、UUID、URI。
- 自定义 validator 描述更具体的业务规则，例如首字符必须是字母、禁止连续连字符、禁止保留用户名等。

例如：

```rust
#[text(
    min_chars = 3,
    max_chars = 32,
    allowed_chars = code,
)]
#[validator("iam.username")]
pub username: String,
```

`code` 本身不限制首字符，也不隐含 `non_blank`；这些由其他约束负责。

### F010-D：`allowed_chars` 的消费语义

同一份 metadata 同时提供给：

- validator：后端和前端校验实际值；
- generator：只生成符合字符集合的随机文本；
- schema/DDL：选择或校验 ASCII、UTF-8/utf8mb4 等存储能力；
- API 文档：描述客户端允许提交的字符范围。

因此：

```rust
#[text(allowed_chars = unicode)]
```

虽然对 Rust `String` 的运行时值几乎没有额外限制，但对数据库 schema 和随机生成器仍有明确意义，应当合法。

只有完全空的：

```rust
#[text]
```

才因为没有表达任何约束或 schema 意图而报错。

请确认或修订 F001-B、F010-B、F010-C、F010-D。

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

同意
补充下 F010-D: allowed_chars 默认值应该是 unicode

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

收到，F010-D 将明确规定：未写 `allowed_chars` 时，其有效默认值就是 `unicode`，并由 validator、generator、schema/DDL 和 API 文档共同读取。

F010-D 已补充确认：`allowed_chars` 的有效默认值为 `unicode`。

下一批确认以下 4 项。

### F013：`#[time(...)]`

```rust
#[time(precision = second)]
pub create_time: DateTime<Utc>,
```

建议只支持一个必填参数：

```text
precision = second
precision = millisecond
precision = microsecond
precision = nanosecond
```

语义：

- 统一数据库存储、实例校验、输入适配和随机生成的时间精度。
- `second` 要求亚秒部分为零。
- `millisecond` 要求纳秒部分是 `1_000_000` 的整数倍。
- `microsecond` 要求纳秒部分是 `1_000` 的整数倍。
- `nanosecond` 允许完整纳秒精度。
- validator 只校验，不修改或截断已有值。
- generator 直接生成满足精度的时间。
- codec/DAO 如果收到更高精度输入，必须先规范化，再构造最终对象。
- `#[time]` 不提供默认精度，无参数时报错。
- 只用于 instant、datetime、time 等具有相应精度能力的类型；纯 date 不需要它。

不建议加入 `past`、`future`、`min`、`max`：

- 过去或未来属于随当前时间变化的业务校验，使用 validator。
- `start_time <= end_time` 属于跨字段约束，使用对象级 validator。
- `time` 只负责稳定的时间表示精度。

### F014：`#[sequence(...)]`

```rust
#[sequence(
    min_items = 1,
    max_items = 10,
    unique_items,
)]
pub tags: Vec<String>,
```

参数：

- `min_items = u32`
- `max_items = u32`
- `unique_items`

规则：

- 至少提供一个参数。
- `min_items <= max_items`。
- `unique_items` 按元素值相等性判断，不等于数据库 unique。
- 对 Set，元素唯一性由类型天然保证，再写 `unique_items` 属于重复语义，编译报错。
- 对固定数组，长度已经由 Rust 类型确定，因此禁止 `min_items` 和 `max_items`，但允许 `unique_items`。
- generator 必须生成满足数量和唯一性要求的集合。
- 如果元素取值空间不足，无法满足 `unique_items + min_items`，生成器返回“约束不可满足”错误，不能无限重试。

`sequence` 只约束容器本身，不约束单个元素内容。

### F015：`#[map(...)]`

```rust
#[map(
    min_entries = 1,
    max_entries = 20,
)]
pub labels: BTreeMap<LabelKey, LabelValue>,
```

参数：

- `min_entries = u32`
- `max_entries = u32`

规则：

- 至少提供一个参数。
- `min_entries <= max_entries`。
- 只允许具有 map capability 的类型。
- Map key 的唯一性由 Map 类型保证，不再提供 `unique_keys`。
- key 和 value 根据各自类型的 descriptor 递归校验、生成。
- 不增加 `key(text(...))`、`value(text(...))` 这种任意嵌套 DSL。
- key/value 需要复用字段约束时，应定义为 Value：

```rust
#[Value(transparent, textual)]
pub struct LabelKey(
    #[text(
        min_chars = 1,
        max_chars = 32,
        allowed_chars = code,
    )]
    pub String,
);
```

Map 不能直接作为 reference 保存形状，这项组合限制稍后统一确认。

### F016：`#[element(...)]`

```rust
#[sequence(min_items = 1, max_items = 10)]
#[element(text(
    min_chars = 1,
    max_chars = 32,
    non_blank,
))]
pub aliases: Vec<String>,
```

支持的元素约束：

- `text(...)`
- `decimal(...)`
- `money(...)`
- `time(...)`

参数与对应字段宏完全一致。

规则：

- 只作用于 sequence、set 或固定数组的第一层元素。
- 不约束容器大小；容器大小由 `sequence` 负责。
- 元素类型必须具有相应 capability。
- `decimal` 和 `money` 互斥。
- 元素为 `Option<T>` 时，`None` 跳过，`Some` 校验内部 `T`。
- 元素为具名 Value、Model 等复杂类型时，由该类型自己的 descriptor 递归处理。
- 不支持在 `element` 内继续嵌套 `sequence`、`map`、`element`。
- 不用于 Map key/value。
- 深层容器或复杂元素需要局部约束时，应提取成具名 Value，避免产生任意深度的属性 DSL。

请确认或修订 F013～F016。

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

基本上同意 F013~F016，但现在可以通过 element 约束 sequence, set的元素，并没有提供方法约束map的key，value，这个是不是应该也对称提供方法来约束

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

是的，这是当前提案中的不对称缺口。Value 类型适合复用约束，但不应该为了给一个 Map 的 key/value 加一次局部约束，就强迫用户创建新类型。我建议保留 `map` 只管容器大小，新增与 `element` 对称的 `map_key`、`map_value`。

应该对称提供。否则 `Vec<String>` 可以局部约束元素，而 `Map<String, String>` 却必须额外定义 Value，设计上不完整。

我建议确认下面 3 项。

### F015-B：新增 `map_key` 和 `map_value`

最终用法：

```rust
#[map(min_entries = 1, max_entries = 20)]
#[map_key(text(
    min_chars = 1,
    max_chars = 32,
    allowed_chars = code,
))]
#[map_value(text(
    max_chars = 256,
    printable_unicode,
))]
pub labels: BTreeMap<String, String>,
```

三者职责严格分开：

- `map(...)`：约束 entry 数量。
- `map_key(...)`：约束每个 key。
- `map_value(...)`：约束每个 value。

不建议使用过于宽泛的 `#[key(...)]`、`#[value(...)]`，因为脱离 Map 后语义不明确，也容易与字段名、Value 角色等概念混淆。

也不建议把它们塞进：

```rust
#[map(
    key(...),
    value(...),
)]
```

因为 `sequence` 与 `element` 已经是容器约束和元素约束分离，Map 保持同样结构更一致。

### F015-C：支持的局部约束

`map_key` 和 `map_value` 与 `element` 支持相同的约束：

- `text(...)`
- `decimal(...)`
- `money(...)`
- `time(...)`

例如：

```rust
#[map_key(text(allowed_chars = code, max_chars = 32))]
#[map_value(decimal(
    precision = 12,
    scale = 2,
    rounding = half_even,
))]
pub prices: BTreeMap<String, Decimal>,
```

规则：

- 每个 Map 字段最多各有一个 `map_key` 和一个 `map_value`。
- 约束必须匹配 key/value 的类型 capability。
- `decimal` 与 `money` 仍然互斥。
- key/value 是 `Option<T>` 时，`None` 跳过局部值约束。
- key/value 是具名复杂类型时，继续通过该类型自己的 descriptor 递归校验。

只允许下沉一层，不支持：

```rust
#[map_value(sequence(...))]
#[map_value(element(...))]
#[map_value(map(...))]
```

深层容器约束仍然通过具名 Value 表达，避免属性 DSL 任意递归。

### F015-D：生成与校验语义

generator 需要同时满足：

- `map` 的 entry 数量；
- `map_key` 的 key 约束；
- Map 类型天然要求的 key 唯一性；
- `map_value` 的 value 约束；
- key/value 类型自身 descriptor 中的约束。

如果 key 的合法取值空间不足以生成要求数量的不同 key，返回明确的“约束不可满足”错误，不能无限重试。

validator 则分别报告准确路径，例如：

```text
labels.<key>
labels["region"].<value>
```

最终形成完全对称的结构：

| 容器 | 容器约束 | 子值约束 |
|---|---|---|
| Sequence / Set / Array | `sequence(...)` | `element(...)` |
| Map | `map(...)` | `map_key(...)`、`map_value(...)` |

F013、F014、F016 已按“基本同意”记为确认；F015 的 entry 数量规则也已确认。现在只需要确认 F015-B～F015-D。

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

同意

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

我会把 F015-B～F015-D 标记为已确认。下一批确认 C003/C004 的四个独立宏：`opaque`、`validator`、`generator`、`codec`，重点会放在策略标识、参数传递和执行边界。

F015-B～F015-D 已记为已确认。

下一批确认 5 项。

### S001：自定义策略的公共参数约定

`validator`、`generator`、`codec` 统一采用：

```rust
id = "qubit.iam.username"
```

不使用 `name`，因为这里保存的是稳定 `StrategyId`，不是展示名称。

完整示例：

```rust
#[validator(
    id = "qubit.iam.username_available",
    depends_on(tenant_id),
    params(
        allow_reserved = false,
        reserved = ["admin", "root"],
    ),
)]
pub username: String,
```

公共参数：

- `id = "..."`：必填，ASCII `lower_snake_case` 点分段。
- `params(key = literal, ...)`：可选静态参数。
- `depends_on(field, ...)`：仅 validator、generator 支持。

`params` 的值支持：

- bool；
- 整数；
- 字符串；
- 上述类型的同类型数组。

不支持浮点字面量。精确十进制、时间和复杂配置使用字符串表示。

`depends_on` 必须使用当前对象中的真实字段路径。策略只能读取显式声明的依赖字段，不能绕过依赖声明访问整个对象。

### F017：`#[opaque]`

```rust
#[opaque]
#[generator(id = "qubit.security.key_material")]
pub key_material: ExternalKeyMaterial,
```

语义：

- 保留最外层 `Option`、sequence、set、array、map、box 等结构。
- 不解析最终叶子类型的内部字段。
- opaque 叶子不要求实现 `HasTypeDescriptor`。
- 默认 validator 不递归 opaque 叶子。
- 默认 generator 无法构造 opaque 叶子；如果调用方不供值，必须声明 generator。
- `text`、`decimal`、`money`、`time` 只有在 opaque 类型显式提供对应 capability adapter 时才允许使用。

限制：

- 不接受参数。
- 不能隐藏 Entity、Projection、Model 或 reference 目标来规避结构检查。
- 不能与 `identifier`、`reference` 组合。
- 与 `indexed`、`unique` 组合时，opaque 类型必须提供查询比较和持久化 adapter，否则 schema 校验失败。

### F018：`#[validator(...)]`

```rust
#[validator(
    id = "qubit.iam.username_available",
    depends_on(tenant_id),
    params(ignore_case = true),
)]
pub username: String,
```

语义：

- 表达标准约束无法描述的自定义校验。
- 接收当前字段值、字段路径、ValidationContext，以及 `depends_on` 声明的其他字段值。
- ValidationContext 可以提供 repository 等外部能力。
- validator 只能报告 violation，不能修改字段值。

规则：

- 同一字段可以声明多个不同 validator。
- 按源码顺序确定性执行。
- 聚合所有 validator 的 violation，不遇到第一个错误就停止。
- 同一字段重复相同 StrategyId 报错。
- `Option<T>` 为 `None` 时默认跳过 validator。
- 是否允许为空由字段类型和可空规则决定，不使用自定义 validator 改写 Option 语义。

### F019：`#[generator(...)]`

```rust
#[generator(
    id = "qubit.iam.username",
    depends_on(tenant_id, display_name),
    params(prefix = "user", max_attempts = 32),
)]
pub username: String,
```

语义：

- 为自动化测试和随机对象装配提供自定义字段生成策略。
- 接收确定性随机源、生成上下文、字段 metadata、静态参数以及已经就绪的依赖字段值。
- 返回准确字段类型的候选值。

规则：

- 每个字段最多只能有一个 generator。
- 自定义 generator 替代该字段的默认生成步骤。
- 生成结果仍然必须经过标准约束和所有 validator。
- 不合法结果不能直接写入对象。
- 策略缺失、返回类型错误、依赖环或生成值不合法时返回结构化错误。
- 不能静默退回默认 generator。
- 重试次数由生成上下文统一限制，不能无限重试。

### F020：`#[codec(...)]`

```rust
#[codec(
    id = "qubit.phone.e164",
    params(default_region = "CN"),
)]
pub phone: PhoneNumber,
```

语义：

- 描述领域字段值与 metadata-aware 外部表示之间的双向转换和规范化。
- 可供 DAO、数据库适配器、API codec 等消费者使用。
- decode 后的值仍然必须执行字段约束和 validator。

规则：

- 每个字段最多一个 codec。
- 支持 `id` 和 `params`，不支持 `depends_on`。
- 编码结果不能依赖同一个对象的其他字段。
- 必须满足：

```text
decode(encode(value)) == canonical(value)
```

- encode/decode 失败返回结构化错误。
- codec 不自动修改 `serde::Serialize`、`serde::Deserialize` 的行为。
- Serde 字段属性仍然属于 C005；需要改变 Serde 表示时显式使用 `#[serde(...)]`。

请确认或修订 S001、F017～F020。

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

S001：同意

F017: 同意

F018:
validator的验证范围和语义要做约束：validator只做语法层次的约束和验证，不做业务层次的约束。
具体而言，
- 对于用户名，validator只验证它的长度，组成字符是否合法，不验证系统中是否已经有这个用户名，后者属于业务层次的验证，应该放在service层，不该由bean的validation层来处理
- 对于身份证号码，validator除了验证它的长度、组成字符，还需要验证它是否是个合法的身份证；如果用户回信息还提供了gender，birthday，validator会一起验证他们是否一致。这些都属于语法合规性验证，不属于业务层的验证；比如validator不会验证这个身份证是否重复，也不会验证这个身份证号码是否真的存在对应的真实的人（只会验证身份证号码的组成是否合规）。

关于这块，你可以参考js的validator库：
~/working/qubit/js-common/js-common-validator 这里面定义了js的validator装饰器
~/working/qubit/js-common/js-validation-rule 这里面定义了一些validation rule
~/working/qubit/js-common/js-common-model  这里面可以展示如何使用 validator 的装饰器

注意，我们在 rs-model-derive 中不需要定义具体的validator和验证规则，只需要提供语法让用户自己去实现。

但是我们是不是要对validator做一些约束，比如他们必须符合某个traits?

未来我们可能会专门写一个 rs-validator 的 crate （qubit-validator)，在其中定义一个 Validator 的traits。在这里如果有需要的话，可以先定义在 rs-model-derive 或 rs-model-metadata 中，未来我们再移植。

所以 F018 #[validator(...)] 这个你可能要重新设计

F019：

generator 这个宏，目前我还是没搞清到底有什么用。先把它去掉吧，以后要用再说

F020:

codec这个应该是有用的，但还是那句话，到底要怎么设计，暂时还没想到好办法。不过，这个应该在java代码中已经有了应用，你可以看下java的代码，

~/working/qubit/java-common/common-annotation
~/working/qubit/java-common/common-model

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

我会按你的边界重做 F018：validator 只描述对象自身可判定的语法/结构一致性，不允许访问 repository、数据库或外部业务状态；同时核对 JS 版的 decorator/rule 协议。F019 将从最终设计删除。F020 先根据 Java 的实际 codec 用法重新归纳，不沿用刚才过于泛化的定义。

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

关于codec，在 ~/working/qubit/rust-common/rs-codec 中也应该有traits定义了。ValueEncoder， ValueDecoder

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

这个补充很关键。我会把 `rs-codec` 的 `ValueEncoder`、`ValueDecoder` 作为 Rust 侧主依据，再与 Java 的 `@TypeCodec` 实际用法对照，判断 codec 应该是类型级能力、字段级覆盖，还是两者分层。

我核对后的结论是：

- JS 版 validator 确实只做对象自身可判定的合法性校验，包括跨字段一致性，不访问外部业务状态。
- Java 的 `@TypeCodec` 实际只标注在类型上，不标注字段；现有 Location、Phone、CredentialInfo codec 都是“领域类型 ↔ String”。
- Rust 的 `qubit-codec` 已经提供合适的 `ValueEncoder`、`ValueDecoder`，不应该再定义一套重复的 codec trait。

基于这些证据，建议确认下面 5 项。

### F018-A：validator 的严格语义边界

`#[validator(...)]` 只允许表达：

- 字段格式是否合法；
- 字段内容是否自洽；
- 校验位是否正确；
- 同一对象内多个字段是否一致；
- 不依赖外部状态的领域语法规则。

例如身份证 validator 可以：

- 检查长度和字符；
- 检查行政区划、出生日期编码和校验位；
- 检查身份证中的性别与 `gender` 是否一致；
- 检查身份证中的出生日期与 `birthday` 是否一致。

但不能：

- 查询身份证是否真实登记；
- 查询身份证是否重复；
- 查询用户名是否被占用；
- 查询关联对象是否存在；
- 验证权限、库存或账户余额。

这些必须由 service/repository 层负责。

validator 必须同步、确定性、无副作用。框架不会向它提供 repository、数据库、网络或 service。

Rust 无法绝对阻止用户在实现中自行访问全局网络，但 trait API 不提供这些能力，并将“纯语法校验”作为强制契约。

### F018-B：保留 StrategyId，通过 registry 约束 trait

建议继续使用：

```rust
#[validator(
    id = "qubit.person.china_identity_card",
    depends_on(credential_type, gender, birthday),
    params(strict_area_code = true),
)]
pub credential_number: String,
```

不直接在模型中写 validator Rust 类型。原因是：

- metadata 需要稳定、可跨 crate 和跨语言识别的 ID；
- model crate 不应该依赖具体 validator 实现 crate；
- 前端和文档生成器也可以识别 ValidatorId；
- validator 实现可以在独立 crate 中替换。

具体实现必须满足类似契约：

```rust
pub trait FieldValidator<T: ?Sized>: Send + Sync + 'static {
    fn validate(
        &self,
        value: &T,
        context: &ValidationContext<'_>,
    ) -> ValidationResult;
}
```

注册时通过泛型约束保证实现类型正确：

```rust
registry.register::<String, ChinaIdentityCardValidator>(
    "qubit.person.china_identity_card",
    ChinaIdentityCardValidator,
);
```

这样 trait 约束发生在注册阶段：

- 实现没有实现 `FieldValidator<String>`：编译失败。
- StrategyId 没有注册：完整 validator registry 校验失败。
- 注册类型和字段类型不一致：registry 校验失败。
- `depends_on` 路径不存在：模型 schema 校验失败。

`ValidationContext` 只提供：

- 当前字段路径；
- `params`；
- `depends_on` 声明的字段值；
- 必要的校验模式和本地化上下文。

不提供完整 service context。

### F018-C：独立 `qubit-validator` 契约层

trait 不适合定义在：

- `rs-model-derive`：proc-macro crate 不应承载运行时验证协议；
- `rs-model-metadata`：它应该只保存静态 metadata，不执行验证。

建议直接把最小运行时契约放进独立的 `qubit-validator` crate，即使第一阶段只包含：

- `FieldValidator<T>`
- `ValidatorRegistry`
- `ValidationContext`
- `ValidationResult`
- `ValidationViolation`
- `ValidatorId`

`rs-model-metadata` 只保存：

```text
ValidatorId
params
depends_on
```

`rs-model-derive` 只生成这些 metadata。

Violation 建议保存稳定错误码、字段路径和消息参数，而不是直接绑定中文错误消息：

```rust
ValidationViolation {
    code: "invalid_china_identity_card",
    path: "credential.number",
    params: ...,
}
```

显示层再负责本地化。

### F020-A：删除字段级 `#[codec(...)]`

Java 实际用法说明 codec 是类型的规范外部表示能力：

```java
@TypeCodec(PhoneCodec.class)
public class Phone { ... }
```

因此建议删除字段级：

```rust
#[codec(...)] // 不再支持
pub phone: Phone,
```

改为类型级配置，具体语法在五种角色阶段确认，例如：

```rust
#[Value(codec = PhoneCodec)]
pub struct Phone {
    // ...
}
```

同一个类型在不同字段需要不同编码时：

- 定义不同的 wrapper Value；或者
- 使用具体 consumer 的 Serde/DAO 配置。

不提供字段级 codec override，避免同一领域类型没有稳定外部表示。

### F020-B：直接复用 `ValueEncoder`、`ValueDecoder`

根据当前 Java 的真实场景，先把模型 codec 精确定义为“规范文本 codec”。

对领域类型 `T` 和 codec 类型 `C`，要求：

```rust
C: Default
    + ValueEncoder<T, Output = String>
    + ValueDecoder<str, Output = T>
```

例如：

```rust
#[derive(Default)]
pub struct PhoneCodec;

impl ValueEncoder<Phone> for PhoneCodec {
    type Output = String;
    type Error = PhoneCodecError;

    fn encode(&mut self, value: &Phone) -> Result<String, Self::Error> {
        // ...
    }
}

impl ValueDecoder<str> for PhoneCodec {
    type Output = Phone;
    type Error = PhoneCodecError;

    fn decode(&mut self, value: &str) -> Result<Phone, Self::Error> {
        // ...
    }
}
```

规则：

- 不再定义新的 ModelCodec encode/decode trait。
- codec 是双向的，必须同时实现 encoder 和 decoder。
- decode 后的值仍需执行字段约束和 validator。
- codec error 转换为统一结构化错误。
- codec 与 Serde 的关系留到五种角色默认派生阶段确认。
- 目前只定义经过 Java 实际场景验证的文本 codec；不提前设计二进制或任意目标类型 codec。

这也意味着需要修订已经确认过的 C003/S001：

- `generator` 删除。
- `codec` 移到类型级能力。
- 字段级自定义策略只保留 `validator`。
- StrategyId、params、depends_on 公共约定最终只适用于 validator。

请确认或修订 F018-A～F018-C、F020-A～F020-B。

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

F018-A 同意
F018-B 同意，但是建议，为 FieldValidator 也提供类型宏，例如

#[Validator(id = "...")]
pub struct EmailValidator {
    fn validate(
        &self,
        value: &str,
        context: &ValidationContext<'_>,
    ) -> ValidationResult;
}

这样就自动为 EmailValidator 实现 FieldValidator<str> 以及自动用id注册这个validator。具体还需要哪些参数你可以设计

F018-C 同意

F020-A 同意，
F020-B 同意，但是也建议增加codec注册表，提供 #[Codec(id = "...")] 让一个类自动实现 ValueEncoder, ValueDecoder，具体还需要哪些参数你可以设计

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

你的建议可以同时保留“模型只依赖稳定 ID”和“实现必须满足 Rust trait”两点。由于 Rust 的 struct 体内不能直接定义方法，我会把语法调整为“宏标注无状态 validator/codec 类型，具体逻辑写在其 inherent impl 中”，宏负责生成 trait impl 和分布式注册项。

这个方向可行，而且能同时实现：

- 模型只依赖稳定 ID；
- validator/codec 实现满足 Rust trait；
- 自动注册；
- 完整 registry 阶段检查缺失和类型不匹配。

需要注意，你示例中把方法写在 struct 体内不是合法 Rust。最终形式应当是“类型宏 + unit struct + inherent impl”。

下面确认 5 项。

### V001：`#[Validator]` 类型宏

建议语法：

```rust
#[Validator(
    id = "qubit.text.email",
    value = str,
)]
pub struct EmailValidator;

impl EmailValidator {
    fn validate(
        &self,
        value: &str,
        context: &ValidationContext<'_>,
    ) -> ValidationResult {
        // 用户实现
    }
}
```

类型宏参数：

- `id = "ValidatorId"`：必填。
- `value = RustType`：必填，表示 validator 接收的值视图类型。

限制：

- 只支持无泛型 unit struct。
- validator 必须无状态。
- 配置通过字段 occurrence 上的 `params(...)` 传递，不保存在 validator 实例中。
- 宏自动生成默认构造能力。
- 宏自动生成 `FieldValidator<value>` 实现。
- 宏自动生成分布式 `ValidatorRegistration`。
- inherent `validate` 的签名或返回类型不正确时编译失败。

`value = str` 可以用于：

- `String`
- `Box<str>`
- 提供 text view adapter 的 Value

其他类型默认要求准确匹配。

### V002：小写 `#[validator]` 表示一次使用

类型定义与字段使用分开：

```rust
#[Validator(
    id = "qubit.person.china_identity_card",
    value = str,
)]
pub struct ChinaIdentityCardValidator;

#[validator(
    id = "qubit.person.china_identity_card",
    depends_on(credential_type, gender, birthday),
    params(strict_area_code = true),
)]
pub credential_number: String,
```

其中：

- 大写 `#[Validator]` 定义并注册实现。
- 小写 `#[validator]` 在字段上引用实现。
- `depends_on` 和 `params` 属于具体使用位置。
- 同一个 validator 可以在不同模型字段上复用不同的依赖路径和参数。
- 同一字段允许使用多个不同 validator。
- 同一字段重复相同 ValidatorId 报错。

完整 registry 校验：

- ValidatorId 是否已经注册；
- 注册值类型是否兼容字段类型；
- `depends_on` 路径是否存在；
- params 的基础字面量是否合法。

### K001：codec 类型宏

我建议宏命名为 `#[ValueCodec]`，而不是 `#[Codec]`。

`qubit-codec` 已经存在底层 `Codec` trait，继续使用 `Codec` 作为属性宏虽然技术上可以工作，但阅读时容易误以为它实现的是低层 buffer codec。这里实际实现的是 whole-value 的 `ValueEncoder`、`ValueDecoder`。

建议语法：

```rust
#[ValueCodec(
    id = "qubit.contact.phone",
    value = Phone,
    encode_error = PhoneEncodeError,
    decode_error = PhoneDecodeError,
)]
pub struct PhoneCodec;

impl PhoneCodec {
    fn encode(
        &mut self,
        value: &Phone,
    ) -> Result<String, PhoneEncodeError> {
        // 用户实现
    }

    fn decode(
        &mut self,
        input: &str,
    ) -> Result<Phone, PhoneDecodeError> {
        // 用户实现
    }
}
```

如果双向使用相同错误类型，可以简写：

```rust
#[ValueCodec(
    id = "qubit.contact.phone",
    value = Phone,
    error = PhoneCodecError,
)]
pub struct PhoneCodec;
```

参数：

- `id`：必填。
- `value`：必填，领域值类型。
- `error`：双向共用错误类型。
- 或者使用 `encode_error`、`decode_error` 分别指定。
- `error` 与两个独立错误参数互斥。

宏自动生成：

```rust
impl Default for PhoneCodec
impl ValueEncoder<Phone> for PhoneCodec
impl ValueDecoder<str> for PhoneCodec
```

以及分布式 `CodecRegistration`。

同样只支持无泛型 unit struct，带状态或配置的 codec 可以手动实现和注册。

这里需要你确认最终名称采用推荐的 `#[ValueCodec]`，还是仍采用 `#[Codec]`。

### K002：CodecRegistry 和 canonical codec

`CodecRegistry` 按稳定 `CodecId` 查询，注册项至少保存：

- CodecId；
- 领域值的 TypeIdentity；
- 外部表示种类，目前固定为 Text；
- codec Rust 类型信息；
- erased encode/decode 调用入口。

规则：

- CodecId 重复时报错。
- 注册类型与 `ValueEncoder`/`ValueDecoder` 类型不匹配时编译或 registry 校验失败。
- 同一领域类型可以注册多个 codec，例如规范格式、紧凑格式、旧协议格式。
- 但领域类型最多选择一个 canonical codec。

后续角色宏通过下面两种方式二选一选择：

```rust
#[Value(codec = PhoneCodec)]
```

或者：

```rust
#[Value(codec_id = "qubit.contact.phone")]
```

前者使用 Rust 类型并进行编译期检查；后者使用稳定 ID，在完整 registry 阶段解析。两者效果等价。

是否允许 Entity、Projection、Model、Enum 使用 canonical codec，放到角色权限阶段确认。

### S001 修订

由于 generator 已删除、codec 改为类型级 Rust codec，之前的策略公共约定修订为：

- StrategyId、`params(...)`、`depends_on(...)` 只用于小写字段宏 `#[validator(...)]`。
- `#[Validator]` 负责定义实现和自动注册。
- `#[ValueCodec]` 使用 Rust 类型、CodecId 和 qubit-codec traits。
- 不再存在字段级 `#[generator]` 和 `#[codec]`。
- C003“字段自定义处理策略”最终只剩 validator。
- codec 移到类型级规范表示能力。

请确认 V001、V002、K001、K002、S001；K001 还需要在 `ValueCodec` 与 `Codec` 两个名称中作出选择。

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

V001, V002, K001, K002 大体上都同意，下面是一些具体的意见

【宏自动生成分布式 ValidatorRegistration】什么叫做“分布式”？
另外 Validator 宏，应该定义在 rs-validator中
ValueCodec 宏，应该定义在 rs-codec 中
小写的validator, codec定义在rs-model-derive中，表示对字段的注解
同意宏定义为ValueCodec，建议注册表叫做 ValueCodecRegistry，id类型叫做 ValueCodecId

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

“分布式注册”指的不是网络分布式，而是：

- `Validator`、`ValueCodec` 可以定义在不同业务 crate。
- 每个类型宏在本 crate 生成一个静态 registration。
- 最终链接程序时，由 `linkme` 一类机制跨 crate 自动收集。
- 用户不需要维护一个中央列表。

最终文档不再使用“分布式注册”，统一称为：

> 链接期跨 crate 自动注册

crate 归属也按你的意见确定：

| 公共 API | 所属 crate |
|---|---|
| `#[Validator]`、`FieldValidator`、`ValidatorRegistry` | `rs-validator` |
| `#[ValueCodec]`、`ValueEncoder`、`ValueDecoder`、`ValueCodecRegistry`、`ValueCodecId` | `rs-codec` |
| `#[validator(...)]`、`#[codec(...)]` 字段 helper | `rs-model-derive` |

由于普通 Rust library 不能直接同时作为常规库和 proc-macro crate，`rs-validator`、`rs-codec` 内部可以使用 companion proc-macro crate，但统一从主 crate 重导出。用户只需要：

```rust
use qubit_validator::Validator;
use qubit_codec::ValueCodec;
```

下面还需要确认 5 个精确决策。

### V003：`#[Validator]` 最终参数

```rust
#[Validator(
    id = "qubit.text.email",
    value = str,
)]
pub struct EmailValidator;

impl EmailValidator {
    fn validate(
        &self,
        value: &str,
        context: &ValidationContext<'_>,
    ) -> ValidationResult {
        // ...
    }
}
```

只支持两个参数：

- `id = "ValidatorId"`：必填。
- `value = RustType`：必填。

宏负责：

- 自动实现 `Default`；
- 自动实现 `FieldValidator<value>`；
- 检查 inherent `validate` 方法签名；
- 生成 `ValidatorRegistration`；
- 参与链接期跨 crate 自动注册。

只支持无泛型 unit struct。需要有状态的特殊 validator 可以手动实现并注册，但不能使用自动宏。

`params` 和 `depends_on` 不属于 validator 类型，继续写在字段 occurrence 上：

```rust
#[validator(
    id = "qubit.text.email",
    depends_on(account_type),
    params(allow_local_domain = false),
)]
pub email: String,
```

### K003：`#[ValueCodec]` 最终参数

```rust
#[ValueCodec(
    id = "qubit.contact.phone",
    value = Phone,
    error = PhoneCodecError,
)]
pub struct PhoneCodec;
```

或者双向错误类型不同：

```rust
#[ValueCodec(
    id = "qubit.contact.phone",
    value = Phone,
    encode_error = PhoneEncodeError,
    decode_error = PhoneDecodeError,
)]
pub struct PhoneCodec;
```

参数：

- `id = "ValueCodecId"`：必填。
- `value = RustType`：必填。
- `error = RustErrorType`：双向共用错误。
- 或使用 `encode_error`、`decode_error`，两者必须同时出现。
- `error` 与两个独立错误参数互斥。

宏自动实现：

```rust
Default
ValueEncoder<Phone, Output = String>
ValueDecoder<str, Output = Phone>
```

并生成链接期自动收集的 `ValueCodecRegistration`。

同样只支持无泛型 unit struct；配置直接固化在不同 codec 类型中，不给 codec 实例传动态 params。

### K004：小写字段 `#[codec(...)]`

根据你最新的意见，保留字段级 codec，但它只负责选择，不负责定义实现。

支持两种等价形式：

```rust
#[codec(with = PhoneCodec)]
pub phone: Phone,
```

```rust
#[codec(id = "qubit.contact.phone")]
pub phone: Phone,
```

规则：

- `with` 接受 Rust codec 类型，进行编译期 trait 和类型检查。
- `id` 接受 `ValueCodecId` 字符串，在完整 `ValueCodecRegistry` 阶段解析。
- 两者必须二选一。
- 每个字段最多声明一个 codec。
- 不支持 `params`；不同配置应定义不同 ValueCodec 类型和 ID。
- codec 必须与字段解开最外层 `Option` 后的值类型兼容。
- `None` 不执行 encode/decode。
- 容器字段默认要求 codec 针对整个容器类型；不会自动把字段 codec 下沉到每个元素。容器元素 codec 的问题留到 F023 统一确认。

### K005：类型 canonical codec 与字段 codec 的优先级

建议规则：

1. 字段显式 `#[codec(...)]` 优先。
2. 字段没有 codec 时，使用字段值类型声明的 canonical codec。
3. 类型也没有 canonical codec 时，该字段没有 model codec。

例如：

```rust
#[Value(codec = PhoneCanonicalCodec)]
pub struct Phone {
    // ...
}

pub struct Contact {
    pub primary_phone: Phone,

    #[codec(with = PhoneCompactCodec)]
    pub compact_phone: Phone,
}
```

其中：

- `primary_phone` 使用 `PhoneCanonicalCodec`。
- `compact_phone` 使用字段显式选择的 `PhoneCompactCodec`。
- 字段显式选择与类型 canonical codec 完全相同时，属于冗余声明，派生报错。
- 同一领域类型可以在 `ValueCodecRegistry` 注册多个 codec，但类型最多选择一个 canonical codec。

角色宏使用：

```rust
#[Value(codec = PhoneCanonicalCodec)]
```

或者：

```rust
#[Value(codec_id = "qubit.contact.phone")]
```

两者二选一。其他角色是否允许 canonical codec，后续按角色确认。

### F017 修订：opaque 不再依赖字段 generator

因为 F019 已经删除，F017 中原来的“opaque 必须声明 generator”需要同步修订为：

- 默认对象生成器不能解析并构造 opaque 叶子。
- opaque 值必须由调用方直接提供，或者由对象生成系统在外部按类型注册生成能力。
- 不存在 `#[generator(...)]` 字段宏。
- 缺少 opaque 值来源时，生成流程返回结构化错误。

请确认 V003、K003、K004、K005，以及 F017 的这项同步修订。

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

V003确认
K003确认
K004确认
K005确认
F017确认

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

我会把 V003、K003～K005 和 F017 修订全部固化为已确认。下一批处理剩余的结构/输出类字段语义：`redact`、Serde/`keep_serializing`、Option/容器递归，以及 `computed` 的最终语法。

我会先把这五项确认状态核对进确认记录，然后检查现有脱敏、序列化与 `computed` 相关 API；下一批给你确认 4 项：`redact`、序列化控制、容器递归边界、`computed` 的精确语法。

核对后发现确认记录里还有两处旧措辞残留：C003 仍提到已删除的 `generator`，F018 仍标成“待确认”。我现在按你刚确认的最终结论修正记录；这只是更新确认文档，不改仓库文档或代码。

已将 V003、K003、K004、K005、F017 的确认结果写入确认记录，并修正了 C003、F018 中遗留的旧状态和 `generator` 表述。

下一批请确认以下 4 项。

### F021 `#[redact(...)]`

建议完全复用 `qubit-redact`，不在 `rs-model-derive` 中设计第二套脱敏规则。

字段只能选择一种模式：

- `#[redact(level = "low")]`
- `#[redact(level = "medium")]`
- `#[redact(level = "high")]`
- `#[redact(level = "secret")]`
- `#[redact(skip)]`
- `#[redact(nested)]`
- `#[redact(map)]`
- `#[redact(keyed_by = key_field)]`
- `#[redact(json)]`

具体语义：

- `level`：按指定敏感级别处理标量叶子。
- `skip`：启用脱敏时完全省略该字段。
- `nested`：调用字段值自身的 `Redact` 实现。比如：

  ```rust
  #[redact(nested)]
  pub email: EmailAddress,
  ```

  `EmailAddress` 内部定义了自己的脱敏策略，外层必须通过 `nested` 委托给它。
- `map`：把字段视为文本 key 的 Map，按照 key 和运行时策略处理 value。
- `keyed_by`：使用同一对象的兄弟文本字段为当前 payload 分类。
- `json`：字段保存 JSON 文本，解析后按照 JSON 字段策略递归脱敏。
- 未标注字段保持普通输出，不根据字段名猜测敏感性。
- 同一字段重复或组合多个模式，编译报错。

`FieldMetadata` 中保存规范化后的 `RedactionMode`，供接口文档、schema 和安全审查使用；真正执行脱敏仍交给 `qubit-redact`。

### F022 Serde 与 `#[keep_serializing]`

五种角色宏完整支持标准 `#[serde(...)]` 属性。Serde 本身已有的 `rename`、`skip`、`with`、`flatten` 等参数不重新定义。

建议默认规则：

- 只对具名字段自动省略：

  - `Option::None`
  - 空的标准集合

- 直接支持的集合：

  - `Vec`
  - `VecDeque`
  - `LinkedList`
  - `HashMap`
  - `BTreeMap`
  - `HashSet`
  - `BTreeSet`
  - `BinaryHeap`

- 反序列化时，缺失的 Option 和集合字段采用默认值。
- 固定数组不做空值省略。
- newtype、tuple struct 和 enum tuple payload 不自动省略字段，避免改变位置和序列化形状。

`#[keep_serializing]` 是无参数标记：

```rust
#[keep_serializing]
pub aliases: Vec<String>,

#[keep_serializing]
pub nickname: Option<String>,
```

它只关闭宏自动添加的 `skip_serializing_if`：

- `None` 输出为 `null`
- 空集合输出为 `[]` 或 `{}`

它不关闭反序列化的缺失默认值，也不覆盖用户显式写下的 `#[serde(skip...)]`。标在不会被自动省略的字段上属于冗余错误。

### F023 Option、Box 与容器递归规则

建议把类型结构分成透明包装层和语义容器。

透明包装层：

- `Option<T>`
- `Box<T>`
- `Rc<T>`
- `Arc<T>`

规则：

- `Option::None` 跳过标准值约束、字段 validator 和字段 codec。
- `Some(value)` 对内部 payload 执行这些规则。
- Box/Rc/Arc 解包后处理内部 payload。
- metadata 仍保留完整包装结构及 optionality。

容器不是透明层：

- sequence
- set
- array
- map

因此：

```rust
#[text(max_chars = 20)]
pub names: Vec<String>, // 非法
```

必须写成：

```rust
#[element(text(max_chars = 20))]
pub names: Vec<String>,
```

Map 使用：

```rust
#[map_key(text(...))]
#[map_value(text(...))]
pub attributes: HashMap<String, String>,
```

小写 `validator` 和 `codec` 在解开外层 Option/Box/Rc/Arc 后，作用于整个剩余值。也就是说，标在 `Vec<T>` 上时处理整个 `Vec<T>`，不会自动逐元素执行。

未标 `opaque` 的命名复杂类型会根据自己的 descriptor 自动递归校验；它出现在 Option、集合元素或 Map key/value 中时也一样。`opaque` 截断递归。

### F024 `#[computed(...)]`

这里存在一个 Rust 过程宏层面的客观限制：标注 struct 的 `#[Entity]` 等宏看不到另一个 `impl` 块中的 getter。因此建议增加实现块宏 `#[ModelProperties]`：

```rust
#[ModelProperties]
impl User {
    #[computed(depends_on(
        first_name,
        last_name,
        email,
    ))]
    pub fn info(&self) -> UserInfo {
        // ...
    }
}
```

规则：

- `#[computed]`：没有精确声明依赖，保守地视为依赖全部存储字段。
- `#[computed(depends_on(...))]`：精确声明依赖的存储字段或其他 computed property。
- `#[computed(depends_on())]`：明确表示无依赖。
- property 名就是方法名，不支持 `name`。
- 不支持 `indexed`、`unique`、`identifier`、`reference` 等持久化参数。

Getter 必须满足：

```rust
pub fn name(&self) -> T
```

并且：

- 必须是 public；
- 只能接收 `&self`；
- 不能是 async、unsafe 或泛型方法；
- 不能有其他参数；
- 返回 owned、`'static` 的值；
- `T` 可以是 Option、Value 或其他复杂对象；
- 必须无副作用，并且结果只能由声明的依赖决定；
- 依赖路径必须存在；
- computed property 之间不能形成依赖环。

computed property 会进入 metadata，允许：

- 被 schema 和接口描述发现；
- 递归校验 getter 返回值；
- 被 `reference(property = info)` 选择；
- 在自动化对象装配完成后读取。

但它不是：

- 持久化字段；
- 数据库列；
- 构造输入；
- 随机生成槽位。

请确认 F021～F024，或者逐项提出调整。

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

F021 同意
但是要额外说明一点，就是这个脱敏策略，是可以穿透容器和Option的。
具体而言，对于
#[redact(level = "high")]
pub phone_numbers: Vec<String>,
#[redact(level = "medium")]
pub email: Option<String>,

这些指定的脱敏策略，可以做用到phone_numbers中每个元素，以及email中的some元素。对 redact的其他选项也是一样可以穿透作用。

F022 基本同意

F023 这里提到了option,box, 容器递归规则，和 F021 我的补充意见有相同的地方也有冲突的地方，按照F021，我希望 redact 可以递归做用到容器内元素，但是 F022 这里的意见也很有价值，所以我觉得也可以考虑用 #[element(redact(...))] 来实现对容器内元素的脱敏，以及 #[map_key(redact(...))], #[map_value(redact(...))] 对map的key和value做脱敏约束。不知道这个技术上能否做到。还要能允许element, map_key, map_value的括号内支持多个约束，例如 #[map_value(text(...), redact(...))]

另外，对于validator, codec 其实也可以用类似的方式，让他们支持对容器内元素的递归处理。
这样直接在 Vec<T>上标注#[validator(..)]处理的是整个Vec<T>，但是如果标注 #[element(validator(...))] 则可以处理Vec<T>中每个元素。

F024 同意

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

我会把 F021、F022、F024 记为已确认，并把 F021 的容器穿透语义补进去。F023 暂不确认：你的 selector 方案技术上可行，我会把它改成“字段级处理整个容器、selector 处理容器成员”的统一规则，同时单独处理 `redact(skip)` 和 Map key 脱敏这两个结构性边界。

F021、F022、F024 已按你的意见记录为确认。F023 的 selector 扩展在技术上完全可行，而且能形成更统一的设计。

建议将 F023 拆成下面四项确认。

### F023-A 透明包装与普通递归

以下类型是透明包装：

- `Option<T>`
- `Box<T>`
- `Rc<T>`
- `Arc<T>`

标准约束、validator、codec 遇到这些包装时：

- `None` 跳过处理；
- `Some(value)` 处理 `value`；
- Box/Rc/Arc 处理内部值；
- metadata 仍保留完整包装结构。

未标注 `opaque` 的 Value、Model、Enum 等命名复杂类型，仍按照自身 descriptor 递归校验。它们位于 Option、集合元素或 Map key/value 中时同样递归。

### F023-B selector 支持多个子规则

`element`、`map_key`、`map_value` 都允许同时声明多个不同类别的规则：

```rust
#[element(
    text(max_chars = 32),
    validator(id = "qubit.phone.syntax"),
    codec(with = PhoneCodec),
    redact(level = "high"),
)]
pub phone_numbers: Vec<String>;
```

```rust
#[map_value(
    text(max_chars = 64),
    validator(id = "qubit.attribute.value"),
    redact(level = "medium"),
)]
pub attributes: HashMap<String, String>;
```

允许的子规则：

- `text`
- `decimal`
- `money`
- `time`
- `validator`
- `codec`
- `redact`

组合限制：

- 可以有多个不同 Validator；
- 最多一个 codec；
- 最多一个 redact；
- 同一种标准约束最多一个；
- decimal 和 money 互斥；
- 不允许 identifier、indexed、unique、reference、computed；
- 不允许递归嵌套 `element(element(...))` 或 `map_value(map_value(...))`。

selector 仍只选择一层容器位置。更深的复杂结构应定义成具名 Value 类型。

### F023-C validator 与 codec 的作用位置

直接标在容器字段上时，处理整个容器：

```rust
#[validator(id = "qubit.phone_list")]
#[codec(with = PhoneListCodec)]
pub phone_numbers: Vec<Phone>;
```

这里 Validator 和 Codec 的目标类型都是 `Vec<Phone>`。

写在 selector 中时，逐个处理成员：

```rust
#[element(
    validator(id = "qubit.phone"),
    codec(with = PhoneCodec),
)]
pub phone_numbers: Vec<Phone>;
```

这里：

- Validator 对每个 `Phone` 执行；
- Codec 对每个 `Phone` 执行；
- 如果元素是 `Option<Phone>`，None 跳过，Some 中的 Phone 被处理。

Map 同理：

```rust
#[map_key(codec(with = AttributeKeyCodec))]
#[map_value(validator(id = "qubit.attribute_value"))]
pub attributes: HashMap<AttributeKey, AttributeValue>;
```

标准值约束不会从容器字段自动下沉，所以以下写法仍然非法：

```rust
#[text(max_chars = 32)]
pub names: Vec<String>;
```

必须使用 `element(text(...))`。

### F023-D redact 的特殊递归能力

`redact` 与 validator、codec 不完全相同。它自身具有容器递归能力。

下面的字段级写法直接递归到所有元素：

```rust
#[redact(level = "high")]
pub phone_numbers: Vec<String>;

#[redact(level = "medium")]
pub email: Option<String>;
```

也允许使用显式 selector：

```rust
#[element(redact(level = "high"))]
pub phone_numbers: Vec<String>;

#[map_key(redact(level = "medium"))]
#[map_value(redact(level = "high"))]
pub secrets: HashMap<String, String>;
```

两种写法的区别是：

- 字段级 redact 自动穿透 Option、Box/Rc/Arc、sequence、set、array；
- `element(redact(...))` 明确只选择元素分支，然后 redact 可以继续穿透元素内部的透明包装和容器；
- Map 上的字段级 redact 默认只进入 value，不修改 key；
- 要脱敏 Map key，必须显式使用 `map_key(redact(...))`；
- 同一作用路径同时声明字段级 redact 和 selector redact，属于重复或歧义，编译报错。

不同模式会寻找与自己能力匹配的内部值：

- `level`：寻找支持标量脱敏的叶子；
- `nested`：寻找实现 `Redact` 的值；
- `json`：寻找支持的 JSON 文本；
- `map`：寻找 Map；
- `keyed_by`：使用兄弟字段分类，然后递归处理当前 payload。

我建议给 `skip` 保留一个必要例外：

- `#[redact(skip)]` 始终省略整个字段，不递归到元素；
- 不允许 `element(redact(skip))`、`map_key(redact(skip))` 或 `map_value(redact(skip))`。

因为“删除数组元素”“删除 Map key 但保留 value”没有统一含义，对固定数组甚至无法保持类型形状。如果要保留字段但输出空容器，应通过专门的输出 DTO 表达，而不是把 `skip` 解释为元素变换。

另外，Map key 脱敏后可能产生相同的输出 key。安全序列化不得静默覆盖条目；发生冲突时应返回结构化序列化错误。

请确认 F023-A～F023-D，重点看两个决定：

1. Map 的字段级 redact 默认只递归 value，key 必须显式 `map_key(redact(...))`。
2. `redact(skip)` 只作用于整个字段，禁止放进 selector。

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

同意

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

我先把 F023-A～F023-D 的最终规则写入确认记录，包括 selector 多规则组合、Map 默认只处理 value，以及 `redact(skip)` 的结构性例外。随后我会核对字段宏清单，给出下一批尚未确认的字段级语义。

字段宏清单核对时发现还有两类不能直接略过：Enum 的 `#[variant(...)]`，以及 Java 实际场景中的 `@Modified` / `@Unmodified`。后者可能表达“更新测试时哪些字段允许或必须变化”，不能在没有评估用途前简单删除。我会先核对它们在 Java 模型和测试框架中的真实使用，再决定是否需要新增一类字段语义。

F023-A～F023-D 已完整记录为确认。

核对 Java 实际使用后，字段/变体语义还剩下面 4 项需要确认。

### F025 `#[variant(...)]`

只允许标在 Enum variant 上：

```rust
#[Enum]
pub enum UserState {
    Pending,
    Active,

    #[variant(name = "LOCKED")]
    Locked,
}
```

建议只支持一个参数：

```rust
#[variant(name = "CANONICAL_NAME")]
```

规则：

- 省略时，从 Rust variant 名转换为 `SCREAMING_SNAKE_CASE`；
- 名称不能为空；
- 同一个 Enum 内不能重复；
- 源码声明顺序形成 ordinal；
- 不增加 `code`、`ordinal`、`weight`、`default` 等参数。

规范名供以下 API 使用：

- `EnumVariantMetadata::canonical_name()`
- `Enum::name()`
- `Display`
- unit Enum 的 `from_name()`

它也作为宏生成的默认 Serde wire name。但按照已确认的 F022，显式 `#[serde(rename = "...")]` 仍然优先，允许 wire name 与 canonical name 有意不同。metadata 会分别暴露：

- canonical name
- serialize name
- deserialize name
- Serde aliases

随机生成 Enum variant 时的概率或权重属于某次生成任务的配置，不进入 `#[variant]`。

### F026 不增加字段级 `modified` / `unmodified`

Java 的 `@Modified`、`@Unmodified` 实际标在 DAO 方法上：

```java
@Modified({"state", "modifyTime"})
void updateState(...);

@Unmodified({"id", "code", "createTime"})
void update(...);
```

它们描述的是“执行某个操作以后，哪些字段应该变化或保持不变”，而不是字段自身永恒不变的属性。同一个字段在 add、update、delete 中可能有不同预期。

因此建议：

- `rs-model-derive` 不提供字段级 `#[modified]` 或 `#[unmodified]`；
- 将来由 DAO/API 的操作宏按结构化字段路径声明；
- model metadata 只负责提供可引用的字段路径。

例如 add 操作修改 `id/create_time/modify_time/delete_time`、软删除操作修改 `delete_time`，都属于 DAO operation metadata。

### F027 不增加通用 `#[exclude]`

Java 的 `@Exclude` 目前只用于 common-random 的测试夹具，表示某个随机化过程不要处理该字段。它不是 validation、持久化或输出层面的不变量。

建议最终设计不提供：

```rust
#[exclude]
```

替代规则：

- 默认生成器无法生成的外部类型：使用 `#[opaque]`，由调用方供值；
- 某一次生成任务需要排除普通字段：在生成请求中使用结构化字段路径配置；
- 数据库负责产生的 identifier：使用已经确认的 `assigned_by = database`；
- computed property 本身不是生成槽位，不需要 exclude。

这样不会把某个随机生成器的临时策略固化到模型定义中。

### F028 不增加 `#[key_index(...)]`

Java `@KeyIndex(n)` 用来规定复杂对象在复合数据库索引或字符串 key 中的成员顺序：

```java
class Owner {
    @KeyIndex(0)
    String type;

    @KeyIndex(1)
    Long id;
}
```

新设计中这三个用途已经分离：

- 可查询性使用结构化字段路径；
- 物理复合数据库索引属于持久化层；
- 复杂值的规范字符串表示由 `ValueCodec` 决定；
- metadata 需要稳定遍历时使用源码字段声明顺序。

因此建议不提供：

```rust
#[key_index(0)]
```

否则它会形成第二套字段顺序，并可能与 struct 声明顺序、ValueCodec 顺序或数据库索引顺序互相冲突。

请确认 F025～F028。确认后，字段与 Enum variant 层面的宏语义应该就完整了，下一阶段可以开始逐项确认 `Entity`、`Projection`、`Model`、`Enum`、`Value`。

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

F025，F026, F027 都接受
F028 不接受，KeyIndex是有用的，具体分析如下：

结论：`@KeyIndex` 是“复合键各分量的稳定排序元数据”，不是数据库/ORM 索引声明。它不会建索引、不会影响 SQL，也不参与 JPA/MyBatis 映射。

- 注解仅能标在字段上、运行期可反射读取；`value` 从 0 开始表示顺序：[KeyIndex.java](/home/starfish/working/qubit/java-common/common-annotation/src/main/java/ltd/qubit/commons/annotation/KeyIndex.java:19)。
- 框架反射层由 `BeanProperty#getKeyIndex()` 读取该注解并返回索引值：[BeanProperty.java](/home/starfish/working/qubit/java-common/common-java/src/main/java/ltd/qubit/commons/reflect/BeanProperty.java:719)。

在你指定的三个模块中：

- `common-model` 的 `Owner` 将 `type/id/property` 定义为 `0/1/2`：[Owner.java](/home/starfish/working/qubit/java-common/common-model/src/main/java/ltd/qubit/model/commons/Owner.java:78)；`Owners` 同样是 `type/ids/property`：[Owners.java](/home/starfish/working/qubit/java-common/common-model/src/main/java/ltd/qubit/model/commons/Owners.java:49)。这些类自身不消费注解，只提供元数据。
- `common-integration-test` 的 DAO 测试工具会对“没有 `@Identifier` ID 字段”的对象，取全部普通属性，按 `KeyIndex` 排序，并用 `-` 拼出确定的字符串：[DaoTestUtils.java](/home/starfish/working/qubit/java-common/common-integration-test/src/main/java/ltd/qubit/commons/test/dao/impl/DaoTestUtils.java:225)。测试明确验证 `Owner` 被编码为 `type-id`：[DaoTestUtilsTest.java](/home/starfish/working/qubit/java-common/common-integration-test/src/test/java/ltd/qubit/commons/test/dao/DaoTestUtilsTest.java:108)。

全仓 Java 源码的实际消费者还有一处：

- `common-random` 的 `UniqueValueRandomizer` 也按 `KeyIndex` 排序后生成对象的字符串表示：[UniqueValueRandomizer.java](/home/starfish/working/qubit/java-common/common-random/src/main/java/ltd/qubit/commons/random/randomizers/misc/UniqueValueRandomizer.java:240)。该表示在忽略大小写的唯一值缓存比较中使用：[UniqueValueRandomizer.java](/home/starfish/working/qubit/java-common/common-random/src/main/java/ltd/qubit/commons/random/randomizers/misc/UniqueValueRandomizer.java:198)。

因此，注释里“数据库复合索引”的说法更像设计背景；代码中的实际用途是让复合对象序列化为键字符串时保持列/键分量顺序，例如 `TYPE-123-property`，避免依赖反射属性遍历顺序。


>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

你给出的证据改变了 F028 的归类：它确实不是物理索引配置，而是复杂值形成稳定复合键时的分量顺序。我会先把 F025～F027 记为确认，再核对 Java 消费逻辑的缺号、重复、未标注字段和 identifier 优先级，然后重新设计 Rust 版 `#[key_index(...)]` 的精确契约。

核对结果与你的分析一致，而且 Java 版本还有一个值得在新设计里收紧的问题：它允许未标注字段和重复/缺号位置，最终会退回反射顺序，无法真正保证稳定。Rust 新版应保留 `key_index`，但把“完整、连续、无重复”的规则放到编译期，并把它独立归类为“规范复合键组成”，不暗示数据库索引。

你的分析成立，我撤回删除 `KeyIndex` 的建议。F025～F027 已记录为确认；F028 改为下面的新设计。

### F028-A 精确定义

新增类别 C007“规范复合键组成”。

```rust
#[Value]
pub struct Owner {
    #[key_index(0)]
    pub entity_type: String,

    #[key_index(1)]
    pub id: Id,

    #[key_index(2)]
    pub property: Option<String>,
}
```

`#[key_index(n)]` 表示字段在所属对象的规范复合键中的分量位置。

它：

- 不创建数据库索引；
- 不影响 SQL、ORM 或字段映射；
- 不隐含 `#[indexed]`；
- 不隐含 `#[unique]`；
- 不表示 identifier；
- 只定义复合键的分量顺序。

DAO 测试、唯一值缓存、自动对象装配等消费者统一读取 `CompositeKeyMetadata`，不再各自按字段反射顺序拼接。

### F028-B 语法与完整性

唯一语法：

```rust
#[key_index(0)]
```

参数是从 0 开始的 `u16`，不支持 `name`、`order` 等其他参数。

建议只允许：

- 具名字段 `Model`
- 具名字段 `Value`

以下角色禁止：

- Entity：始终以 identifier 作为对象键；
- Projection：始终使用来源 Entity 的 identifier；
- Enum：使用规范 variant 和 payload 表示；
- newtype Value：只有一个隐含的第 0 分量，无需标注；
- computed property：不是存储字段。

为了真正保证稳定性，只要一个类型出现了 `key_index`：

- 所有存储字段都必须标注；
- 索引必须完整覆盖 `0..field_count`；
- 不允许重复；
- 不允许缺号；
- 不允许依赖未标注字段的源码顺序。

这比 Java 版本更严格：Java 对 null KeyIndex 的排序最终仍可能依赖反射属性顺序，新设计直接在编译期消除这种不稳定性。

### F028-C 分量的递归编码能力

每个复合键分量必须具有规范键表示能力，包括：

- 内建 scalar；
- Enum；
- Entity/Projection，通过 identifier 表示；
- 具有 canonical ValueCodec 的类型；
- 自身具有完整 CompositeKeyMetadata 的 Model/Value。

容器规则：

- `Option::None` 与 `Some("")` 必须产生不同表示；
- sequence 和 array 保留元素顺序；
- Set 必须按元素的规范编码排序；
- Map 必须按 key、value 的规范编码排序；
- HashSet/HashMap 不得依赖运行时迭代顺序；
- opaque 只有提供 key capability adapter 或 canonical codec 时才能作为分量。

`key_index` 只定义分量顺序，不规定用 `-` 拼接。Java 的 `TYPE-123-property` 容易出现分隔符碰撞，新版公共 key encoder 应使用带长度或类型边界的无歧义编码；具体 API 在 runtime metadata 阶段确认。

### F028-D 键来源与安全边界

对象键规则：

- Entity/Projection：始终使用 identifier；
- scalar：使用内建规范值；
- Enum：使用规范 variant 和 payload；
- 具名 Model/Value：可以使用完整 CompositeKeyMetadata；
- newtype Value：使用其唯一内层值。

如果同一个 Model/Value 同时声明：

- canonical ValueCodec；
- `key_index` 复合键；

会产生两个竞争的规范表示来源，因此建议编译报错，要求明确二选一。

复合键用于：

- 等值比较；
- 唯一值缓存；
- DAO 自动化测试；
- 对象装配；
- 稳定协议键。

它不等同于 Display 或 Serde，也不能绕过 `redact` 直接输出到日志。敏感字段可以参与内部复合键计算，但输出键时仍必须经过安全策略。

请确认 F028-A～F028-D。

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

补充下java版的KeyIndex 的目前实际用途

当前代码里，它只用在“把对象规范化成字符串”的两个场景：

1. `common-integration-test`：组装 `@Unique` 的 `respectTo` 值字符串，供 DAO 测试校验 `DuplicateKeyException` 的重复键内容。
   `Owner(type,id,property)` 会表示成 `type-id-property`。
   [DaoTestUtils.java](/home/starfish/working/qubit/java-common/common-integration-test/src/main/java/ltd/qubit/commons/test/dao/impl/DaoTestUtils.java:268)

2. `common-random`：当 `@Unique` 字段本身是对象类型、并且开启忽略大小写时，把该字段对象转成字符串后作为随机去重缓存的比较值。
   [UniqueValueRandomizer.java](/home/starfish/working/qubit/java-common/common-random/src/main/java/ltd/qubit/commons/random/randomizers/misc/UniqueValueRandomizer.java:198)

它不用于：

- 数据库真实复合索引的创建或查询；
- ORM 映射；
- `@Unique(respectTo=...)` 的随机生成去重键——这条旧逻辑已被注释移除。

所以更准确地说，`KeyIndex` 目前是一个“对象转键字符串时的字段排序约定”。在你关心的 `Owner` 用作 `respectTo` 的常见场景中，复合值主要是 DAO 测试框架用于构造、核对重复键异常信息，而非业务运行时的数据库键。

你可以根据这个信息更新下对key_index的设计（如果有必要更新的话）

另外我对这个key_index名字不满意，它不能很准确地描述出其用途

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

这个补充会明显收窄设计：我之前把它扩展成“通用对象键/稳定协议键”确实过度了。新版应只表达“对象投影为比较或诊断用键文本时，字段作为第几个分量”，不参与对象身份、查询、装配或数据库键；名称也应去掉容易让人联想到数据库索引的 `index`。

这个补充有必要更新设计。我之前将它扩展为通用稳定协议键，范围过大。现在建议把 F028 收敛为下面的语义。

### F028-R2-A 精确定义

它只描述：

> 将复杂对象投影为比较或诊断用的键文本时，各存储字段作为分量的稳定顺序。

主要消费者是：

- DAO 测试构造和核对重复键异常文本；
- `unique(ignore_case)` 复杂字段的随机去重比较。

它不用于：

- 业务身份；
- 数据库索引；
- ORM 映射；
- 查询过滤；
- 对象装配；
- `respect_to` 的随机生成键；
- Display 或 Serde。

因此将其独立归入 C007“键文本投影”，不再归入身份、查询和持久化类别。

### F028-R2-B 更名为 `#[key_part(...)]`

我建议最终名称为：

```rust
#[key_part(order = 0)]
```

例如：

```rust
#[Value]
pub struct Owner {
    #[key_part(order = 0)]
    pub entity_type: String,

    #[key_part(order = 1)]
    pub id: Id,

    #[key_part(order = 2)]
    pub property: Option<String>,
}
```

选择这个名字的原因：

- `key` 表示用途是形成比较或诊断键；
- `part` 表示字段是复合表示的一个分量；
- `order` 明确数字表达的是顺序；
- 不再使用容易被误解为数据库索引的 `index`。

其他候选名中：

- `key_text_part` 更具体，但会让人误以为 metadata 已经规定最终字符串编码；
- `canonical_part` 没说清楚“什么东西”的 canonical；
- `key_order` 只表达顺序，没有表达字段也是键的组成部分。

因此推荐 `key_part(order = n)`。

### F028-R2-C 完整性规则

建议只允许具名字段的 Model 和 Value 使用。

只要一个类型出现 `key_part`：

- 所有存储字段都必须标注；
- `order` 必须从 0 开始；
- 必须连续；
- 不允许重复或缺号；
- computed property 不参与。

例如三个字段必须形成 `0, 1, 2`。

禁止用于：

- Entity、Projection：对象键始终由 identifier 决定；
- Enum：由规范 variant 和 payload 表示；
- newtype Value：只有一个隐含分量，无需排序；
- computed property：不是存储字段。

这比 Java 版更严格，因为 Java 的未标注字段最终可能退回反射遍历顺序，无法真正保证稳定性。

### F028-R2-D metadata 与运行时表示

宏只生成有序字段元数据：

```rust
pub struct KeyTextMetadata {
    parts: &'static [KeyPartMetadata],
}

pub struct KeyPartMetadata {
    order: u16,
    field: &'static FieldMetadata,
}
```

消费者通过 metadata 取得按 `order` 排列的字段值。

底层比较不建议直接拼接字符串，而应先形成结构化值，例如：

```rust
pub enum KeyComponentValue {
    Null,
    Scalar(/* ... */),
    Sequence(Vec<KeyComponentValue>),
    Object(Vec<KeyComponentValue>),
}
```

这样可以区分：

- `None`
- `Some("")`
- 字符串中原本包含 `-`
- 不同的容器和嵌套边界

DAO 测试需要异常文本时，再使用指定格式渲染：

```text
type-id-property
```

随机去重缓存则直接比较结构化分量，并对文本分量执行大小写规范化，不需要依赖可能碰撞的连接字符串。

`ValueCodec` 与 `key_part` 可以共存：

- ValueCodec：外部编码和解码；
- key_part：内部比较或诊断的字段投影。

消费者必须明确选择哪一种，不设置 Java 版那样的隐式优先级。

另外，key-text projection 可能包含敏感字段；它不会自动进入 Display、Serde 或日志，也不能绕过 `redact`。

请确认 F028-R2-A～F028-R2-D，尤其是最终命名 `#[key_part(order = n)]`。

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

F028-R2-A：部分同意
几个更正：
1.  key_part 不仅可以用于 unique(ignore_case=true)，也可以用于 unique(ignore_case=false)的情况；
2. key_part 也应该可以用于为 respect_to(...) 里面的属性值产生复合键，java没这么做是因为后来把这个功能去除了，为了更安全地生成唯一键；但不排除以后还会加上这个能力，所以rust要保留这个可能
其他都正确

F028-R2-B: 完全同意

F028-R2-C: 部分同意，
【只要一个类型出现 key_part，所有存储字段都必须标注】这一条不同意，允许有的字段不标注
其他都同意

F028-R2-D : 同意

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

我会按你的修订将 F028-R2 定稿：`key_part` 既支持大小写敏感和不敏感的 unique，也为 `respect_to(...)` 中的复杂属性保留复合投影能力；未标注字段不参与键投影，不再要求所有存储字段全部标注。这样 `key_part` 同时负责“选择分量”和“规定所选分量顺序”。

F028 已按补充信息最终修订并记录：

- 最终名称：`#[key_part(order = n)]`
- 未标注字段不参与投影；
- 已标注分量的 order 必须从 0 连续、无重复、无缺号；
- 同时支持 `unique(ignore_case = true/false)`；
- 支持 `respect_to(...)` 中复杂属性的复合键投影；
- 只产生结构化键文本投影 metadata，不涉及数据库索引、ORM、查询或对象身份；
- ValueCodec 可以与 key_part 共存。

至此，字段和 Enum variant 层面的语义已经全部确认。下面开始确认五种角色本身的精确定义。

### R001 `#[Entity]`

Entity 是拥有独立领域身份和独立持久化生命周期的对象。

核心特征：

- 参与全局注册；
- 必须有稳定 `ModelId`；
- 必须有且只能有一个直接的、准确类型为 `Id` 的 identifier；
- identifier 同时表示领域实例身份和持久化主键；
- 可以被 `#[reference(...)]` 指定为目标 Entity；
- 可以独立创建、读取、更新和删除；
- 可以声明 unique、indexed、reference 等持久化和查询语义。

典型类型：

- User
- Order
- Tenant
- Organization
- Device

“独立持久化生命周期”是关键。一个类型即使最终嵌入同一张表，只要它没有独立身份和 repository 生命周期，就不应定义成 Entity。

### R002 `#[Projection]`

Projection 是从一个 Entity 实例派生出来的只读表示。

核心特征：

- 参与全局注册；
- 没有自己的 `ModelId`；
- 必须有且只能有一个直接的、准确类型为 `Id` 的 identifier；
- identifier 是来源 Entity identifier 的复制或投影，不是 Projection 自己的新身份；
- 没有独立创建、更新、删除和持久化生命周期；
- 不作为 `reference(entity = ...)` 的目标；
- 可以作为 Entity 的 computed property 返回值，并被：

  ```rust
  #[reference(
      entity = User,
      property = info,
  )]
  ```

  这样的 relation 保存。

典型类型：

- UserInfo
- OrderSummary
- OrganizationOverview

一个 Projection 可以只有一个固定来源 Entity，也可以由多个 Entity 产生。是否通过 `source` / `source_id` 限制来源，将在 Projection 参数批次单独确认。

### R003 `#[Model]`

Model 是可被框架发现和操作的数据结构契约，但没有独立领域身份和持久化生命周期。

核心特征：

- 参与全局注册；
- 没有 `ModelId`；
- 禁止 identifier；
- 不是独立数据库记录；
- 不能作为 `reference(entity = ...)` 的目标；
- 可以包含指向 Entity 的 reference；
- 可以作为 validation、随机生成和 schema 处理的根节点；
- 可以嵌套其他 Model、Value 和 Enum。

典型类型：

- REST 请求参数；
- REST 响应；
- 查询过滤器；
- Command；
- 配置对象；
- 多个 Entity/Value 组成的结果；
- DAO 自动化测试的输入模型。

Model 和 Value 的核心区别是：Model 是需要全局发现、可以成为处理根节点的数据契约；Value 是不注册、只能依附于外层类型存在的值。

### R004 `#[Enum]`

Enum 是封闭值域或代数和类型。

核心特征：

- 参与全局注册；
- 没有 `ModelId`；
- 禁止 identifier；
- 没有独立持久化生命周期；
- 不能声明 direct reference；
- 支持 unit、tuple、struct 和混合 variant；
- 每个 variant 有稳定 canonical name；
- data variant 的 payload 拥有完整字段 descriptor，可以递归校验和生成。

典型类型：

- Gender
- State
- OrderStatus
- 带数据的成功/失败结果；
- 互斥的领域状态及其载荷。

Enum 注册的目的是让框架动态发现其封闭取值、variant metadata 和 payload 结构，不代表 Enum 是有身份的 Model。

### R005 `#[Value]`

Value 是只由内容决定语义和相等性的值对象。

核心特征：

- 不参与全局注册；
- 没有 `ModelId`；
- 禁止 identifier；
- 没有独立生命周期；
- 没有 relation 或持久化属性；
- 拥有完整静态 descriptor；
- 外层 Entity、Projection、Model、Enum 或其他 Value 可以递归读取其字段约束；
- 因此即使不注册，也能自动递归 validation 和合法随机生成。

典型类型：

- EmailAddress
- Phone
- Money
- Coordinate
- Revision
- Owner 这样的复合值

Value 可以包含：

- scalar；
- Enum；
- 其他 Value；
- Option 和容器；
- 显式 opaque 外部值。

Value 不能直接或间接包含 Entity、Projection、Model 或 reference，否则它就不再是纯内容值。

请确认 R001～R005。确认之后，下一批开始逐项确定五种角色分别支持的 Rust 形状和宏参数。

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

全部确认

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

我会把 R001～R005 全部写入确认记录。下一批将确认 4 项：Entity 的形状和参数、Projection 的 `source/source_id` 是否保留及其真实作用、Projection 的可执行转换协议、Model 的形状和参数。这里会明确区分“来源约束”和“如何实际生成 Projection”。

本批只确认角色专属的形状和参数；`codec`、默认派生以及 `no_*` 等通用行为稍后统一确认。

### R006 `#[Entity]` 的形状与参数

只支持具名字段 struct：

```rust
#[Entity(id = "qubit.platform.iam.User")]
pub struct User {
    #[identifier]
    pub id: Id,

    // ...
}
```

唯一必填的角色专属参数：

```rust
id = "ModelId"
```

这里的属性名就是 `id`，不是 `model_id`。

规则：

- ModelId 必须全局唯一；
- 区分大小写；
- 由至少两个点分隔的 ASCII 段组成；
- 每段必须以 ASCII 字母开头，后续允许字母、数字和下划线；
- 推荐 namespace 使用 lower_snake_case、末段使用 UpperCamelCase；
- 末段不强制等于 Rust 类型名，使 Rust 重命名不必改变稳定协议 ID。

例如：

```text
qubit.platform.iam.User
qubit.platform.order.Order
```

Entity 禁止：

- unit struct；
- tuple struct；
- enum；
- 泛型和 lifetime 参数；
- `where` 子句；
- union。

字段可以是 public 或 private，宏生成的字段访问器不要求破坏领域封装。

Entity 角色参数中不提供：

- `index(...)`
- `unique(...)`
- `projection(...)`
- `primary_key(...)`

这些语义已经分别由字段 annotation、identifier 和 computed property 表达。

### R007 `#[Projection]` 的形状与来源参数

Projection 只支持具名字段 struct，并且必须包含唯一的 `Id` identifier。

支持三种形式：

```rust
#[Projection]
pub struct CommonInfo {
    #[identifier]
    pub id: Id,
}
```

```rust
#[Projection(source = User)]
pub struct UserInfo {
    #[identifier]
    pub id: Id,
}
```

```rust
#[Projection(source_id = "qubit.platform.iam.User")]
pub struct UserInfo {
    #[identifier]
    pub id: Id,
}
```

规则：

- `source = RustType` 与 `source_id = "ModelId"` 最多提供一个；
- `source` 在编译期要求目标实现 Entity 协议；
- `source_id` 在完整注册表校验时解析到 Entity；
- 两者效果等价；
- Projection 自己没有 `id`/ModelId 参数；
- `source` 不是 Projection 的注册 ID。

三种语义：

- 省略来源：开放 Projection，允许由多个 Entity 产生；
- 指定 `source`：限制为某个 Rust Entity 类型；
- 指定 `source_id`：限制为某个稳定 ModelId 对应的 Entity。

我认为 `source/source_id` 有用，但它只表达“来源约束和数据血缘”，不负责实际转换。它能够：

- 防止 UserInfo 被错误地声明成 Order 的 Projection；
- 让 schema 和接口文档显示 Projection 来源；
- 支持由 SQL 查询直接构造、没有 computed getter 的 Projection；
- 在完整注册表校验时检查 identifier 和来源角色。

### R008 Projection 的可执行转换协议

`source` 只说明“从谁来”，不会自动生成转换逻辑。

真正的转换由已确认的 computed getter 提供：

```rust
#[ModelProperties]
impl User {
    #[computed(depends_on(
        id,
        username,
        nickname,
        email,
        state,
        delete_time,
    ))]
    pub fn info(&self) -> UserInfo {
        UserInfo {
            id: self.id,
            username: self.username.clone(),
            nickname: self.nickname.clone(),
            email: self.email.clone(),
            state: self.state,
            delete_time: self.delete_time,
        }
    }
}
```

`ModelProperties` 会登记：

- producer Entity：User；
- computed property：info；
- output Projection：UserInfo；
- 依赖字段；
- erased getter。

于是下面的 reference 可以精确解析并执行该 getter：

```rust
#[reference(
    entity = User,
    property = info,
)]
pub user: UserInfo,
```

完整注册表必须校验：

- getter 所属类型是 Entity；
- getter 返回 Projection；
- fixed-source Projection 的 source 与 getter 所属 Entity 一致；
- 返回 Projection 的 identifier 在运行时等于来源 Entity identifier；
- reference 字段类型与 getter 返回类型一致。

开放 Projection 可以有多个 producer。引用时通过 `entity` 和 `property` 唯一确定具体 producer。

如果 Projection 只有 `source/source_id`，但没有 computed getter：

- 它仍可以由 DAO/SQL mapper 或反序列化过程直接构造；
- metadata 仍能表达来源；
- 默认随机对象生成器不能凭空知道映射逻辑；
- 需要生成时必须提供 Projection projector adapter，否则返回明确的缺少 projector 错误。

因此，`source` 有数据血缘和约束价值，但不冒充转换函数。

### R009 `#[Model]` 的形状与参数

Model 支持：

```rust
#[Model]
pub struct FindUserRequest {
    // ...
}
```

以及无字段标记类型：

```rust
#[Model]
pub struct RefreshCache;
```

也就是：

- 具名字段 struct；
- unit struct。

不支持 tuple struct。单字段领域包装应使用 Value，而不是 Model：

```rust
#[Value(transparent)]
pub struct EmailAddress(String);
```

Model 不接受角色专属参数：

```rust
#[Model]
```

以下写法非法：

```rust
#[Model(id = "...")]
```

Model：

- 参与注册；
- 通过 Rust `TypeIdentity` 标识；
- 没有 ModelId；
- 禁止 identifier；
- 不能作为 reference 的 Entity 目标；
- 可以包含 reference 字段；
- 可以作为 validation、schema 和自动生成的根节点。

同样禁止泛型、lifetime 参数、`where` 子句、enum 和 union。

请确认 R006～R009，尤其是 Projection 的结论：保留 `source/source_id`，但把它严格限定为来源约束，不让它承担 projector 的职责。

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

R006：大部分赞同
意见：
- 【ModelId必须由至少两个点分隔的 ASCII 段组成】：这个不赞同，不应该给这个限制，允许有0个或多个句点分割成若干段，0个句点说明它就只有一段
-  【每段必须以 ASCII 字母开头，后续允许字母、数字和下划线】这个同意
其他都同意
基本上，我们希望，ModelId 符合 java 的 class 的 full name 的形式

R007：同意。

但我有个想法，我们是否可以在 Entity 中增加 projection/projection_id 的可选属性，即

Entity(id="qubit.platform.address.Country", projection = Info)
pub struct Country {
   ...
}

Entity(id="qubit.platform.iam.User", projection_id = "qubit.platform.iam.UserInfo")
pub struct User {
   ...
}

这样感觉更能表现entity和projection的关系。不过这个就限制了一个entity只能有一个projection。你从第一性原理出发，结合我们在java中对 xxx info 这些projection对象的使用情况，考虑一下真实的业务场景是不是会出现一个entity多个projection的情况。

这个场景的一个可能的未来用途是，我们也许不用再反复为每个 entity 实现 HasInfo, WithSpeciifedInfo<...>  接口。但目前暂时不需要这样，因为这么做还需要把 WithSpeciifedInfo 这个traits 从 rs-platform 中上升到更上游的地方。

先提前确认R009：部分同意，但是否有办法让 Model 支持泛型？感觉会很有用。

R008：部分同意。但是我觉得，computed 干脆不要加depends_on属性了。因为一方面这个暂时用不到，另一方面，为每个 info() 属性加 computed(depends_on(...)) 就没办法用宏来实现 info(), set_info()了。所以干脆 computed 不需要参数，它就表示这个 属性 （getter）是个计算出的属性。

reference 对 property 的解析，不应该依赖 getter 上的 computed 属性。事实上computed属性不仅仅用于这类会被property引用的返回projection的getter，它应该可以用于任意的，通过计算而不是直接从struct的字段值中得到的getter。

具体而言，你参考下
/home/starfish/working/qubit/java-common/common-java/src/main/java/ltd/qubit/commons/reflect/BeanInfo.java

我觉得rust也应该实现类似这样的一个 ModelMetadata 的类，提供一个Model的元信息（可能现在已经有了）
这里面的关键是，要区分 field 和 property
field 应该是这个struct的字段，无论是pub还是private，或其他什么访问范围的，
property应该是字段 + getter，但是要去重。getter的定义我记得前面我们讨论过，原则上类似java的getter，但是不需要写成 getXxxx() 形式，而是按照rust风格直接写 xxxx()
如果一个 getter 有个同名的field，他们合并为一个名字的 property，只是这个property提供了getter。还有些getter不是直接返回field，而是返回其他什么；这样区分这个property是否是computed的就有意义了。

不过想到这里我忽然发现一个问题：似乎 computed 也可以不需要的，我们是否能根据有没有同名的field来确定这个getter是否是计算出来的？这个你需要认真评估下

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

我先把已经明确的部分记录下来：R006 的 ModelId 改成一段或多段 Java fully-qualified-class-name 风格；R007 保留并确认。然后重点核对三件事：Java 中一个 Entity 实际对应多少种 Info/Projection、BeanInfo 如何合并 field/getter/property、Rust 泛型 Model 在静态 descriptor 与链接期注册下能否成立。R008、R009 暂不定稿。

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

【ModelId 改成一段或多段 Java fully-qualified-class-name 风格】 这个不准确，应该是 ModelId 就是一个 java full class name风格的字符串，允许用句点隔开几段，每一段都要符合java的名字空间的段的要求

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

明白。这里不应描述成“由一段或多段组成”这种实现视角，而应直接把公共契约定义为：`ModelId` 是符合项目约定的 Java fully-qualified class name 风格字符串；句点分隔的每个名称段都必须合法。此前确认的 ASCII 段规则继续作为该风格的精确语法。

已修正为精确定义：

```text
ModelId := Segment ("." Segment)*
Segment := [A-Za-z][A-Za-z0-9_]*
```

即 `ModelId` 是 Java fully-qualified class name 风格的字符串：

- 句点分隔的每一段都必须是合法名称段；
- 不允许空段、前导句点或尾随句点；
- 不含句点的单段名称也合法；
- 推荐 namespace 段使用 lower_snake_case、类型段使用 UpperCamelCase；
- 不强制最后一段与 Rust 类型名相同。

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

同意

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

关于 Projection 关系，我已经看到一个关键事实：Java/Rust 现状中的单一 `info()` 更多是接口方法名和 Java 返回类型不能参与重载造成的形态，不是领域上“一种 Entity 永远只有一种 Projection”的保证。接下来我会再做一次定向清点，区分通用 Info、StatefulInfo、具体 UserInfo 等投影族，再给出是否应在 Entity 上放单数 projection 的结论。

评估已经得到三个明确方向：Entity 不应声明单数 projection；`computed` 可以由“是否存在同名 field”可靠推导，显式宏可以删除；泛型 Model 可以支持，但必须把“泛型定义注册”和“具体实例 descriptor”分成两层，不能假装链接期能够枚举无限个单态化类型。我现在把这三项候选方案写入确认记录。

ModelId 的精确定义已确认并记录。R007 也已确认。针对其余问题，我的结论如下。

### R006-P：不在 Entity 上增加单数 `projection`

我不建议增加：

```rust
#[Entity(
    id = "qubit.platform.iam.User",
    projection = UserInfo,
)]
```

也不建议增加对应的 `projection_id`。

原因不是技术限制，而是 Entity 与 Projection 的真实基数不是一对一。

现有 Java/Rust 代码看起来常常只有一个 `info()`，主要受接口设计限制：

- `HasSpecificInfo<T>` 固定使用 `info()`；
- Java 不能仅根据返回类型重载方法；
- 一个类很难同时自然实现多套同名 `getInfo()` 接口。

但实际领域中已经存在多种投影层级：

- `Info`
- `StatefulInfo`
- `InfoWithEntity`
- `UserInfo`
- `PersonInfo`
- `EmployeeInfo`
- `OperationLogInfo`

未来同一个 User 很自然会有：

```rust
pub fn info(&self) -> UserInfo;
pub fn public_profile(&self) -> PublicUserProfile;
pub fn admin_summary(&self) -> AdminUserSummary;
pub fn authentication_info(&self) -> AuthenticationInfo;
```

另外，Java 的 `Product.getInfo(specification)` 说明投影甚至可能依赖额外上下文，不一定只有一个无参数默认映射。

更合理的关系来源是：

- Projection 通过 `source/source_id` 声明固定来源；
- getter-only property 的返回类型登记 producer 关系；
- Registry 反向计算某个 Entity 有哪些 Projection。

也就是：

```text
Projection.source ───────┐
                         ├──> Registry 计算 Entity.projections()
Property getter output ──┘
```

不需要 Entity 再重复列举。

如果未来确实需要自动实现 `HasSpecificInfo<T>`，那时可以设计：

```rust
default_projection = UserInfo
```

它表示“多个 Projection 中的默认 Info Projection”，而不是把 Entity 限制成只能有一个 Projection。目前不预留这个参数。

### R008-A：删除 `#[computed]`

认真评估后，我同意你的判断：`#[computed]` 可以完全删除，包括 `depends_on`。

computed 的真正语义不是“方法内部有没有做计算”，而是：

> 这个 property 有没有对应的真实存储字段。

因此可以可靠推导：

- 有同名 field：stored property，`computed = false`；
- 没有同名 field，只有 getter：computed property，`computed = true`。

例如：

```rust
#[Entity(id = "qubit.platform.iam.User")]
pub struct User {
    pub username: String,
    pub first_name: String,
    pub last_name: String,
}
```

```rust
#[ModelProperties]
impl User {
    // 与 username 字段合并，是 stored property。
    pub fn username(&self) -> &String {
        &self.username
    }

    // 没有 info 字段，是 computed property。
    pub fn info(&self) -> UserInfo {
        // ...
    }

    // 没有 full_name 字段，也是 computed property。
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    // 有参数，不是 getter，不形成 property。
    pub fn set_info(&mut self, info: UserInfo) {
        // ...
    }
}
```

即使同名 getter 对 field 做了转换，它仍然是 field-backed property，因为真实存储槽位存在。如果需要同时暴露转换后的计算结果，应使用另一个 property 名称。

删除 `depends_on` 后：

- getter 宏更容易生成；
- `info()/set_info()` 可以由 declarative macro 生成整个 `#[ModelProperties] impl`；
- 生成器只在完整对象构造完成后调用 getter，不需要调度依赖；
- getter 内部是否调用其他 getter，是普通 Rust 运行逻辑，不建立 metadata 依赖图。

这会推翻此前确认的 F024/C006，最终文档中改成“Property getter 与存储来源”，不再存在 `computed` annotation。

### R008-B：Field 与 Property 必须分开

建议 `ModelMetadata` 同时提供两套集合：

```rust
pub struct ModelMetadata {
    fields: &'static [FieldMetadata],
    properties: PropertyCollection,
}
```

Field 是真实 struct 存储字段：

- 包含 public、private、crate-visible 等所有字段；
- 只由 struct 声明决定；
- 拥有字段类型、约束、关系、Serde、redact 等 metadata；
- computed getter 永远不进入 fields。

Property 是按名称合并后的可读属性：

```rust
pub struct PropertyMetadata {
    name: &'static str,
    field: Option<&'static FieldMetadata>,
    getter: Option<PropertyGetter>,
    // 其他访问 API 后续确认
}
```

合并规则：

| Field | Getter | Property 结果 |
| --- | --- | --- |
| 有 | 无 | field-backed，使用生成的 field accessor |
| 有 | 有 | 合并为一个 property，优先通过显式 getter 读取 |
| 无 | 有 | getter-only，即 computed property |
| 无 | 无 | 不存在 |

`computed` 不需要存成独立声明，可以直接定义为：

```rust
property.field().is_none()
```

或者对外提供：

```rust
property.storage_kind()
```

返回：

```rust
pub enum PropertyStorageKind {
    FieldBacked,
    Computed,
}
```

`reference(property = ...)` 直接查询统一 PropertyMetadata：

```rust
#[reference(
    entity = User,
    property = info,
)]
pub user: UserInfo;
```

它不关心 `info` 是不是 computed，只关心：

- User 是否有名为 `info` 的 property；
- property 是否可读；
- property 返回类型是否与当前字段兼容。

`#[ModelProperties]` 本身就是显式边界。只收集其中符合以下形状的方法：

```rust
pub fn name(&self) -> T
```

要求：

- public；
- `&self`；
- 无额外参数；
- 非 async；
- 非 unsafe；
- 非泛型；
- 返回非 unit。

因此不需要再给每个 getter 写 `#[property]` 或 `#[computed]`。带参数的 `set_info()` 可以和 getter 放在同一 impl 中，但不会被误判为 getter。setter metadata 和 borrowed/owned getter 的 erased ABI 留到 Runtime API 阶段精确确认。

### R009-G：支持泛型 Model

泛型 Model 技术上可以支持，而且 `Page<T>`、`ApiResponse<T>`、`TreeNode<T>` 确实很有价值：

```rust
#[Model]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: u64,
}
```

但必须承认一个事实：链接期不可能枚举 `Page<T>` 的所有潜在单态化类型。因此需要两层 descriptor。

第一层是链接期注册的泛型模板：

```rust
GenericModelTemplate {

    rust_name: "Page",
    type_parameters: ["T"],
    fields: [
        FieldShape::Sequence(TypeParameter(0)),
        FieldShape::Scalar(U64),
    ],
}
```

第二层是具体实例 descriptor：

```rust
metadata_of::<Page<UserInfo>>()
metadata_of::<Page<OrderSummary>>()
```

模板替换参数后产生：

```text
Page<UserInfo>     -> ConcreteTypeDescriptor
Page<OrderSummary> -> ConcreteTypeDescriptor
```

规则建议为：

- 支持类型参数；
- 支持 `where` 子句；
- 暂不支持 lifetime 参数，因为注册和动态访问要求具体值为 `'static`；
- 首版暂不支持 const generic，后续有明确场景再扩展；
- 宏生成的 descriptor impl 自动增加所需的 `T: HasTypeDescriptor + 'static` 约束；
- validation、schema、random generation 都在具体实例 descriptor 上运行；
- generic template 只能执行与具体参数无关的结构校验。

注册语义必须明确：

- `Page<T>` 的泛型定义参与链接期注册；
- `Page<UserInfo>` 的 descriptor 在请求时由模板实例化并缓存；
- 不声称所有潜在 `Page<T>` 都已经在链接期注册。

建议 Registry API 最终区分：

```rust
registry.models()
registry.generic_model_templates()

metadata_of::<Page<UserInfo>>()
template.instantiate(arguments)
```

这可以保留 Model“参与注册”的基本语义，同时诚实处理 Rust 泛型不可能被预先穷举的问题。

请确认：

- R006-P：Entity 不增加单数 projection；
- R008-A：删除 `#[computed]`，根据同名 field 自动推导；
- R008-B：ModelMetadata 分开 fields 和 properties；
- R009-G：通过“泛型模板注册 + 具体 descriptor 实例化”支持泛型 Model。

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

R006-P ：同意
R008-A：同意
R008-B：同意，不过property还要检测setter，确认是否可读、可写；setter约定为 set_xxxx(...)其中xxxx为属性名，换句话说，我们采用类似java bean的约定。注意有可能一个property可写但不可读（只有setter）。对于所有的field，正常应该是可写、可读，但对于注入 name: &str 这样的，可能只能可读（我不太熟悉rust语法，可能说的不准确）
R009-G：同意

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

我会把这四项记为确认，并把 property 模型补成完整的 Java Bean 风格：名称集合是 field、getter、setter 的并集；getter 使用 `xxxx()`，setter 使用 `set_xxxx(value)`；分别计算 readable/writable，允许只写 property。对于 Rust 字段本身，我也会把可写性说明得更准确。

已按修订记录 R006-P、R008-A、R008-B、R009-G。

关于字段可写性补充一点：Rust struct 中的普通字段只要持有 `&mut self`，原则上都可以重新赋值，无论字段是否 public。`name: &str` 通常要求 struct 带 lifetime，而当前角色不支持 lifetime；`&'static str` 字段仍可重新赋另一个静态字符串，并非天然只读。真正的只读/只写主要出现在只有 getter 或 setter 的虚拟 property。

下一批建议确认以下 4 项。

### R008-C Property 的 getter/setter 协议

`#[ModelProperties]` 中自动识别：

```rust
#[ModelProperties]
impl User {
    // getter：property 名为 info
    pub fn info(&self) -> UserInfo {
        // ...
    }

    // setter：同一个 info property
    pub fn set_info(&mut self, value: UserInfo) {
        // ...
    }
}
```

严格形状：

```rust
pub fn xxxx(&self) -> T;
pub fn set_xxxx(&mut self, value: T);
```

两者都必须：

- public；
- 同步；
- safe；
- 非泛型。

Getter：

- 只能有 `&self`；
- 返回值不能是 `()`。

Setter：

- 只能有 `&mut self` 和一个值参数；
- 必须返回 `()`。

Property 合并结果：

| Field | Getter | Setter | 可读 | 可写 | 类型 |
| --- | --- | --- | --- | --- | --- |
| 有 | 无 | 无 | 是 | 是 | field-backed |
| 有 | 有 | 无 | 是 | 是 | field-backed |
| 有 | 无 | 有 | 是 | 是 | field-backed |
| 有 | 有 | 有 | 是 | 是 | field-backed |
| 无 | 有 | 无 | 是 | 否 | computed，只读 |
| 无 | 无 | 有 | 否 | 是 | virtual，只写 |
| 无 | 有 | 有 | 是 | 是 | virtual，可读写 |

显式 getter/setter 优先于生成的 field accessor，以保留方法中的封装和不变量。

Rust 常见借用 getter应允许：

```rust
pub fn name(&self) -> &str;
pub fn items(&self) -> &[Item];
```

因此 getter 与 field/setter 不要求 Rust 类型字面完全相同，而是通过 property type adapter 检查兼容性。首批内建兼容关系至少包括：

- `T` ↔ `&T`
- `String` ↔ `str` / `&str`
- `Vec<T>` ↔ `[T]` / `&[T]`
- `Option<T>` ↔ `Option<&T>`

精确的 borrowed/owned erased accessor API 在 runtime API 阶段确认。

### R009-H 泛型 Model 的参数范围

建议支持：

- 类型参数；
- `const N: usize`；
- `where` 子句。

例如：

```rust
#[Model]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: u64,
}
```

```rust
#[Model]
pub struct FixedPage<T, const N: usize> {
    pub items: [T; N],
}
```

暂不支持 lifetime 参数：

```rust
#[Model]
pub struct Borrowed<'a> {
    pub name: &'a str,
}
```

原因是动态 metadata、全局注册和 erased accessor 都要求具体实例满足 `'static`。

类型参数和 const 参数进入 `GenericModelTemplate`；具体 `Page<UserInfo>` 或 `FixedPage<UserInfo, 20>` 在请求 descriptor 时实例化。

### R010 `#[Enum]` 的形状与参数

支持：

- unit variant；
- tuple variant；
- struct variant；
- 任意混合；
- 类型泛型；
- `const N: usize`；
- `where` 子句。

禁止 lifetime 参数和 union。

`#[Enum]` 没有角色专属参数：

```rust
#[Enum]
pub enum ResultState<T> {
    Pending,
    Success(T),
    Failure {
        message: String,
    },
}
```

泛型 Enum 与泛型 Model 一样注册 template，具体实例按需获得 descriptor。

Variant 规范继续使用已经确认的：

```rust
#[variant(name = "SUCCESS")]
```

### R011 `#[Value]` 的形状与参数

建议支持：

- 具名字段 struct；
- 单字段 tuple newtype；
- 类型泛型；
- `const N: usize`；
- `where` 子句。

禁止：

- unit struct；
- 多字段 tuple struct；
- enum；
- lifetime 参数；
- union。

建议删除旧设计中的 `transparent` 和 `textual` 参数，使 `#[Value]` 没有角色专属参数。

单字段 newtype 天然按内层值表示：

```rust
#[Value]
pub struct EmailAddress(
    #[text(
        min_chars = 3,
        max_chars = 320,
        format = email,
    )]
    pub String,
);
```

其行为：

- descriptor 保留名义上的 `EmailAddress` Value 类型；
- shape 是 `Newtype(String)`；
- 默认 Serde/Display 表示与内层值一致；
- 自动继承内层的 text/numeric/time 等 capability；
- 不自动生成 `From<String>`，避免绕过约束构造非法值。

旧的 `textual` 对具名 Value 只有标记、没有提供如何取得文本的可执行协议，因此删除。具名 Value 需要规范文本能力时，应：

- 声明 canonical ValueCodec；或
- 实现后续定义的明确文本 capability trait。

请确认 R008-C、R009-H、R010、R011。

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

R008-C：同意
R009-H：同意
R010：同意
R011：同意

不过，后面你还要说明，Entity, Projection, Enum, Model, Value分别默认derive了哪些宏（比如Display, Debug, Redact, Serialize, Deserialize, Clone, Default, Copy等）
以及哪些参数可以去掉默认的derive
哪些参数又可以加上默认的derive（比如 Copy 如果不默认derive，是不是可以通过 copy 参数derive）
transparent 和 copy 我觉的还是有不少适用场景的，比如 Id, EmailAddress, Phone这些 Value

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

我先记录 R008-C、R009-H、R010、R011 的确认；同时把 R011 中“删除 transparent”单独重新打开，因为你明确指出了 Id、EmailAddress、Phone 这类 Value 的实际需求。下一批我会统一设计五种角色的默认派生、移除参数、追加参数，以及 `Value(transparent)` 的精确含义，避免 `copy`、`Default`、`Redact` 之间出现互相冲突的零散规则。

我会先把本轮已确认的 R008-C、R009-H、R010、R011 写入确认记录；随后给出 `transparent` 与默认派生能力的 5 项精确定义，供你逐项确认。这里继续按 brainstorming 技能的“先确认语义、后定稿”流程推进。

R008-C、R009-H、R010、R011 已经写入正式确认记录。下面继续确认 `transparent` 与默认派生能力。

### R011-T：`#[Value(transparent)]` 的精确定义

`transparent` 表示“具有独立领域类型身份，但外部值表示与内部唯一字段一致”的透明值包装。

```rust
#[Value(transparent, copy)]
pub struct UserId(Id);

#[Value(transparent)]
pub struct EmailAddress {
    value: String,
}
```

约束：

- 只能用于 `#[Value]`。
- Value 必须恰好有一个存储字段。
- 单字段 tuple struct 和只有一个字段的 named struct 都允许。
- getter、setter 等 property 不计入存储字段数量。
- 多字段 `Phone` 不能使用 `transparent`；单字段包装形式的 `Phone` 可以使用。

具体语义：

- Value 仍然是独立的名义类型，不能与内部类型混为同一个 Rust 类型。
- metadata 保留 Value 自己的类型描述，同时记录：
  - `transparent = true`
  - 唯一内部字段
  - 内部字段的类型描述和约束
- `Serialize`、`Deserialize` 使用内部值的表示，不额外产生对象层或 newtype 层。

例如：

```rust
#[Value(transparent)]
pub struct EmailAddress(String);
```

序列化结果是：

```json
"alice@example.com"
```

而不是：

```json
{"value":"alice@example.com"}
```

也不是带有 `EmailAddress` 标签的结构。

- `Display` 直接委托内部值，因此输出 `alice@example.com`。
- `Debug` 保留名义类型，输出类似 `EmailAddress("alice@example.com")`，便于诊断时区分 `EmailAddress` 和普通 `String`。
- `Redact` 保持透明表示，但仍执行唯一字段上的脱敏规则。
- 字段约束、validator、codec、key_part 等 metadata 不会因为透明表示而消失。
- `transparent` 本身不自动生成：
  - `Deref`
  - `From<Inner>`
  - `Into<Inner>`
  - `TryFrom<Inner>`

原因是表示透明不等于构造过程必然无条件合法。例如 `EmailAddress(String)` 通常需要经过校验，自动生成 `From<String>` 会绕过这个语义。需要这些转换时由类型显式实现。

### R012-A：五种角色默认实现的能力

建议五种角色统一默认实现以下 trait：

| 能力 | Entity | Projection | Model | Enum | Value |
|---|---:|---:|---:|---:|---:|
| `Clone` | 是 | 是 | 是 | 是 | 是 |
| `Debug` | 是 | 是 | 是 | 是 | 是 |
| `Display` | 是 | 是 | 是 | 是 | 是 |
| `PartialEq` | 是 | 是 | 是 | 是 | 是 |
| `Eq` | 是 | 是 | 是 | 是 | 是 |
| `Hash` | 是 | 是 | 是 | 是 | 是 |
| `Redact` | 是 | 是 | 是 | 是 | 是 |
| `Serialize` | 是 | 是 | 是 | 是 | 是 |
| `Deserialize` | 是 | 是 | 是 | 是 | 是 |
| `Copy` | 否 | 否 | 否 | 仅全 unit Enum | 否 |
| `Default` | 否 | 否 | 否 | 否 | 否 |
| `PartialOrd` | 否 | 否 | 否 | 否 | 否 |
| `Ord` | 否 | 否 | 否 | 否 | 否 |

这里有几个关键约定：

1. `PartialEq`、`Eq`、`Hash` 使用标准结构化语义。

   Entity 不会被擅自实现成“只比较 identifier”。如果业务需要 identifier-only equality，应关闭这些默认实现并自行实现。

2. `Default` 不默认提供。

   自动产生的默认值可能违反 `text`、`sequence`、`validator` 等约束，因此不能把“能够构造一个 Rust 值”误认为“能够构造一个合法业务值”。

3. `Ord` 不默认提供。

   字段顺序或 Enum variant 声明顺序不一定代表领域顺序，框架不应擅自创造这种语义。

4. 只有全部为 unit variant 的 Enum 默认 `Copy`。

   这种 Enum 是纯符号值，复制成本和语义都明确。带数据的 Enum 不默认 `Copy`。

5. 泛型类型的自动实现带有相应的条件约束。

   例如 `Model<T>` 的 `Clone` 实现只在 `T: Clone` 时可用，而不是要求所有可能的 `T` 都实现 `Clone`。

### R012-B：移除默认能力的参数

五种角色宏统一支持下列关闭参数：

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
```

另外，全 unit Enum 支持：

```rust
#[Enum(no_copy)]
```

依赖规则如下：

- `no_clone`：
  - 移除 `Clone`。
  - 同时不能启用 `Copy`，因为 Rust 的 `Copy` 要求 `Clone`。

- `no_partial_eq`：
  - 同时移除 `Eq`、`Hash`、`PartialOrd`、`Ord`。
  - 因为这些能力都依赖或约定依赖相等性。

- `no_eq`：
  - 保留 `PartialEq`。
  - 同时移除默认 `Hash`。
  - 禁止启用 `Ord`。
  - 这是为了维持 Rust 中 `Eq`/`Hash` 和全序关系的语义契约。

- `no_hash`：
  - 只移除 `Hash`。

- `no_debug`、`no_display`：
  - 只移除对应的格式化接口。
  - 不移除 `Redact`，仍然可以通过 `Redactor` 显式输出脱敏结果。

- `no_serialize` 和 `no_deserialize` 相互独立：
  - 允许只序列化。
  - 允许只反序列化。

- `no_redact`：
  - 仅当该类型及其 `element`、`map_key`、`map_value` selector 中完全没有 `redact(...)` 规则时允许。
  - 如果存在任何脱敏标注，产生编译错误。
  - 关闭后，默认的 `Debug`、`Display`、`Serialize` 使用普通非脱敏实现。

不提供 `no_default`、`no_ord` 等参数，因为这些能力本来就不默认启用。

### R012-C：增加非默认能力的参数

统一提供以下 opt-in 参数：

```rust
#[Value(transparent, copy, default, ord)]
pub struct Revision(u64);
```

支持：

- `copy`
- `default`
- `partial_ord`
- `ord`

具体规则：

#### `copy`

为类型增加 `Copy`。

- Entity、Projection、Model、带数据 Enum 和 Value 都可以显式请求。
- 所有存储字段必须实现 `Copy`，否则由编译器报告类型能力不满足。
- 同时要求 `Clone` 没有被关闭。
- `#[Value(transparent, copy)]` 是 `Id`、数值代码、固定大小值对象等类型的典型写法。
- `String`、`Vec` 等拥有堆资源的字段不能 `Copy`，因此以 `String` 为内部值的 `EmailAddress` 不能启用 `copy`。

#### `default`

增加结构化 `Default` 实现。

- struct 要求所有存储字段实现 `Default`。
- Enum 必须有且只有一个标有标准 `#[default]` 的 unit variant。
- `default` 只表示能够产生 Rust 默认值，不保证该值通过模型约束和 validator。
- 如果默认值必须满足业务约束，建议手工实现 `Default`。

#### `partial_ord`

增加结构化 `PartialOrd`。

- 要求保留 `PartialEq`。
- 所有参与比较的字段必须实现 `PartialOrd`。

#### `ord`

增加结构化 `Ord`，并同时启用其依赖：

- `PartialEq`
- `Eq`
- `PartialOrd`
- `Ord`

如果同时声明 `ord` 和 `no_eq`、`no_partial_eq`，产生编译错误。

对于用户已经显式写在 `#[derive(...)]` 中的相同 trait，角色宏把它视为已经启用，不重复产生实现。

### R014：默认派生能力与脱敏的联动

五种角色默认实现 `Redact`，因此字段上的 `#[redact(...)]` 不再要求类型宏额外添加 `redact` 参数。

```rust
#[Value(transparent)]
pub struct EmailAddress(
    #[redact(level = "medium")]
    String,
);

#[Entity(id = "qubit.platform.iam.User")]
pub struct User {
    #[identifier]
    pub id: Id,

    #[redact(nested)]
    pub email: EmailAddress,
}
```

默认输出规则：

- `Redact`：始终提供显式脱敏输出能力。
- `Debug`：默认执行字段声明的脱敏策略。
- `Display`：默认执行字段声明的脱敏策略。
- `Serialize`：默认执行字段声明的结构化脱敏策略。
- `Deserialize`：只负责输入，不应用脱敏策略。
- 没有 `#[redact(...)]` 的字段保持正常输出。
- `nested`、`level`、`map`、`keyed_by`、`json` 以及容器 selector 的递归规则，沿用已经确认的 F021～F023。
- `no_debug`、`no_display`、`no_serialize` 只关闭对应的对外接口，不关闭 `Redact`。
- `no_redact` 只有在类型完全没有脱敏规则时才允许，此时 `Debug`、`Display`、`Serialize` 恢复普通实现。
- 删除旧的类型级 `redact` 参数，因为 `Redact` 已经成为五种角色的默认能力。

请确认：

- R011-T：透明 Value
- R012-A：默认能力矩阵
- R012-B：移除默认能力
- R012-C：增加非默认能力
- R014：脱敏联动规则

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

R011-T： 同意
R012-A： 同意
R012-B：同意
R012-C：同意
R014： 同意

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

这 5 项已全部确认。我会按 brainstorming 确认流程立即写入正式记录，然后进入下一组：五种角色的注册、类型身份与 metadata 获取入口。

已写入并验证正式确认记录：

[2026-08-27-rs-model-derive-semantics-confirmation.md](/home/starfish/working/qubit/rust-platform/rs-model-derive/doc/2026-08-27-rs-model-derive-semantics-confirmation.md)

继续确认注册与类型身份设计。

### R015-A：`ModelId` 的唯一职责

`ModelId` 只属于 Entity，用于在 Rust 类型系统之外稳定标识一种具有持久化身份的实体。

```rust
#[Entity(id = "qubit.platform.iam.User")]
pub struct User {
    #[identifier]
    pub id: Id,
}
```

规则：

- 只有 Entity 拥有 `ModelId`。
- `#[Entity(id = "...")]` 的 `id` 必填。
- Projection、Model、Enum、Value 禁止声明 `id`。
- `ModelId` 在整个最终注册表中必须唯一。
- `ModelId` 是跨进程、跨服务、跨语言和持久化 metadata 中使用的稳定标识。
- Rust 类型改名、移动模块时，Entity 可以继续保持原 `ModelId`。
- `reference(entity_id = "...")` 和 `Projection(source_id = "...")` 中的字符串必须引用 Entity 的 `ModelId`。
- Projection 的 identifier 是来源 Entity 的实例 ID，但 Projection 本身没有 `ModelId`。

因此：

- `Entity.id` 宏参数标识“实体类型”。
- `#[identifier]` 字段标识“实体实例”。

二者不是同一个概念。

### R015-B：五种角色的注册规则

| 角色 | 自动参与注册 | 注册时拥有 `ModelId` | 可通过具体 Rust 类型获取 metadata |
|---|---:|---:|---:|
| Entity | 是 | 是 | 是 |
| Projection | 是 | 否 | 是 |
| Model | 是 | 否 | 是 |
| Enum | 是 | 否 | 是 |
| Value | 否 | 否 | 是 |

Entity、Projection、Model、Enum 由宏产生链接期自动注册项，不需要用户维护中央类型列表。

注册表提供两类查询：

1. 按 Rust 具体类型查询

```rust
metadata_of::<User>()
metadata_of::<UserInfo>()
metadata_of::<Gender>()
```

五种角色都支持，包括不参与注册的 Value。

2. 按 `ModelId` 查询

```rust
registry.entity(ModelId::new("qubit.platform.iam.User")?)
```

只可能返回 EntityMetadata。

Projection、Model、Enum 不增加另一套人为字符串 ID。它们在当前程序中按 Rust 的具体类型身份识别，并可以从注册表中枚举。

这里的 Rust 类型身份只保证在当前编译产物和进程中唯一，不承诺跨版本、跨语言或跨进程稳定。需要这种稳定性的类型必须是 Entity，并使用 `ModelId`。

### R015-C：Value 不注册但仍拥有完整 metadata

Value 不进入全局注册表，并不意味着它没有 metadata。

每个 Value 都实现统一的静态类型描述接口，因此可以直接获取：

```rust
let metadata = metadata_of::<EmailAddress>();
```

当 Value 被其他类型使用时，外层 descriptor 可以递归访问它：

```rust
#[Value(transparent)]
pub struct EmailAddress(
    #[text(format = email)]
    String,
);

#[Entity(id = "qubit.platform.iam.User")]
pub struct User {
    #[identifier]
    pub id: Id,

    #[redact(nested)]
    pub email: EmailAddress,
}
```

`User.email` 的字段 metadata 包含：

- 字段类型是 `EmailAddress`；
- `EmailAddress` 是 Value；
- 它是 transparent Value；
- 内部类型是 `String`；
- 内部存在 `text(format = email)` 约束；
- 它具有相应的 validator、codec 和 redact metadata。

自动校验、schema 生成和随机值生成从 `User` descriptor 递归进入 `EmailAddress` descriptor，不需要先从注册表查找 Value。

Value 不注册带来的含义仅仅是：

- 不能通过注册表枚举“系统中所有 Value”；
- 不能通过字符串 ID 查找 Value；
- 不产生全局注册项；
- 不承担跨模块实体关系的目标身份。

### R015-D：泛型类型的注册规则

对于泛型 Model 和 Enum，注册的是泛型模板，而不是所有可能的具体实例。

```rust
#[Model]
pub struct Page<T, const N: usize> {
    pub items: Vec<T>,
}
```

链接期注册表包含一个 `Page<T, N>` 模板描述，但不会提前注册：

```rust
Page<UserInfo, 20>
Page<OrderSummary, 50>
Page<String, 100>
```

具体实例通过 Rust 类型按需获得并缓存：

```rust
metadata_of::<Page<UserInfo, 20>>()
```

规则：

- Registry 可以枚举已注册的泛型模板。
- 泛型模板只有当前程序内部的 opaque template identity 和可读 Rust 路径，没有 `ModelId`。
- 具体实例以当前进程内的 Rust `TypeId` 作为缓存键。
- 具体实例不是新的链接期注册项。
- 调用 `metadata_of::<ConcreteType>()` 后，可以缓存该 concrete descriptor，但这不改变其注册语义。
- 泛型 Value 同样按具体类型生成 descriptor，但模板和值实例都不进入全局注册表。
- Entity 和 Projection 当前禁止泛型，因此不存在泛型 Entity/Projection 注册问题。

### R015-E：五种类型宏的最终参数集合

在目前已经确认的设计下，类型宏只支持以下参数。

#### `#[Entity]`

```rust
#[Entity(
    id = "qubit.platform.iam.User",
    copy,
    default,
    partial_ord,
    ord,
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
```

其中只有 `id` 必填，其余都是能力控制参数。

禁止 `projection`、`projection_id`、`index`、`unique`、`source`、`transparent` 等参数。

#### `#[Projection]`

```rust
#[Projection(
    source = User,
    // 或 source_id = "qubit.platform.iam.User",
    copy,
    default,
    partial_ord,
    ord,
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
```

- `source` 和 `source_id` 最多提供一个。
- 二者均可省略，表示开放 Projection。
- 不允许 `id`。

#### `#[Model]`

```rust
#[Model(
    copy,
    default,
    partial_ord,
    ord,
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
```

Model 没有其他角色专属参数。

#### `#[Enum]`

除通用能力参数外，额外支持 `no_copy`：

```rust
#[Enum(no_copy, default)]
pub enum Status {
    #[default]
    Pending,
    Active,
}
```

Enum 没有 `id`、`source`、`transparent` 等角色参数。

#### `#[Value]`

除通用能力参数外，只额外支持 `transparent`：

```rust
#[Value(transparent, copy, default)]
pub struct Revision(u64);
```

Value 没有 `id`、`source` 等注册或关系参数。

标准 Rust 属性不属于这些类型宏的参数，例如：

```rust
#[must_use]
#[repr(u8)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
```

它们仍按各自标准语义使用；角色宏不得静默覆盖用户显式提供的兼容配置。

请确认：

- R015-A：ModelId 唯一职责
- R015-B：五种角色的注册规则
- R015-C：Value 的递归 metadata
- R015-D：泛型模板注册
- R015-E：五种类型宏的最终参数集合

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

R015-A：确认
R015-B：部分同意
问题在于，对于没有id的其他几类Projection, Model, Enum，放入注册表的意义何在？不能通过他们的id找到他们（因为他们没有id），只能从枚举列表中看到他们，这个实用价值不大啊。
要么允许他们有id，这样可以通过id在注册表中找到他们。要么他们干脆也不加入注册表。
另外，如果Projection, Model, Enum都可以有id且在注册表中可以通过id找到，为何Value不能有？
你从第一性原理出发，评估下我的意见，允许反驳

R015-C: 同意
R015-D：同意
R015-E：除了R015-B提到的有没有id的问题待确认，其他都同意。

R015-E 加一条意见：Value要能支持copy

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

补充意见收到：`Value` 明确支持 `copy`，例如 `#[Value(transparent, copy)]`；这与已确认的 R012-C 一致，我会在 R015-E 的最终参数表中再次明确，而不是只依赖通用能力说明。现在一并记录本轮状态。

你的质疑成立。此前的设计把“能够自动收集类型”误当成了“值得加入注册表”，这两个概念并不等价。

注册表的核心价值应当是：调用方只持有一个稳定 ID、并不知道对应 Rust 类型时，仍能动态找到 metadata。仅仅为了枚举类型而注册，确实缺乏足够的实际价值。

虽然无 ID 注册还能用于全局图校验、反向查找 Projection 等，但这些收益不足以支撑这种不对称设计：

- 使用 Rust 类型声明的关系可以直接通过类型 metadata 校验。
- `entity_id`、`source_id` 只会动态引用 Entity。
- 普通嵌套类型可以递归获得 descriptor。
- “列出程序中的全部 DTO 和 Enum”并不是一个足够稳定、明确的业务能力。

因此我建议修订此前的结论。

### R015-B2：注册表的第一性原理

一个类型只有在需要“脱离 Rust 静态类型进行动态发现”时，才应该进入全局注册表。

动态发现包括：

- 根据稳定 ID 查找 metadata；
- 从配置、协议、schema 或外部请求中的类型 ID 解析类型；
- 动态选择 validator、codec、schema、对象生成器等处理能力；
- 建立跨 crate、跨进程或跨语言的类型契约；
- 枚举所有被明确声明为公共契约的类型。

不包括：

- 已经知道具体 Rust 类型时调用 `metadata_of::<T>()`；
- 从外层字段递归进入成员类型；
- 仅仅为了证明某个类型“存在”；
- 把所有内部 DTO、辅助 Enum 和 Value 自动暴露成公共类型目录。

因此：

> 注册必须以稳定 ID 为前提；没有稳定 ID 的类型不进入注册表。

不再存在“没有 ID、但仍然自动注册”的类型。

### R015-F：所有角色都可以拥有 `ModelId`

建议把 `ModelId` 的定义修订为：

> `ModelId` 是模型系统中某个可动态发现的类型或泛型模板的稳定标识。

它不再被定义为 Entity 专属，但 Entity 必须拥有它。

最终规则：

| 角色 | `id` | 是否注册 |
|---|---|---|
| Entity | 必填 | 始终注册 |
| Projection | 可选 | 提供 `id` 时注册 |
| Model | 可选 | 提供 `id` 时注册 |
| Enum | 可选 | 提供 `id` 时注册 |
| Value | 可选 | 提供 `id` 时注册 |

例如：

```rust
#[Entity(id = "qubit.platform.iam.User")]
pub struct User {
    #[identifier]
    pub id: Id,
}
```

```rust
#[Projection(
    id = "qubit.platform.iam.UserInfo",
    source = User,
)]
pub struct UserInfo {
    #[identifier]
    pub id: Id,
}
```

```rust
#[Model(id = "qubit.platform.iam.LoginRequest")]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}
```

```rust
#[Enum(id = "qubit.platform.iam.Gender")]
pub enum Gender {
    Male,
    Female,
    Unknown,
}
```

```rust
#[Value(
    id = "qubit.commons.value.EmailAddress",
    transparent,
)]
pub struct EmailAddress(
    #[text(format = email)]
    String,
);
```

不需要动态发现时可以省略：

```rust
#[Model]
pub struct InternalQueryContext {
    pub trace_enabled: bool,
}

#[Enum]
pub enum LocalState {
    Pending,
    Complete,
}

#[Value(transparent, copy)]
pub struct Revision(u64);
```

所有角色的 `ModelId` 使用已经确认的相同格式和全局命名空间，并在注册表中全局唯一。不能出现 Entity 和 Enum 使用相同 ID 的情况。

这项修订会替换刚确认的 R015-A 中“只有 Entity 拥有 ModelId”这一条，但保留以下内容：

- Entity 的 `id` 必填；
- `ModelId` 与实例字段 `#[identifier]` 完全不同；
- `reference(entity_id = "...")` 只能指向 Entity；
- `Projection(source_id = "...")` 只能指向 Entity。

### R015-G：注册不会改变角色语义

给 Projection、Model、Enum 或 Value 增加 `id`，只会增加动态发现能力，不会改变其角色。

例如，注册后的 Value 仍然：

- 没有 identifier；
- 不能作为 Entity reference 的目标；
- 没有独立持久化生命周期；
- 不能包含 Entity、Projection、Model 或 relation；
- 按内容表达领域值。

注册后的 Enum 仍然：

- 没有 identifier；
- 不能作为 Entity reference 的目标；
- 没有持久化实体身份。

注册后的 Model 仍然可以包含 Entity reference，但不会因此成为 Entity。

注册后的 Projection 仍然借用来源 Entity 的实例 ID，不会获得独立实体身份。

因此需要区分两个维度：

```text
角色：这个类型是什么、允许表达什么
注册：能否通过稳定 ModelId 动态找到它
```

这两个维度正交。

### R015-H：泛型类型的 ID 标识模板

对于泛型 Model、Enum 或 Value，`id` 标识泛型模板，不标识某个具体单态实例。

```rust
#[Model(id = "qubit.commons.model.Page")]
pub struct Page<T, const N: usize> {
    pub items: Vec<T>,
}
```

注册表通过 ID 返回 `Page<T, N>` 的模板 metadata：

```rust
registry.template("qubit.commons.model.Page")
```

具体类型仍通过 Rust 类型实例化：

```rust
metadata_of::<Page<UserInfo, 20>>()
```

首版不自动拼接具体实例的稳定 ID。例如不会自动生成：

```text
qubit.commons.model.Page<qubit.platform.iam.UserInfo,20>
```

原因是这会立即引入一套新的跨语言泛型类型表达语法，而且类型参数可能根本没有 `ModelId`。

所以：

- 泛型类型没有 `id`：不注册模板。
- 泛型类型有 `id`：注册模板。
- 具体实例 descriptor 按 Rust 类型生成和缓存。
- 具体实例不自动加入全局注册表。
- 将来若确实需要通过字符串表达具体泛型实例，应单独设计规范化的 `TypeExpression`，不能临时拼字符串。

### R015-E2：类型宏参数相应修订

#### Entity

`id` 必填：

```rust
#[Entity(id = "qubit.platform.iam.User")]
```

#### Projection

`id` 可选，`source`/`source_id` 可选且二选一：

```rust
#[Projection(
    id = "qubit.platform.iam.UserInfo",
    source = User,
)]
```

#### Model

`id` 可选：

```rust
#[Model(id = "qubit.platform.iam.LoginRequest")]
```

#### Enum

`id` 可选：

```rust
#[Enum(id = "qubit.platform.iam.Gender")]
```

#### Value

`id` 可选，并明确支持 `transparent` 和 `copy`：

```rust
#[Value(
    id = "qubit.commons.value.Revision",
    transparent,
    copy,
)]
pub struct Revision(u64);
```

Value 的 `copy` 与是否注册无关：

```rust
// 不注册，但支持 Copy。
#[Value(transparent, copy)]
pub struct Revision(u64);

// 注册，并支持 Copy。
#[Value(
    id = "qubit.commons.value.Revision",
    transparent,
    copy,
)]
pub struct Revision(u64);
```

其余已经确认的能力增删参数保持不变。

请确认：

- R015-B2：只有稳定 ID 的类型才进入注册表
- R015-F：Entity 强制 ID，其他四种角色可选 ID
- R015-G：注册与角色语义正交
- R015-H：泛型 ID 标识模板
- R015-E2：修订后的类型宏参数集合

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

R015-B2: 同意
R015-F：同意
R015-G：同意
R015-H：同意
R015-E2：同意

最后一个建议， metadata_of::<Page<UserInfo>>() 的用法不太符合我的审美观，是否可以改为某个类的静态泛型函数？当然这不是强制要求

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

这 5 项已全部确认。我会先更新正式记录，并明确标注 R015-A、R015-C 中被新结论替换的部分，避免文档内部自相矛盾。然后给出 metadata 静态入口的建议。

你这个审美判断我赞同。自由函数：

```rust
metadata_of::<Page<UserInfo>>()
```

虽然符合 Rust 常见写法，但 metadata 本身已经是一个明确的领域对象，把查询入口放在 metadata 类型上更集中。

建议改为：

### A001-S：静态 metadata 获取入口

```rust
let metadata = TypeMetadata::of::<Page<UserInfo>>();
```

精确定义：

```rust
impl TypeMetadata {
    pub fn of<T>() -> &'static TypeMetadata
    where
        T: HasTypeMetadata + 'static;
}
```

适用于全部五种角色，无论是否注册：

```rust
TypeMetadata::of::<User>();
TypeMetadata::of::<UserInfo>();
TypeMetadata::of::<LoginRequest>();
TypeMetadata::of::<Gender>();
TypeMetadata::of::<EmailAddress>();
TypeMetadata::of::<Page<UserInfo>>();
```

我建议使用 `TypeMetadata`，而不是 `ModelMetadata`，原因是 `ModelMetadata` 容易被理解成只对应 `#[Model]` 角色，而这个入口实际覆盖 Entity、Projection、Model、Enum、Value。

同时建议：

- 删除自由函数 `metadata_of::<T>()`，不保留两套同义 API。
- 保留底层公开 trait `HasTypeMetadata`，用于泛型约束和用户自定义实现。
- 不给每个类型生成 `User::metadata()` 这样的固有方法，避免与用户自己定义的方法发生命名冲突。
- `TypeMetadata::of::<T>()` 是静态类型查询，类型不满足 `HasTypeMetadata` 时产生编译错误，不返回 `Option`。
- 动态 ID 查询仍由注册表负责：

```rust
ModelRegistry::global().get(model_id)
```

两种入口职责清晰：

```rust
// 已知 Rust 类型
TypeMetadata::of::<Page<UserInfo>>();

// 只知道稳定 ModelId
ModelRegistry::global().get(model_id);
```

如果你同意，我会把它记为 A001-S 已确认，并把确认记录中已有的 `metadata_of::<T>()` 全部替换为 `TypeMetadata::of::<T>()`。

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

A001-S：同意

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

A001-S 已确认。我现在把它写入正式记录，并将确认文档中已有的 `metadata_of::<T>()` 表述统一改为 `TypeMetadata::of::<T>()`。

记录已更新且文档中已无 `metadata_of`。下一组我会确认 `TypeMetadata` 本身的公共结构：类型身份、角色判断、字段与属性入口，以及角色专属 metadata 的访问方式。

A001-S 已写入正式确认记录，文档内已有的 `metadata_of::<T>()` 也已全部替换为 `TypeMetadata::of::<T>()`。

继续确认 metadata 公共模型。

### A001-T：`TypeMetadata` 与 `TypeDescriptor` 的边界

需要两个不同层次的类型描述。

#### `TypeMetadata`

只描述由五种角色宏声明的领域类型：

```rust
TypeMetadata::of::<User>();
TypeMetadata::of::<UserInfo>();
TypeMetadata::of::<LoginRequest>();
TypeMetadata::of::<Gender>();
TypeMetadata::of::<EmailAddress>();
```

公开接口：

```rust
impl TypeMetadata {

    pub fn of<T>() -> &'static TypeMetadata
    where
        T: HasTypeMetadata + 'static;
}
```

#### `TypeDescriptor`

描述任意可被模型系统理解的 Rust 类型，包括：

- 标量：`String`、`bool`、整数、浮点数等；
- 时间、Decimal、Id 等基础类型；
- `Option<T>`；
- `Vec<T>`、集合、数组；
- Map；
- tuple；
- `Box<T>`、`Rc<T>`、`Arc<T>`；
- Entity、Projection、Model、Enum、Value；
- opaque 类型；
- 泛型参数和具体泛型实例。

例如：

```rust
TypeDescriptor::of::<String>();
TypeDescriptor::of::<Option<EmailAddress>>();
TypeDescriptor::of::<Vec<UserInfo>>();
TypeDescriptor::of::<Page<UserInfo>>();
```

公开接口：

```rust
impl TypeDescriptor {
    pub fn of<T>() -> &'static TypeDescriptor
    where
        T: HasTypeDescriptor + 'static;

    pub fn metadata(&self) -> Option<&'static TypeMetadata>;
}
```

因此：

- `TypeMetadata` 回答“这个领域类型是什么”。
- `TypeDescriptor` 回答“这个 Rust 类型的完整结构是什么”。
- 字段和 property 的类型统一返回 `TypeDescriptor`。
- 对普通 `String` 调用 `TypeMetadata::of::<String>()` 是编译错误。
- 对普通 `String` 调用 `TypeDescriptor::of::<String>()` 合法。

### A001-I：类型身份、注册 ID 与泛型来源

`TypeMetadata` 暴露以下身份信息：

```rust
impl TypeMetadata {
    pub fn rust_identity(&self) -> RustTypeIdentity;
    pub fn rust_name(&self) -> &'static str;

    pub fn model_id(&self) -> Option<ModelId>;
    pub fn template(&self) -> Option<&'static GenericTypeMetadata>;

    pub fn is_registered(&self) -> bool;
}
```

语义：

#### `rust_identity()`

表示当前编译产物中的具体 Rust 类型身份，内部包含：

- Rust `TypeId`；
- 完整 Rust 类型名称。

它只在当前程序内有效，不具有跨进程稳定性。

#### `rust_name()`

返回便于诊断的 Rust 类型名称，例如：

```text
qubit_platform_iam::model::User
qubit_commons::Page<qubit_platform_iam::model::UserInfo>
```

它不是稳定协议 ID，不能代替 `ModelId`。

#### `model_id()`

只在当前具体类型本身是注册项时返回 `Some`：

```rust
TypeMetadata::of::<User>().model_id();
// Some("qubit.platform.iam.User")
```

未注册的非泛型类型返回 `None`。

具体泛型实例不是注册项，因此：

```rust
TypeMetadata::of::<Page<UserInfo>>().model_id();
// None
```

#### `template()`

具体泛型实例返回产生它的模板：

```rust

TypeMetadata::of::<Page<UserInfo>>()
    .template()
    .unwrap()
    .model_id();
// 可能是 Some("qubit.commons.model.Page")
```

#### `is_registered()`

只表示当前具体 `TypeMetadata` 是否直接存在于注册表。

因此，来自已注册模板的具体实例仍然返回 `false`。

### A001-R：角色判断与角色专属 metadata

定义：

```rust
pub enum ModelRole {
    Entity,
    Projection,
    Model,
    Enum,
    Value,
}
```

`TypeMetadata` 提供：

```rust
impl TypeMetadata {
    pub fn role(&self) -> ModelRole;

    pub fn as_entity(&self) -> Option<&EntityMetadata>;
    pub fn as_projection(&self) -> Option<&ProjectionMetadata>;
    pub fn as_model(&self) -> Option<&ModelMetadata>;
    pub fn as_enum(&self) -> Option<&EnumMetadata>;
    pub fn as_value(&self) -> Option<&ValueMetadata>;
}
```

并提供统一的角色数据枚举：

```rust
pub enum RoleMetadata {
    Entity(EntityMetadata),
    Projection(ProjectionMetadata),
    Model(ModelMetadata),
    Enum(EnumMetadata),
    Value(ValueMetadata),

}

impl TypeMetadata {
    pub fn role_metadata(&self) -> &RoleMetadata;
}
```

约定：

- `role()` 适合分支判断。
- `role_metadata()` 适合模式匹配。
- `as_entity()` 等适合调用方已预期某种角色时使用。
- 角色不匹配返回 `None`，不 panic。
- 不提供 `unwrap_entity()` 等 panic 型便利接口。
- 这里的 `ModelMetadata` 专指 `#[Model]` 的角色专属 metadata；总入口仍叫 `TypeMetadata`，从而避免歧义。

### A002-F：存储字段 metadata

所有真实存储字段，无论可见性，都会出现在 `FieldMetadata` 中。

```rust
impl TypeMetadata {
    pub fn fields(&self) -> &[FieldMetadata];
    pub fn field(&self, name: &str) -> Option<&FieldMetadata>;
    pub fn field_at(&self, index: usize) -> Option<&FieldMetadata>;
}
```

规则：

- Entity、Projection、具名 Model、具名 Value 返回 struct 的存储字段。
- unit Model 返回空切片。
- 单字段 tuple Value 返回一个 index 为 `0`、name 为 `None` 的字段。
- Enum 顶层 `fields()` 返回空切片；variant payload 字段从 `EnumVariantMetadata` 获取。
- `field(name)` 只查找具名字段。
- `field_at(index)` 按 Rust 声明顺序查找。

`FieldMetadata` 至少提供：

```rust
impl FieldMetadata {
    pub fn index(&self) -> usize;
    pub fn name(&self) -> Option<&'static str>;
    pub fn ty(&self) -> &'static TypeDescriptor;
    pub fn visibility(&self) -> FieldVisibility;
    pub fn attributes(&self) -> &[FieldAttributeMetadata];

    pub fn is_identifier(&self) -> bool;
    pub fn is_indexed(&self) -> bool;
    pub fn unique(&self) -> Option<&UniqueMetadata>;
    pub fn reference(&self) -> Option<&ReferenceMetadata>;

    pub fn constraints(&self) -> &[ConstraintMetadata];
    pub fn validators(&self) -> &[ValidatorMetadata];
    pub fn codec(&self) -> Option<&CodecMetadata>;
    pub fn redact(&self) -> Option<&RedactMetadata>;
}
```

`is_indexed()` 返回最终语义，而不是只判断是否显式标注：

- `#[indexed]` 为 `true`；
- identifier 为 `true`；
- unique 为 `true`；
- reference 为 `true`。

如果调用方需要区分“显式 indexed”与“由其他规则隐含”，另提供：

```rust
pub fn indexing_source(&self) -> Option<IndexingSource>;
```

### A002-P：Property metadata

`TypeMetadata` 同时提供独立的 property 视图：

```rust
impl TypeMetadata {
    pub fn properties(&self) -> &[PropertyMetadata];
    pub fn property(&self, name: &str) -> Option<&PropertyMetadata>;
}
```

`PropertyMetadata` 至少提供：

```rust
impl PropertyMetadata {
    pub fn name(&self) -> &'static str;
    pub fn ty(&self) -> &'static TypeDescriptor;

    pub fn field(&self) -> Option<&FieldMetadata>;
    pub fn getter(&self) -> Option<&GetterMetadata>;
    pub fn setter(&self) -> Option<&SetterMetadata>;

    pub fn is_readable(&self) -> bool;
    pub fn is_writable(&self) -> bool;
    pub fn is_computed(&self) -> bool;
    pub fn storage_kind(&self) -> PropertyStorageKind;
}
```

其中：

```rust
pub enum PropertyStorageKind {
    FieldBacked,
    Computed,
    Virtual,
}
```

精确定义：

- `FieldBacked`：存在同名 field，可以有或没有显式 getter/setter。
- `Computed`：没有同名 field，但存在 getter。
- `Virtual`：没有同名 field、没有 getter，但存在 setter。
- `is_computed()` 等价于 `storage_kind() == Computed`。
- field 或 getter 使 property 可读。
- field 或 setter 使 property 可写。
- 显式 getter/setter 优先于宏生成的 field accessor。
- tuple Value 的无名存储字段不自动产生 property。
- Enum variant payload 字段不进入类型级 property 集合。

请确认：

- A001-T：TypeMetadata 与 TypeDescriptor 的边界
- A001-I：Rust 身份、ModelId 与泛型模板来源
- A001-R：角色判断和角色专属 metadata
- A002-F：存储字段 metadata
- A002-P：Property metadata

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

A001-T： 同意
A001-I：部分同意
TypeMetadata::rust_identity 是否可以简化为 rust_type_id，类型 RustTypeIdentity 是否可以简化为 RustTypeId
TypeMetadata::rust_name 是否可以改名为 rust_type_name
TypeMetadata::template 这个名称不好，看不出来是什么；这个名称你可以借鉴下java的Class类里面类似的东西。
另外 RustTypeIdentity 或 RustTypeId , 到底是什么类型? rust标准库有 TypeId 类型吗

A001-R：同意
A002-F： 大部分同意
FieldMetadata.ty()  这个名字不好，改为 FieldMetadata.descriptor()
FieldMetadata增加便利的 is_unique(), is_reference(),

A002-P: 部分同意
PropertyMetadata.ty() 改名为 PropertyMetadata.descriptor()
PropertyMetadata 提供便利的 is_field(), is_getter(), is_setter()

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

我先记录本轮确认与改名意见。关于 Rust 类型身份，我会核对标准库语义后给出简化方案；如果标准 `TypeId` 已完整满足需求，就不再额外包装一个 `RustTypeId`。

你的命名修改更准确。标准库已经提供 `TypeId`，没有必要再包装一个几乎相同的 `RustTypeId`。

### A001-I2：Rust 类型身份 API 修订

Rust 标准库提供：

```rust
std::any::TypeId
```

典型用法：

```rust
let type_id = std::any::TypeId::of::<User>();
```

它具有以下性质：

- 对当前程序中的 `'static` Rust 类型提供唯一

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

A001-I:
TypeMetadata::rust_identity 是否干脆改名为 TypeMetadata::type_id
TypeMetadata::rust_name 是否干脆改名为 TypeMetadata::type_name
RustTypeIdentifier 改名为 TypeId

rust标准库 有的话，TypeMetadata.type_id() 直接返回标准库TypeId

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

可以，直接采用标准库类型最简洁：`TypeMetadata::type_id() -> std::any::TypeId`，`TypeMetadata::type_name() -> &'static str`，不再定义 `RustTypeIdentity`、`RustTypeIdentifier` 或自有 `TypeId`。我会把这一决定写入确认记录。

同意。Rust 标准库已经提供 `std::any::TypeId`，无需再包装。

最终接口调整为：

```rust
use std::any::TypeId;

impl TypeMetadata {
    pub fn type_id(&self) -> TypeId;
    pub fn type_name(&self) -> &'static str;

    pub fn model_id(&self) -> Option<ModelId>;
    pub fn generic_definition(&self)
        -> Option<&'static GenericTypeMetadata>;

    pub fn is_registered(&self) -> bool;
}
```

其中：

- `type_id()` 直接返回 `std::any::TypeId`。
- `type_name()` 返回 `std::any::type_name::<T>()` 对应的完整 Rust 类型名。
- 不再定义：
  - `RustTypeIdentity`
  - `RustTypeIdentifier`
  - `RustTypeId`
  - rs-model-metadata 自己的 `TypeId`

需要注意：

- `std::any::TypeId` 是当前编译产物内具体 `'static` Rust 类型的唯一、不透明标识。
- 它可以比较、Hash，但不能序列化成稳定的跨进程类型标识。
- `type_name()` 主要用于日志和诊断，其字符串格式不是 Rust 的稳定协议。
- 跨进程、跨语言和跨版本的稳定身份仍然使用 `ModelId`。

对于原来的 `template()`，建议改为：

```rust
pub fn generic_definition(
    &self,
) -> Option<&'static GenericTypeMetadata>;
```

`generic_definition()` 比 `template()` 更明确。它对应“这个具体泛型实例来自哪个泛型类型定义”：

```rust

let metadata = TypeMetadata::of::<Page<UserInfo>>();

let definition = metadata
    .generic_definition()
    .expect("Page<UserInfo> is a generic instance");
```

Java 中最接近的是 `ParameterizedType.getRawType()`，.NET 则直接叫 `GetGenericTypeDefinition()`。这里采用 `generic_definition()` 更准确，因为 Rust 没有 Java raw type 的类型擦除语义。

如果确认 `generic_definition()`，A001-I 就可以整体完成确认。

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

同意

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

A001-I 现已完整确认。我会把 `generic_definition()` 和标准库 `TypeId` 的最终接口写入正式记录，然后继续确认修订后的 FieldMetadata 与 PropertyMetadata。

A001-I 已写入正式确认记录。

继续确认修订后的字段和 Property API。

### A002-F2：`FieldMetadata` 最终基础接口

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

便利方法严格等价于：

```rust
field.is_identifier() == field.identifier().is_some()
field.is_unique() == field.unique().is_some()
field.is_reference() == field.reference().is_some()
```

`descriptor()` 返回字段完整的类型描述，包括 Option、容器、Value 及嵌套 selector，不再使用含义较弱的 `ty()`。

### A002-I：索引原因必须支持多个值

之前提出的单数 `indexing_source()` 不够准确，应改为：

```rust
pub fn indexing_reasons(&self) -> IndexingReasons;
```

因为一个字段可能同时是：

```rust
#[unique]
#[reference(entity = User)]
pub user: User;
```

此时它既因为 unique，也因为 reference 而具有查询能力。

定义：

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

语义：

```rust
pub fn is_indexed(&self) -> bool {
    !self.indexing_reasons().is_empty()
}
```

示例：

```rust
field.indexing_reasons()
    .contains(IndexingReasons::REFERENCE);
```

其中：

- `EXPLICIT`：显式 `#[indexed]`；
- `IDENTIFIER`：由 `#[identifier]` 隐含；
- `UNIQUE`：由 `#[unique]` 隐含；
- `REFERENCE`：由 `#[reference(...)]` 隐含。

重复的 `#[indexed]` 仍按已确认规则处理：

- identifier、unique、reference 字段不需要且不允许再显式标注 `#[indexed]`；
- 因此合法 metadata 通常不会同时出现 `EXPLICIT` 和其他原因；
- `IndexingReasons` 仍采用集合，是为了准确表达由多个隐含语义共同产生的查询能力。

### A002-P2：`PropertyMetadata` 最终基础接口

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
```

便利方法定义为：

```rust
property.is_field() == property.field().is_some()
property.is_getter() == property.getter().is_some()
property.is_setter() == property.setter().is_some()
```

需要特别注意：

- `is_getter()` 表示存在显式 getter 方法，不表示 property 是否可读。
- `is_setter()` 表示存在显式 setter 方法，不表示 property 是否可写。
- field 本身已经使 property 可读、可写。

因此，只有字段、没有显式方法的 property：

```text
is_field()    = true
is_getter()   = false
is_setter()   = false
is_readable() = true
is_writable() = true
```

同一个 property 可以同时满足：

```text
is_field()  = true
is_getter() = true
is_setter() = true
```

### A002-V：字段可见性 metadata

定义：

```rust
pub enum FieldVisibility {
    Public,

    Crate,
    Restricted(&'static str),
    Private,
}
```

映射规则：

- `pub` → `Public`
- `pub(crate)` → `Crate`
- `pub(super)`、`pub(in path)` → `Restricted(...)`
- 无 `pub` → `Private`

可见性只描述 Rust 源代码声明，不决定 metadata 的访问能力：

- private field 同样存在于 `fields()`。
- private field 仍然形成 field-backed property。
- private field 仍然可通过宏生成的安全字段访问接口读取和写入。
- `is_readable()`、`is_writable()` 不根据 Rust visibility 判断。

请确认：

- A002-F2：FieldMetadata 基础接口
- A002-I：多重索引原因
- A002-P2：PropertyMetadata 便利方法
- A002-V：字段可见性 metadata

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

A002-F2： 确认
A002-I：确认
A002-P2：确认
但是有两个问题：
1. is_getter() 表示存在显式 getter 方法，不表示 property 是否可读：这个能举出例子吗？
2. is_setter() 表示存在显式 setter 方法，不表示 property 是否可写：能举出例子吗？

A002-V：确认，
但是 FieldVisibility 是否能修改为
pub enum FieldVisibility {
    Public,
    Crate,
    Super,
    Path(&'static str),
    Private,
}
其中 Super 表示 pub(super)，Path表示 pub(path)

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

这两个问题暴露了我上一轮措辞不准确：存在 getter 必然可读，存在 setter 必然可写；真正需要强调的是反方向不成立。可见性枚举按你的拆分更精确，我会一并修订并记录。

你问得对，上一轮那两句话不准确，应该改成单向关系。

### Getter 与 readable

正确关系是：

```text
is_getter() == true  => is_readable() == true

is_readable() == true 不一定 is_getter() == true
```

不存在“有 getter 但不可读”的合法 property。

反例是只有 field、没有显式 getter：

```rust
#[Model]
pub struct User {
    name: String,
}
```

`name` property：

```text
is_field()    = true
is_getter()   = false

is_readable() = true
```

因为宏可以为真实 field 生成读取访问器，不需要显式 getter。

Getter-only computed property：

```rust
#[ModelProperties]
impl User {
    pub fn display_name(&self) -> String {
        self.name.to_uppercase()
    }
}
```

结果为：

```text
is_field()    = false
is_getter()   = true
is_readable() = true
is_computed() = true
is_writable() = false
```

最终公式是：

```rust
is_readable() == is_field() || is_getter()
```

### Setter 与 writable

正确关系是：

```text
is_setter() == true => is_writable() == true
is_writable() == true 不一定 is_setter() == true
```

不存在“有 setter 但不可写”的合法 property。

只有 field、没有显式 setter：

```rust
#[Model]
pub struct User {
    name: String,
}
```

`name` property：

```text
is_field()    = true
is_setter()   = false
is_writable() = true
```

因为宏可以为真实 field 生成写入访问器。

Setter-only virtual property：

```rust
#[ModelProperties]
impl User {
    pub fn set_full_name(&mut self, value: String) {
        // 将 value 拆分并写入其他真实字段
    }
}
```

如果不存在 `full_name` field 和 getter：

```text
is_field()    = false
is_getter()   = false
is_setter()   = true
is_readable() = false
is_writable() = true
storage_kind  = Virtual
```

最终公式是：

```rust
is_writable() == is_field() || is_setter()
```

### FieldVisibility

同意修改为：

```rust
pub enum FieldVisibility {
    Public,
    Crate,
    Super,
    Path(&'static str),
    Private,
}
```

精确映射：

| Rust 声明 | FieldVisibility |
|---|---|
| `pub` | `Public` |
| `pub(crate)` | `Crate` |
| `pub(super)` | `Super` |
| `pub(in crate::module)` | `Path("crate::module")` |
| `pub(self)` 或无 `pub` | `Private` |

Rust 的准确语法是 `pub(in path)`，不是 `pub(path)`。

等价形式进行归一化：

- `pub(in crate)` → `Crate`
- `pub(in super)` → `Super`
- `pub(in self)` → `Private`

以上修订均已写入正式确认记录。

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

同意，继续

>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

继续确认五种角色各自的专属 metadata。公共字段、Property、身份和能力仍放在 `TypeMetadata`，角色 metadata 只保存该角色独有的信息，避免重复。

### A003-E：`EntityMetadata`

```rust
pub struct EntityMetadata {
    // 私有字段
}

impl EntityMetadata {
    pub fn model_id(&self) -> ModelId;
    pub fn identifier(&self) -> &'static FieldMetadata;
}
```

使用：

```rust
let metadata = TypeMetadata::of::<User>();
let entity = metadata.as_entity().unwrap();

assert_eq!(
    entity.model_id().as_str(),
    "qubit.platform.iam.User",
);

let identifier = entity.identifier();
assert_eq!(identifier.name(), Some("id"));
```

约定：

- `model_id()` 一定存在，因为 Entity 的 `id` 必填。
- `identifier()` 一定存在，且对应唯一的直接 `Id` 字段。
- 不返回 `Option`，因为缺失 identifier 的 Entity 无法通过编译。
- Entity 的普通字段和 Property 仍通过以下接口访问：

```rust
metadata.fields();
metadata.properties();
```

- `EntityMetadata` 不保存 `projection`、`index`、持久化表名或 DAO 策略。
- 可用于生成查询过滤器的 indexed 路径将在独立的 QueryMetadata 中提供，不直接塞进 EntityMetadata。

### A003-P：`ProjectionMetadata`

```rust
pub struct ProjectionMetadata {
    // 私有字段
}

impl ProjectionMetadata {
    pub fn identifier(&self) -> &'static FieldMetadata;

    pub fn source(&self) -> Option<&ProjectionSource>;
    pub fn is_open(&self) -> bool;
    pub fn is_fixed(&self) -> bool;
}
```

来源定义：

```rust
pub enum ProjectionSource {
    Type(&'static TypeMetadata),
    Id(ModelId),
}
```

对应：

```rust
#[Projection(source = User)]
```

产生：

```rust
ProjectionSource::Type(TypeMetadata::of::<User>())
```

而：

```rust
#[Projection(source_id = "qubit.platform.iam.User")]
```

产生：

```rust
ProjectionSource::Id(
    ModelId::new("qubit.platform.iam.User"),
)
```

规则：

- `source() == None` 表示开放 Projection。
- `is_open()` 等价于 `source().is_none()`。
- `is_fixed()` 等价于 `source().is_some()`。
- `identifier()` 一定返回 Projection 唯一的直接 `Id` 字段。
- `ProjectionSource::Type` 指向的类型必须是 Entity。
- `ProjectionSource::Id` 最终也必须由注册表解析为 Entity。
- metadata 保留用户采用的是 `source` 还是 `source_id`，但二者解析后的业务效果一致。

后续由统一 resolver 提供：

```rust
projection.resolve_source(registry)
```

不让 `ProjectionMetadata::source()` 隐式访问全局注册表。

### A003-M：`ModelMetadata`

`#[Model]` 当前没有角色专属参数，因此：

```rust
pub struct ModelMetadata {
    // 首版没有公开属性
}
```

仍然保留这个类型，而不是把 `RoleMetadata::Model` 设计成无数据枚举项，原因是：

- `TypeMetadata::as_model()` 可以保持和其他角色一致的返回形式；
- 未来增加 Model 专属能力时不需要破坏 `RoleMetadata` 结构；
- 可以明确区分“这是 Model”与普通 structural metadata。

使用：

```rust
let metadata = TypeMetadata::of::<LoginRequest>();

if let Some(_model) = metadata.as_model() {
    // 确认它是 #[Model]
}
```

Model 的以下信息属于公共 TypeMetadata，而不是 ModelMetadata：

- 可选 `ModelId`；
- 是否注册；
- fields；
- properties；
- 泛型定义；
- 默认实现的 trait capabilities。

### A003-N：`EnumMetadata` 与 Variant API

```rust
impl EnumMetadata {
    pub fn variants(&self) -> &[EnumVariantMetadata];

    pub fn variant(
        &self,
        name: &str,
    ) -> Option<&EnumVariantMetadata>;

    pub fn variant_by_rust_name(
        &self,
        rust_name: &str,
    ) -> Option<&EnumVariantMetadata>;

    pub fn variant_by_serialized_name(
        &self,
        serialized_name: &str,
    ) -> Option<&EnumVariantMetadata>;
}
```

`variant(name)` 按已经确认的 canonical variant name 查询。

```rust
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
```

```rust
pub enum EnumVariantKind {
    Unit,
    Tuple,
    Struct,
}
```

三个名称分别表示：

- `rust_name()`：Rust 源代码中的 variant 标识符，例如 `InReview`。
- `name()`：模型系统的 canonical name，例如 `IN_REVIEW`，可由 `#[variant(name = "...")]` 修改。
- `serialized_name()`：Serde 最终序列化名称，受 `#[serde(rename = "...")]` 等标准属性影响。

三者不强制相同。

字段规则：

- unit variant 的 `fields()` 为空。
- tuple variant 字段只有 index，没有 name。
- struct variant 字段同时具有 index 和 name。
- `is_default()` 表示该 variant 是否标有标准 `#[default]`。
- `variant(name)` 不混合匹配 Rust 名称或序列化名称，避免同一个字符串产生歧义。

### A003-V：`ValueMetadata`

```rust
impl ValueMetadata {
    pub fn is_transparent(&self) -> bool;

    pub fn transparent_field(
        &self,
    ) -> Option<&'static FieldMetadata>;
}
```

规则：

- `#[Value(transparent)]`：
  - `is_transparent()` 返回 `true`；
  - `transparent_field()` 返回唯一存储字段。
- 普通具名 Value：
  - `is_transparent()` 返回 `false`；
  - `transparent_field()` 返回 `None`。
- 单字段 Value 不一定 transparent，只有显式声明 `transparent` 才是。
- Value 的 `copy`、`default`、`serialize` 等能力不放入 `ValueMetadata`，而是通过公共 TypeCapabilities 查询。
- Value 的字段、Property、约束、validator、codec、key_part 等仍从 `TypeMetadata` 和 `FieldMetadata` 查询。
- Value 是否具有 `ModelId`、是否注册，也由公共 `TypeMetadata` 查询，不属于 Value 角色语义。

请确认：

- A003-E：EntityMetadata
- A003-P：ProjectionMetadata
- A003-M：ModelMetadata
- A003-N：EnumMetadata 与 Variant API
- A003-V：ValueMetadata
