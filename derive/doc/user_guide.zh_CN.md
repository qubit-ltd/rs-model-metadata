# 领域模型声明指南

[English](user_guide.md) · [设计文档](design.zh_CN.md) · [README](../README.zh_CN.md) · [运行时指南](../../doc/user_guide.zh_CN.md)

本指南面向定义业务模型的开发者，适用于 0.2.0。假设账户服务同时需要识别用户身份、
查找可查询字段、保护敏感信息和校验输入：如果这些规则分别写在数据库映射、验证器和输出代码里，
字段改名时就容易遗漏一处。`qubit-model-derive` 让你在 Rust 类型上声明这些**领域事实**，
生成的元数据可由 `qubit-model-metadata` 读取。它不会替应用生成数据库查询或自动执行所有规则。

第一次接入请按“安装 → [实战场景](#实战场景声明用户模型并读取属性) →
[运行时指南](../../doc/user_guide.zh_CN.md)”阅读。已在使用时，可直接跳到下面的
[功能速查](#功能速查)：每节给出使用场合、代码和结果；精确的参数与类型签名以 Rustdoc 为准。
项目要求 Rust 1.94、edition 2024。清单设置了 `publish = false`，安装方式见下文。

## 安装与示例上下文

示例统一使用以下检出布局，应用命令在 `rs-platform/app` 中执行。
直接依赖必须与 runtime 使用同一份检出路径：

```text
checkout/
  rs-platform/
    app/                 # Cargo.toml 与 src/main.rs
    rs-model-metadata/   # runtime 与 derive/
    rs-reflect/
  rust-common/
    rs-id/
    rs-validator/
    rs-validation-rules/
    rs-redact/
    rs-datatype/
```

下面的 `core` 安装方案用于 `core/...` 示例；`validation/...` 程序使用
运行时指南中的独立 validation 安装方案。每次把一份完整程序复制到 `src/main.rs`，执行 `cargo run`。
标记为 `rust,ignore` 的片段需要旁边说明的 API 对象或应用自定义类型。
`publish = false` 仅说明当前清单禁止发布，不能据此断言某版本没有发布到 crates.io；
离线解析失败也只能说明当前本地依赖缓存不足。

<!-- example: core -->
```toml
[dependencies]
qubit-model-metadata = { version = "0.2", path = "../rs-model-metadata", default-features = false }
qubit-model-derive = { version = "0.2", path = "../rs-model-metadata/derive" }
qubit-id = "0.7"
qubit-reflect = { version = "0.2", path = "../rs-reflect" }
```

## 实战场景：声明用户模型并读取属性

账户资料由 `User` 保存。它需要稳定身份，`nickname` 可供上层构造查询条件，`token`
在默认输出中按脱敏策略处理；页面显示名由方法计算，不另存一份。下面先验证这些**声明确实进入元数据**。

<!-- example: core/quick-start -->
```rust
use qubit_id::Id;
use qubit_model_derive::{Entity, ModelImpl};
use qubit_model_metadata::metadata::TypeMetadata;

#[Entity(id = "guide.User")]
struct User {
    #[identifier]
    id: Id,
    #[indexed]
    nickname: String,
    #[redact(level = "secret")]
    token: String,
    tags: Vec<String>,
}

#[ModelImpl]
impl User {
    pub fn display_name(&self) -> String { self.nickname.clone() }
    #[model_property(skip)]
    pub fn diagnostic(&self) -> usize { self.tags.len() }
}

fn main() {
    let metadata = TypeMetadata::of::<User>();
    assert!(metadata.field("nickname").unwrap().is_indexed());
    assert!(metadata.try_property("display_name").unwrap().unwrap().is_computed());
    assert!(metadata.try_property("diagnostic").unwrap().is_none());
}
```

运行 `cargo run` 后三个断言都通过：`indexed` 被记录在 `nickname` 上，公开 getter
`display_name` 成为计算属性，`diagnostic` 因 `skip` 不出现在 Property 集合。
存储字段本身也是 Property；`#[ModelImpl]` 可以把公开 getter/setter 与同名字段合并。
这些结果只证明声明和属性组装成功；要解析跨模型关系或执行验证，继续看运行时指南。

默认 Debug、Display、Serialize 委托 rs-redact。`redact(skip)` 在具名、位置字段、Enum payload 和透明
包装上都省略整个字段；关闭应用脱敏策略后恢复输出。Map 键脱敏后发生重名时，序列化返回错误。
观察原始字段值的脱敏模式不能与绕过它的自定义序列化适配器混用。

可以直接检查生成的 `Debug` 是否泄漏原值：

```rust,ignore
#[Model]
struct Secrets {
    #[redact(level = "secret")]
    token: String,
    #[redact(skip)]
    internal: String,
}

let value = Secrets { token: "raw-token".into(), internal: "internal".into() };
let output = format!("{value:?}");
assert!(!output.contains("raw-token"));
assert!(!output.contains("internal"));
```

片段需引入 `Model`。`skip` 省略整个字段；`level` 交给当前脱敏策略处理，
因此不要把固定掩码文本写死在断言里。

## 功能速查

### 选择模型角色

先根据业务含义选择角色；角色决定结构解析规则，不是为了获得某个自动 trait 而选。

| 角色 | 适用形状和用途 |
| --- | --- |
| Entity | 非泛型具名 struct，恰有一个直接使用 `qubit_id::Id` 的 identifier |
| Projection | 非泛型具名视图，具有 identifier，可指定固定 Entity 来源 |
| Model | 具名或 unit struct，可使用泛型 |
| Enum | unit、tuple、具名或混合 variant，可使用泛型 |
| Value | 具名值对象或单字段 tuple 包装，可指定 `transparent` |

下面的声明展示场景中未用到的三个角色。`Projection` 表示 `Account` 的视图，
`Email` 是可嵌入的值，`State` 是有限状态；需要按稳定 ID 跨 crate 查找时写 `id`。

```rust,ignore
#[Entity(id = "account.Account")]
struct Account { #[identifier] id: Id }

#[Projection(id = "account.Summary", source = Account)]
struct Summary { #[identifier] id: Id }

#[Value(id = "account.Email", transparent)]
struct Email(String);

#[Enum(id = "account.State")]
enum State { Active, Disabled }
```

这里的 `Id` 来自 `qubit_id::Id`；片段需加上相应宏和 `Id` 的 `use`。
`TypeMetadata::of::<Summary>().as_projection()`、`as_value()` 和 `as_enum()`
可分别取得角色元数据。`Entity` 必须有稳定 `id`；其他角色可匿名，但匿名类型不能按稳定 ID 查询。

泛型定义与具体类型需要启用运行时 `generic` feature，不支持 lifetime 参数。

### 控制自动生成的 trait 与序列化

五种角色均默认生成 Clone、Debug、Display、PartialEq、Redact、Serialize 与 Deserialize。
结构性相等和哈希的默认值按角色区分：

| 角色 | PartialEq | Eq | Hash |
| --- | --- | --- | --- |
| Entity | 默认启用 | 显式启用 | 显式启用 |
| Projection | 默认启用 | 显式启用 | 显式启用 |
| Model | 默认启用 | 显式启用 | 显式启用 |
| Value | 默认启用 | 默认启用 | 默认启用 |
| Enum | 默认启用 | 默认启用 | 默认启用 |

确实需要结构性 Eq 和 Hash 时显式开启，例如：

```rust,ignore
#[Entity(id = "account.User", eq, hash)]
struct User { #[identifier] id: Id, name: String }
```

片段需引入 `Entity` 和 `Id`。比较与哈希会覆盖全部存储字段；如果用户姓名可改，
集合键通常应使用稳定 `id`，不要把整个可变 `User` 作为键。
这是源码兼容性变更：只为实际依赖这些 trait 的模型补开关。
`eq` 启用 Eq；`hash` 要求 Eq 已启用，包括 `ord` 或值角色提供的 Eq。
`ord` 隐含 Eq 与排序能力，但不启用 Hash；`partial_ord` 只依赖 PartialEq。
`eq, no_eq` 和 `hash, no_hash` 为冲突组合。`no_*` 关闭自动能力，`no_eq` 同时移除默认 Hash。
全部 variant 为 unit 的 Enum 还默认生成 Copy。`copy`、`default`、`partial_ord`、`ord` 启用额外能力。
后续新增 trait 默认关闭，须先增加显式宏选项才能启用。

Enum 的 `default` 要求恰有一个标准 `#[default]` unit variant。可构造默认值不等于满足领域约束。
未声明类型级 `serde(rename_all)` 时，宏会为每个 variant 安装与 metadata canonical 名一致的默认 Serde
wire 名（通常为 SCREAMING_SNAKE_CASE）；`#[variant(name = "...")]` 或 variant 级 `#[serde(rename = "...")]` 仍优先。

角色属性放在显式 derive 之前，宏才能识别并避免重复生成。手写实现使用对应的关闭开关。
`no_redact` 禁止当前字段或 selector 上存在脱敏规则，但嵌套类型保留自己的安全输出。
包含 HashMap 的 Value 类型通常需要 `no_hash`。泛型约束按实际存储字段生成，涵盖 PhantomData 和 const 数组。

具名 Option 与标准集合字段在缺失时使用默认值，空值默认不序列化。`keep_serializing` 只关闭自动省略，
显式 Serde 配置优先；位置字段不自动省略。

```rust,ignore
#[Model]
struct Response {
    #[serde(rename = "mail")]
    email: Option<String>,
    #[keep_serializing]
    tags: Vec<String>,
}
```

片段需引入 `Model`。序列化时 `email` 使用 `mail` 名称；空 `tags` 仍保留在输出中。
这控制输出形状，不会改变字段的 Rust 名称或自动执行字段验证。

默认序列化委托 rs-redact，不支持 Serde `flatten`。需要普通 Serde 的字段展开时，
显式选择 `#[Model(no_redact)]`；该类型不能再声明本地字段或 selector 脱敏规则。
元数据仍保留展开、双向名称、跳过控制和默认值来源。

### 合并字段与访问器

当应用通过方法读取或更新字段时，使用 `#[ModelImpl]` 把访问能力加入 Property：

```rust,ignore
#[Model]
struct Profile { name: String }

#[ModelImpl]
impl Profile {
    pub fn name(&self) -> &str { &self.name }
    pub fn set_name(&mut self, value: String) { self.name = value; }
}

let metadata = TypeMetadata::of::<Profile>();
let name = metadata.try_property("name")?.expect("name 属性");
assert!(name.getter().is_some() && name.setter().is_some());
```

片段需引入 `Model`、`ModelImpl`、`TypeMetadata`，并放进返回 `Result` 的函数中。
同名字段与合格访问器合并成一个 Property；冲突会通过 `try_property` 返回组装错误。
私有方法和不符合访问器形状的方法不会贡献 Property。

### 声明引用、索引和逻辑键

```rust,ignore
#[Entity(id = "shop.Order")]
struct Order { #[identifier] id: Id }

#[Model(id = "shop.Line")]
struct Line {
    #[reference(entity = Order, property = id)]
    order_id: Id,
    #[indexed]
    sku: String,
}

#[Value(id = "shop.Coordinate")]
struct Coordinate {
    #[key_part(order = 0)] x: i32,
    #[key_part(order = 1)] y: i32,
}
```

`order_id` 记录目标 Entity 和目标属性；`sku` 记录显式 indexed 原因；`Coordinate`
按 `x, y` 声明逻辑复合键。先用 `TypeMetadata` 读取本地声明，再由
`StructureResolver` 检查引用目标、路径和键的跨模型关系。`#[indexed]` 不会创建数据库索引或查询对象。
上述片段需引入 `Id`、`Entity`、`Model`、`Value`，并与运行时 crate 一起编译。

需要复用当前对象或父对象上的引用绑定时，才写 `path`：

```rust,ignore
#[reference(entity = Order, property = id, path = "..")]
order_id: Id,
```

`..` 表示领域父对象；它不是 Rust 字段路径。省略 `path` 表示不请求复用绑定。

`reference.path` 使用 `/` 和 `..` 导航对象绑定。
路径经过引用字段时，继续访问所绑定的完整 Entity，即使该字段只保存 ID 或 Projection。
省略 path 表示没有请求复用绑定；容器本身不增加领域父对象。独立的 `property` 仍使用点分隔路径。
引用可放在 Option、智能指针、序列、Set、数组及其组合中，不能直接放在 Map 中。
Enum payload 可以声明引用，但 Value 不能通过嵌套 Enum 隐藏引用。

identifier、unique、reference 都贡献隐含 indexed 原因，再添加 `#[indexed]` 会报冗余错误。
例如同一租户内邮箱不能重复，可在 Entity 的邮箱字段上写：

```rust,ignore
#[unique(respect_to(tenant_id), ignore_case = true)]
email: String,
```

它记录“按 `tenant_id` 划分作用域、比较邮箱时忽略大小写”的声明；本库不访问数据库，
唯一性是否成立仍由消费它的组件判断。`unique` 通过 `respect_to(...)` 表达作用域。
文本能力字段默认忽略大小写，真实类型别名同样有效；
非文本字段不能显式指定 ignore_case。具名 Model、Value 的 `key_part(order = n)` 表达逻辑键组成及顺序。

下游可以把 indexed nickname 设计为子串匹配，把 birthday、create_time、age 设计为上下界参数。
这些例子解释元数据的用途，不表示本库生成 filter 对象。本库公开直接 indexed 声明，
操作选择、物理索引和数据库匹配由消费者设计。

### 声明验证规则、容器规则与 codec

内置约束适合字段自身的规则；自定义 validator 适合需要业务代码或其他属性的规则。
下面的字段声明可分别用于文本、集合和依赖验证：

```rust,ignore
#[text(non_blank, max_chars = 80)]
label: String,
#[sequence(min_items = 1, unique_items)]
#[element(text(non_blank))]
tags: Vec<String>,
#[validator(id = "profile.matches_region", depends_on(region))]
phone: String,
```

宏把约束、元素作用位置、规则 ID 和依赖写入元数据。`#[element(...)]` 描述元素，
`#[sequence(...)]` 描述容器；二者不能互换。`phone` 的规则要由应用注册同 ID 的 validator，
并在运行时建立验证计划。声明成功并不保证当前执行后端支持每种容器形状，见
[运行时支持范围](../../doc/user_guide.zh_CN.md#限制执行范围与构建拒绝)。

金额、时间和 Map 使用各自的约束，按字段的实际类型声明：

```rust,ignore
#[Model]
struct Invoice {
    #[decimal(precision = 8, scale = 2, min = "0.00")]
    amount: BigDecimal,
    #[time(precision = second)]
    issued_at: DateTime<Utc>,
    #[map(min_entries = 1, max_entries = 3)]
    #[map_key(text(max_chars = 8))]
    labels: HashMap<String, String>,
}
```

片段需引入 `Model`、`bigdecimal::BigDecimal`、`chrono::{DateTime, Utc}` 和
`std::collections::HashMap`。当前验证后端可检查准确的 BigDecimal、受支持时间类型及
Map 外层条目数；`map_key` 的标准约束虽然会被保留，当前执行后端会明确返回
`UnsupportedExecution`。需要它时应换用支持该声明的消费者，而不是假定宏会执行。

`#[opaque]` 用于只处理外层字段、不沿内部模型递归的边界；例如第三方载荷由另一组件验证：

```rust,ignore
#[opaque]
third_party_payload: Payload,
```

它不会删掉这个字段本身的规则。若需要调整序列化名称或保留空集合，可分别使用
`#[serde(rename = "mail")]`、`#[keep_serializing]`；后者只关闭自动省略，显式 Serde 配置优先。

```rust,ignore
#[validator(id = "person.birthday",
    depends_on(gender, birthday(path = "..", property = birthday)),
    params(strict = true))]
#[validator(id = "person.birthday", params(strict = false))]
value: String,
```

以上双 `#[validator]` 是**同一字段的两次独立规则调用**，不是后者覆盖前者。
同 ID 的多个 occurrence 独立保留参数、依赖和声明位置。裸依赖选择当前对象的 Property；
结构化依赖把对象 path 与属性 property 分开。声明不会执行 validator；可选运行时适配器显式绑定
validator registry，并由调用方另行提供父上下文。

text、decimal/money、time、sequence、map 记录声明的约束。
`element(...)`、`map_key(...)`、`map_value(...)` 保留各自作用位置。
Set 已有的唯一性和数组固定长度不能重复声明。opaque 截断内部遍历，但保留外层形状。
旧宏参数 target、on_none、validate_nested 已移除，执行策略由消费者掌握。
运行时的 `FieldAttributeMetadata::ValidateNested` 和 `FieldMetadata::validate_nested()` 也已移除。
嵌套声明由计划构建器按支持矩阵发现，不需要额外标记；reference 和 opaque 仍界定遍历边界。

需要为字段指定编码器时，可以选择稳定 ID；值类型也可声明默认 codec：

```rust,ignore
#[Value(id = "profile.Email", transparent, codec = EmailCodec)]
struct Email(String);

#[Model]
struct Profile {
    #[codec(id = "profile.email.v2")]
    email: Email,
}
```

`EmailCodec` 和 `profile.email.v2` 的实现及注册项由应用提供。上例字段显式选择优先于
`Email` 的默认选择；没有相应注册项或准确类型不匹配，会在 `codec::bind_codecs`
阶段报错，不会在宏展开时自动补齐。

## 能够声明不等于能够执行

宏接受某项声明，只说明模型可以表达该语义。当前 runtime 验证适配器支持具名字段的现有 text 约束
和自定义 validator，并递归处理直接及 Option 嵌套模型。需要穿过可选子对象时，应提供实际可用的
借用 getter，例如 `Option<&Child>`。字段存储类型为 `Vec<T>` 本身不能证明可以读取元素，
显式 element validator 需要借用 slice getter。

含规则的 Enum payload、普通 tuple/newtype 内部、容器元素模型、有可达执行声明的循环，以及缺少集合
适配器、selector 内约束或依赖，都属于当前明确拒绝的执行形状。
需要但缺少解包适配器，或 owned 中间对象无法继续借用时，也在计划构建阶段返回 `UnsupportedExecution`。
透明单字段 tuple `Value` 的内部约束在受支持形状下可以执行；不能把普通 tuple 的限制套用到它。
unit Enum 和没有可达执行声明的循环仍可作为普通值通过。reference 只验证存储字段的显式规则，
opaque 只截断内部遍历，不删除外层声明。

先用 `ValidationCapabilities::check(root, &graph)` 检查访问能力，再把真实自定义注册表传给
`ValidationPlan::build`。两者都保留声明来源；即使缺少规则注册项，错误中也有原始 ID。
运行时指南提供[完整执行矩阵](../../doc/user_guide.zh_CN.md#限制执行范围与构建拒绝)、
[可运行示例](../../doc/user_guide.zh_CN.md#核心工作流从声明到验证报告)、报告上限与部分错误处理，
并说明真实 `rs-platform` 的集成边界。不能为让某个后端接受模型而删除约束或添加 opaque。

生成代码使用 checked `__private::v7`，runtime 与宏 crate 必须同步升级，不保留 v5/v6 兼容门面。
具体字段通过 owner TypeId、variant 序号和字段序号识别；稳定外部命名仍使用 ModelId。
`rs-reflect` 的协议版本独立维护。

### 角色闭包与条件访问器

`Value` 可以包装合法的值或枚举，却不能把 `Entity`、`Model`、引用或原始反射结构
包在内部以绕过关系检查。直接嵌套 `Entity` 也会被结构解析拒绝；若字段保存目标身份，
应明确声明 `#[reference(...)]`。例如：

```rust,ignore
#[Entity(id = "shop.Order")]
struct Order { #[identifier] id: Id }

#[Entity(id = "shop.Line")]
struct Line {
    #[identifier] id: Id,
    #[reference(entity = Order, property = id)]
    order_id: Id,
}
```

上例在 `Line` 中保存的是 `Order` 的 ID，不是内嵌的 `Order` 实例。
建立结构图后，可用 `graph.reference(field.location().unwrap())` 确认目标已解析。

`#[ModelImpl]` 方法若受 `cfg` 控制，只有当前构建启用的方法贡献 Property：

```rust,ignore
#[ModelImpl]
impl Profile {
    #[cfg(feature = "display-name")]
    pub fn display_name(&self) -> String { self.nickname.clone() }
}
```

在未启用该 feature 的构建中，`try_property("display_name")` 返回 `Ok(None)`；
若同一构建启用两个冲突访问器，属性组装会返回错误。

## 出错时按阶段排查

### 声明规则时区分发生阶段

| 写在模型上的内容 | 宏阶段确认 | 后续必须确认 |
| --- | --- | --- |
| `#[reference(...)]` | 属性语法、字段形状和本地限制 | 目标 Entity、目标 Property 与对象路径能否解析 |
| `#[unique(...)]`、`#[key_part(...)]` | 局部声明形式及位置 | 完整作用域、逻辑键在结构图中是否成立 |
| `#[text]`、`#[decimal]`、`#[time]` 等 | 声明能够表达 | 选定验证器是否支持该形状与具体类型 |
| `#[validator(id = ...)]` | ID、依赖语法、参数声明 | 注册表是否有匹配规则，依赖是否可读、父上下文是否提供 |
| `#[codec(...)]` | 明确记录选择 | 实际 codec 注册表是否包含精确类型与 ID |
| `#[redact(...)]` | 本地冲突、与自动序列化能力的关系 | 应用是否按预期启用脱敏输出策略 |

因此编译错误、`StructureResolver` 错误、`ValidationPlan::build` 错误和执行错误应分别处理。尤其不能把计划构建的 `UnsupportedExecution` 解释成声明无效；它只说明当前执行适配器还不能安全完成这条路径。

### 排障顺序

| 现象 | 先检查 | 修复位置 |
| --- | --- | --- |
| 宏报角色或字段形状错误 | struct/Enum 形状、identifier 数量、Value 载荷、lifetime | 模型声明 |
| 报重复或冲突的 trait 实现 | 角色宏与手写 `derive`/impl 的顺序和 `no_*` 开关 | 模型声明与 trait 实现 |
| `try_property` 返回错误 | 同名字段/getter/setter、签名、`cfg`、元数据 ABI | `ModelImpl` 与访问器 |
| ID 查询不到模型 | 稳定 ID、最终链接 crate、注册表快照 | 应用链接与注册范围 |
| 结构图报错 | reference 目标、Value 闭包、Projection 来源、路径导航 | 跨模型关系 |
| 验证计划报 `UnsupportedExecution` | 当前后端的形状矩阵、借用 getter、容器适配器 | 执行适配器或模型访问能力 |
| 自定义规则或 codec 未绑定 | 精确 ID、Rust 类型、显式注册表和参数签名 | 应用注册入口 |

宏生成的 `__private::v7` 是运行时握手层，应用代码应引用公开 API；runtime 与 derive 使用对应版本。诊断时优先保留错误中的 owner、字段身份和声明来源，再根据阶段判断是修改声明、链接范围还是执行注册。

## 方法反射、错误与排查

ModelImpl 转发 reflect_impl 支持的选项。trait 方法和不能充当 Property 的方法仍然保留反射。
泛型运行时适配器沿用 rs-reflect 的显式 `specialize(T = String, ...)` 绑定。
只有公开、安全、同步且方法自身无泛型的固有访问器贡献 Property；借用 getter 保留生命周期，不强制 Send。

本地形状、冗余和能力冲突在编译期报告；跨 crate 目标与完整结构关系由 StructureResolver 检查。
父上下文需求会显式保留，不会因此判定声明非法。使用 `try_*` 查询处理能力和 Property 组装错误，
执行层错误处理见运行时指南。

当前契约见[设计文档](design.zh_CN.md)；历史需求仍见[需求记录](rs-model-derive-requirements.zh_CN.md)。
运行 `cargo doc --workspace --all-features --no-deps` 可生成本地 API 文档。
