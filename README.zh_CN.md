# qubit-model-metadata

[![Rust CI](https://github.com/qubit-ltd/rs-model-metadata/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-model-metadata/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-model-metadata/coverage-badge.json)](https://qubit-ltd.github.io/rs-model-metadata/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-model-metadata.svg?color=blue)](https://crates.io/crates/qubit-model-metadata)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

`qubit-model-metadata` 为已由 `qubit-reflect` 描述结构的 Rust 类型补充稳定的领域语义。
它适合框架与应用开发者：通过生成的模型角色、字段语义和稳定 ID，在多个已链接的模型 crate
之间显式解析关系，同时避免另起一套反射系统。

## 安装

运行时 crate 需要 Rust 1.94，使用 edition 2024。metadata、model-id 和 derive crate 均设置了
`publish = false`，因此应从应用 crate 使用 platform 工作区中的本地检出：

示例统一使用以下检出布局，应用命令在 `rs-platform/app` 中执行。
直接依赖必须与 runtime 使用同一份检出路径：

```text
checkout/
  rs-platform/
    app/                 # Cargo.toml 与 src/main.rs
    rs-model-metadata/   # runtime、model-id/ 与 derive/
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

仅传递或保存稳定模型身份的库，可以直接依赖轻量的 `qubit-model-id`，无需完整 metadata 运行时：

<!-- example: core/model-id -->
```toml
[dependencies]
qubit-model-id = { version = "0.1", path = "../rs-model-metadata/model-id" }
```

身份协议类型为 `qubit_model_id::{ModelId, ModelIdBuf, ModelIdError}`。
模型声明和注册表访问仍依赖 `qubit-model-metadata`；其 `metadata` 模块重导出相同的 ID 类型。
具名、非泛型角色声明显式提供 `id` 时，派生宏还会实现
`qubit_model_id::HasModelId`；泛型定义和匿名模型没有具体模型 ID。

<!-- example: validation -->
```toml
[dependencies]
qubit-model-metadata = { version = "0.2", path = "../rs-model-metadata", features = ["validation"] }
qubit-model-derive = { version = "0.2", path = "../rs-model-metadata/derive" }
qubit-reflect = { version = "0.2", path = "../rs-reflect" }
qubit-validator = { version = "0.1.0", path = "../../rust-common/rs-validator" }
```

`qubit-id` 提供 `Entity` 和 `Projection` 标识字段必须使用的 `Id` 类型。默认 feature 集为空；
按需启用 `codec` 或 `validation` 执行适配器，以及提供泛型定义元数据的 `generic`。

## 快速开始

账户服务只需声明一次账户类型，就能在无需维护第二套模型注册流程的前提下读取模型元数据。派生宏生成
角色感知的元数据，`TypeMetadata` 则通过 `qubit-reflect` 采用的同一个 `TypeDescriptor` 暴露它。

<!-- example: core/quick-start -->
```rust
use qubit_model_derive::Entity;
use qubit_id::Id;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;

#[Entity(id = "example.User")]
struct User {
    #[identifier]
    id: Id,
    #[unique(ignore_case = true)]
    email: String,
}

fn main() {
    let metadata = TypeMetadata::of::<User>();
    assert_eq!(metadata.model_id().unwrap().as_str(), "example.User");
    let registry = ModelRegistry::try_global().expect("链接模型图有效");
    assert!(registry.metadata_for(metadata.descriptor()).unwrap().is_some());
}
```

得到的是 `User` 的静态元数据；`TypeMetadata::of` 不会初始化全局模型注册表。
`ModelRegistry::from_static_metadata` 根据显式传入的元数据构造隔离注册表，查询 Property 时
只使用 `TypeMetadata::local_properties()`。若通过 `ModelRegistry::from_reflect_registry`
投影冻结的 `ReflectRegistry` 快照，`metadata_for` 和 `properties_for` 则以该快照为准，
能够读取独立注册的 `ModelImpl` provider。显式 Property 查询返回拥有型视图，动态合并数据随视图释放；
`ModelRegistry` 的缓存只在 registry 生命周期内复用。跨 crate 关系的解析流程请参阅用户指南。

## 为什么需要这个项目

反射可以回答类型有哪些字段、字段使用什么 Rust 类型等结构问题。领域模型还需要标识符、约束、
引用、Property、角色和可持久化的模型 ID 等信息。本 crate 将这些语义附着在反射 descriptor 上，
而不是重复维护一份反射模型。

## 核心能力

- `qubit-model-derive` 可为 `#[Entity]`、`#[Projection]`、`#[Model]`、`#[Enum]`、`#[Value]`
与 `#[ModelImpl]` 声明生成 metadata。
- `metadata` 模块拥有与执行引擎无关的声明词汇，包括 `TypeMetadata`、`ModelId`、codec 引用、
  validation 参数和脱敏敏感度。
- `registry::ModelRegistry` 可从冻结的 `ReflectRegistry` 快照投影具体模型，也可从显式静态元数据
  构造隔离注册表。启用 `generic` 后，可通过 `generic_metadata_for` 按进程内的
  `TypeDefinitionId` 查询泛型定义；`generic_definitions()` 同时列出有 ID 和匿名定义。
  `entries()` 只包含具有稳定 `ModelId` 的注册项，因此不列出匿名泛型定义。
- `resolve::StructureResolver` 只解析结构关系并生成不可变的 `resolve::ModelGraph`。
- 启用 `codec` 后，`codec::bind_codecs` 在图构建后绑定 codec occurrence，并确定性地汇总错误。
- 启用 `validation` feature 后，`ValidationPlan::build` 编译 Property 路径，并绑定调用方提供的
  `qubit-validator::ValidatorRegistry`；`ValidationPlan::validate` 执行这些不可变绑定并返回结构化的
  `ValidationReport`。构建诊断以 `ConstraintRuleRef::Registry` 表示经注册表绑定的规则，
  以 `ConstraintRuleRef::ModelIntrinsic` 表示由模型计划直接执行的规则。
  外层序列唯一性使用保留 ID `qubit_validation_rules::ids::COLLECTION_UNIQUE`；
  自定义 validator 注册该 ID 会使计划构建返回 `InvalidDeclaration`。
  原 `constraint_rule_ids()` 方法已由 `constraint_rules()` 取代；每个引用仍可通过
  `.id().as_str()` 取得稳定的规则 ID。
  借用的可选模型路径遇到缺失值会跳过，并保留完整 Property 路径；受支持的终端
  `Option<String>` text 规则会跳过 `None`、校验 `Some`。准备好的模型级规则可以通过一次有序的
  `with_model_rules` 调用追加。
- 结构、codec 和 validation 错误分别由其所属层返回；resolver 不创建任何可执行绑定。

本 crate 不会取代 `qubit-reflect`，静态元数据查询也不会隐式注册模型或解析跨模型关系。生成的
metadata 在穿过隐藏的 metadata-only ABI v7 边界前，会校验 descriptor、Field、Property 和角色
不变量；生成代码只依赖经过收窄的模块 facade 及其精确私有 ABI。

全局入口 `ModelRegistry::try_global()` 表示完整的链接注册集合；发生冲突时，整体初始化会失败，source chain 会保留反射错误及 capability conflict。需要隔离模型视图时，只将所需描述符加入 `ReflectRegistry` 快照，再把同一个快照传给 `ModelRegistry::from_reflect_registry`：

下列片段使用安装方案已声明的 `qubit-reflect` 依赖，要求应用提供 `MyModel`，
并放在返回 `Result` 的函数中。

```rust,ignore
let mut builder = qubit_reflect::registry::RegistrySnapshotBuilder::new();
builder.add_type(
    qubit_reflect::TypeDescriptor::of::<MyModel>(),
    qubit_reflect::identity::FragmentIdentity::new("example", "models", 1, 1, "type", 1),
);
let snapshot = builder.build()?;
let models = ModelRegistry::from_reflect_registry(&snapshot)?;
```

显式快照从空集合开始，不会自动包含进程中链接的所有注册信息。不要使用旧的隐藏 testing registry helper。

具备相应 getter 和适配器时，验证计划可执行外层 sequence 数量与去重、外层 map entry 数量，
以及支持类型上的 `BigDecimal` 和 chrono 时间约束。可选值缺失时跳过执行；不支持的 selector 遍历、
Enum payload、缺失的集合适配器或不匹配的值类型会在构建阶段明确失败。
FailFast 和报告上限会停止整个计划，基础执行错误则保留部分报告。
具体边界见用户指南的[执行矩阵](doc/user_guide.zh_CN.md#限制执行范围与构建拒绝)与
[API 迁移说明](doc/user_guide.zh_CN.md#进阶用法字段身份与-api-迁移)。

## 可恢复查询

`ModelRegistry::metadata_for` 返回 `Result<Option<&TypeMetadata>, ModelMetadataError>`。
注册表构建失败时，`ModelRegistryError::origins()` 会保留相关能力来自类型内建能力还是注册片段，
并为注册能力保留精确的片段身份；`sources()` 仍可用于直接检查片段。
`Ok(None)` 表示没有匹配的模型元数据；能力冲突和描述符 ABI 不匹配返回结构化错误。
`TypeMetadata::try_properties_in`、`try_property_in`、`property_fragments_in` 传播
`PropertyResolutionError`，显式 snapshot 查询不会初始化全局注册表。
解析器通过 `ResolveError::cause()` 保留原始错误，并附加模型、属性路径和来源。
独立错误继续聚合，基础失败不会被改写成属性缺失。

### 动态属性路径

使用 `PropertyAccessPath::compile(&registry, root, segments)` 编译请求提供的属性段，
再通过 `read` 或 `write` 访问实例。读取只穿过可借用属性，并支持可投影的 Option 中间节点；
缺失值返回 `MissingIntermediate`。写入要求中间段都是普通字段。转换输入前先调用
`check_writable()`，并可通过 `leaf_property()` 查看末端属性声明类型。关于对象路径与来源信息，参阅
[用户指南对应章节](doc/user_guide.zh_CN.md#进阶用法对象路径引用与声明位置)。

每个 `qubit.model.metadata.v1` 能力目标也必须是反射快照的成员；缺少成员声明时，
`ModelRegistry::from_reflect_registry` 会在调用任何模型 provider 前返回 `UnregisteredModelTarget`。
快照投影的 `ModelEntry::source()` 和 `ModelRegistry::source()` 指向元数据能力片段；
`ModelEntry::declaration_source()` 指向反射类型或泛型定义片段。静态元数据条目的该方法返回 `None`。

借用切片可按索引直接读取。显式 `into_invocation_output` 会用 O(n) 时间物化元素借用包装，
不复制底层元素；切片 adapter 自身也需要一次装箱。该转换的成本与原始访问分别测量。

## 声明默认能力与显式根

角色宏默认生成 Clone、Debug、Display、PartialEq、Eq、Hash、Redact、Serialize 和 Deserialize。
有意关闭某项能力时使用 `no_*`；`no_eq` 同时关闭默认 Hash。Copy、Default 和排序能力通过
`copy`、`default`、`partial_ord`、`ord` 启用。具名 Option 与标准集合字段支持缺失默认值和空值省略。

需要审计链接注册表中的全部模型时使用 `new`；业务请求只需要根模型及其可达模型时使用
`for_roots`。两种调用都借用同一个 `ModelRegistry` 快照：按根解析会跳过无关模型的结构初始化，
但可达模型仍从该快照读取 capability 和 `ModelImpl` Property provider。下面的局部验证示例需要启用
`validation` feature，并直接依赖 `qubit-validator`；依赖配置见用户指南。

<!-- example: validation/root-scoped -->
```rust
use std::sync::Arc;
use qubit_model_derive::Model;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_model_metadata::validation::ValidationBuildInputs;
use qubit_model_metadata::validation::ValidationOptions;
use qubit_model_metadata::validation::ValidationPlan;
use qubit_reflect::ReflectedRef;
use qubit_validator::ValidatorRegistry;

#[Model]
struct Request {
    #[text(non_blank)]
    input: String,
}

fn main() {
    let models = ModelRegistry::try_global().expect("链接注册表有效");
    let _complete = StructureResolver::new(ResolveInputs { models: &models, roots: &[] })
        .resolve().expect("审计所有已链接模型");

    let root = TypeMetadata::of::<Request>();
    let roots = [root];
    let local = Arc::new(StructureResolver::for_roots(ResolveInputs { models: &models, roots: &roots })
        .resolve().expect("解析请求所需的模型结构"));
    assert!(local.model(root.type_id()).is_some());
    let validators = ValidatorRegistry::empty();
    let plan = ValidationPlan::build(root, ValidationBuildInputs {
        graph: Arc::clone(&local),
        validators: &validators,
    }).expect("构建请求验证计划");
    let report = plan.validate(
        ReflectedRef::new(&Request { input: String::new() }),
        &ValidationOptions::default(),
    ).expect("执行请求验证");
    assert!(!report.is_valid());
}
```

基于反射快照的注册表会通过同一快照发现根可达的匿名模型。通过
`ModelRegistry::from_static_metadata` 构造的静态元数据注册表没有反射发现能力，必须显式提供
所有可达子模型的元数据。需要处理生成元数据 ABI 错误时，可调用
`TypeMetadata::try_of::<T>()` 并接收 `AbiViolation`；若 ABI 校验失败，`TypeMetadata::of::<T>()`
会 panic。无效显式根和注册项会在解析或注册表构造期间报告。
泛型具体类型即使没有稳定 ID，也保留泛型定义关联。`QueryMetadata::declarations()` 返回直接 indexed
声明及 identifier、unique、reference 等隐含原因。filter 生成和匹配规则由消费者设计。
`reference.path` 使用 `/` 和 `..`；普通 Property 路径仍使用 `.`。

## 当前执行能力与迁移

| 执行声明 | 当前合同 |
| --- | --- |
| 外层 Map entry count | 生成的 `HashMap`/`BTreeMap` 可读长度适配器；违规报告在字段路径 |
| Decimal / Money | 准确的 `BigDecimal` 及 Option；检查 scale、`DECIMAL(p,s)` precision 和区间，不舍入 |
| Time precision | 支持 `DateTime<Utc>`、`NaiveDateTime`、`NaiveTime` 的秒/毫秒/微秒/纳秒精度；拒绝 `NaiveDate` |
| Option | 借用的 `Option<T>` 中间对象遇到 `None` 会跳过，遇到 `Some` 会继续访问；终端 `Option<String>` text 规则跳过 `None`、校验 `Some`；构建时仍检查具体类型 |
| selector 内约束或依赖、MapKey/MapValue、tuple 路径、含约束的 Enum payload、容器内模型、返回 owned 中间对象且还需继续遍历的 getter | `UnsupportedExecution`；外层支持不能推导内部遍历能力 |
| Enum/raw wrapper 与递归路径 | 发现可达工作后，不支持的使用路径在能力检查和计划构建时明确拒绝；无工作包装可以通过 |

基于反射快照的注册表能从根发现可达匿名子模型，包括 raw reflection wrapper 内的模型；
`ResolveInputs.roots` 只需传入根，发现范围始终受传入快照限制。仅静态元数据注册表需要显式子元数据，
不会引入反射能力。单字段 tuple `Value` 与具名 Value 遵循相同的值闭包检查；`transparent` 只控制表示，
不保证内部约束可执行。Entity 的角色检查同样穿过 tuple 字段：没有显式引用的 `(InnerEntity,)`
返回 `InvalidEntityNesting`。newtype Value 不能在值闭包中隐藏 Model/Entity/Projection、引用、
未解析描述符或 raw struct（`InvalidValueClosure`）；基本值及合法 Value/Enum 闭包仍可通过。
无名载荷字段不会成为具名 Property。`ModelImpl` 的 provider、签名和访问适配器与方法/impl 的 `cfg` 及嵌套
`cfg_attr` 同步启用；互斥访问器可用，同时启用的冲突组合仍会诊断。

## 延伸阅读

- [English user guide](doc/user_guide.md)
- [简体中文用户指南](doc/user_guide.zh_CN.md)
- [`qubit-model-derive` 声明指南](derive/doc/user_guide.zh_CN.md)
- [覆盖率测量与工具复现检查](doc/coverage_measurement.zh_CN.md)
- 配套的 derive crate 位于本仓库的 [`derive/`](derive/) workspace member 中。
- 本地 API 文档：运行 `cargo doc --open`
- [English README](README.md)

## 测试

```bash
# 使用默认 feature 集运行测试
cargo test --workspace --locked

# 使用项目声明的全部 feature 运行测试
cargo test --workspace --all-features --locked

# 运行项目 CI 检查
./.infra/bin/ci-check.sh

# 检查代码覆盖率
./.infra/bin/coverage.sh
```

## 许可证

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

本项目基于 Apache License 2.0 授权。完整许可证文本请参阅
[LICENSE](LICENSE)。

## 贡献

欢迎贡献。请遵循 Rust API 指南，及时更新公共 API 文档与测试，并在提交
Pull Request 前运行 `./.infra/bin/align-ci.sh`格式化代码，运行`./.infra/bin/ci-check.sh`对齐CI要求。

## 作者

**Haixing Hu** - *Qubit Co. Ltd.*

仓库地址：[https://github.com/qubit-ltd/rs-model-metadata](https://github.com/qubit-ltd/rs-model-metadata)
