# `Entity`、`Projection` 与 `Reference` 过程宏重构设计

- 日期：2026-08-27
- 状态：阶段性设计；已确认约束与待定 API 明确分列
- 范围：`rs-model-derive` 及其依赖的 `rs-model-metadata` 公共元数据协议
- 不包含：带数据 `#[Enum]` 的实现、ORM/DAO 实现、随机数据生成器的具体实现

本文是 `Entity`、`Projection`、模型角色、identifier 和 `Reference` 宏重构的当前基准；其他阶段性文档中
与本文冲突的早期宏设计，以本文为准。

## 1. 背景与目标

当前所有结构体主要通过 `#[Model]` 注册，但平台实际存在三种不同的结构体角色：

- 拥有独立数据库表的持久化实体；
- 表示实体简化形态的查询投影；
- 不要求独立持久化、也不表示实体投影的普通数据模型。

这三种角色在 identifier、数据库映射、引用解析和测试数据生成方面具有不同约束。继续使用同一个
`#[Model]` 角色会迫使下游根据字段形态猜测语义，例如根据是否存在 `id` 或 `#[identifier]` 猜测一个
模型是否是实体。

本次重构的目标是：

1. 通过 `#[Entity]`、`#[Projection]` 和 `#[Model]` 显式表达模型角色；
2. 保留稳定的 `ModelId` 作为跨 crate 模型协议，避免通过 Rust 类型路径建立不必要的编译依赖；
3. 为 Entity 和 Projection 建立明确、可验证的 identifier 规则；
4. 明确 `#[reference]` 的值选择、合法性校验和自动生成依赖语义；
5. 让下游持久化、查询、模型注册和测试数据生成只依赖统一元数据，而不是字段名或特殊字符串约定。

## 2. 模型角色

### 2.1 角色定义

```rust
pub enum ModelRole {
    Model,
    Entity,
    Projection,
    Enum,
}
```

- `Entity`：需要独立持久化并拥有单独数据库表的结构体；
- `Projection`：某个 Entity 的简化描述，用于关联查询和数据传输，本身没有独立数据库表；
- `Model`：通用数据模型，可以是值对象，也可以是复杂 DTO、快照、配置或其他非 Entity 结构；
- `Enum`：枚举模型，包括无数据枚举与带数据枚举。

`ModelRole` 是领域及持久化语义，现有 `TypeKind::{Struct, Enum, Newtype}` 是 Rust 数据结构形态。两个维度
必须独立保留，不能相互替代。

### 2.2 过程宏 API

```rust
#[Entity(id = "qubit.platform.metadata.Category")]
pub struct Category {
    #[identifier]
    pub id: Id,
}

#[Projection(id = "qubit.platform.core.Info")]
pub struct Info {
    #[identifier]
    pub id: Id,
    pub code: String,
    pub name: String,
}

#[Model(id = "qubit.platform.address.Coordinate")]
pub struct Coordinate {
    pub longitude: Decimal,
    pub latitude: Decimal,
}

#[Enum(id = "qubit.platform.core.Scope")]
pub enum Scope {
    System,
    Entity(EntityRef),
}
```

`#[Model]` 不等同于值对象宏。它表示没有 Entity 或 Projection 专用语义的普通模型。

### 2.3 Trait 层次

所有注册模型共享基础能力，特殊角色增加 marker trait：

```rust
pub trait Model: HasTypeMetadata + HasModelRegistration {}

pub trait EntityModel: Model {
    const IDENTIFIER_FIELD: &'static str;

    fn id(&self) -> Id;
}

pub trait ProjectionModel: Model {
    const IDENTIFIER_FIELD: &'static str;

    fn projected_id(&self) -> Id;
}

pub trait EnumModel: Model {}
```

上面的具体方法名仍可在实现前调整，但必须保留以下能力边界：

- 四类宏均实现 `Model`；
- `#[Entity]` 额外实现 `EntityModel`；
- `#[Projection]` 额外实现 `ProjectionModel`；
- `#[Enum]` 额外实现 `EnumModel`；
- 普通 `#[Model]` 不能被当作 Entity 或 Projection 使用。

## 3. Identifier 规则

### 3.1 Entity identifier

每个 `#[Entity]` 必须且只能声明一个 `#[identifier]`：

```rust
#[Entity(id = "qubit.platform.person.Person")]
pub struct Person {
    #[identifier]
    pub person_id: Id,
}
```

约束如下：

- identifier 字段可以使用任意合法字段名；
- 字段类型必须是 `qubit_id::Id`；
- identifier 同时表示 Entity 的领域身份和独立数据表主键；
- 当前不支持复合主键；
- 业务唯一性通过 `#[unique]` 等独立元数据表达，不能冒充 identifier。

过程宏生成返回类型为 `Id` 的访问方法，使 Rust 类型检查器验证 identifier 字段确实能够作为 `Id`
返回。`EntityRef::of::<T: EntityModel>(id)` 因而只能接受合法 Entity 类型。

### 3.2 Projection identifier

每个 `#[Projection]` 也必须且只能声明一个 `#[identifier]`：

```rust
#[Projection(id = "qubit.platform.core.Info")]
pub struct Info {
    #[identifier]
    pub id: Id,
    pub code: String,
    pub name: String,
}
```

Projection identifier 的语义是“被投影 Entity 的身份”，不是 Projection 自身的主键。因此：

- Projection 不生成独立数据表；
- Projection identifier 不生成 Projection 主键元数据；
- Projection identifier 用于从投影值中提取关联实体身份；
- 持久化层可以把 `Foo.category: Info` 展开为 `foo.category_id`；
- 查询和关联层可以据此将投影值与目标 Entity 的 identifier 对齐。

Entity identifier 和 Projection identifier 应在元数据中共享“身份字段”概念，但是否产生数据库主键必须由
`ModelRole` 决定，不能继续无条件把 `#[identifier]` 归一化为 `PrimaryKey`。

### 3.3 普通 Model 的 identifier

普通 `#[Model]` 是否允许 `#[identifier]` 尚未最终确认。实现前必须完成现有模型清点，区分以下情况：

- 应迁移为 `#[Entity]` 的持久化结构；
- 应迁移为 `#[Projection]` 的简化实体结构；
- 确实需要局部身份、但不属于 Entity 或 Projection 的普通模型。

在该清点完成前，不应直接把普通 `#[Model]` 上的 `#[identifier]` 全部判为编译错误。

## 4. Projection 来源

Projection 支持通用投影和固定来源投影。

### 4.1 通用 Projection

```rust
#[Projection(id = "qubit.platform.core.Info")]
pub struct Info {
    #[identifier]
    pub id: Id,
    pub code: String,
    pub name: String,
}
```

通用 Projection 不绑定固定来源。具体来源由使用该 Projection 的 `#[reference]` 字段指定。`Info` 可以用于
Category、Country、Organization 等多个 Entity。

### 4.2 固定来源 Projection

```rust
#[Projection(
    id = "qubit.platform.person.PersonInfo",
    source = Person
)]
pub struct PersonInfo {
    #[identifier]
    pub id: Id,
}
```

- `source` 可选且最多出现一次；
- `source` 接受 Rust 类型路径；
- `source` 必须实现 `EntityModel`；
- 固定来源适用于与来源 Entity 位于合理依赖边界内的专用 Projection；
- 通用 Projection 省略 `source`，不使用 `source = None` 或 `generic = true` 等冗余写法。

`ProjectionMetadata` 至少需要暴露可选来源。其最终存储为 Rust 类型引用还是 `ModelId`，应与注册表现有的
`NamedTypeRef` 能力一起评估，但公共语义必须保持上述约束。

## 5. `#[reference]` 的稳定协议

### 5.1 API 形式

```rust
pub struct User {
    #[reference(
        entity = "qubit.platform.person.Person",
        property = id
    )]
    pub person_id: Id,
}
```

```rust
pub struct Foo {
    #[reference(
        entity = "qubit.platform.metadata.Category",
        property = info
    )]
    pub category: Info,
}
```

`entity` 必须继续接受稳定的 `ModelId` 字符串，不能改为 `entity = Category` 这样的 Rust 类型参数。原因是
领域模型可能分布在多个 crate 中；Rust 类型参数会把领域引用提升为 crate 编译依赖，并可能形成循环依赖。

这里接受的是过程宏中的字符串字面量，而不是任意运行时字符串。过程宏展开为经过格式校验的
`ModelId`，完整注册表随后校验目标模型是否真实存在以及角色是否合法。

### 5.2 `property` 的准确语义

`property` 表示从被引用 Entity 中选择哪个属性或投影来填写当前字段：

```text
当前模型.当前字段 <- 被引用 Entity.property
```

因此：

```text
User.person_id <- Person.id
Foo.category   <- Category.info()
```

`property` 不能被 Projection identifier 替代：

- `property = info` 决定如何从 Category 取得完整的 `Info` 值；
- `Info.id` 上的 `#[identifier]` 决定如何从已有 `Info` 值中取得 Category 的关联身份。

两者分别描述值的生成方向和关联身份的提取方向：

```text
生成：Category --info--> Info --赋值--> Foo.category
关联：Foo.category --identifier--> Info.id --对应--> Category.id
```

`property` 可以是普通字段选择器，例如 `id`、`code`，也可以是投影选择器，例如 `info`。省略 `property`
表示使用整个目标 Entity 值。

现有 `ReferenceTarget::Property(FieldPath)` 的总体抽象可以保留，但其文档应明确它是“目标值选择器”，不应
被简单描述为数据库外键列。

### 5.3 `existing` 与生成依赖

`existing` 默认值保持为 `true`。

当 `existing = true` 时，被引用 Entity 必须在当前 Entity 有效或持久化之前存在。自动数据生成器据此建立
依赖边：

```text
User -> Person
```

生成 User 时，生成器应先查找可复用的 Person；如果生成策略要求创建新对象，则先生成并持久化 Person，
再提取 `Person.id` 填入 `User.person_id`，最后生成和持久化 User。

`existing = false` 表示该引用不要求目标记录预先存在，因此不能建立相同的强制前置依赖。目标对象如何被
内联生成、稍后持久化或仅作为临时数据，应由生成器和持久化层的后续设计决定。

`existing` 不表示字段是否为 `Option`。字段是否必填仍由 Rust 类型及 required 元数据决定。

### 5.4 `path`

现有 `path` 用于描述包含对象图中与当前引用等价的另一条引用路径，例如 Address 中行政区层级之间的
一致性约束。它与 `property` 的值选择语义正交，应继续保留：

- `property`：从目标 Entity 选择什么值；
- `path`：在当前对象图中从哪里找到等价引用；
- `existing`：目标 Entity 是否必须预先存在。

### 5.5 分阶段校验

过程宏编译期能够完成：

- `entity` 必须是字符串字面量；
- `ModelId` 格式合法；
- `property` 和 `path` 的语法及路径段合法；
- 重复参数和互斥参数校验；
- `existing` 必须是布尔值。

完整模型注册表建立后必须完成：

- `entity` 指向的 ModelId 已注册；
- 目标模型角色是 `Entity`；
- 普通字段 `property` 确实存在；
- 投影选择器 `property` 已注册且其结果类型与当前字段兼容；
- Projection 字段具有合法 identifier；
- 标量属性、投影属性与当前字段的类型兼容；
- `path` 指向另一条语义一致的 reference；
- `existing = true` 形成的必需依赖图不存在无法构造的强依赖环。

不用 Rust 类型表达 `entity` 意味着编译器无法通过 trait bound 直接证明目标 Entity 已链接。该检查必须在
完整模型注册表建立后完成。这是避免 crate 循环依赖所必须接受的边界。

## 6. 投影选择器注册需求

当前注册表对 `property = info` 采取特殊跳过策略，不能验证 `info` 是否存在、返回何种 Projection，也不能
为自动生成器提供可执行的值提取能力。重构后必须删除这种“看到 `info` 就认为合法”的特殊分支。

新的投影选择器协议必须满足：

1. Entity 能够声明自己提供哪些命名投影，例如 `info`；
2. 每个选择器记录返回 Projection 的 ModelId 或等价类型元数据；
3. 注册表可以验证 reference 字段类型与选择器返回类型兼容；
4. 自动生成器能够根据选择器从目标 Entity 值取得投影值；
5. 协议不能要求引用方 crate 依赖目标 Entity 的 Rust crate；
6. 通用 `Info` 与 `PersonInfo` 等专用 Projection 都能使用该协议。

投影选择器的具体声明 API 尚未定稿。候选方向包括在 Entity 宏参数中声明、在投影转换实现上单独注册，或
提供专门的投影注册宏。该选择会影响可执行提取函数的类型擦除方式，应在下一阶段单独评审，不在本文中
假定某一种实现。

## 7. 元数据结构调整

`TypeMetadata` 至少增加以下语义：

```rust
pub struct TypeMetadata {
    // existing fields
    role: ModelRole,
    identifier: Option<IdentifierMetadata>,
    projection: Option<ProjectionMetadata>,
}
```

要求如下：

- `role` 显式存储，不通过其他属性推断；
- `identifier` 表示身份字段，不直接等同于数据库主键；
- Entity 根据 identifier 派生主键语义；
- Projection 保留 identifier，但不派生独立主键；
- `projection` 只对 `ModelRole::Projection` 有效；
- 构造 API 应尽量防止产生角色与专属元数据不一致的非法状态。

`ReferenceMetadata` 继续保存：

- 目标 `ModelId`；
- 整体目标或属性/投影选择器；
- `existing`；
- 可选等价引用 `path`。

如果投影选择器需要独立元数据，不应继续把它伪装成 Entity 的普通 struct 字段。

## 8. `#[ownership]` 与 `#[key]` 的处理

现有 `#[ownership(owner = Type)]` 仅保存一个所有者模型类型，不能表达 Principal 对具体 Entity 实例的
所有权。平台已经把 Ownership 定义为可独立持久化的关系 Entity，因此两者存在语义冲突。该属性是否废弃
或重新命名尚待最终确认，本次宏重构不能默认照搬。

现有 `#[key(name = ..., fields = ...)]` 只表达命名逻辑字段组，不自动表示主键、唯一约束或数据库索引，且
当前缺少明确消费方。是否保留为自然键元数据，需要在清点实际使用场景后决定；不能把它用于替代 Entity
identifier。

## 9. 迁移与兼容性原则

这是模型公共 API 的破坏性重构，建议按以下顺序迁移：

1. 在 metadata 中增加 `ModelRole`、通用 identifier 和 Projection 元数据；
2. 增加 `#[Entity]`、`#[Projection]` 及对应 trait，不立即移除 `#[Model]`；
3. 迁移明确的 Entity 和 Projection；
4. 注册表同时校验角色、identifier 和 reference；
5. 设计并实现投影选择器注册协议，替换 `info` 特殊分支；
6. 完成所有调用方迁移后，再评估是否删除旧的兼容行为。

迁移期间不得改变既有 `ModelId`。Rust 类型、module 或 crate 的调整不能导致已经持久化或跨服务传输的
模型协议 ID 变化。

带数据 `#[Enum]` 已完成重构，不属于本阶段实现范围；本阶段只需保证它注册为 `ModelRole::Enum` 并实现
统一的基础 `Model` 能力。

## 10. 验收条件

重构完成后至少应满足：

- Entity 缺少 identifier、存在多个 identifier 或 identifier 不是 `Id` 时编译失败；
- Projection 缺少 identifier、存在多个 identifier 或 identifier 不是 `Id` 时编译失败；
- Projection identifier 不被注册为独立数据表主键；
- `ModelRole` 能从注册元数据直接读取；
- `entity = "..."` 不引入目标 Entity crate 的编译依赖；
- ModelId 格式错误在过程宏展开阶段失败；
- 缺失目标 Entity、错误模型角色和错误 property 在完整注册表校验阶段失败；
- `property = id` 能描述标量关联值生成；
- `property = info` 能通过正式注册的投影选择器完成验证和取值，不再特殊跳过；
- `existing = true` 能为自动生成器提供稳定的前置依赖边；
- 必需 reference 形成不可构造依赖环时，注册表报告包含来源模型、字段和目标模型的确定性错误；
- 通用 Projection 和固定来源 Projection 均有覆盖测试；
- 已完成的带数据 Enum 行为不回退。

## 11. 待继续设计

以下问题尚未定稿，进入实现计划前必须逐项确认：

1. Entity 与 Projection 是否提取共同的 identifier trait，以及最终方法名；
2. 投影选择器的声明、注册和类型擦除 API；
3. Projection `source` 在元数据中的最终表示；
4. 普通 `#[Model]` 是否允许局部 identifier；
5. `#[ownership]` 的废弃或重命名方案；
6. `#[key]` 是否保留，以及是否重新定义为 natural key；
7. reference 指向非 identifier 唯一字段时的数据库约束规则；
8. `existing = false` 在自动生成与持久化阶段的完整生命周期语义。
