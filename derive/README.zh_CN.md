# qubit-model-derive

[![Rust CI](https://github.com/qubit-ltd/rs-model-metadata/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-model-metadata/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-model-metadata/coverage-badge.json)](https://qubit-ltd.github.io/rs-model-metadata/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-model-derive.svg?color=blue)](https://crates.io/crates/qubit-model-derive)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

`qubit-model-derive` 把 Rust 领域类型声明编译为 Qubit 模型元数据。它面向需要同时获得 Rust
结构反射和领域语义的应用、框架作者：身份、约束、关系、脱敏、序列化策略与安全属性都从同一份类型声明生成，无需手工维护另一套 schema。

## 安装

本 crate 使用 Rust 1.94 和 edition 2024，清单设置了 `publish = false`。
下例使用宏 crate 与 `qubit-model-metadata` 运行时的本地检出路径：

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
qubit-model-derive = { version = "0.2", path = "../rs-model-metadata/derive" }
qubit-model-metadata = { version = "0.2", path = "../rs-model-metadata" }
qubit-id = "0.7"
```

生成代码会通过 `proc-macro-crate` 解析 `qubit-model-metadata` 的实际依赖名，因此支持重命名 runtime
依赖；只使用生成声明的业务 crate 不必直接依赖 `qubit-reflect`；
调用其公开 API 时，必须声明与 runtime 相同的检出路径。

## 快速开始

以登录服务为例：用户必须有稳定身份，邮箱不能在日志中明文输出，框架还需要发现可写的 `email`
属性。只需声明一次模型：

<!-- example: core/quick-start -->
```rust
use qubit_id::Id;
use qubit_model_derive::{Entity, ModelImpl};
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;

#[Entity(id = "example.User")]
pub struct User {
    #[identifier]
    id: Id,
    #[unique(ignore_case = true)]
    #[redact(level = "high")]
    email: String,
}

#[ModelImpl]
impl User {
    pub fn email(&self) -> &str { &self.email }
    pub fn set_email(&mut self, value: String) { self.email = value; }
}

fn main() {
    let metadata = TypeMetadata::of::<User>();
    assert!(metadata.field("id").unwrap().is_identifier());
    assert!(metadata.try_property("email").unwrap().unwrap().is_writable());
    let registry = ModelRegistry::try_global().expect("链接模型图有效");
    assert!(registry.metadata_for(metadata.descriptor()).unwrap().is_some());
}
```

角色宏会委托 `qubit-reflect` 生成 Rust 结构描述符，再将唯一的 `TypeMetadata` 类型化能力
附加到同一个描述符上。角色默认生成常用 Rust 能力，并将 Debug、Display 与 Serialize 委托 rs-redact。

生成的模型代码使用隐藏的 metadata-only ABI v7 facade。具体模型和泛型定义都通过统一的冻结反射快照发现；模型层不再维护独立 inventory。

`#[key_part(order = n)]` 描述具名 `Model` 或具名 `Value` 的逻辑复合键及字段顺序。逻辑键可以只选择
部分字段，但已选择字段的 order 必须从零开始、连续且不重复。它不是 Entity identifier，因此不能用于
`Entity`、`Projection`、`Enum` 或 tuple/newtype Value。

## 为什么需要这个项目

模型框架既要理解 Rust 结构，也要掌握领域规则。若另行维护 schema，字段名、类型、访问器和约束很容易
逐渐失配。本 crate 直接编译模型声明，并复用唯一的 `qubit-reflect` descriptor，让 metadata 消费者与
业务代码始终基于同一份结构事实。

## 提供的能力

六个属性宏共用解析、规范化、校验和展开流程：

- `#[Entity]`：声明带持久化身份的模型。
- `#[Projection]`：声明实体的开放或固定视图。
- `#[Model]`：声明普通结构化数据。
- `#[Enum]`：声明领域枚举，并保留 Rust 名、canonical 名和 Serde 名。
- `#[Value]`：声明值对象；`transparent` 支持单字段包装类型。
- `#[ModelImpl]`：把公开固有方法中的 getter/setter 与字段合并为安全的属性元数据。

五种角色默认生成 Clone、Debug、Display、PartialEq、Eq、Hash、Redact、Serialize 与 Deserialize； 后续新增 trait 默认保持关闭，须先增加显式宏选项才能启用。
全部 variant 为 unit 的 Enum 还默认生成 Copy。`no_*` 关闭自动能力，`no_eq` 同时移除默认 Hash；
`copy`、`default`、`partial_ord`、`ord` 启用额外能力。角色属性应放在显式 derive 之前，以便识别并去重；
手写实现通过对应的关闭开关避免冲突。

`#[ModelImpl]` 转发反射选项，保留普通方法与 trait impl 的反射。只有公开、安全、同步且方法自身无泛型的
固有访问器贡献 Property。没有同名存储字段的 getter 自动成为 computed Property；
`#[model_property(skip)]` 只排除当前方法的 Property 贡献。泛型运行时适配器沿用反射的显式
`specialize(...)` 绑定。

## 边界

直接通过 `TypeMetadata::of::<T>()` 查询静态元数据不会初始化全局模型注册表；descriptor capability 与
Property 查询会使用冻结的反射快照。只有在所有参与 crate 都已链接后，才使用
`registry::ModelRegistry` 和 `resolve::StructureResolver` 解析稳定 ID、reference、Projection 来源与 Query。
启用 metadata runtime 的 `validation` feature 后，由下游 `ValidationPlan::build` 接收显式的
`qubit-validator::ValidatorRegistry`，负责 validator 绑定与执行。
codec 执行属于独立的可选 adapter：启用 runtime 的 `codec` feature，并以 `qubit-codec` registry
调用 `codec::bind_codecs`。

小写 `#[validator(...)]` 生成经过语法校验的 occurrence；下游 validation plan 会按稳定 ID 绑定已准备的
`qubit-validator` 规则并解析可读依赖。Rust codec 声明只保留稳定 ID 或 Rust 类型身份；codec adapter
负责绑定可执行 descriptor 并校验精确 value type。若多个原始 map key
脱敏后相同，序列化会失败，避免静默覆盖数据。

本 crate 只描述模型语义，不定义物理数据库索引，不负责执行 validator，也不会把 Rust `type_name()`
当作稳定模型身份。相关职责由显式的下游消费者和完成解析的模型图承担。

## 当前执行能力与迁移

| 执行声明 | 当前合同 |
| --- | --- |
| 外层 Map entry count | 生成的 `HashMap`/`BTreeMap` 可读长度适配器；违规报告在字段路径 |
| Decimal / Money | 准确的 `BigDecimal` 及 Option；检查 scale、`DECIMAL(p,s)` precision 和区间，不舍入 |
| Time precision | 支持 `DateTime<Utc>`、`NaiveDateTime`、`NaiveTime` 的秒/毫秒/微秒/纳秒精度；拒绝 `NaiveDate` |
| Option | `None` 跳过内层约束，`Some` 执行；构建时仍检查具体类型 |
| selector 内约束或依赖、MapKey/MapValue、容器内模型 | `UnsupportedExecution`；外层支持不能推导内部遍历能力 |
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
- [中文用户指南](doc/user_guide.zh_CN.md)
- 本地 API 文档：在 crate 根目录运行 `cargo doc --open`
- [最终设计](doc/rs-model-derive-final-design.zh_CN.md)
- [English README](README.md)

以下命令均在 **rs-model-metadata 仓库根目录**运行，不能直接在 `derive` 子目录复制执行；
CI、格式化与覆盖率脚本位于仓库根目录。

## 测试

```bash
# 使用默认 feature 集运行测试
cargo test --workspace --locked

# 使用项目声明的全部 feature 运行测试
cargo test --workspace --all-features --locked

# 只测试 derive package
cargo test -p qubit-model-derive --all-features --locked

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

仓库地址：[https://github.com/qubit-ltd/rs-model-metadata/tree/main/derive](https://github.com/qubit-ltd/rs-model-metadata/tree/main/derive)
