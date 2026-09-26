# 模型元数据使用指南

[English](user_guide.md) · [README](../README.zh_CN.md) · [声明指南](../derive/doc/user_guide.zh_CN.md)

## 手册目标与读者

本指南面向框架开发者，适用于 0.1.0 契约。以用户目录为例，先读取模型结构和 indexed 声明，
再按需显式绑定执行服务。项目要求 Rust 1.94、edition 2024。默认 feature 集为空；generic、
validation、codec 分别启用对应 API。

## 概念模型

Field 表示存储槽位，Property 合并存储字段和符合要求的访问器，两者复用 rs-reflect descriptor。
TypeMetadata 是不依赖模型实例的静态信息。Entity 必须声明稳定的 ModelId；其他角色可省略 ModelId，匿名模型仍有准确的 TypeId，
可以作为显式根纳入结构解析。

## 实战场景与最小配置

两个 model-metadata package 都设置了 `publish = false`，需要使用本地检出。
以下路径假设应用 crate 与 `rs-model-metadata` 同属 `rs-platform` 工作区。只有构建验证计划时，
才为运行时 crate 启用 `validation`：

```toml
[dependencies]
qubit-model-metadata = { version = "0.1", path = "../rs-model-metadata", default-features = false }
qubit-model-derive = { version = "0.1", path = "../rs-model-metadata/derive" }
qubit-id = { version = "0.6.0", path = "../../rust-common/rs-id" }
```

`ModelRegistry::from_static_metadata` 只读取显式传入的元数据，查询 Property 时仅使用
`TypeMetadata::local_properties()`，因此看不到独立注册的 `ModelImpl` provider。
需要参与声明处理的匿名模型（包括嵌套子模型）都应加入显式 roots；也可以改用从指定反射快照构建的注册表。
验证器只消费得到的图，不会从进程级注册表补入缺失声明。

```rust
use qubit_id::Id;
use qubit_model_derive::Entity;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::{ResolveInputs, StructureResolver};

#[Entity(id = "guide.directory.User")]
struct User {
    #[identifier]
    id: Id,
    #[indexed]
    nickname: String,
}

fn main() {
    let root = TypeMetadata::of::<User>();
    assert_eq!(root.model_id().expect("Entity ID").as_str(), "guide.directory.User");
    let models = ModelRegistry::from_static_metadata(&[]).unwrap();
    let roots = [root];
    let graph = StructureResolver::new(ResolveInputs { models: &models, roots: &roots })
        .resolve().unwrap();
    let query = graph.query(root.type_id()).unwrap();
    assert_eq!(query.declarations().len(), 2);
    assert_eq!(query.declarations()[1].path().segments(), &["nickname"]);
}
```

查询视图包含 id 和 nickname，因为 identifier 也贡献 indexed 原因。每条 QueryDeclaration 保留来源 Field、
路径、索引原因，并可通过 Field 读取类型。这里不会展开 filter，也不检查平面字段名是否冲突。
未来的 filter crate 可以选择 nickname 子串匹配，以及 age、时间字段的上下界参数；这些是下游策略。

已链接的模型 crate 可以使用 `ModelRegistry::try_global()`，也可以先构造显式 rs-reflect 快照，再通过
`ModelRegistry::from_reflect_registry` 投影。冻结注册表前应完成所需 crate 的链接。
快照注册表的 `properties_for` 能合并独立注册的 `ModelImpl` 访问器。
`from_reflect_registry` 只投影快照 `types()` 中的描述符；仅附加能力的 overlay 仍可通过
rs-reflect 查询，但不会成为模型根。
解析器遍历注册类型与显式根的并集，按 TypeId 去重，检查引用、Projection 来源、Property 冲突和 Value 闭包。
显式注册表不会从全局注册表补入未提供的注册项。启用 `generic` 后，
`ModelRegistry::from_static_metadata_with_generics` 可接收显式泛型定义；快照注册表则收集已注册的
定义 provider。两种注册表都可用 `generic_metadata_for(definition.id())` 按进程内定义身份查询，
`generic_definitions()` 也会列出匿名定义。`entries()` 和其他按 ID 查询的入口只包含具有稳定
`ModelId` 的注册项，匿名泛型定义仍可通过定义身份入口取得。泛型具体 metadata 始终保留定义关联；
并发重复查询同一具体类型会共享 metadata 分配。

验证入口通过传入根的 TypeId 选择图中的元数据。同一 Rust 类型即使有另一份经过检查的 overlay，
也不能借此增加或删除图中的声明。`ValidationPlan::root()` 返回实际采用的图内元数据，
能力检查遵循同一快照边界。

## 进阶用法：字段身份与 API 迁移

具体字段的 `location()` 返回 `Some(FieldLocation)`，由 owner 的 `TypeId`、可选 Enum variant 序号
和字段序号组成；复制 `FieldMetadata` 不会改变身份。泛型定义字段尚未对应具体 Rust 类型，
因此没有具体 location。`DeclarationLocation` 用于追溯声明来源，不是结构图的查询键。
`TypeId` 和 `FieldLocation` 只在当前进程中有效；持久化外部标识时应使用 `ModelId`。

| 旧 API | 当前 API |
| --- | --- |
| `ModelRegistry::from_metadata` | `ModelRegistry::from_static_metadata` |
| `ModelRegistry::from_metadata_with_generics` | `ModelRegistry::from_static_metadata_with_generics`（启用 `generic`） |
| `graph.reference(field)` | 处理 `field.location()` 后调用 `graph.reference(location)` |
| `graph.query(entity_payload)` | `graph.query(entity_type_id)` |
| `graph.projection_source(projection_payload)` | `graph.projection_source(projection_type_id)` |
| 按地址查找节点 | `graph.model(type_id)` |
| `capabilities.supports(position)` | `ValidationCapabilities::check(root, &graph)` |
| 仅查看绑定错误的 `rule()` | `declared_rule_id()` 在注册项缺失时也保留自定义声明 ID |

生成代码采用 checked `__private::v7`。升级时同步更新 runtime、derive 与手写生成协议 fixture；
旧私有协议没有兼容层。应用代码使用上表的公开接口即可，`rs-reflect` 的生成协议版本独立维护。

### 快照拥有的 Property 视图

`TypeMetadata::try_properties[_in]` 现在返回 `ResolvedProperties`。需要读取切片或查找属性时，
先保留这个视图：

```rust,ignore
let resolved = metadata.try_properties_in(&reflection)?;
let properties = resolved.properties();
let title = resolved.property("title");
```

`try_property[_in]` 返回复制后的 `Option<PropertyMetadata>`。fragment 查询返回
`ResolvedPropertyFragments`，其 `fragments()` 切片不能超过视图本身的生命周期。直接调用 `*_in`
会自行持有合并结果，但不会写入全局缓存；重复查询可通过 `ModelRegistry::properties_for` 复用。
缓存归所属 registry 管理，registry 丢弃后即可释放。`ModelGraph` 会在自身生命周期内保留属性视图，
从 `graph.properties()` 借出的引用也不能超过 graph。

旧的 `FieldMetadata::validate_nested()` 方法和 `FieldAttributeMetadata::ValidateNested` 标记已移除；
它们不控制执行。计划构建器负责发现受支持的嵌套声明，遍历边界按文档中的 reference 和 opaque 语义处理，
不再使用递归开关。

## 进阶用法：对象路径、引用与声明位置

ObjectPath 使用 NavigationStep::Property 和 NavigationStep::Parent，显示为 `/` 分隔路径；
PropertyPath 使用 `.` 选择普通属性。依赖的空 ObjectPath 表示当前对象，reference 省略 path 则表示未请求复用。
普通容器不增加领域父对象。

引用绑定路径经过只保存 ID 或 Projection 的字段时，仍导航所绑定的完整 Entity；目标 property 单独选择。
依赖父对象的 reference、validator 声明会保留 ContextRequirement::ParentObject。
`ModelGraph::dependencies()` 保存各自独立的依赖 occurrence。DeclarationLocation 保存文件、行列、
owner 名称、variant/field 序号与 selector 位置，无名 Enum payload 也能定位。

## 进阶用法：显式绑定执行适配器

启用 validation 后，调用 `ValidationPlan::build(root, ValidationBuildInputs { graph: &graph,
validators: &validators })`，传入自己的 validator registry。
绑定检查稳定 ID、参数、可读 Property 路径及已知的输入、依赖类型。预备实例的形状还会与签名逐项核对。
共享的借用参数类型由 `qubit-validation-vocabulary` 提供，metadata 直接重新导出，不转换参数也不分配第二个向量。
静态可判定的 optional getter 路径不能满足必需依赖；deferred 父路径会在上下文提供后检查。同 ID 声明分别绑定，不会互相覆盖。
标准约束使用现有 validation-rules 适配器。

父依赖可通过 `build_with_context` 提供类型 metadata，通过 `validate_with_context` 提供借用实例，
两者都按“最近父对象优先”排列。绑定时也允许暂缺父类型；执行前应把父模型纳入 graph，
执行器才能解析并核对延后的路径后缀。缺少父对象会返回结构化依赖错误，不能当作 Option::None。
即使计划为空，传入错误 Rust 类型的根实例也会被拒绝。

计划只读，不修改对象。ValidationOptions 控制字段选择、快速失败和遍历预算。
当前支持边界内的直接、Option 嵌套模型 validator 自动纳入计划，opaque 截断遍历。
元数据能描述的范围大于某个执行后端。借用切片 getter 支持显式 element validator；生成的集合适配器
还可在受支持的具体类型上执行外层 sequence 去重和 map entry 数量约束。selector 内的标准约束及
MapKey/MapValue 遍历仍会明确返回构建错误。
选择后端前检查 ValidationCapabilities，不应把“能够声明”理解成“所有后端都能执行”。

启用 codec 后，在结构解析完成后使用 CodecBindInputs 与显式 codec registry 调用 `codec::bind_codecs`。
选择顺序为字段显式 codec、Value canonical codec、无 codec。Rust 类型形式也要求相应注册项存在；
显式指定同一个 canonical codec 合法。occurrence 身份包含准确 TypeId、可选 ModelId、Property 路径和来源。

## 核心工作流：从声明到验证报告

给 runtime 依赖启用 `validation`，并为下列**可独立运行的完整程序**添加已发布的执行 API 直接依赖：

```toml
[dependencies]
qubit-model-metadata = { version = "0.1", path = "../rs-model-metadata", features = ["validation"] }
qubit-model-derive = { version = "0.1", path = "../rs-model-metadata/derive" }
qubit-reflect = "0.1.0"
qubit-validator = "0.1.0"
```

下面的用户资料要求 label 与可选联系人的 name 非空白。先发现元数据、解析结构图，再绑定规则。
嵌套执行需要读取借用的中间对象，因此示例显式提供 `Option<&Contact>` getter；
返回 `&Option<Contact>` 的 getter 具有不同的访问形状，不能互相替代。

```rust
use std::num::NonZeroUsize;
use qubit_model_derive::Model;
use qubit_model_derive::ModelImpl;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_model_metadata::validation::ValidationBuildInputs;
use qubit_model_metadata::validation::ValidationCapabilities;
use qubit_model_metadata::validation::ValidationMode;
use qubit_model_metadata::validation::ValidationOptions;
use qubit_model_metadata::validation::ValidationPlan;
use qubit_reflect::ReflectedRef;
use qubit_validator::ExecutionErrorKind;
use qubit_validator::ValidatorRegistry;

#[Model]
struct Contact {
    #[text(non_blank)]
    name: String,
}

#[Model]
struct Profile {
    #[text(non_blank)]
    label: String,
    contact: Option<Contact>,
}

#[ModelImpl]
impl Profile {
    pub fn contact(&self) -> Option<&Contact> { self.contact.as_ref() }
}

fn main() {
    let root = TypeMetadata::of::<Profile>();
    let roots = [root];
    let models = ModelRegistry::try_global().expect("linked metadata and accessors");
    let graph = StructureResolver::new(ResolveInputs { models: &models, roots: &roots })
        .resolve().expect("valid structure");
    ValidationCapabilities::check(root, &graph).expect("supported access shapes");
    let validators = ValidatorRegistry::empty();
    let plan = ValidationPlan::build(root, ValidationBuildInputs {
        graph: &graph,
        validators: &validators,
    }).expect("built-in text constraints bind without custom registrations");

    let invalid = Profile {
        label: String::new(),
        contact: Some(Contact { name: String::new() }),
    };
    let report = plan.validate(ReflectedRef::new(&invalid), &ValidationOptions::default())
        .expect("validation executed");
    assert!(!report.is_valid());
    assert_eq!(report.violations().len(), 2);
    assert_eq!(report.violations()[0].path().render(), "label");
    assert_eq!(report.violations()[1].path().render(), "contact.name");

    let absent = Profile { label: "personal".to_owned(), contact: None };
    assert!(plan.validate(ReflectedRef::new(&absent), &ValidationOptions::default())
        .expect("absent optional child is skipped").is_valid());

    let fail_fast = ValidationOptions::builder().mode(ValidationMode::FailFast).build();
    let report = plan.validate(ReflectedRef::new(&invalid), &fail_fast)
        .expect("policy stop is a successful, truncated execution");
    assert_eq!(report.violations().len(), 1);
    assert!(report.is_truncated());

    let limited = ValidationOptions::builder()
        .max_nodes(NonZeroUsize::new(3).expect("positive budget")).build();
    let error = plan.validate(ReflectedRef::new(&invalid), &limited)
        .expect_err("root, label read, and rule invocation consume all three nodes");
    assert_eq!(error.error().kind(), ExecutionErrorKind::TraversalLimit);
    assert_eq!(error.partial_report().violations().len(), 1);
    assert_eq!(error.partial_report().violations()[0].path().render(), "label");
}
```

普通验证报告保留两条完整字段路径。contact 缺失时跳过其内部规则；FailFast 只保留 label 的首条违规，
不会继续读取 contact。节点预算不足则属于执行错误，已经收集的 label 违规仍在部分报告中。
处理错误时不能把 `Err` 转换为“空报告，所以验证通过”。

示例中的空 registry 只表示没有**自定义 validator**，标准 text 约束由内置适配器绑定。
使用 `#[validator(id = "...")]` 时，调用方必须在传入的 validator registry 中提供对应注册项。
能力检查只检查声明与访问形状，不调用 getter、不绑定自定义注册项，也不能证明实例有效。

## 限制：执行范围与构建拒绝

结构图合法不代表当前验证后端可以执行全部声明。下面的 Enum payload 能保留约束及声明来源，
但构建执行计划时必须明确拒绝，不能返回遗漏了约束的空计划：

```rust
use qubit_model_derive::Enum;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_model_metadata::validation::ValidationBuildErrorKind;
use qubit_model_metadata::validation::ConstraintRuleRef;
use qubit_model_metadata::validation::ValidationBuildInputs;
use qubit_model_metadata::validation::ValidationPlan;
use qubit_validator::ValidatorRegistry;

#[Enum]
enum ContactMethod {
    Named { #[text(non_blank)] name: String },
}

fn main() {
    let root = TypeMetadata::of::<ContactMethod>();
    let roots = [root];
    let models = ModelRegistry::from_static_metadata(&[]).expect("isolated registry");
    let graph = StructureResolver::new(ResolveInputs { models: &models, roots: &roots })
        .resolve().expect("enum declarations are structurally supported");
    let validators = ValidatorRegistry::empty();
    let errors = match ValidationPlan::build(root, ValidationBuildInputs {
        graph: &graph,
        validators: &validators,
    }) {
        Err(errors) => errors,
        Ok(_) => panic!("constrained enum payloads cannot produce an empty successful plan"),
    };
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].kind(), ValidationBuildErrorKind::UnsupportedExecution);
    let location = errors[0].field_location().expect("concrete payload field");
    assert_eq!(location.owner(), root.type_id());
    assert_eq!(location.variant(), Some(0));
    assert_eq!(location.index(), 0);
    assert!(errors[0].constraint().is_some());
    let mapped = errors[0].constraint_rules();
    assert_eq!(mapped[0].id().as_str(), "qubit.rules.text.non_blank");
    match mapped[0] {
        ConstraintRuleRef::Registry(id) => assert_eq!(id.as_str(), "qubit.rules.text.non_blank"),
        ConstraintRuleRef::ModelIntrinsic(_) => panic!("文本规则应使用注册表"),
    }
    assert!(errors[0].source_error().is_none());
}
```

`ValidationBuildError::constraint_rules()` 即使在声明无法执行时，也会按执行顺序列出
已知映射。`Registry(id)` 仍须由相应注册表按输入类型和参数完成绑定；
`ModelIntrinsic(id)` 由模型计划直接执行，不能通过注册表绑定。序列 `unique_items`
属于后一类。原 `constraint_rule_ids()` 方法已删除；如只需读取旧有 ID 字符串，
可使用 `rule.id().as_str()`。

| 声明或访问形状 | 当前验证后端 |
| --- | --- |
| 具名字段的现有 text 约束、自定义 validator | 绑定并执行 |
| 直接或 Option 嵌套的具名模型 | 共用 binder；可选中间对象缺失时跳过；要求真实的借用访问能力 |
| 借用 slice getter 上的显式 element validator，包括嵌套路径 | 绑定并执行，检查准确的元素输入类型 |
| 外层 sequence item count | 需要可读取的借用切片 |
| 外层 `#[sequence(unique_items)]` | 生成的 `Vec<T>` 或 `[T; N]` 配合借用切片 getter 可用，要求 `T: PartialEq + 'static` 及元素相等性适配器。第一处重复报告在 `field[second_index]`，并附 `first_index`。 |
| 外层 `#[map(min_entries = ..., max_entries = ...)]` | 通过生成的可读借用 getter 适配器统计 `HashMap<K, V>` 或 `BTreeMap<K, V>`；违规路径是字段本身。 |
| `#[decimal(...)]` / `#[money(...)]` | 对准确的 `BigDecimal` 值或可选值执行；检查 scale、precision 与精确区间，不舍入。 |
| `#[time(precision = ...)]` | 对 `DateTime<Utc>`、`NaiveDateTime` 或 `NaiveTime` 及其可选值执行；检查秒、毫秒、微秒或纳秒精度。 |
| selector 内的标准约束或依赖，MapKey/MapValue | `UnsupportedExecution` |
| 缺少 map 长度或 sequence 相等性适配器、未知集合形状、不支持的时间类型、标量输入类型不符 | 构建阶段返回 `UnsupportedExecution` |
| 含执行声明的 Enum payload、tuple/newtype、容器元素模型 | `UnsupportedExecution` |
| 有可达执行声明的递归实例路径 | `UnsupportedExecution`，不会无限展开 |
| unit Enum，以及无可达执行声明的 payload 或循环 | 可作为普通值通过 |
| reference 字段 | 只执行存储字段的显式规则，不读取被引用的完整 Entity 实例 |
| opaque 字段 | 只检查外层声明，不穿透内部对象 |
| 返回 owned 中间对象的 getter | 后续需要继续借用时返回 `UnsupportedExecution` |
| Option、智能指针解包 | 必须有实际适配器；切片元素需要隐式解包时明确拒绝 |

同一子类型分别出现在 `primary.name` 和 `secondary.name` 时，各有一份独立 occurrence；
同 ID 的多条规则也不会去重。构建阶段按声明顺序聚合独立错误，不会忽略不支持的声明。
`ValidationCapabilities::check(root, &graph)` 与计划构建共用声明和访问检查。
出现 `RootNotInGraph` 时，应先通过 `ResolveInputs.roots` 或传入的注册表把根模型纳入图中。

不支持的结构路径包含变体名和元组序号，例如 `choice.First.name`、`pair.1.0.name`。
容器内模型路径使用 `[]`、`[key]` 或 `[value]`，例如 `items[].name`；这些标记表示静态声明位置，
不是某个实例中的元素下标。同一模型出现在不同元组位置时分别保留诊断。仅提供反射的包装类型内部，
也会发现已纳入图中的嵌套模型声明。

真实下游 `rs-platform` 的 testkit 覆盖了生产 `CredentialInfo` 的 Option 包装和两个独立使用位置。
`PersonInfo.delete_time` 使用受支持的 `DateTime<Utc>` 形状；计划可检查有值情况，并跳过 `None`。
构建阶段仍会在查看实例前检查声明与具体输入类型。

## 进阶用法：停止条件与执行预算

使用 `ValidationOptions::default()` 获取默认策略。需要定制时，通过
`ValidationOptions::builder()` 设置 `mode`、`selection` 和所需的 `max_*` 预算，
最后调用 `build()`。独立的 `ValidationOptionsBuilder` 持有配置并在构造结束时转移所有权；
原有的 `with_*` 配置方法已移除。

`ValidationOptions` 默认选择 CollectAll、全部字段，深度上限 64、节点上限 100,000、
报告违规上限 100、比较上限 1,000,000。预算设置都要求 `NonZeroUsize`。
字段选择匹配完整的已绑定字段路径，忽略集合索引；例如需要指定 `contact.name` 才能选择该嵌套字段，
只指定 `contact` 不会包含后代。`FieldPath::from_segments` 拥有传入名称，不会再次拆分段内的点号。
使用 `Fields` 选择模型级规则时，需要加入空段序列；普通字段路径不会选中模型级规则。
被选中的模型级规则先于被选中的字段规则执行。选择只影响执行，不能绕过声明或绑定错误。

- FailFast 只保留首条违规，包括失败前置条件产生的违规。单次返回多条违规也不能突破限制。
- 违规数量是整个报告的硬上限，前置失败同样计入。达到上限后，不再调用后续 getter、规则，
  也不继续读取依赖或 selector 元素。
- 策略停止返回 `Ok(report)`，并设置 `is_truncated() == true`。合法 skip 本身不会触发 FailFast。
  报告容量不会限制外部 validator 在返回结果时自行分配的内存。
- 根计一个节点；每次实际属性、依赖、元素读取，以及每次规则调用，各计一个节点。重复读取重复计费。
- 深度按属性和元素路径段计算，依赖导航的 parent hop 也计入；访问发生前检查预算。
- comparisons 统计 selector 元素规则调用次数，以及外层 sequence 去重检查的每一对元素；
  元素读取也消耗节点预算。去重按索引顺序调用 `PartialEq`，最坏需要 O(n²) 次比较，
  每次比较前先检查预算；不测量自定义 validator 内部的比较。
  深度、节点或比较预算不足返回 `TraversalLimit` 和部分报告，计数溢出不会回绕。

Decimal 先规范化数值表示，再依次检查 scale、可选 precision 和精确区间。因此 `1.2300`
符合 scale 2，`1.234` 不符合。precision 按 `DECIMAL(p,s)` 总容量解释：scale 为 2、precision
为 3 时接受 `1.2300`，拒绝 `12`，因为整数最多占 `p-s` 位。这是有意的破坏性变更：旧版
precision 为 1、scale 为 0 时会接受 `1e3`，新版拒绝。元数据中的 `rounding`、`semantic`
供其他消费者理解领域策略；验证阶段不会舍入或改写输入。Time 检查纳秒部分能否被声明单位
整除，不调整日期，也不舍入。旧 `format = email` 和 `TextFormat::Email` 应分别改成
`format = email_ascii` 和 `TextFormat::EmailAscii`；持久规则 ID `qubit.rules.text.email_ascii` 不变。

同时检查 `ModelValidationError::error()` 与 `partial_report()`，结合 root、owner、occurrence、
字段身份和声明来源定位失败操作。依赖的对象导航与属性选择分别有独立 getter；
标准 `Error::source()` 链到 `ExecutionError` 为止。可信诊断代码可通过
`ModelValidationError::error().trusted_source()` 显式读取可用的拥有型原因；普通错误链输出不包含该原因。

## 错误处理与排查

不要提供替换内置规则 ID 的注册项。此类冲突返回根级 `InvalidDeclaration`，
可通过 `rule()` 取得冲突 ID。同一次失败构建仍会报告独立的未支持形状和缺失规则；
应处理所有诊断后重新构建计划。
`MatchesDependency` 比对文本不等时产生 `text.dependency_mismatch`；依赖缺失或不是文本时属于执行错误，
不是值违规。

| 现象 | 检查方向 |
| --- | --- |
| 找不到引用目标 | 稳定 ID、已链接 crate、传入注册表的内容 |
| 能力或 Property 冲突 | fallible 查询的 cause 与原始声明来源 |
| 匿名模型不在图中 | 是否把 TypeMetadata 加入 ResolveInputs.roots |
| 父依赖缺失 | 最近父对象优先的实例上下文与 graph 中的类型信息 |
| selector 无执行支持 | ValidationCapabilities 或其他消费后端 |
| codec 缺失或类型不符 | 注册项、显式引用和准确 value type |

`ModelRegistry::metadata_for` 返回 Result<Option<_>>，Ok(None) 只表示不存在；能力和 ABI 失败保持错误。
`try_properties_in`、`try_property_in`、`property_fragments_in` 保留组装错误。
ResolveErrors 聚合独立问题；同一错误 descriptor 可能分别在可达闭包检查和具体引用路径处报告。
ResolveError 提供 owner Rust 身份、可选稳定 ID、已知声明位置和底层 cause，不应把失败转换成空 metadata。

结构图借用不可变注册表；静态 metadata 不借用 graph 分配或模型实例。
Property 的借用值不能逃出源实例生命周期，也不强制 Send。新生成代码采用 rs-reflect codegen v3 上的
私有 checked v7 门面，应用应使用公开 API。

数据库访问、随机对象创建、物理索引、filter 生成、Unicode 比较执行，以及父对象缺失时的业务回退，
仍由下游组件实现。完整契约见[冻结需求](../derive/doc/rs-model-derive-requirements.zh_CN.md)、
[最终设计](../derive/doc/rs-model-derive-final-design.zh_CN.md)。
运行 `cargo doc --workspace --all-features --no-deps` 可生成本地 API 文档。

## 延伸阅读

- [README](../README.zh_CN.md)
- [English user guide](user_guide.md)
- [派生宏声明指南](../derive/doc/user_guide.zh_CN.md)
