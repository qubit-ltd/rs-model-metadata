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

运行时 crate 需要 Rust 1.94，使用 edition 2024。以下示例假设所需 crate 已发布到 crates.io：

```toml
[dependencies]
qubit-model-metadata = { version = "0.1", default-features = false }
qubit-model-derive = "0.1"
qubit-id = "0.6.0"
```

`qubit-id` 提供 `Entity` 和 `Projection` 标识字段必须使用的 `Id` 类型。默认 feature 集为空；
按需启用 `codec` 或 `validation` 执行适配器，以及提供泛型定义元数据的 `generic`。

## 快速开始

账户服务只需声明一次账户类型，就能在无需维护第二套模型注册流程的前提下读取模型元数据。派生宏生成
角色感知的元数据，`TypeMetadata` 则通过 `qubit-reflect` 采用的同一个 `TypeDescriptor` 暴露它。

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

得到的是 `User` 的静态元数据；`TypeMetadata::of` 不会初始化全局模型注册表。从冻结的
`ReflectRegistry` 构造 `ModelRegistry` 后，其 `metadata_for` 和 `properties_for` 查询会使用
该快照，以便看见独立生成的模型 overlay。跨 crate 关系的解析流程请参阅用户指南。

## 为什么需要这个项目

反射可以回答类型有哪些字段、字段使用什么 Rust 类型等结构问题。领域模型还需要标识符、约束、
引用、Property、角色和可持久化的模型 ID 等信息。本 crate 将这些语义附着在反射 descriptor 上，
而不是重复维护一份反射模型。

## 核心能力

- `qubit-model-derive` 可为 `#[Entity]`、`#[Projection]`、`#[Model]`、`#[Enum]`、`#[Value]`
与 `#[ModelImpl]` 声明生成 metadata。
- `metadata` 模块拥有与执行引擎无关的声明词汇，包括 `TypeMetadata`、`ModelId`、codec 引用、
  validation 参数和脱敏敏感度。
- `registry::ModelRegistry` 从冻结的 `ReflectRegistry` 快照投影具体模型；启用 `generic` 后才包含泛型定义。
- `resolve::StructureResolver` 只解析结构关系并生成不可变的 `resolve::ModelGraph`。
- 启用 `codec` 后，`codec::bind_codecs` 在图构建后绑定 codec occurrence，并确定性地汇总错误。
- 启用 `validation` feature 后，`ValidationPlan::build` 编译 Property 路径，并绑定调用方提供的
  `qubit-validator::ValidatorRegistry`；`ValidationPlan::validate` 执行这些不可变绑定并返回结构化的
  `ValidationReport`。
- 结构、codec 和 validation 错误分别由其所属层返回；resolver 不创建任何可执行绑定。

本 crate 不会取代 `qubit-reflect`，静态元数据查询也不会隐式注册模型或解析跨模型关系。生成的
metadata 在穿过隐藏的 metadata-only ABI v7 边界前，会校验 descriptor、Field、Property 和角色
不变量；生成代码只依赖经过收窄的模块 facade 及其精确私有 ABI。

需要隔离反射上下文时，应使用 `qubit-reflect` 的 `RegistrySnapshotBuilder` 构造快照，再传给显式的 `*_in` 查询；不要继续使用旧的隐藏 testing registry helper。全局初始化失败仍保持结构化错误：`ModelRegistry::try_global()` 的 source chain 会保留反射 registry 错误及其 capability conflict。

验证计划会收集受支持的各处嵌套声明，并明确拒绝不支持的执行形状；Enum、Time 等声明结构合法，
不代表后端能够执行。FailFast 和报告上限会停止整个计划，基础执行错误则保留部分报告。
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

借用切片可按索引直接读取。显式 `into_invocation_output` 会用 O(n) 时间物化元素借用包装，
不复制底层元素；切片 adapter 自身也需要一次装箱。该转换的成本与原始访问分别测量。

## 声明默认能力与显式根

角色宏默认生成 Clone、Debug、Display、PartialEq、Eq、Hash、Redact、Serialize 和 Deserialize。
有意关闭某项能力时使用 `no_*`；`no_eq` 同时关闭默认 Hash。Copy、Default 和排序能力通过
`copy`、`default`、`partial_ord`、`ord` 启用。具名 Option 与标准集合字段支持缺失默认值和空值省略。

无 ID 模型通过 `ResolveInputs { models: &registry, roots: &[metadata] }` 纳入结构图。
泛型具体类型即使没有稳定 ID，也保留泛型定义关联。`QueryMetadata::declarations()` 返回直接 indexed
声明及 identifier、unique、reference 等隐含原因。filter 生成和匹配规则由消费者设计。
`reference.path` 使用 `/` 和 `..`；普通 Property 路径仍使用 `.`。

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
cargo test

# 使用项目声明的全部 feature 运行测试
cargo test --all-features

# 运行项目 CI 检查
./ci-check.sh

# 检查代码覆盖率
./coverage.sh
```

## 许可证

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

本项目基于 Apache License 2.0 授权。完整许可证文本请参阅
[LICENSE](LICENSE)。

## 贡献

欢迎贡献。请遵循 Rust API 指南，及时更新公共 API 文档与测试，并在提交
Pull Request 前运行 `./align-ci.sh`格式化代码，运行`./ci-check.sh`对齐CI要求。

## 作者

**Haixing Hu** - *Qubit Co. Ltd.*

仓库地址：[https://github.com/qubit-ltd/rs-model-metadata](https://github.com/qubit-ltd/rs-model-metadata)
