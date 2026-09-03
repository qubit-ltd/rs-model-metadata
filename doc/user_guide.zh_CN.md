# qubit-model-metadata 用户指南

[English](user_guide.md) | [README](../README.zh_CN.md) | 本地 API 文档：`cargo doc --open`

适用于 `qubit-model-metadata` 0.1.x、Rust 1.94 和 edition 2024。

## 手册目标与读者

本指南面向使用 `qubit-model-derive` 声明领域模型的框架和应用开发者。它说明结构反射与领域语义的
边界，并以账户模型为例，展示如何从模型声明走到不可变的解析结果图。本指南对应 model ABI v3，
并使用反射层 `codegen_v2` 协议。

## 概念模型

`qubit-reflect` 负责 Rust 类型的结构信息，`qubit-model-metadata` 在结构之上附加领域语义，
`qubit-model-derive` 根据同一份声明生成这两层信息。

```text
Rust 声明 -> TypeDescriptor -> TypeMetadata -> ModelRegistry -> ResolvedModelGraph
                |                  |
          FieldDescriptor      Field / Property 语义
```

`TypeRef` 可处于 `Resolved`、`Opaque` 或 `Symbolic` 状态。因此，
`FieldMetadata::descriptor()` 和 `PropertyMetadata::descriptor()` 返回 `Option`：
没有具体 descriptor 可能只是 opaque 或 symbolic 类型的正常表示，并非 metadata 缺失。

## 贯穿场景

一个账户服务需要立即检查自身的模型声明，并在应用链接完全部模型 crate 后，解析登录模型的引用。
完成标志有两个：不依赖全局模型注册表即可读取账户的 metadata；完整注册表能够将
`Login.account_id` 解析到 `Account` Entity 的 `id` Property。

## 安装与最小配置

Qubit 模型 crate 目前仅供内部使用且不发布。请从相邻检出目录加入依赖，并按工作区调整路径：

```toml
[dependencies]
qubit-model-metadata = { version = "0.1", path = "../rs-model-metadata" }
qubit-model-derive = { version = "0.1", path = "../rs-model-derive" }
qubit-id = { version = "0.6", path = "../../rust-common/rs-id" }
qubit-validator = { version = "0.1", path = "../../rust-common/rs-validator" }
qubit-codec = { version = "0.14", features = ["registry"] }
```

`ValueCodecRegistry` 只有启用 `qubit-codec` 的 `registry` feature 后才可用。直接导入它并把它传给
`ModelResolver` 的应用 crate 必须保留该 feature。resolver 要求显式传入三个注册表，因此即使当前模型
没有声明 validator 或 codec，运行解析的应用也需要直接依赖 `qubit-validator` 和 `qubit-codec`。
`qubit-id` 则提供 `Entity` 与 `Projection` 标识字段必须使用的 `Id` 类型。

传给 `TypeMetadata::of` 的类型必须使用模型角色派生宏。该宏会生成所需的 metadata provider 与 trait
bound；仅派生结构反射的类型不能满足这一要求。

## 核心工作流

先声明 Entity，再声明引用它的 Model。`#[reference]` 使用稳定的 Entity ID 和公开的 Property 名称，
留待后续解析。

```rust,ignore
use qubit_id::Id;
use qubit_model_derive::Entity;
use qubit_model_derive::Model;

#[Entity(id = "example.Account")]
pub struct Account {
    #[identifier]
    pub id: Id,
    #[unique(ignore_case = true)]
    pub email: String,
}

#[Model(id = "example.Login")]
pub struct Login {
    #[reference(entity_id = "example.Account", property = id)]
    pub account_id: Id,
}
```

先对账户模型做静态查询。`TypeMetadata::of` 不会初始化全局模型注册表；Property 查询会冻结反射快照，
以合并独立生成的 `ModelImpl` capability fragment：

```rust,ignore
use qubit_model_metadata::TypeMetadata;

let account = TypeMetadata::of::<Account>();
assert!(account.field("id").unwrap().is_identifier());
assert!(account.try_property("email").unwrap().unwrap().is_readable());
```

待所有模型 crate 都完成链接后，取得三个显式注册表并执行一次解析：

```rust,ignore
use qubit_codec::ValueCodecRegistry;
use qubit_model_metadata::ModelRegistry;
use qubit_model_metadata::ModelResolver;
use qubit_model_metadata::ResolveInputs;
use qubit_model_metadata::TypeMetadata;
use qubit_validator::ValidatorRegistry;

fn resolve_models() -> Result<(), Box<dyn std::error::Error>> {
    let models = ModelRegistry::try_global()?;
    let validators = ValidatorRegistry::try_global()?;
    let codecs = ValueCodecRegistry::try_global()?;
    let graph = ModelResolver::new(ResolveInputs {
        models,
        validators,
        codecs,
    })
    .resolve_all()?;
    let field = TypeMetadata::of::<Login>().field("account_id").unwrap();
    let reference = graph.reference(field).unwrap();
    assert_eq!(reference.target().model_id().unwrap().as_str(), "example.Account");
    assert_eq!(reference.property().unwrap().name(), "id");
    Ok(())
}
```

成功后会得到不可变的 `ResolvedModelGraph`。resolver 只会在全部关系解析成功后返回完整结果，不会逐步
发布半成品图。

## 进阶用法

### 按阶段选择查询入口

只检查单个类型的静态 metadata 时，使用 `TypeMetadata::of::<T>()`。它会先核对生成结果与唯一的
reflection descriptor；若需要自行处理隐藏 ABI 校验失败，则改用 `TypeMetadata::try_of::<T>()`，避免
直接 panic。

需要按稳定 ID 或精确 Rust `TypeId` 查找时，使用 `ModelRegistry`。`ModelRegistry::try_global()` 会先冻结
`ReflectRegistry`，从中投影每个具体模型 capability 及其权威反射来源，再加入模型层自有的泛型模板注册；
它不会发现未链接的 crate。测试或工具若要控制兼容模型集合，可通过 `ModelRegistry::from_registrations`
显式构建；`from_reflect_registry` 则直接从指定的冻结反射快照构建。

只有完整模型集合已经就绪后才运行 `ModelResolver`。它集中校验跨模型关系与可执行策略绑定；全部成功时
返回一个不可变的 `ResolvedModelGraph`，失败时不会留下可供误用的部分解析图。

### 角色、稳定 ID 与泛型模型

`Entity` 和 `Projection` 必须是非泛型具名 struct，并各有且仅有一个 `qubit_id::Id` 标识字段。
`Entity` 必须声明稳定模型 ID；`Projection` 可以是开放视图，也可以通过 Rust 类型或稳定 ID 固定到某个
Entity。`Model`、`Enum` 和 `Value` 分别表达结构化记录、领域枚举和值对象。声明稳定 ID 后，类型才会
进入注册表；没有 ID 的类型仍可通过 `TypeMetadata::of` 查询，但不能按注册表 ID 查找。

静态字符串使用 `ModelId::try_new` 做可恢复校验，动态输入交给 `ModelIdBuf::parse`。每个点分段必须匹配
`[A-Za-z][A-Za-z0-9_]*`，比较时区分大小写。

### Field、约束与 Property

Field 是反射层提供的真实存储槽位。Property 以一个公开名称组合 Field 与可选的 getter、setter。
getter adapter 会保留借用生命周期；若 setter 在执行前失败，`PropertySetFailure` 会保留 replacement，
调用方仍可继续处理该值。

可先通过 `TypeMetadata::fields`、`field` 或 `field_at` 定位 Field。`FieldMetadata` 把结构事实委托给
reflection 层的 `FieldDescriptor`，自身的强类型 getter 则提供 identifier、unique、reference、key part、
validator、codec、redaction 与 Serde 语义。只关注某一类约束时，可调用 `text_constraint`、
`decimal_constraint`、`time_constraint`、`sequence_constraint` 或 `map_constraint`；需要保留全部声明时，
遍历 `constraints()`。使用 `descriptor()` 前应先检查 `type_ref()`，不要假定所有类型引用都有具体 descriptor。

带有模型 ID 的泛型声明只注册一个 `GenericModelMetadata` 模板。具体单态类型没有 `ModelId`，也不会
进入注册表；可通过 `generic_definition()` 回到该模板。模板字段可以使用 symbolic 类型，而具体实例的
字段类型可以是已解析的。

### 解析结果与查询 metadata

跨模型结果统一从解析图读取：字段引用使用 `reference`，固定 Projection 使用 `projection_source`，
Entity 上可读且能生成 Projection 的 Property 使用 `projection_producers`。若 producer 带有可执行 getter，
可调用 `project`；运行时会确认来源 Entity 与生成的 Projection 保持相同 `Id`。

`properties(model)` 返回 resolver 接受的 Field/getter/setter 合并结果。对于 Entity，`query(entity)`
提供由索引字段生成的 filter，以及由标识字段和 unique 声明生成的唯一查询键。不同路径若展平为同一个
查询名，整次解析会失败。

validator occurrence 和 codec 声明在解析前都只是描述信息。`validator` 返回匹配的可执行注册项及其
可读依赖 Property；`codec` 返回可执行 codec descriptor，按 ID 声明时还会给出对应注册项。

## 错误与诊断

`ModelId::try_new` 和 `ModelIdBuf::parse` 在 ID 为空、点分段为空或分段不符合 ASCII 标识符规则时返回
`ModelIdError`。`ModelRegistry::try_global()` 会以 `ModelRegistryError` 报告重复模型 ID、注册冲突或
reflection registry 初始化失败。

`resolve_all()` 返回 `ModelResolveErrors`，按确定顺序汇总问题，不会发布带未解析关系的图。处理错误时应
遍历 `errors()` 并匹配 `ModelResolveError::kind()`，不要解析展示文本。错误类型覆盖本地 Property 合并、
Entity 嵌套、opaque 模型、引用、角色与类型、Projection 契约、validator/codec 绑定、selector 类型、
Value 闭包以及查询名冲突。按错误场景，还可读取模型 ID、Property 路径、预期与实际角色或类型、来源片段。

生成的 metadata 在发布前还会检查 model ABI v3 不变量，并使用反射层 `codegen_v2` 协议。若 panic 信息以
`QMM-ABI-` 开头，说明生成代码或手写的
隐藏 ABI metadata 违反了相应不变量，已被拒绝。

## 排障

- `TypeMetadata::of::<T>()` 无法编译时，确认 `T` 使用了模型角色宏，并满足生成的 trait bound。
- Entity 或 Projection 的标识字段被拒绝时，确认类型精确为 `qubit_id::Id`；整数基础类型和应用自定义
  ID wrapper 都不满足角色契约。
- `descriptor()` 返回 `None` 时，先检查 `type_ref()`；opaque 和 symbolic 引用本来就没有具体 descriptor。
- `ModelRegistry::try_global()` 中缺少预期模型时，确认声明带有稳定模型 ID，且所属 crate 确实链接进了
  最终二进制；注册收集无法越过静态链接边界。
- `resolve_all()` 返回错误时，逐项检查 `ModelResolveError`，修正稳定 ID、目标角色、Property 名称或对应的
  validator/codec 注册，然后重新执行完整解析。
- 成功解析的 validator 会提供强类型注册项和可读的依赖 Property；成功解析的 codec 会提供可执行 descriptor，
  对于 ID 声明还会提供匹配的注册项。

## 限制与最佳实践

稳定链接的声明使用 `ModelId`，动态输入先交给 `ModelIdBuf::parse` 解析；不要把 Rust 诊断类型名用作持久化
ID。直接调用 `TypeMetadata::of` 的静态查询应与全局模型注册表初始化分离，只有在完整模型集合都链接后才
进行解析；descriptor capability 与 Property 查询则有意共享冻结的反射注册表。本 crate 不提供另一套
反射系统，也不会在静态查询时隐式绑定跨模型引用。

最终依赖图中只保留一个 `qubit-model-metadata` 版本；不同版本拥有彼此独立的注册清单，会把模型集合拆开。
模型 ID 应视为应用协议：改名时要有意识地同步更新所有文本引用。库代码和诊断工具优先使用可恢复的
`try_*` 入口；只有在应用启动阶段确认配置错误无法恢复时，才使用会 panic 的 `of` 或 `global` 入口。

## 延伸阅读

- [README](../README.zh_CN.md)
- [English user guide](user_guide.md)
- [`qubit-model-derive` 声明指南](https://github.com/qubit-ltd/rs-model-derive/blob/main/doc/user_guide.zh_CN.md)
- 本地 API 文档：运行 `cargo doc --open`
