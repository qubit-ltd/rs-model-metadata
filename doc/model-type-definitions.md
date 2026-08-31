# 模型类型体系：五种类型的精确定义

- 日期：2026-08-27
- 状态：已确认的基础语义
- 范围：`#[Entity]`、`#[Projection]`、`#[Model]`、`#[Enum]`、`#[Value]`

本文从领域身份、生命周期、持久化、关系、运行时发现和递归处理六个维度，精确定义模型系统中的五种类型。
它是后续过程宏、元数据、注册表、校验器和随机对象生成器设计的语义基线。

本文不规定宏参数的最终语法、Rust trait 名称、元数据结构名称或迁移方案，也不保留旧设计的兼容性假设。

## 1. 首要原则：不要混淆五种不同能力

模型系统必须把以下概念分开。一个类型拥有其中某项能力，不代表它自动拥有其他能力。

### 1.1 静态类型元数据

静态类型元数据描述一个已知 Rust 类型的结构和行为，包括：

- 类型形状，例如 struct、newtype 或 enum；
- 字段、变体及其嵌套类型；
- `Option`、集合、Map、数组等容器形状；
- 文本、数值、长度、时间等局部约束；
- codec、generator、脱敏等字段处理策略；
- 该类型所属的模型类别。

五种类型都必须拥有静态类型元数据。否则，外层类型无法递归进入嵌套类型，也无法统一完成校验、随机生成、
序列化描述等工作。

### 1.2 全局注册

全局注册回答的是：

> 当前程序中有哪些可以被框架主动发现、枚举和调度的类型？

注册使用当前程序内的 Rust 类型身份或等价的进程内类型键。显示名称可以用于诊断，但不能成为持久化协议身份。

`Entity`、`Projection`、`Model` 和 `Enum` 参与全局注册；`Value` 不参与全局注册。

注册不等于拥有 `ModelId`。一个类型可以参与注册，但只在当前程序中按类型身份被发现。

### 1.3 `ModelId`

`ModelId` 是 Entity 种类的稳定协议身份，用于跨 crate、进程和构建版本引用同一种领域实体。它标识的是
“这是哪一种 Entity”，而不是某个 Entity 实例。

只有 `Entity` 拥有 `ModelId`。`Projection`、`Model`、`Enum` 和 `Value` 都没有 `ModelId`。

### 1.4 `#[identifier]`

`#[identifier]` 标识一个具体 Entity 实例的领域身份：

- `Entity` 必须且只能有一个类型为 `Id` 的 identifier；
- `Projection` 必须且只能有一个类型为 `Id` 的 identifier，它借用来源 Entity 的实例身份；
- `Model`、`Enum` 和 `Value` 禁止 identifier。

`ModelId` 与 identifier 是两个不同层级：

```text
ModelId     = Entity 的种类身份
identifier  = 某个 Entity 实例的身份
```

### 1.5 独立生命周期与持久化

独立生命周期表示一个对象可以被独立创建、查找、更新和删除，并在领域中独立存在。只有 `Entity` 具有这种
语义，也只有 `Entity` 可以直接声明主键、唯一约束、索引等持久化属性。

其他四种类型都不能因为自身字段形状看起来像一张表，就被推断为可独立持久化对象。

### 1.6 跨 Entity 关系

关系的目标必须是 `Entity`。关系字段可以保存目标 Entity 的 identifier、某个字段值、某个 Projection，或
按后续关系设计允许的其他选择结果，但这些值都只是目标 Entity 的表示，不会因此成为新的关系目标。

`Entity`、`Projection` 和 `Model` 可以声明指向 Entity 的关系。`Enum` 和 `Value` 本身不能直接声明关系。

## 2. 总览

| 类型 | 静态类型元数据 | 全局注册 | `ModelId` | `#[identifier]` | 独立持久化 | 可作为关系目标 | 可声明 Entity 关系 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `Entity` | 是 | 是 | 必须且稳定 | 必须且只能有一个 `Id` | 是 | 是 | 是 |
| `Projection` | 是 | 是 | 无 | 必须且只能有一个 `Id` | 否 | 否 | 是 |
| `Model` | 是 | 是 | 无 | 禁止 | 否 | 否 | 是 |
| `Enum` | 是 | 是 | 无 | 禁止 | 否 | 否 | 否 |
| `Value` | 是 | 否 | 无 | 禁止 | 否 | 否 | 否 |

## 3. `Entity`：有独立身份和生命周期的领域对象

### 3.1 精确定义

`Entity` 是能够在领域中独立存在，并具有独立身份、生命周期和持久化记录的对象。

判断一个类型是不是 Entity，核心问题不是“它有没有很多字段”，而是：

> 在其他对象都不存在时，业务是否仍然需要区分、查找和管理这个对象的不同实例？

如果答案是肯定的，它才可能是 Entity。

### 3.2 必须具备的语义

- 参与全局注册；
- 声明唯一且稳定的 `ModelId`；
- 必须且只能有一个 `#[identifier]` 字段；
- identifier 字段类型必须是 `Id`；
- identifier 同时表示领域实例身份和独立持久化记录的主键身份；
- 可以声明主键、唯一约束、索引以及指向其他 Entity 的关系；
- 是五种类型中唯一合法的关系目标；
- 自动生成器可以围绕它建立创建、复用和持久化依赖。

业务唯一字段不能替代 identifier；复合自然键也不能冒充 Entity 的实例身份。自然键、唯一约束与 Entity
identifier 必须分别建模。

### 3.3 `ModelId` 与 identifier 的组合

一个 Entity 实例的完整领域身份由两个层次组成：

```text
(Entity ModelId, instance Id)
```

例如，`Person` 与 `Organization` 即使碰巧拥有相同的 `Id` 值，也不是同一个领域对象，因为它们的
`ModelId` 不同。

## 4. `Projection`：某个 Entity 实例的派生表示

### 4.1 精确定义

`Projection` 是从一个 Entity 实例中裁剪、重组或计算得到的表示。它描述同一个 Entity 实例的部分信息，
而不是创造一个新的领域对象。

Projection 可以用于关联字段、列表摘要、查询结果或传输对象，但没有独立生命周期和独立持久化记录。

### 4.2 identifier 的语义

Projection 必须且只能有一个类型为 `Id` 的 identifier。该值借用来源 Entity 的 identifier：

```text
Projection.identifier == source Entity.identifier
```

它用于：

- 从 Projection 值中恢复被表示 Entity 的实例身份；
- 校验关系字段中的 Projection 与目标 Entity 对齐；
- 让持久化或查询层提取底层关联 ID。

它不是 Projection 自己的主键，也不能使 Projection 获得独立生命周期。

### 4.3 为什么注册但没有 `ModelId`

Projection 参与注册，是为了让框架发现其字段结构、identifier、约束和投影能力，并对关系选择结果进行
类型检查。Projection 不需要 `ModelId`，因为它不是独立领域对象种类。

一个 Projection 类型可以是：

- **通用 Projection**：例如 `Info`，可用于多种 Entity；
- **固定来源 Projection**：例如只表示 `Person` 的 `PersonInfo`。

因此，单凭 Projection 类型或其中的 identifier，不一定能确定来源 Entity。关系上下文必须另外保留目标
Entity 的 `ModelId`。当后续设计命名投影选择器时，完整的选择身份至少应包含：

```text
(Entity ModelId, projection selector) -> Projection type identity
```

这个映射是 Entity 与 Projection 之间的能力登记，不是 Projection 自己的 `ModelId`。

### 4.4 禁止的语义

Projection：

- 不能成为关系目标；
- 不能声明独立主键、唯一约束或数据库索引；
- 不能被独立创建、更新或删除；
- 不能脱离来源 Entity 声称自己的 identifier 表示独立对象。

Projection 可以包含指向其他 Entity 的关系字段，因为一个派生表示可能包含其他关联对象的信息。

## 5. `Model`：可独立发现的数据契约和对象图节点

### 5.1 精确定义

`Model` 是可以被框架独立发现和处理，但没有领域实例身份、独立生命周期或独立持久化记录的数据契约。

典型用途包括：

- 请求和响应 DTO；
- 配置与参数对象；
- 任务输入、输出和状态快照；
- 不对应单一 Entity 的组合查询结果；
- DAO 自动化测试或随机生成系统的顶层对象图入口。

### 5.2 必须具备的语义

- 参与全局注册；
- 没有 `ModelId`；
- 禁止 `#[identifier]`；
- 不能成为关系目标；
- 不能声明主键、唯一约束或数据库索引；
- 可以声明指向 Entity 的关系，从而描述对象图依赖；
- 可以作为框架主动发现、校验或生成的根类型。

### 5.3 与 Projection、Value 的区别

Model 不承诺自己来源于某个 Entity，因此不需要 Projection identifier。

Model 与 Value 都没有独立领域身份，但 Model 参与注册、可以作为框架根类型，也可以声明 Entity 关系；
Value 只能通过已知 Rust 类型被使用，不进入全局注册，也不能声明关系。

## 6. `Enum`：封闭的值域或代数和类型

### 6.1 精确定义

`Enum` 表示一个封闭的取值集合，或者由若干互斥变体构成的代数和类型。它描述“一个值可能有哪些形态”，
而不是一个具有实例身份和生命周期的领域对象。

`Enum` 同时包括：

- 无数据变体枚举；
- tuple 变体；
- struct 变体；
- 混合以上形态的带数据枚举。

### 6.2 必须具备的语义

- 参与全局注册；
- 没有 `ModelId`；
- 禁止 `#[identifier]`；
- 没有独立持久化生命周期；
- 不能成为关系目标；
- 不能直接声明 Entity 关系或持久化属性；
- 必须公开变体、变体字段、局部约束和表示策略等元数据。

Enum 注册的目的，是让校验器、schema、随机生成器和其他运行时消费者发现它的完整变体集合。序列化变体名
可以是稳定的外部协议名称，但它属于序列化协议，不是 `ModelId`。

如果带数据变体包含其他类型，校验和生成应递归进入其 payload。关系语义属于被嵌套类型自身，不能挂在
Enum 变体或 Enum 本体上规避角色边界。

## 7. `Value`：按内容定义、仅通过类型引用使用的值对象

### 7.1 精确定义

`Value` 是按内容而不是按身份区分的可复用值类型。两个 Value 是否相同，只取决于其内容，不取决于对象
身份、创建时间或持久化记录。

典型例子包括：

- `Revision(u64)`；
- `EmailAddress(String)`；
- `Money { amount, currency }`；
- `Address { ... }`；
- 数值区间、电话号码、邮政编码等领域值。

### 7.2 必须具备的语义

- 不参与全局注册；
- 没有 `ModelId`；
- 禁止 `#[identifier]`；
- 没有独立生命周期和持久化记录；
- 不能成为关系目标，也不能直接声明 Entity 关系；
- 不能声明主键、唯一约束、索引或所有权等持久化属性；
- 必须拥有完整静态类型元数据；
- 必须能够被外层类型递归校验、生成、描述和序列化。

“不注册”只表示框架不会在没有 Rust 类型上下文时主动发现 Value，不表示 Value 是 opaque 类型，更不表示
其字段约束不可见。

## 8. `Value` 不注册时如何自动校验

### 8.1 注册与校验是两条独立路径

注册解决的是“有哪些类型”；校验解决的是“这个已知类型的值是否合法”。

校验开始时，调用方已经拥有一个具体 Rust 类型，或者已经显式取得某个根类型的描述。因此校验器可以从
根类型的静态元数据出发，沿字段类型递归访问嵌套 Value，无需在全局注册表中搜索 Value。

例如：

```rust
#[Value]
pub struct Address {
    #[text(min_chars = 1, max_chars = 100)]
    pub city: String,
}

#[Model]
pub struct CreateOrganization {
    pub address: Address,
}
```

校验 `CreateOrganization` 时，逻辑路径是：

```text
CreateOrganization
└── address: Address
    └── city: String
        └── text(min_chars = 1, max_chars = 100)
```

校验错误必须携带完整字段路径，例如：

```text
address.city: 字符数不能少于 1
```

### 8.2 容器递归

容器不能中断 Value 校验：

- `Option<Value>`：值为 `Some` 时递归校验；
- `Vec<Value>`、数组或其他序列：逐元素校验，并在路径中记录下标；
- `Map<K, Value>`：逐值校验，并在路径中记录 key；
- `Box<Value>` 或其他透明包装：进入内部值继续校验。

例如，第三个地址的城市不合法时，错误路径应类似：

```text
addresses[2].city
```

### 8.3 Value 也可以被直接校验

当 Rust 类型在编译期已知时，Value 可以直接调用统一的强类型校验入口。它不必先成为 Entity、Model 或
注册类型。

具体校验 trait 和方法名称由后续 API 设计决定，但必须保留以下能力：

```text
validate(value: &Address) -> validation result
```

### 8.4 无类型输入的边界

如果系统只有一段无类型数据以及字符串 `"Address"`，并希望仅凭这个字符串动态发现 schema 并校验，
那么全局注册确实是必要的。这种需求表示该类型是可独立发现的数据契约，应使用 `Model`，而不是 `Value`。

对于 Value，调用方必须满足以下条件之一：

- 编译期已知具体 Rust 类型；
- 从已知 Entity、Projection、Model、Enum 或外层 Value 的字段元数据递归到达它；
- 显式把该 Value 的类型描述传给通用处理器。

系统不得为了方便无类型字符串查找而偷偷将所有 Value 加入全局注册表。

## 9. 元数据与注册表的分层要求

后续实现无论采用什么 Rust 类型名称，都必须体现以下三层能力：

```text
静态类型描述
├── Entity
├── Projection
├── Model
├── Enum
└── Value

全局注册条目
├── Entity
├── Projection
├── Model
└── Enum

Entity 稳定身份索引
└── ModelId -> Entity 注册条目
```

因此：

- 静态类型描述不能强制包含 `ModelId`；
- 注册条目不能强制要求每种已注册类型都有 `ModelId`；
- Entity 注册信息必须额外携带并索引稳定 `ModelId`；
- Value 的字段描述必须能够引用其静态类型描述，而不依赖全局注册条目；
- 递归校验和递归生成必须从类型描述导航，不能把注册表当作嵌套类型目录使用。

这一分层应尽量通过类型结构防止非法状态，而不是创建一个包含大量 `Option` 字段、允许任意角色组合的通用
元数据对象。

## 10. 必须拒绝的非法状态

后续宏和元数据校验至少必须拒绝：

- Entity 缺少 `ModelId`；
- 非 Entity 声明 `ModelId`；
- Entity 或 Projection 缺少 identifier；
- Entity 或 Projection 存在多个 identifier；
- Entity 或 Projection 的 identifier 不是 `Id`；
- Model、Enum 或 Value 声明 identifier；
- Projection、Model、Enum 或 Value 声明独立持久化属性；
- Enum 或 Value 直接声明 Entity 关系；
- 关系把 Projection、Model、Enum 或 Value 当作目标；
- Value 的嵌套字段没有可用类型元数据且未显式声明为 opaque；
- 因 Value 未注册而静默跳过其字段约束。

## 11. 验收场景

完成后，至少应满足以下可观察行为：

1. Entity、Projection、Model 和 Enum 都能从当前程序的全局注册表中枚举到，Value 不能。
2. 全局注册表只要求 Entity 具有 `ModelId`，并能按 `ModelId` 唯一查找 Entity。
3. Projection、Model 和 Enum 在没有 `ModelId` 的情况下仍然可以正常注册和按 Rust 类型身份访问。
4. Entity 和 Projection 都严格要求唯一的 `Id` identifier；其他三类严格禁止 identifier。
5. 关系的目标始终解析为 Entity；Projection 只能作为从目标 Entity 选择出的表示值。
6. 嵌套 Value 的字段约束可以从任意已知根类型递归读取。
7. 自动校验器能够校验 Value 内部约束，并返回包含完整嵌套路径的错误。
8. `Option`、序列、Map、数组和透明包装不会中断 Value 的递归校验。
9. 已知 Rust 类型的 Value 可以直接校验，不要求全局注册。
10. 只有字符串类型名而没有目标类型上下文时，系统不会假装能够动态发现未注册 Value。
11. Enum 的变体和 payload 可被递归描述、校验和生成，但 Enum 自身不获得身份或关系语义。
12. 任何违反第 10 节角色不变量的声明，都在尽可能早的阶段给出确定、可定位的错误。
