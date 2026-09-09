# 模型元数据使用指南

[English](user_guide.md) · [README](../README.zh_CN.md) · [声明指南](../derive/doc/user_guide.zh_CN.md)

本指南面向框架开发者，适用于工作区 0.1 契约。以用户目录为例，先读取模型结构和 indexed 声明，
再按需显式绑定执行服务。项目要求 Rust 1.94、edition 2024，路径依赖见 README。
默认 feature 集为空；generic、validation、codec 分别启用对应 API。

## 身份、字段与结构图

Field 表示存储槽位，Property 合并存储字段和符合要求的访问器，两者复用 rs-reflect descriptor。
TypeMetadata 是不依赖模型实例的静态信息。ModelId 可省略：匿名模型仍有准确的 TypeId，
可以作为显式根纳入结构解析。

```rust
use qubit_id::Id;
use qubit_model_derive::Entity;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::{ResolveInputs, StructureResolver};

#[Entity]
struct User {
    #[identifier]
    id: Id,
    #[indexed]
    nickname: String,
}

let root = TypeMetadata::of::<User>();
assert!(root.model_id().is_none());
let models = ModelRegistry::from_metadata(&[]).unwrap();
let roots = [root];
let graph = StructureResolver::new(ResolveInputs { models: &models, roots: &roots })
    .resolve().unwrap();
let query = graph.query(root.as_entity().unwrap()).unwrap();
assert_eq!(query.declarations().len(), 2);
assert_eq!(query.declarations()[1].path().segments(), &["nickname"]);
```

查询视图包含 id 和 nickname，因为 identifier 也贡献 indexed 原因。每条 QueryDeclaration 保留来源 Field、
路径、索引原因，并可通过 Field 读取类型。这里不会展开 filter，也不检查平面字段名是否冲突。
未来的 filter crate 可以选择 nickname 子串匹配，以及 age、时间字段的上下界参数；这些是下游策略。

已链接的模型 crate 可以使用 `ModelRegistry::try_global()`，也可以先构造显式 rs-reflect 快照，再通过
`ModelRegistry::from_reflect_registry` 投影。冻结注册表前应完成所需 crate 的链接。
解析器遍历注册类型与显式根的并集，按 TypeId 去重，检查引用、Projection 来源、Property 冲突和 Value 闭包。
显式注册表不会从全局注册表补入未提供的注册项。泛型具体 metadata 始终保留定义关联，
只有具有稳定 ID 的定义进入按 ID 枚举的注册表；并发重复查询同一具体类型会共享 metadata 分配。

## 对象路径、引用与声明位置

ObjectPath 使用 NavigationStep::Property 和 NavigationStep::Parent，显示为 `/` 分隔路径；
PropertyPath 使用 `.` 选择普通属性。依赖的空 ObjectPath 表示当前对象，reference 省略 path 则表示未请求复用。
普通容器不增加领域父对象。

引用绑定路径经过只保存 ID 或 Projection 的字段时，仍导航所绑定的完整 Entity；目标 property 单独选择。
依赖父对象的 reference、validator 声明会保留 ContextRequirement::ParentObject。
`ModelGraph::dependencies()` 保存各自独立的依赖 occurrence。DeclarationLocation 保存文件、行列、
owner 名称、variant/field 序号与 selector 位置，无名 Enum payload 也能定位。

## 显式绑定执行适配器

启用 validation 后，调用 `ValidationPlan::build(root, ValidationBuildInputs { graph: &graph,
validators: &validators })`，传入自己的 validator registry。
绑定检查稳定 ID、参数、可读 Property 路径及已知的输入、依赖类型。同 ID 声明分别绑定，不会互相覆盖。
标准约束使用现有 validation-rules 适配器。

父依赖可通过 `build_with_context` 提供类型 metadata，通过 `validate_with_context` 提供借用实例，
两者都按“最近父对象优先”排列。绑定时也允许暂缺父类型；执行前应把父模型纳入 graph，
执行器才能解析并核对延后的路径后缀。缺少父对象会返回结构化依赖错误，不能当作 Option::None。
即使计划为空，传入错误 Rust 类型的根实例也会被拒绝。

计划只读，不修改对象。ValidationOptions 控制字段选择、快速失败和遍历预算。
当前支持边界内的直接、Option 嵌套模型 validator 自动纳入计划，opaque 截断遍历。
元数据能描述的范围大于某个执行后端：当前集合适配器通过借用切片 getter 支持 element validator；
不支持的 selector 位置或约束适配器会明确返回构建错误。Map 声明仍可供其他消费者使用。
选择后端前检查 ValidationCapabilities，不应把“能够声明”理解成“所有后端都能执行”。

启用 codec 后，在结构解析完成后使用 CodecBindInputs 与显式 codec registry 调用 `codec::bind_codecs`。
选择顺序为字段显式 codec、Value canonical codec、无 codec。Rust 类型形式也要求相应注册项存在；
显式指定同一个 canonical codec 合法。occurrence 身份包含准确 TypeId、可选 ModelId、Property 路径和来源。

## 错误处理与排查

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
私有 checked v6 门面，应用应使用公开 API。

数据库访问、随机对象创建、物理索引、filter 生成、Unicode 比较执行，以及父对象缺失时的业务回退，
仍由下游组件实现。完整契约见[冻结需求](../derive/doc/rs-model-derive-requirements.zh_CN.md)、
[最终设计](../derive/doc/rs-model-derive-final-design.zh_CN.md)。
运行 `cargo doc --workspace --all-features --no-deps` 可生成本地 API 文档。
