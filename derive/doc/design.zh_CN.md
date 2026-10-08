# rs-model-metadata-derive 设计文档

[English](design.md) · [用户手册](user_guide.zh_CN.md) · [运行时设计](../../doc/design.zh_CN.md)

本文对应当前 `qubit-model-derive` 0.2.0 过程宏实现（Rust 1.94、edition 2024）。生成结果由 `qubit-model-metadata` 0.2.0 与 `qubit-reflect` 0.2 消费。当前 manifest 设置了 `publish = false`。

## 职责与编译流水线

公开属性宏包括 `Entity`、`Projection`、`Model`、`Enum`、`Value` 和 `ModelImpl`。前五者声明类型角色，`ModelImpl` 从 impl 块添加属性访问器。入口将每个宏送入共同流水线：解析 Rust 和属性、规范化声明、检查局部语义与冲突、展开反射和模型能力。源码由 `parse`、`ir`、`normalize`、`validate`、`compiler`、`expand` 分担。非法的局部声明直接形成编译诊断，不生成不完整元数据。

宏输出静态 `TypeMetadata`、字段与变体声明、反射能力和注册片段。具体类型的元数据从冻结反射快照投影；泛型模型定义保留模型层自己的注册片段。生成代码通过 `__private::v7` 调用受检运行时构造器，因此 derive 与运行时 ABI 必须同步。

## 角色与身份

`Entity` 必须是非泛型具名结构体，恰有一个 `#[identifier]` 字段；`Projection` 同样必须恰有一个此类字段。两者标识字段的实际类型都必须是 `qubit_id::Id`；该类型的别名可用，因为别名不创建新的 Rust 类型，newtype 包装和容器类型不可用。宏生成的代码通过私有 sealed `IdentifierType` 契约在编译期检查此要求，下游不能自行实现该契约来扩展允许的类型。`Projection` 还可以指定固定来源或保持开放。`Model` 表示结构化数据，`Enum` 表示领域枚举；`Value` 的值闭包不能藏入实体、投影、模型或引用语义。需要稳定身份时通过 `id = "..."` 声明。具体 Rust 类型另有 `TypeId`；它与稳定模型 ID 的用途、有效期不同。

角色宏按文档生成反射、Clone、比较、脱敏格式化及 Serde 等默认 Rust 能力，并受相应 trait bound 限制。`Value` 与 `Enum` 默认还生成 Eq、Hash。显式能力开关改变生成的 Rust 行为，但仍保留模型元数据；无法满足的 trait bound 由 Rust 编译器报告。枚举 payload 的元数据保留所属变体，泛型声明则需结合具体实例检查。

## 字段、属性与声明

字段属性记录标识符、逻辑键、索引、唯一性、引用、约束、validator、selector、codec、脱敏与 Serde 行为。每个具体字段都有 `FieldLocation`，声明位置和顺序也会保留供诊断使用。`ModelImpl` 反射方法，并把符合条件的 getter/setter 与普通字段组合。只有 getter 的方法默认成为计算属性；`#[model_property(skip)]` 可排除其属性贡献。`cfg` 等存在条件同时作用于 impl 反射和访问器适配，因此被禁用的方法不会留下访问器。

引用对象路径使用 `/` 与 `..`，属性路径使用 `.`。宏检查局部语法和类型形状；跨模型目标、结构关系、validator 注册及 codec 执行由后续 registry、resolver 或显式执行绑定器检查。能声明不代表当前执行器支持该形状。为集合内部或 selector 附加可执行约束前，请核对[运行时支持矩阵](../../doc/user_guide.zh_CN.md#限制执行范围与构建拒绝)。

## 错误阶段与验证

局部属性冲突和不支持的角色形状会导致编译失败。Rust trait 或适配器 bound 在使用位置报错。Registry 初始化检查生成 ABI 和身份，`StructureResolver` 检查关系，可选校验/codec 计划绑定执行行为。宏测试覆盖元数据运行断言、编译失败 UI 样例、重命名运行时依赖和跨 crate 场景。声明示例见[用户手册](user_guide.zh_CN.md)，快照及执行所有权见[运行时设计](../../doc/design.zh_CN.md)。

## 输入契约与角色选择

过程宏作用于编译期的 Rust 语法树，不读取数据库或运行时对象。调用方应先按领域语义选角色，再决定要公开哪些 Rust 能力：

| 宏 | 接受的主要声明 | 关键约束 | 生成结果的主要用途 |
| --- | --- | --- | --- |
| `Entity` | 非泛型具名 struct | 必须有稳定 `id`；恰有一个满足 `IdentifierType` 的 `#[identifier]` | 持久实体身份、结构与引用目标 |
| `Projection` | 非泛型具名 struct | 恰有一个 identifier；可 `open`，也可通过 `source` 或 `source_id` 指定唯一固定来源 | 实体视图及来源关系 |
| `Model` | 具名或 unit struct，可泛型 | 不接受 tuple struct | 普通结构化模型、可选逻辑键 |
| `Enum` | enum，可有 unit、具名或 tuple payload，可泛型 | 变体 canonical 名需唯一 | 领域枚举与逐变体元数据 |
| `Value` | 非空具名 struct 或单字段 tuple struct | `transparent` 仅在恰有一个字段时合法；值闭包不得隐藏实体等角色 | 值对象及可选 canonical codec |
| `ModelImpl` | impl 块 | 仅符合条件的固有方法贡献 Property；trait impl 仍可反射方法 | getter/setter 属性适配与方法反射 |

模型角色不支持 lifetime 参数。允许的 const 泛型仅限基本整数、`bool`、`char`；泛型定义的运行时能力由 `generic` feature 守护。`Entity` 和 `Projection` 不能是泛型，也不能携带泛型 where 条件。宏对输入形状先报本地错误，避免后续生成代码因不相关问题产生大量连锁诊断。

`Entity` 的 `id` 是稳定外部名称，与字段里的 `#[identifier]` 值不是同一概念：前者标识模型类型，后者标识某个实体实例。`Projection` 可以开放，也可以固定来源；`open` 不能与 `source`、`source_id` 同时使用，后两者也互斥。其他角色可匿名，因此宏生成 `TypeMetadata` 不等于全局 registry 一定有可按字符串 ID 查找的条目。

## 编译流水线及失败位置

`derive/src/lib.rs` 的六个公开入口统一转到 `entry::expand`。对五种类型角色，`expand::pipeline::run` 依次进行：

1. 使用 `syn` 解析宏参数和被标注的 `DeriveInput`，先检查角色与 Rust 形状；同时解析运行时 crate 的实际依赖名称。运行时依赖缺失时产生明确诊断，重命名后的依赖路径仍可找到。
2. 拒绝与宏将生成的 Reflect 实现重复的显式 derive，解析字段、变体、Serde、角色参数和声明位置，形成内部 IR。具名与位置字段保留索引，Enum payload 保留所属 variant。
3. 规范化声明：将简写整理成一致的 IR，补充符合条件的 Option/集合缺失默认值与空值省略语义，记录索引原因、依赖路径和 selector 的位置。
4. 校验 IR：角色选项、identifier 数量、variant 名、字段属性冲突、逻辑键顺序、类型形状和约束组合等。独立局部错误通过 `syn::Error::combine` 收集后报告。
5. 准备默认输出能力，把模型字段 helper 改写为 Reflect/Serde/Redact 可消费的属性，然后给类型加入反射 derive、`definition_provider_v2`（泛型时）和 `__private::v7::model_capability`。
6. 输出原类型、静态 metadata provider、注册片段和 trait/输出实现。泛型输出包在运行时的 feature 门面中；若没有启用对应 feature，编译时明确拒绝。

`ModelImpl` 走独立分支：先解析 `ItemImpl`，保留原 impl 及 `reflect_impl` 能力，扫描方法并按目标类型生成访问器适配器和 provider。宏剥离 `#[model_property(skip)]` 这一仅用于属性选择的标记；被排除的方法仍属于普通方法反射。局部非法声明、缺失依赖、trait bound 和跨模型关系分别在不同阶段报告，调用方不应期待一次宏展开完成全项目验证。

## 生成能力与输出边界

五种角色默认拥有 Clone、PartialEq、Redact、Debug、Display、Serialize、Deserialize；`Value`、`Enum` 默认另有 Eq、Hash。仅全 unit 变体的 Enum 默认 Copy。`Entity`、`Projection`、`Model` 需要显式 `eq, hash` 才获得对应结构能力；`ord` 会要求 Eq，却不会自动打开 Hash。`default`、`partial_ord`、`ord` 等为显式开关，`no_*` 可关闭对应自动实现。冲突开关在宏阶段拒绝，字段或泛型参数不满足 trait bound 时由 Rust 类型检查精确定位。

宏先识别用户可见的显式 derives，以免重复生成同一 trait。默认 Debug、Display 和 Serialize 经 `qubit-redact` 输出受控视图；Deserialize 仍遵循输入方向的 Serde 合同。`no_redact` 与字段或 selector 的脱敏规则冲突；自定义输出 derive 也不能绕开仍启用的本地脱敏规则。`#[redact(skip)]` 作用于整个字段，map key 脱敏后若发生键碰撞，应由序列化返回错误，而不是静默覆盖数据。

宏会给符合条件的具名 Option 和标准集合字段生成缺失默认值及空值省略。显式 Serde 配置优先，`keep_serializing` 只关闭隐式省略；位置字段不套用具名字段的默认策略。Enum 的 Rust 变体名、canonical 名和 Serde wire 名分别保存；无类型级 `rename_all` 时默认 wire 名按 canonical 名设置，变体级显式 rename 优先。

## 字段元数据与位置保证

字段声明在源顺序上保留 occurrence，而不是把相同 ID 的规则合并成一项。每个具体字段可获得 `FieldLocation`；每个属性、约束、引用、validator 和 selector 还记录足以追踪回声明位置的信息。`#[identifier]`、`#[unique]`、`#[reference]` 会形成隐含 indexed 原因，因此同字段再写冗余 `#[indexed]` 会在编译期报错。具名 `Model`、`Value` 的 `#[key_part(order = n)]` 表达逻辑键顺序，不等于数据库主键自动生成。

引用声明只保存目标 ID/类型、目标 Property 和对象导航信息。对象路径使用 `/`、`..`，属性路径使用点号；两者不能混成同一字符串。宏能检查语法和本地冲突，却无法在当前 crate 编译时确认另一个 crate 的模型一定被最终二进制链接。引用目标是否存在、类型和 Property 是否兼容，必须由同一快照中的结构解析器检查。

约束、validator、selector 与 codec 均先形成声明。`#[validator(id = "...")]` 可以重复出现，每个 occurrence 独立保留参数、依赖和位置。codec 可声明稳定 ID 或 Rust 类型；字段显式 codec 与 Value canonical codec 的选择在运行时绑定阶段决定。宏不调用规则、编码器、getter，也不验证某个对象实例。

## ModelImpl 的属性判定

`ModelImpl` 先把整个 impl 交给 `reflect_impl`，再决定哪些方法能贡献属性。贡献者必须是公开、同步、安全、非方法泛型的固有方法；trait 方法和其他不符合访问器合同的方法可被反射，却不进入 Property 集合。可用的 getter 与 setter 按属性名合并到存储字段；没有字段的 getter 成为计算属性。启用时冲突的访问器会诊断，不能依赖源顺序任意选择其中一个。

访问器的 `cfg` 与嵌套 `cfg_attr` 条件同时附在 provider、签名和适配器上。这保证条件编译关闭的方法既不会参与属性组装，也不会留下引用不存在类型的适配器。泛型 impl 的具体专门化遵循 `rs-reflect` 的显式 `specialize(...)` 能力，不推断任意泛型实例。Getter 返回值的借用形状直接影响下游是否能继续遍历、执行约束；只生成 Property 元数据并不保证所有消费者都可执行它。

## 跨组件验收与迁移

开发或升级宏时，应分别确认：合法声明的 Rust 能力与元数据、非法声明的编译诊断、跨 crate 与运行时重命名、独立 `ModelImpl` provider、泛型/匿名模型、真实平台领域声明，以及运行时 v7 ABI。`derive/tests` 的 UI 与运行 fixture、根 crate 的元数据测试、`rs-platform` 的实际模型消费者分别覆盖这些边界。本文解释当前实现；可运行声明教程和已知执行限制见[用户手册](user_guide.zh_CN.md)。
