# rs-model-metadata 设计文档

[English](design.md) · [用户手册](user_guide.zh_CN.md) · [派生宏设计](../derive/doc/design.zh_CN.md)

本文对应 `qubit-model-metadata` 0.2.0（Rust 1.94、edition 2024）。`qubit-reflect` 描述 Rust 结构，`qubit-model-derive` 为领域声明增加业务语义，本运行时让其他库读取这些语义并解析其关系。持久化和应用策略由消费者决定。

## 领域语义与消费者

字段标注表达结构反射无法推断的事实：`text`、`decimal` 的限制，`unique` 的作用域，`reference` 的目标，`indexed` 的可查询性，以及自定义 `validator` 的绑定。普通整数和浮点数的范围检查可通过显式 validator 声明，并在运行时注册和绑定后执行；支持的值类型取决于 validator 的输入契约。规则与领域类型写在一起，校验、测试数据生成和查询层便可共用一份声明。

| 消费者 | 读取的元数据 | 仍需自行决定的工作 |
| --- | --- | --- |
| 合法性校验 | 字段约束、唯一性作用域、validator、依赖与访问路径 | 检查实例，以及执行数据库唯一性查询等外部操作 |
| 满足约束的测试数据生成 | 长度、范围、唯一性和引用关系 | 生成值、预留唯一值、选择已存在的关联记录 |
| REST 查询层 | `ModelGraph` 中的 `indexed` 等查询声明 | 校验过滤条件、生成 SQL 并执行查询 |

可选的 `validation` feature 可将受支持的声明绑定为 `ValidationPlan`。随机对象生成和 REST 查询执行可以消费这份元数据契约，但不是 `ModelGraph` 自身的行为。

启用 `codec` 后，绑定只处理具体 descriptor 可解析字段上的声明；descriptor 缺失时会跳过该字段的整个阶段，包括显式 codec 与 selector。`Ok` 表示已处理声明没有绑定错误，不代表检查了所有源声明。消费者应检查预期的 `CodecOccurrenceId`，并决定缺失时启动失败还是回退。此边界独立于验证支持矩阵。opaque 集合只是 descriptor 可能不可用的一个例子，并非所有 opaque 字段都会跳过。详见[运行时用户指南](user_guide.zh_CN.md#codec-绑定)。

## 边界与数据流

```text
Rust 声明 --derive--> TypeMetadata + 反射能力
                         |
ReflectionContext 快照 --> ModelRegistry --> StructureResolver --> ModelGraph
                               |                   |
                       PropertyAccessPath    可选的校验/codec 绑定
```

`TypeMetadata` 无需实例即可描述一个 Rust 类型的角色、字段、声明和本地属性。`FieldMetadata` 对应存储字段；`PropertyMetadata` 组合字段与符合条件的 `ModelImpl` 访问器，也可表示计算属性。读取属性时，能借用实例就保留借用。`FieldLocation` 用所属 `TypeId`、可选枚举变体和字段序号定位具体字段；泛型定义的字段尚无具体位置。外部持久身份使用 `ModelId`，`TypeId` 与 `FieldLocation` 仅在进程内有效。

反射边界采用 `rs-reflect` codegen v4：运行时和生成代码通过
`DynamicRef<'a, Local>`、`DynamicMut<'a, Local>`、`DynamicOwned<Local>`
进行受检动态访问，并用 `#[ReflectImpl]` 注册生成的能力。`__private::v8`
是另一个独立版本的 runtime 与 derive 内部 ABI，不是应用 API。枚举 payload 执行前会调用反射
`VariantDescriptor::is_active_dynamic`；动态根类型不匹配会作为执行错误返回。

公开功能分别放在 `metadata`、`registry`、`resolve`、`access_path`；`generic`、`validation`、`codec` 是独立可选 feature，默认均不启用。稳定 ID 协议还可单独通过 `qubit-model-id` 使用。

## 快照与模型图

`ModelRegistry::try_global()` 对已链接的反射注册进行一次冻结，并保留初始化错误。需要隔离时，可从指定 `ReflectionContext` 快照或静态元数据构造 registry。Registry 同时按稳定 ID 和精确 Rust 类型建立索引；匿名根由解析器接收，不伪造 ID。属性、元数据缓存属于各自的 registry。属性路径缓存合计最多保留 256 条成功的读写路径；失败的编译下次会重试。调用方已经持有的编译路径在缓存逐出后仍可使用。

下面的完整程序创建显式反射上下文，将其投影到模型注册表，并查询已注册模型：

<!-- example: core -->
```toml
[dependencies]
qubit-model-metadata = { version = "0.2", path = "../rs-model-metadata", default-features = false }
qubit-model-derive = { version = "0.2", path = "../rs-model-metadata/derive" }
qubit-id = "0.7"
qubit-reflect = { version = "0.2", path = "../rs-reflect" }
```

<!-- example: core/design-context -->
```rust
use qubit_id::Id;
use qubit_model_derive::Entity;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_reflect::ReflectionContext;

#[Entity(id = "docs.design.ContextRegistered")]
struct ContextRegistered {
    #[identifier]
    id: Id,
}

fn main() {
    let reflection = ReflectionContext::builder()
        .include_linked()
        .build()
        .expect("链接的反射上下文");
    let models = ModelRegistry::from_context(&reflection).expect("上下文模型注册表");
    assert!(models.get("docs.design.ContextRegistered").is_some());
}
```

`StructureResolver::new` 审计全部已注册模型；`for_roots` 与 `for_static_roots` 只解析指定根及其可达模型。解析时检查角色、引用、Projection 来源、Value 闭包、唯一性作用域和查询声明。`ModelGraph` 借用选定的 registry，以 `TypeId` 或 `FieldLocation` 查询关系。根生成的 nested metadata provider 在能提供元数据时可发现可达子模型；启用 `generic` 后，这包括具体化的嵌套泛型子模型。仅有非泛型子字段并不会使其模型元数据自动可用，绑定 context 需要注册该子模型的 provider。registry 索引、显式根、生成的 provider、context provider 和反射事实各有职责；解析 descriptor 仍需 context 中的反射事实。模型图不会从全局 registry 补入缺失声明。

Opaque 字段停止内部遍历，解析器仍会查询字段所声明具体类型的元数据。`Ok(None)` 表示允许缺少模型；能力或 ABI 查询错误会形成带字段上下文且保留原始 cause 的 `MetadataResolution` 诊断。ReflectionContext 初始化错误保留 Reflection cause。因此此前被忽略的错误 provider 现在可能使解析失败，解析器调用签名保持不变。

所有解析模式中，同一具体类型的显式根优先于注册表元数据；重复根时最后一个根胜出。每个根都会进行 ABI 校验，包括被覆盖的根。模式只改变发现范围：`new` 扫描所有注册类型，根作用域构造器遍历可达类型。例如，若注册表中 `User` 的元数据为 A，而根为同一 Rust 类型提供元数据 B，旧版 `AllRegistered` 会保留 A，现在解析器选择 B。若调用方需要 A 的声明，应省去不需要的 B 根，或传入 A。`ModelId` 仍先由注册表定位类型，再为该类型选择显式 overlay；overlay 不会注册别名。注册表 provenance 仍表示注册来源，声明位置则来自选中的 overlay。注册表索引保持不变。

## 声明与执行

核心 crate 保存约束、validator、selector、codec 引用，以及对象路径和属性路径。对象路径使用 `/` 与 `..`，属性路径使用 `.`。查询元数据提供声明及原因，不替应用生成任意筛选器。`PropertyAccessPath` 先编译结构路径，再对实例读写；读写支持范围不同。结构性写入错误会在调用末级 setter 前报告。授权、持久化及查询执行由应用负责。

启用 `validation` 后，`ValidationCapabilities::check` 检查根模型声明能否执行。`ValidationPlan::build` 使用调用方提供的 `ValidatorRegistry` 和 `Arc<ModelGraph>` 绑定受支持的声明；不支持的形状或缺少规则会形成聚合的构建错误。`validate` 将规则违例作为报告返回；执行失败返回带部分报告的 `ModelValidationError`。`ValidationOptions` 控制字段选择、停止策略和预算。构建后，`with_model_rules` 会消费计划，并在追加前逐项检查新绑定是否接受计划根类型，返回 `Result<Self, BindError>`。首个错配会返回带规则 ID 的 `PreparedSignatureMismatch`，不会调用 validator 或 getter；调用失败后原计划已被消费。可选 codec 绑定也需要显式 registry。具体操作和执行边界见[用户手册](user_guide.zh_CN.md)。

## 错误与兼容契约

宏语法错误在编译时诊断。受检元数据构造与 registry 初始化报告 ABI 或注册错误；模型图报告结构错误；执行适配器报告绑定或运行错误。查无对象与查找失败是不同结果。生成代码使用同步的 `__private::v8` ABI 和反射 provider 契约，因此运行时与 derive 版本应保持一致。持久化时保存 `ModelId`，不要保存进程内的 `TypeId` 或 `FieldLocation`。

平台中的生产接入见 [rs-platform 设计文档](../../rs-platform/doc/design.zh_CN.md)。历史需求及迁移台账可供追溯，当前行为以本设计和代码为准。

## 核心对象与所有权

下面的对象分别回答不同问题，不能用其中一种查询结果代替另一种：

| 对象 | 解决的问题 | 身份与生命周期 |
| --- | --- | --- |
| `TypeMetadata` | 一个具体 Rust 类型是什么角色，有哪些字段、声明和本地属性 | 静态元数据；通过 `TypeId` 对应具体类型，`model_id()` 可以为空 |
| `FieldMetadata` | 类型实际存储了哪个字段，字段上声明了什么 | 具名、位置和 Enum payload 字段均可表示；具体字段可取 `FieldLocation` |
| `PropertyMetadata` | 某属性能否从实例读取或写入，访问器如何与存储字段组合 | 可对应字段，也可来自只读计算 getter；查询可能因组装冲突失败 |
| `ModelRegistry` | 某个快照中哪些模型拥有稳定 ID，如何按 ID 或类型查找 | 借用反射快照来源；索引和缓存属于该 registry |
| `ModelGraph` | 注册表中的跨类型关系能否成立，引用、Projection、查询声明指向何处 | 借用构建它的 registry；持有解析好的属性视图 |
| `ValidationPlan` | 已解析的声明中哪些规则可执行，如何以确定顺序执行 | 启用 `validation` 后构建；持有 `Arc<ModelGraph>` 和绑定结果 |

`TypeMetadata::of::<T>()` 适合已知、受检的生成类型；需要处理 ABI 故障时使用 `try_of::<T>()`。`fields()` 和 `field(name)`读取声明，不会访问实例。`try_properties()` 返回拥有本次合并结果的 `ResolvedProperties`，调用方应保留该视图，再借用其中的属性切片。`try_property(name)` 则返回复制后的单项属性。属性组装失败应保留 `PropertyResolutionError`，不能解释成“属性不存在”。

同一字段的结构身份由 `FieldLocation { owner TypeId, variant, index }` 表示；`DeclarationLocation` 另记声明来源和位置，供错误诊断使用。两者均不适合写入数据库或跨进程传输。外部协议使用稳定的 `ModelId`；匿名模型仍能作为显式根参与解析，但不会凭空获得稳定 ID。

## 注册表构建与快照隔离

有三种主要入口，按调用方掌握的范围选用：

1. `ModelRegistry::try_global()` 从进程已链接的反射注册构造全局实例。初始化结果通过一次性单元保存；注册冲突或 ABI 错误不会在后续调用中偷偷变成成功。
2. `ModelRegistry::from_context(&reflection)` 投影调用方提供的 `ReflectionContext` 快照。它保留来源信息，合并上下文中的 `ModelImpl` provider，并能在可达遍历中发现匿名子模型；不会查询全局上下文补缺。
3. `ModelRegistry::from_static_metadata(...)` 用明确给出的元数据和来源建立索引。传入的 `ReflectionContext` 仍绑定到注册表，用于 capability 和属性解析，因此该 context 中注册的 provider 仍可见；若 context 没有匹配的属性 provider，则返回类型的本地属性。显式根生成的 nested metadata provider 可以发现子模型，而不把它加入 registry 索引；例如启用 `generic` 后，可发现具体化的嵌套泛型子模型。仅将非泛型子类型加入反射 context 不会提供其模型元数据；若要发现它，应在 context 中注册相应 metadata provider。绑定 context 仍须提供解析子模型 descriptor 所需的反射事实。

注册表按稳定 ID、精确 `TypeId` 建索引。`get(id)` 返回注册条目，`metadata(id)` 返回具体类型元数据；启用 `generic` 后，`generic(id)` 与 `generic_metadata_for(definition_id)` 处理泛型定义。匿名定义可以按进程内定义身份查到，却不进入稳定 ID 列表。构建阶段会拒绝重复 ID、互相矛盾的具体类型注册、能力目标不属于快照、能力适配器类型不匹配和元数据与反射 descriptor 不一致等情形。

缓存也遵守快照边界。属性和元数据结果由所属 registry 的惰性单元保存；路径缓存按根元数据、路径段及读写模式区分，最多保留 256 条成功编译结果。失败不入缓存，下次请求可以重试；已交给调用方的 `Arc<PropertyAccessPath>` 不会因缓存逐出而失效。应用若建立 overlay 或不同反射快照，应使用对应 registry 编译路径，不能把另一快照的结果当成相同结构。

## 结构解析的范围和步骤

`ResolveInputs` 指定一个 registry 和一组显式根。`StructureResolver::new` 从所有已注册具体模型与显式根出发，适合启动审计；`for_roots` 只处理给定根的可达子图，适合单次业务操作；`for_static_roots` 拥有传入的根数组，便于把生成图保存在长生命周期的计划里。

解析器以精确类型身份收集节点，组装同一快照中的属性，再检查角色和关系：Entity 嵌套规则、Value 闭包、Reference 目标与目标 Property、Projection 来源、唯一性作用域以及依赖路径。错误以 `ResolveErrors` 聚合，尽可能保留源字段和声明位置。结构图合法只说明关系成立，不说明某个 validator 或 codec 已注册，也不说明实例的值有效。

`ModelGraph` 提供以下方向明确的查询：

| 查询 | 参数 | 结果用途 |
| --- | --- | --- |
| `model` | 具体 `TypeId` | 找到图内节点；查不到返回 `None` |
| `properties` | `TypeMetadata` | 取得图内已组装的属性视图 |
| `reference` | 具体字段的 `FieldLocation` | 找到目标模型、目标 Property 与上下文需求 |
| `projection_source` | Projection 的 `TypeId` | 找到固定来源关系 |
| `query` | Entity 的 `TypeId` | 查看 `QueryDeclaration`、路径与 indexed 原因 |
| `dependencies` | 无 | 查看各声明独立保留的依赖 occurrence |

查询视图列出 identifier、indexed、global unique、reference 等原因；它不生成 SQL、筛选器、物理索引，也不把无限可达路径预先展开。`ObjectPath` 负责对象绑定导航，使用 `/` 和 `..`；`PropertyPath` 负责属性选择，使用点号。路径声明与某次实例访问是两件事：缺少父对象上下文时，图可记录 `ContextRequirement`，后续执行必须由调用方提供实际父实例。

## 属性访问与写入边界

`PropertyAccessPath` 先针对给定根元数据和路径进行结构编译，之后才读取或写入实例。编译读路径不能推出写路径也合法。读取中间 `Option<Model>` 时，缺失可作为缺失值处理；写入中间节点需要可变、可遍历的存储路径。应用若用动态属性编辑界面，应分别编译读写路径，并在调用前检查当前用户是否有权访问该字段。本库不决定授权规则。

`PropertyValue<'a>` 可以持有来自输入实例的借用；调用方不能把它延长到实例生命周期之外。Getter 可返回借用、可选借用或受支持的切片形状；返回 owned 中间对象不意味着可继续无损借用遍历。Setter 失败通过 `PropertySetFailure` 区分调用前拒绝与调用后失败，并可携带尚未消费的替换值。调用方不能把任何 setter 错误都当成“对象未改变”；特别是自定义 setter 已运行后，是否回滚由该 setter 和业务层决定。

## 可选执行层的完整阶段

默认构建只保存声明。启用 `validation` 后，一次校验按以下顺序进行：

1. `ValidationCapabilities::check(root, &graph)` 检查图内根的声明发现与访问形状；它不绑定自定义规则，也不调用 getter。
2. `ValidationPlan::build(root, ValidationBuildInputs { graph, validators })` 对每个可执行 occurrence 绑定内置规则或传入的 `ValidatorRegistry`。构建时聚合独立的缺失注册、类型不匹配和 `UnsupportedExecution` 等错误。根以图内同一 `TypeId` 的规范元数据为准，调用方另传的 overlay 不能篡改图内声明。
3. `plan.validate(&value, &options)` 核对具体实例的根类型，按模型规则、字段及 selector 的声明次序读取并执行。规则违例返回 `Ok(ValidationReport)`；访问器、依赖、预算等基础设施故障返回 `Err(ModelValidationError)`，并保留已经产生的部分报告。动态调用方使用 `validate_dynamic(PropertyBorrowed::new_any(&value), &options)`。

以下完整程序为具体值构建并复用验证计划：

<!-- example: validation -->
```toml
[dependencies]
qubit-model-metadata = { version = "0.2", path = "../rs-model-metadata", features = ["validation"] }
qubit-model-derive = { version = "0.2", path = "../rs-model-metadata/derive" }
qubit-reflect = { version = "0.2", path = "../rs-reflect" }
qubit-validator = { version = "0.1.0", path = "../../rust-common/rs-validator" }

[patch.crates-io]
qubit-validator = { path = "../../rust-common/rs-validator" }
```

<!-- example: validation/design-plan -->
```rust
use std::sync::Arc;
use qubit_model_derive::Model;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_model_metadata::validation::ValidationBuildInputs;
use qubit_model_metadata::validation::ValidationOptions;
use qubit_model_metadata::validation::ValidationPlan;
use qubit_validator::ValidatorRegistry;

#[Model]
struct Account {
    #[text(non_blank)]
    name: String,
}

fn main() {
    let root = TypeMetadata::of::<Account>();
    let models = ModelRegistry::try_global().expect("已链接模型元数据");
    let graph = Arc::new(StructureResolver::for_static_roots(&models, [root])
        .resolve().expect("模型图有效"));
    let validators = ValidatorRegistry::empty();
    let plan = ValidationPlan::build(root, ValidationBuildInputs {
        graph: Arc::clone(&graph),
        validators: &validators,
    }).expect("内置文本规则已绑定");

    let valid = Account { name: "Ada".to_owned() };
    assert!(plan.validate(&valid, &ValidationOptions::default())
        .expect("验证已执行").is_valid());
    let invalid = Account { name: String::new() };
    let report = plan.validate(&invalid, &ValidationOptions::default())
        .expect("验证已执行");
    assert!(!report.is_valid());
    assert_eq!(report.violations()[0].path().render(), "name");
}
```

遍历会发现每条实际使用路径上的声明，例如两个不同字段都指向同一种子类型时保留两个 occurrence。执行边界如下：

| 形状 | 执行约定 |
| --- | --- |
| Enum 具名或位置 payload 字段上的直接值规则 | 仅活动变体执行；非活动变体跳过，违例路径包含变体和字段位置。 |
| Enum payload 中的 selector 或嵌套模型规则 | 拒绝；payload 目前只支持直接值规则。 |
| `Vec<T>` / `[T; N]` 的 `Element`；`HashMap<K, V>` / `BTreeMap<K, V>` 的 `MapKey`、`MapValue` | 通过生成的借用适配器逐个执行；借用 slice getter 也能提供元素。`Option` 包装的集合为 `None` 时跳过 selector。 |
| 带依赖的 selector 规则、未知/自定义容器、owned 集合 getter、隐式解包成员、集合成员内部的模型规则 | 能力检查和计划构建阶段拒绝。 |

没有执行工作的递归形状可以终止；含执行工作的递归路径会拒绝，不会生成遗漏规则的“成功空计划”。Reference 停在存储字段，`opaque` 停在内部结构，但两者的外层显式规则仍保留。具体支持形状见[用户手册](user_guide.zh_CN.md#验证消费者按声明和访问形状绑定)。

`ValidationOptions::default()` 采用 `CollectAll`、全部字段，以及深度 64、节点 100,000、违例 100、比较 1,000,000 的非零上限。`FailFast` 或达到违例上限属于成功执行后的截断报告；深度、节点、比较等遍历预算耗尽属于执行错误并附部分报告。共享预算覆盖根、属性、依赖和集合成员的读取及规则调用；selector 元素规则调用和外层 sequence 去重的每一对比较都会消耗比较预算。未知长度 selector 的探测会在调用 `next()` 前，为可能的读取和规则调用预留节点预算，并预留比较预算；用于确认耗尽的探测即使没有产出成员，也可能消耗这些预算。预算不约束外部自定义 validator 内部自行分配或执行的工作。Map entry 路径序号沿容器迭代顺序，`HashMap` 的顺序不稳定。字段选择匹配完整字段路径，不代表自动展开某个路径前缀。

计划构建单独使用固定 `BuildBudget`：模型、wrapper 和类型表达式深度最多 64；发现和绑定工作最多计费 100,000 次；声明 occurrence 和绑定规则各最多 10,000 条；普通诊断最多 100 条。每次检查模型、字段、结构边、表达式节点、声明、selector 或 validator、反向可达边，以及每次绑定尝试，各计一个工作单位；同一项在不同阶段再次检查时分别计费。尝试产生第 101 条普通诊断时停止构建并添加唯一的 `BuildLimitExceeded` 终止错误，因此错误列表最多 101 条。能力检查与计划构建遵循相同的发现预算，绑定输出在追加前检查。该上限不约束调用方已创建的 graph 或 reflection context，也不限制外部 validator binder 内部的工作和分配；它与上述逐实例执行预算相互独立。

`ValidationBuildError::build_limit()` 为超限错误提供 `ValidationBuildLimit`，其中包含 `ValidationBuildLimitKind` 和固定容量；其他构建错误返回 `None`。五类为 Depth（64）、Work（100,000）、Occurrences（10,000）、Bindings（10,000）和 Diagnostics（100 条普通诊断，之后增加一条终止错误）。类别 enum 标记为 `#[non_exhaustive]`，下游匹配需包含 wildcard 分支。根级错误可能在声明遍历前发生，因此可能没有 path、声明来源或 binder source。为保持兼容，`BuildLimitExceeded` 仍是无 payload 的 unit variant；调用方应使用访问器，不应解析显示字符串。

MM-003 使用固定 Work 上限复现：修复前，同一个 root 的单 root 图在 10 个工作单位成功，共享无关模型的图需要 31 个单位。root-scoped 结构发现修复后，两图都在 10 个单位成功；共享图最先需要 14 个单位。剩余的全图工作来自 expression fallback 的名字查找，它为保留全局唯一名称语义而扫描全图，并将实际查找计入预算。该修复减少无关结构扫描，但没有消除所有全图工作，也没有改变名字解析语义。

公共 `ValidationBuildErrorKind` 新增 `BuildLimitExceeded`，下游穷尽匹配可能需要增加显式分支或 fallback。rs-platform 当前包装聚合错误，没有需要迁移的穷尽 kind 匹配。

启用 `codec` 时，声明中的稳定 codec ID 或 Rust codec 类型仍要经显式传入的 registry 绑定。选择优先级是字段显式 codec、Value 的 canonical codec、无 codec。`bind_codecs` 解析声明并返回已选实现的绑定结果；它不会调用 `encode` 或 `decode`、保存数据或执行应用策略。元数据本身不含 codec 实例，不承担持久化流程。

`CodecOccurrenceId` 用于在当前进程内查找一次 codec 绑定。完整键包含 `TypeId`、可选 `ModelId`、属性路径和声明来源。由于键中仍有 `TypeId`，即使存在稳定的 `ModelId`，相等比较、哈希和排序也只对当前进程有效。不要把该值、它的 `TypeId`、哈希值或 `Debug`/`Display` 输出当作持久化键或外部身份。需要跨进程标识声明时，消费者应另行定义稳定协议，并处理匿名模型。

## 错误分期与集成约束

| 阶段 | 典型失败 | 应用应采取的动作 |
| --- | --- | --- |
| 宏展开与 Rust 类型检查 | 属性语法、角色形状、trait bound、重复访问器 | 修正声明或显式能力开关，不能在运行时“重试” |
| 受检元数据与 registry | `AbiViolation`、重复 ID、能力目标或 provider 冲突 | 检查 runtime/derive 版本、链接集合、反射快照来源 |
| `StructureResolver` | 引用目标、Value 闭包、唯一性作用域等结构错误 | 查看 owner、字段位置、声明来源，修正模型关系 |
| plan/codec 绑定 | 缺少注册项、类型不匹配、当前执行器不支持形状 | 补充适配器或调整消费者，不应删去真实业务约束换取通过 |
| 实例执行 | `ValidationReport` 中的违例，或带部分报告的执行错误 | 违例交给业务决策；错误保留原因与部分结果，避免误判为有效 |

`__private::v8` 只供生成代码和运行时协同使用，应用应依赖公开 `metadata`、`registry`、`resolve`、`validation`、`codec` API。未来改动如影响生成 ABI，必须同步更新 runtime 与 derive；历史私有门面没有兼容保证。

## 源码与验收入口

- 静态类型、字段及属性：`src/type_metadata.rs`、`src/field_metadata.rs`、`src/property.rs`、`src/local_property_set.rs`。
- 注册、缓存和来源：`src/registry/`；结构解析与图查询：`src/resolve/`。
- 声明 vocabulary、路径和位置：`src/metadata/`、`src/relation/`；执行绑定：`src/validation/`、`src/codec/`。
- 可复制使用的公开 API、成功/失败场景及限制：本目录的[用户手册](user_guide.zh_CN.md)。
