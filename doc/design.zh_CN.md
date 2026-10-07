# rs-model-metadata 设计文档

[English](design.md) · [用户手册](user_guide.zh_CN.md) · [派生宏设计](../derive/doc/design.zh_CN.md)

本文对应当前 `qubit-model-metadata` 0.2.0 实现（Rust 1.94、edition 2024）。该 crate 负责运行时元数据与模型关系解析；`qubit-model-derive` 生成声明，持久化、查询和业务校验的最终决策由应用负责。当前 manifest 设置了 `publish = false`。

## 边界与数据流

```text
Rust 声明 --derive--> TypeMetadata + 反射能力
                         |
ReflectRegistry 快照 --> ModelRegistry --> StructureResolver --> ModelGraph
                               |                   |
                       PropertyAccessPath    可选的校验/codec 绑定
```

`TypeMetadata` 无需实例即可描述一个 Rust 类型的角色、字段、声明和本地属性。`FieldMetadata` 对应存储字段；`PropertyMetadata` 组合字段与符合条件的 `ModelImpl` 访问器，也可表示计算属性。读取属性时，能借用实例就保留借用。`FieldLocation` 用所属 `TypeId`、可选枚举变体和字段序号定位具体字段；泛型定义的字段尚无具体位置。外部持久身份使用 `ModelId`，`TypeId` 与 `FieldLocation` 仅在进程内有效。

公开功能分别放在 `metadata`、`registry`、`resolve`、`access_path`；`generic`、`validation`、`codec` 是独立可选 feature，默认均不启用。稳定 ID 协议还可单独通过 `qubit-model-id` 使用。

## 快照与模型图

`ModelRegistry::try_global()` 对已链接的反射注册进行一次冻结，并保留初始化错误。需要隔离时，可从指定 `ReflectRegistry` 快照或静态元数据构造 registry。Registry 同时按稳定 ID 和精确 Rust 类型建立索引；匿名根由解析器接收，不伪造 ID。属性、元数据缓存属于各自的 registry。属性路径缓存合计最多保留 256 条成功的读写路径；失败的编译下次会重试。调用方已经持有的编译路径在缓存逐出后仍可使用。

`StructureResolver::new` 审计全部已注册模型；`for_roots` 与 `for_static_roots` 只解析指定根及其可达模型。解析时检查角色、引用、Projection 来源、Value 闭包、唯一性作用域和查询声明。`ModelGraph` 借用选定的 registry，以 `TypeId` 或 `FieldLocation` 查询关系。仅由静态元数据构建的 registry 需要显式提供子模型；基于反射快照的 registry 可发现可达匿名子模型。模型图不会从全局 registry 补入缺失声明。

## 声明与执行

核心 crate 保存约束、validator、selector、codec 引用，以及对象路径和属性路径。对象路径使用 `/` 与 `..`，属性路径使用 `.`。查询元数据提供声明及原因，不替应用生成任意筛选器。`PropertyAccessPath` 先编译结构路径，再对实例读写；读写支持范围不同。结构性写入错误会在调用末级 setter 前报告。授权、持久化及查询执行由应用负责。

启用 `validation` 后，`ValidationCapabilities::check` 检查根模型声明能否执行。`ValidationPlan::build` 使用调用方提供的 `ValidatorRegistry` 和 `Arc<ModelGraph>` 绑定受支持的声明；不支持的形状或缺少规则会形成聚合的构建错误。`validate` 将规则违例作为报告返回；执行失败返回带部分报告的 `ModelValidationError`。`ValidationOptions` 控制字段选择、停止策略和预算。可选 codec 绑定也需要显式 registry。具体支持矩阵和完整操作见[用户手册](user_guide.zh_CN.md#限制执行范围与构建拒绝)。

## 错误与兼容契约

宏语法错误在编译时诊断。受检元数据构造与 registry 初始化报告 ABI 或注册错误；模型图报告结构错误；执行适配器报告绑定或运行错误。查无对象与查找失败是不同结果。生成代码使用同步的 `__private::v7` ABI 和反射 provider 契约，因此运行时与 derive 版本应保持一致。持久化时保存 `ModelId`，不要保存进程内的 `TypeId` 或 `FieldLocation`。

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
2. `ModelRegistry::from_reflect_registry(&reflection)` 投影调用方提供的冻结快照。它保留来源信息，合并快照中的 `ModelImpl` provider，并能在可达遍历中发现匿名子模型；不会查询全局快照补缺。
3. `ModelRegistry::from_static_metadata(...)` 用明确给出的元数据建立隔离注册表。它只使用每个 `TypeMetadata` 的本地属性，不会看到另行注册的 impl provider；需要解析匿名子模型时，调用方必须将子元数据列入输入。

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
3. `plan.validate(ReflectedRef::new(&value), &options)` 核对实例根类型，按模型规则、字段及 selector 的声明次序读取并执行。规则违例返回 `Ok(ValidationReport)`；访问器、依赖、预算等基础设施故障返回 `Err(ModelValidationError)`，并保留已经产生的部分报告。

遍历会发现每条实际使用路径上的声明，例如两个不同字段都指向同一种子类型时保留两个 occurrence。没有执行工作的递归形状可以终止；含执行工作的递归路径、受约束的 Enum payload 或当前无适配器的集合内部路径会明确拒绝，不会生成遗漏规则的“成功空计划”。Reference 停在存储字段，`opaque` 停在内部结构，但两者的外层显式规则仍保留。完整形状矩阵见[用户手册](user_guide.zh_CN.md#限制执行范围与构建拒绝)。

`ValidationOptions::default()` 采用 `CollectAll`、全部字段，以及深度 64、节点 100,000、违例 100、比较 1,000,000 的非零上限。`FailFast` 或达到违例上限属于成功执行后的截断报告；深度、节点、比较等遍历预算耗尽属于执行错误并附部分报告。预算计算覆盖本库的读取、元素访问和规则调用，不约束外部自定义 validator 内部自行分配或执行的工作。字段选择匹配完整字段路径，不代表自动展开某个路径前缀。

启用 `codec` 时，声明中的稳定 codec ID 或 Rust codec 类型仍要经显式传入的 registry 绑定。选择优先级是字段显式 codec、Value 的 canonical codec、无 codec；元数据本身不含 codec 实例，不承担数据持久化或编码流程。

## 错误分期与集成约束

| 阶段 | 典型失败 | 应用应采取的动作 |
| --- | --- | --- |
| 宏展开与 Rust 类型检查 | 属性语法、角色形状、trait bound、重复访问器 | 修正声明或显式能力开关，不能在运行时“重试” |
| 受检元数据与 registry | `AbiViolation`、重复 ID、能力目标或 provider 冲突 | 检查 runtime/derive 版本、链接集合、反射快照来源 |
| `StructureResolver` | 引用目标、Value 闭包、唯一性作用域等结构错误 | 查看 owner、字段位置、声明来源，修正模型关系 |
| plan/codec 绑定 | 缺少注册项、类型不匹配、当前执行器不支持形状 | 补充适配器或调整消费者，不应删去真实业务约束换取通过 |
| 实例执行 | `ValidationReport` 中的违例，或带部分报告的执行错误 | 违例交给业务决策；错误保留原因与部分结果，避免误判为有效 |

`__private::v7` 只供生成代码和运行时协同使用，应用应依赖公开 `metadata`、`registry`、`resolve`、`validation`、`codec` API。未来改动如影响生成 ABI，必须同步更新 runtime 与 derive；历史私有门面没有兼容保证。

## 源码与验收入口

- 静态类型、字段及属性：`src/type_metadata.rs`、`src/field_metadata.rs`、`src/property.rs`、`src/local_property_set.rs`。
- 注册、缓存和来源：`src/registry/`；结构解析与图查询：`src/resolve/`。
- 声明 vocabulary、路径和位置：`src/metadata/`、`src/relation/`；执行绑定：`src/validation/`、`src/codec/`。
- 可复制使用的公开 API、成功/失败场景及限制：本目录的[用户手册](user_guide.zh_CN.md)。
