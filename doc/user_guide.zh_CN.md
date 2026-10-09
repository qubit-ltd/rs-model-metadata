# 模型元数据使用指南

[English](user_guide.md) · [设计文档](design.zh_CN.md) · [README](../README.zh_CN.md) · [声明指南](../derive/doc/user_guide.zh_CN.md)

## 它解决什么问题

本指南面向需要**读取和利用领域模型语义**的应用、框架及工具开发者，适用于 0.2.0。`rs-reflect` 提供 Rust 类型、字段和访问能力等语言层面的反射；领域业务还需要表达长度限制、唯一性、身份、引用及自定义规则。`qubit-model-derive` 用宏把这些事实标在领域对象上，`qubit-model-metadata` 则提供读取、注册和解析这些声明的入口。

同一份元数据可供不同消费者使用：验证器检查实际对象；随机对象生成器按约束准备数据库层或业务层测试数据；REST 查询层依据 `indexed` 等声明校验查询字段并构造过滤器。库提供**语义信息及其结构关系**，具体执行策略由消费者决定。例如，`unique` 说明必须满足领域唯一性，验证现有数据库记录需要调用方提供数据访问能力；`reference` 指明目标关系，准备已存在的测试对象需要调用方提供仓储能力。

首次阅读从[实战场景](#实战场景从声明到结构检查与验证)进入；已在编写消费者时可按下表查找入口。如何写宏标注见[声明指南](../derive/doc/user_guide.zh_CN.md)。Rust 最低版本为 1.94、edition 为 2024；默认 feature 集为空，`generic`、`validation`、`codec` 分别开启对应能力。

| 要完成的任务 | 从哪里开始 | 成功信号 |
| --- | --- | --- |
| 读取已知 Rust 类型的字段和角色 | `TypeMetadata::try_of::<T>()` | 获得字段和角色元数据，无需构造实例 |
| 读取 getter/setter 合并后的属性 | `try_properties()` | 找到可读或可写的 Property，错误不会伪装成缺失 |
| 按稳定 ID 找模型 | `ModelRegistry::try_global()` | 在当前注册范围内查到元数据 |
| 检查引用、查询声明和子模型 | `StructureResolver::for_roots` | 得到 `ModelGraph` 或结构错误 |
| 校验一个实例 | `ValidationPlan::build`、`validate` | 得到有效、带违例或截断的报告；执行故障返回错误 |
| 绑定 codec | `codec::bind_codecs` | 得到绑定结果或含来源的错误 |

这里的 Field 是存储字段；Property 是字段与合格访问器合并后的可访问属性；
`TypeMetadata` 是不依赖实例的静态描述。只有当任务涉及多个模型的关系时才需要结构图。

## 实战场景：从声明到结构检查与验证

沿用[声明指南中的 `User` 与 `PersonInfo`](../derive/doc/user_guide.zh_CN.md#实战场景声明用户与人员资料的业务关系)：`User.username` 有长度与唯一性要求，`PersonInfo.user_id` 引用 `User.id`。消费者要做两件事：先确认关系声明在最终链接的模型集合中能解析，再把可执行的字段约束绑定为验证计划。随机测试生成器也会从相同元数据得知用户名应满足什么约束、`user_id` 应从哪个已存在用户取得。

当前 checkout 中相关 manifest 均为 `publish = false`，示例因此使用本地路径依赖。路径假设应用位于 `rs-platform/app`，与 `rs-model-metadata`、`rs-reflect` 同属 `rs-platform`；版本字段不代表这些版本已发布。将完整程序放入应用的 `src/main.rs`，并从该应用目录运行 `cargo run`。

<!-- example: core -->
```toml
[dependencies]
qubit-model-metadata = { version = "0.2", path = "../rs-model-metadata", default-features = false }
qubit-model-derive = { version = "0.2", path = "../rs-model-metadata/derive" }
qubit-id = "0.7"
qubit-reflect = { version = "0.2", path = "../rs-reflect" }
```

只需要稳定模型 ID 的库可单独依赖协议 crate：

<!-- example: core/model-id -->
```toml
[dependencies]
qubit-model-id = { version = "0.1", path = "../rs-model-metadata/model-id" }
```

首先读取本地语义，再解析两个模型的关系。下例沿用声明指南的模型类型；完整程序需引用定义它们的 crate。

```rust,ignore
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::{ResolveInputs, StructureResolver};

let user_metadata = TypeMetadata::try_of::<User>()?;
let person_metadata = TypeMetadata::try_of::<PersonInfo>()?;
let username_field = user_metadata.field("username").expect("declared field");
assert_eq!(username_field.text_constraint().unwrap().max_chars(), Some(64));
assert!(username_field.is_unique());

let models = ModelRegistry::try_global()?;
let roots = [user_metadata, person_metadata];
let graph = StructureResolver::for_roots(ResolveInputs {
    models, roots: &roots,
}).resolve()?;
let field = person_metadata.field("user_id").unwrap();
let relation = graph.reference(field.location().unwrap()).expect("resolved relation");
assert_eq!(relation.target().type_id(), user_metadata.type_id());
```

`try_of` 读取具体类型的静态元数据并检查生成协议；`try_global` 投影最终二进制已链接的模型；`for_roots` 解析所选根及其可达关系。`reference()` 能查到结果，说明目标模型与属性已在**这一注册范围**中解析成功。字段上有 `#[reference]` 但图内查不到目标时，应先核对稳定 ID、链接范围和目标属性类型。`#[unique]` 的存在只说明规则已声明；是否与现存记录冲突仍由具备数据访问能力的消费者判断。

同一图还能提供查询声明。REST 层可以从 `graph.query(user_metadata.type_id())` 取得带索引原因的字段，建立可接受查询参数的白名单：

```rust,ignore
let query = graph.query(user_metadata.type_id()).expect("Entity query declarations");
let requested_field = "display_name";
let declared = query.declarations().iter().any(|entry| {
    entry.path().segments() == [requested_field]
});
assert!(declared);
// 再结合字段类型、调用方权限和业务策略决定比较操作与 SQL 映射。
```

这里先确认查询字段出现在模型声明中；实际接口还必须确定该字段是否对当前调用方开放、接受哪种比较操作，再构造过滤器并映射到 SQL。`QueryDeclaration::field()` 可继续读取字段类型和约束，`reasons()` 返回索引原因。`identifier`、`unique`、`reference` 也贡献索引原因，因此不能把所有声明直接暴露给客户端。

## 实战场景续：校验一个用户实例

给运行时依赖启用 `validation`，并添加验证执行 API：

<!-- example: validation -->
```toml
[dependencies]
qubit-model-metadata = { version = "0.2", path = "../rs-model-metadata", features = ["validation"] }
qubit-model-derive = { version = "0.2", path = "../rs-model-metadata/derive" }
qubit-id = "0.7"
qubit-reflect = { version = "0.2", path = "../rs-reflect" }
qubit-validator = { version = "0.1.0", path = "../../rust-common/rs-validator" }
```

验证执行依赖同一 checkout 的 `rs-validator`；添加该依赖后，从 `rs-platform/app` 运行完整示例。

沿用前述用户模型，这里只校验输入中的 `username` 文本约束。`#[unique]` 告诉需要访问数据源的消费者应检查什么，却不等于只凭当前对象就能判断数据库中是否重名。下列完整程序故意提供长度不合格的用户名，展示元数据如何变成可执行的验证报告。

<!-- example: validation/user -->
```rust
use std::sync::Arc;
use qubit_model_derive::Entity;
use qubit_id::Id;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_model_metadata::validation::ValidationBuildInputs;
use qubit_model_metadata::validation::ValidationOptions;
use qubit_model_metadata::validation::ValidationPlan;
use qubit_reflect::ReflectedRef;
use qubit_validator::ValidatorRegistry;

#[Entity(id = "guide.iam.User")]
struct User {
    #[identifier]
    id: Id,
    #[unique(ignore_case = true)]
    #[text(non_blank, min_chars = 3, max_chars = 64, allowed_chars = ascii)]
    username: String,
    #[indexed]
    display_name: String,
}

fn main() {
    let root = TypeMetadata::of::<User>();
    let roots = [root];
    let models = ModelRegistry::try_global().expect("linked models");
    let graph = Arc::new(StructureResolver::for_roots(ResolveInputs { models: &models, roots: &roots })
        .resolve().expect("valid structure"));
    let validators = ValidatorRegistry::empty();
    let plan = ValidationPlan::build(root, ValidationBuildInputs {
        graph: Arc::clone(&graph),
        validators: &validators,
    }).expect("text constraint bound");

    let invalid = User {
        id: Id::new(1),
        username: "a".to_owned(),
        display_name: "Alice".to_owned(),
    };
    let report = plan.validate(ReflectedRef::new(&invalid), &ValidationOptions::default())
        .expect("validation executed");
    assert!(!report.is_valid());
    assert_eq!(report.violations()[0].path().render(), "username");
}
```

`ValidationPlan::build` 把图内可执行约束与规则实现绑定；`validate` 返回报告，`is_valid()` 才表示该实例通过。`Ok(report)` 中可以含有违规；`Err` 表示结构、绑定或执行故障，不能当作“没有违规”。空 `ValidatorRegistry` 只表示没有自定义规则，标准文本约束由内置适配器绑定；使用 `#[validator(id = "...")]` 时须提供对应注册项。验证计划可复用，报告对应每次传入的实例。

## 速查：注册表、全图审计与泛型定义

只想按稳定 ID 找已链接模型时，无需解析结构图。在上例的 `User` 已链接到最终程序时：

```rust,ignore
let models = ModelRegistry::try_global()?;
let user_metadata = models.metadata("guide.iam.User").expect("已注册 User");
assert_eq!(user_metadata.type_id(), TypeMetadata::of::<User>().type_id());
```

`None` 表示所选注册范围内没有这个 ID；如果注册表构造失败，应处理 `Err`，不能当作不存在。

| 注册范围 | 入口 | 何时选用 |
| --- | --- | --- |
| 最终二进制已链接的模型 | `ModelRegistry::try_global()` | 应用启动时检查全部链接声明 |
| 明确冻结的反射快照 | `from_reflect_registry(&reflection)` | 测试隔离、插件视图或需要独立注册的 `ModelImpl` 访问器 |
| 手头的一组静态元数据 | `from_static_metadata(...)` | 不需要反射快照能力的工具；子模型也须显式提供 |

注册表不会从进程全局范围补入当前范围缺失的模型。快照可从根发现可达匿名子模型，
静态元数据注册表只能读取显式传入的类型和本地 Property；因此计划构建前要确认使用了同一注册范围。
启用 `generic` 后，定义 provider 也随反射快照收集；静态注册表可用
`from_static_metadata_with_generics` 显式提供。按 ID 的 `entries()` 不包含匿名定义，
但 `generic_definitions()` 和 `generic_metadata_for(definition.id())` 可以访问它们。

启用 `generic` 且模型声明了泛型定义后，可以按定义身份查询，而不是用具体类型的稳定 ID 猜测：

```rust,ignore
let concrete = TypeMetadata::of::<Page<String>>();
let definition = concrete.generic_definition().expect("泛型定义");
let declared = models.generic_metadata_for(definition.id());
assert!(declared.is_some());
```

这里的 `Page<T>` 和 `models` 需来自同一注册范围；只需要具体类型字段时，
直接读取 `concrete.fields()` 即可。

验证入口通过传入根的 TypeId 选择图中的元数据。同一 Rust 类型即使有另一份经过检查的 overlay，
也不能借此增加或删除图中的声明。`ValidationPlan::root()` 返回实际采用的图内元数据，
能力检查遵循同一快照边界。

启动或发布前审计时，可以用 `new` 和空根列表解析所有注册模型。处理单个请求时则使用
`for_roots`，下方的验证流程就是这种用法。它会跳过无关模型的结构初始化，同时从同一注册表快照
读取可达模型的 capability 和 Property provider。

<!-- example: core/audit -->
```rust
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;

fn main() {
    let models = ModelRegistry::try_global().expect("链接模型注册表有效");
    let graph = StructureResolver::new(ResolveInputs { models: &models, roots: &[] })
        .resolve().expect("所有已链接模型结构均有效");
    println!("已审计 {} 个已链接模型", graph.models().len());
}
```

## 速查：字段身份与 Property 访问

Field 是存储声明；Property 合并同名字段和合格的 getter/setter。需要读取业务语义时从 Field 开始；需要读取实例或沿嵌套路径执行规则时检查 Property 的访问能力。

```rust,ignore
let field = person_metadata.field("user_id").expect("declared field");
let location = field.location().expect("concrete field");
let target = graph.reference(location).expect("resolved relation").target();
assert_eq!(target.type_id(), user_metadata.type_id());

let resolved = user_metadata.try_properties()?;
let username_property = resolved.property("username").expect("readable property");
```

`FieldLocation` 由当前进程的 owner 类型、variant 和字段序号组成，是图中查询具体声明的键；外部持久化身份应使用稳定 `ModelId`。`try_properties()` 返回持有合并结果的视图，借出的 Property 不能超过 `resolved` 的生命周期。组装错误会作为 `Err` 返回，而非伪装成属性不存在。

重复按路径访问实例时，可以在同一个 `ModelRegistry` 中缓存编译结果；读写能力分别编译：

```rust,ignore
let read = models.compile_read_path_cached(user, &["username"])?;
let write = models.compile_write_path_cached(user, &["username"])?;
```

上例沿用已取得的 `models` 与 `user`。编译成功表示结构路径和对应访问能力成立，不等于应用授权调用方读写该字段。缓存属于注册表快照，不跨不同快照复用；实际 JSON 读写可参考 [rs-platform 用户指南](../../rs-platform/doc/user-guide.zh_CN.md#属性路径的实际使用顺序)。

## 进阶用法：对象路径、引用与声明位置

ObjectPath 使用 NavigationStep::Property 和 NavigationStep::Parent，显示为 `/` 分隔路径；
PropertyPath 使用 `.` 选择普通属性。依赖的空 ObjectPath 表示当前对象，reference 省略 path 则表示未请求复用。
普通容器不增加领域父对象。

引用绑定路径经过只保存 ID 或 Projection 的字段时，仍导航所绑定的完整 Entity；目标 property 单独选择。
依赖父对象的 reference、validator 声明会保留 ContextRequirement::ParentObject。
`ModelGraph::dependencies()` 保存各自独立的依赖 occurrence。DeclarationLocation 保存文件、行列、
owner 名称、variant/field 序号与 selector 位置，无名 Enum payload 也能定位。

如果根是一个 `Projection`，可从图中确认其 Entity 来源：

```rust,ignore
let source = graph.projection_source(TypeMetadata::of::<UserView>().type_id())
    .expect("视图来源已解析");
assert_eq!(source.target().type_id(), TypeMetadata::of::<User>().type_id());
```

`UserView` 须声明 `#[Projection(source = User)]`，且两者要纳入同一结构图。
这与上面的字段引用查询是两类关系；都需要先成功构建 `ModelGraph`。

## 进阶用法：显式绑定执行适配器

启用 validation 后，调用 `ValidationPlan::build(root, ValidationBuildInputs { graph: Arc::clone(&graph),
validators: &validators })`，传入自己的 validator registry。
计划持有 `Arc<ModelGraph>`；构建后即使调用方释放自己的 graph 句柄，计划仍可执行。普通请求按根解析，
完整链接模型审计则解析全图；局部计划成功不表示无关模型也通过结构检查。
绑定检查稳定 ID、参数、可读 Property 路径及已知的输入、依赖类型。预备实例的形状还会与签名逐项核对。
metadata 保存借用的声明参数，并在绑定每条规则时将其转换为 `qubit-validator` 的参数。
静态可判定的 optional getter 路径不能满足必需依赖；deferred 父路径会在上下文提供后检查。同 ID 声明分别绑定，不会互相覆盖。
标准约束使用现有 validation-rules 适配器。

父依赖可通过 `build_with_context` 提供类型 metadata，通过 `validate_with_context` 提供借用实例，
两者都按“最近父对象优先”排列。绑定时也允许暂缺父类型；执行前应把父模型纳入 graph，
执行器才能解析并核对延后的路径后缀。缺少父对象会返回结构化依赖错误，不能当作 Option::None。
即使计划为空，传入错误 Rust 类型的根实例也会被拒绝。

例如某个 `Child` 规则通过 `path = ".."` 读取 `Parent` 属性时，应用在建立计划和执行时分别提供：

```rust,ignore
let plan = ValidationPlan::build_with_context(
    child,
    ValidationBuildInputs { graph: Arc::clone(&graph), validators: &validators },
    &[parent],
)?;
let report = plan.validate_with_context(
    ReflectedRef::new(&child_value),
    &[ReflectedRef::new(&parent_value)],
    &ValidationOptions::default(),
)?;
```

这里的 `child`、`parent` 是对应类型的 `TypeMetadata`，两个值是实际实例；
`graph` 必须包含依赖路径涉及的模型。少传一个父实例属于执行错误，不表示依赖值为 `None`。

计划只读，不修改对象。ValidationOptions 控制字段选择、快速失败和遍历预算。
当前支持边界内的直接、Option 嵌套模型 validator 自动纳入计划，opaque 截断遍历。
元数据能描述的范围大于某个执行后端。借用的可选中间对象既可以来自字段 `Option<T>`，也可以来自返回
`&Option<T>` 的 getter；遇到 `None` 时跳过该分支，遇到 `Some` 则沿编译好的路径继续访问。
终端字段若为 `Option<String>`，也能执行 text 规则：`None` 不执行规则，`Some(value)` 按文本校验。
违规 occurrence 保留完整 Property 路径，例如 `mobile.country_area`。这些路径要求借用访问；返回
owned 中间对象的 getter 无法继续遍历。借用切片 getter 支持显式 element validator；生成的集合适配器
还可在受支持的具体类型上执行外层 sequence 去重和 map entry 数量约束。selector 内的标准约束及
MapKey/MapValue 遍历仍会明确返回构建错误。选择后端前检查 ValidationCapabilities，不应把“能够声明”
理解成“所有后端都能执行”。

### 批量追加模型级规则

当 validator 针对整个模型而非某个声明字段时，使用 `ModelRuleBinding`。计划构建完成后，把准备好的
绑定一次性交给 `with_model_rules`：

```rust,ignore
let plan = plan.with_model_rules(prepared_model_rules);
```

`prepared_model_rules` 可以是任意 `IntoIterator<Item = ModelRuleBinding>`，例如
`Vec<ModelRuleBinding>`。方法会消费迭代器，并返回拥有这些新增规则的计划。新规则按输入顺序追加在已有模型规则之后；
传入空迭代器不会改变现有规则。同一规则 ID 不会去重，重复项仍是独立 occurrence。
API 允许分批追加，但每次调用都会重新构造拥有型规则数组；通常应先收集本次要追加的规则，再调用一次。
被选中的模型级规则先于字段规则执行，详见[停止条件与执行预算](#进阶用法停止条件与执行预算)。

启用 codec 后，在结构解析完成后使用 CodecBindInputs 与显式 codec registry 调用 `codec::bind_codecs`。
选择顺序为字段显式 codec、Value canonical codec、无 codec。Rust 类型形式也要求相应注册项存在；
显式指定同一个 canonical codec 合法。occurrence 身份包含准确 TypeId、可选 ModelId、Property 路径和来源。

```rust,ignore
let bindings = qubit_model_metadata::codec::bind_codecs(
    qubit_model_metadata::codec::CodecBindInputs {
        graph: &graph,
        codecs: &codecs,
    },
)?;
for binding in bindings.bindings() {
    // 应用在此把绑定结果交给自己的编解码流程。
}
```

此片段假定 `codecs` 是应用已建立的 `ValueStringCodecRegistry`，`graph` 是解析成功的图。
缺少注册项或值类型不匹配时，绑定返回错误；声明 `#[codec(...)]` 本身不会执行编码。

## 验证消费者：按声明和访问形状绑定

验证计划在构建时检查声明、字段访问能力和规则实现；执行时检查一个具体值。对业务代码最重要的是区分这两个结果：构建失败意味着规则无法安全绑定，执行成功但 `report.is_valid() == false` 意味着实例违反了规则。

| 声明 | 执行时如何使用 | 需要提供什么 |
| --- | --- | --- |
| `#[text(...)]` | 按字符数、非空白或格式检查字段值 | 可读取的文本属性 |
| `#[decimal(...)]`、`#[money(...)]`、`#[time(...)]` | 对准确的数值或时间类型检查精度与范围 | 与声明匹配的具体 Rust 类型 |
| `#[sequence(...)]`、`#[map(...)]` | 检查容器长度；`unique_items` 检查序列内部重复 | 可借用容器的访问器；去重需要元素相等性适配器 |
| `#[validator(id = ...)]` | 调用按 ID 绑定的自定义规则 | 同 ID 的注册项；有依赖时还需可读依赖路径 |
| `#[reference(...)]` | 检查保存引用的字段上显式声明的值约束 | 数据库存在性等跨对象检查由具备仓储能力的消费者完成 |
| `#[opaque]` | 执行外层字段约束，停止内部遍历 | 在模型上标出交给其他组件处理的边界 |

可选子对象为 `None` 时跳过内部值约束，为 `Some` 时继续访问；需要继续穿过该属性时，应提供借用 getter：

```rust,ignore
#[Model]
struct Profile { contact: Option<Contact> }

#[ModelImpl]
impl Profile {
    pub fn contact(&self) -> Option<&Contact> { self.contact.as_ref() }
}
```

片段需引入 `Model`、`ModelImpl`，并定义 `Contact`。在计划构建前可检查访问能力；检查不会读取具体对象：

```rust,ignore
ValidationCapabilities::check(root, graph.as_ref())?;
let plan = ValidationPlan::build(
    root,
    ValidationBuildInputs { graph: Arc::clone(&graph), validators: &validators },
)?;
```

这里的 `root`、`graph` 和 `validators` 与上文验证流程相同。`UnsupportedExecution` 表示消费方无法安全执行某条已发现的声明；错误保留字段位置和声明来源。调用方应处理该错误，不能把它转换为空计划或“验证通过”。

## 限制：执行支持与构建拒绝

结构图能够解析某种声明，不代表当前验证后端能沿该路径执行它。下表描述当前后端的执行边界：

| 声明或访问形状 | 当前验证后端 |
| --- | --- |
| 具名字段上的 text 约束、自定义 validator | 可绑定并执行；自定义 validator 还需要匹配的注册项 |
| 直接或 `Option` 包装的具名嵌套模型 | 可执行；`None` 跳过内部规则，继续访问要求实际的借用 getter |
| 借用 slice getter 上的显式元素 validator（含嵌套路径） | 可绑定并执行，并检查元素输入类型 |
| 外层 sequence item count | 需要可读取借用 slice 的长度适配器 |
| 外层 `#[sequence(unique_items)]` | 生成的 `Vec<T>` 或 `[T; N]` 配合借用 slice getter 可执行；要求 `T: PartialEq + 'static` 及元素相等性适配器。首次重复报告在 `field[second_index]`，并带有 `first_index` |
| 外层 `#[map(min_entries = ..., max_entries = ...)]` | 生成的可读借用 getter 适配器可统计 `HashMap<K, V>` 或 `BTreeMap<K, V>`；违规位置是字段本身 |
| `#[decimal(...)]` / `#[money(...)]` | 对准确的 `BigDecimal` 值或可选值执行；检查 scale、precision 与精确区间，不舍入 |
| `#[time(precision = ...)]` | 支持 `DateTime<Utc>`、`NaiveDateTime`、`NaiveTime` 及其可选值，检查秒、毫秒、微秒或纳秒精度；`NaiveDate` 在构建时拒绝 |
| selector 内的标准约束或依赖、`MapKey` / `MapValue` | `UnsupportedExecution` |
| 缺少 map 长度或 sequence 相等性适配器、未知集合形状、不支持的时间类型、标量输入类型不匹配 | 在构建阶段返回 `UnsupportedExecution` |
| 透明单字段 tuple `Value` 内部约束 | 可穿过唯一字段绑定并执行；外层可选值为 `None` 时跳过 |
| 含执行声明的 Enum payload、普通 tuple/newtype、容器元素模型内部规则 | `UnsupportedExecution` |
| 存在可达执行声明的递归实例路径 | `UnsupportedExecution`，不会无限展开 |
| unit Enum，以及没有可达执行声明的 payload 或循环 | 可作为普通值通过 |
| `reference` 字段 | 只执行存储字段上的显式规则，不读取被引用的完整 Entity 实例 |
| `opaque` 字段 | 只执行外层声明，不继续遍历内部对象 |
| 后续访问需要借用、但 getter 返回 owned 中间对象 | `UnsupportedExecution` |
| `Option` 或智能指针解包 | 必须有实际适配器；slice 元素需要隐式解包时会拒绝 |

`ValidationCapabilities::check(root, &graph)` 与 `ValidationPlan::build` 使用相同的声明和访问形状检查。能力检查只检查静态声明与可用访问能力，不调用 getter，也不绑定自定义 validator；因此它不能证明运行时访问一定成功，也不能证明注册表中有匹配规则。`ValidationPlan::build` 还会绑定支持的规则：缺少注册项或签名/参数不匹配会产生 `ValidatorBinding(...)` 诊断；缺少访问适配器、访问类型不匹配或某条可达声明落在不支持的路径上，则产生 `UnsupportedExecution`。结构路径可解析但其中有可达执行工作时，计划构建仍会拒绝；没有可达执行工作的包装可作为普通值通过。

遇到构建错误时，先按诊断中的模型、字段/variant 坐标、selector 和完整路径定位声明，再检查 getter 是否返回所需的借用形状、容器或元素适配器是否存在，以及 getter 暴露的 Rust 类型是否与规则输入一致。若是 `ValidatorBinding(...)`，再检查 validator ID、输入类型、参数和依赖注册。不要忽略错误或把失败转成空计划。英文指南包含完整的 [拒绝示例与诊断说明](user_guide.md#limitations-execution-support-and-explicit-refusal)。

## 进阶用法：停止条件与执行预算

使用 `ValidationOptions::default()` 获取默认策略。需要定制时，通过
`ValidationOptions::builder()` 设置 `mode`、`selection` 和所需的 `max_*` 预算，
最后调用 `build()`。独立的 `ValidationOptionsBuilder` 持有配置并在构造结束时转移所有权；
原有的 `with_*` 配置方法已移除。

`ValidationOptions` 默认选择 CollectAll、全部字段，深度上限 64、节点上限 100,000、
报告违规上限 100、比较上限 1,000,000。预算设置都要求 `NonZeroUsize`。
例如只检查实战场景中的嵌套字段：

```rust,ignore
use qubit_model_metadata::validation::{FieldPath, ValidationSelection};

let options = ValidationOptions::builder()
    .selection(ValidationSelection::Fields(vec![
        FieldPath::from_segments(["contact", "name"]),
    ]))
    .build();
let report = plan.validate(ReflectedRef::new(&profile), &options)?;
```

这里的 `plan` 是前文已构建的计划，`profile` 是 `Profile` 实例。只选择完整路径
`contact.name`；仅写 `contact` 不会自动选中其内部规则。构建阶段仍检查所有声明。
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

不要提供替换内置规则 ID 或占用 `qubit_validation_rules::ids::COLLECTION_UNIQUE` 的注册项。
此类冲突返回根级 `InvalidDeclaration`，
可通过 `rule()` 取得冲突 ID。同一次失败构建仍会报告独立的未支持形状和缺失规则；
应处理所有诊断后重新构建计划。
`MatchesDependency` 比对文本不等时产生 `text.dependency_mismatch`；依赖缺失或不是文本时属于执行错误，
不是值违规。

| 现象 | 检查方向 |
| --- | --- |
| 找不到引用目标 | 稳定 ID、已链接 crate、传入注册表的内容 |
| 能力或 Property 冲突 | fallible 查询的 cause 与原始声明来源 |
| 元数据注册冲突 | 查看 `ModelRegistryError::sources()`，按发现顺序取得双方片段身份 |
| 匿名模型不在图中 | 检查传入反射快照；仅静态元数据注册表需要显式子元数据 |
| 父依赖缺失 | 最近父对象优先的实例上下文与 graph 中的类型信息 |
| selector 无执行支持 | ValidationCapabilities 或其他消费后端 |
| codec 缺失或类型不符 | 注册项、显式引用和准确 value type |

`ModelRegistry::metadata_for` 返回 `Result<Option<_>>`：`Ok(None)` 才表示当前注册范围内没有目标；注册或 ABI 错误应保留原始错误。`try_properties_in`、`try_property_in` 保留访问器组装错误；`ResolveErrors` 聚合结构问题并附模型与声明位置。诊断时先确认所用注册表、根模型和链接范围一致，再根据错误中的 owner、字段位置和 cause 修正声明或消费方注册。

需要具体签名和错误类型时看 Rustdoc；设计依据见[运行时设计](design.zh_CN.md)与[派生宏设计](../derive/doc/design.zh_CN.md)。

## 延伸阅读

- [README](../README.zh_CN.md)
- [设计文档](design.zh_CN.md)
- [English user guide](user_guide.md)
- [派生宏声明指南](../derive/doc/user_guide.zh_CN.md)
