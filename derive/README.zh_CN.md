# qubit-model-derive

[![Rust CI](https://github.com/qubit-ltd/rs-model-metadata/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-model-metadata/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-model-metadata/coverage-badge.json)](https://qubit-ltd.github.io/rs-model-metadata/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-model-derive.svg?color=blue)](https://crates.io/crates/qubit-model-derive)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

`qubit-model-derive` 给 Rust 领域类型补充业务语义。角色宏与字段标注把同一份类型声明
变成可供其他库读取的元数据：标识、唯一性、约束、引用、查询索引等。
`qubit-model-metadata` 在 `qubit-reflect` 提供的结构反射基础上读取并解析这些声明。

## 安装

<!-- example: core -->
```toml
[dependencies]
qubit-model-metadata = { version = "0.2", path = "../rs-model-metadata", default-features = false }
qubit-model-derive = { version = "0.2", path = "../rs-model-metadata/derive" }
qubit-id = "0.7"
qubit-reflect = { version = "0.2", path = "../rs-reflect" }
```

示例假定应用 crate 位于 `rs-platform/app`，与 `rs-model-metadata`、`rs-reflect`
同级；在该应用目录运行完整示例。metadata、model-id 和 derive 的 manifest 均设置了
`publish = false`。版本字段标识包版本，实际来源由对应的检出路径确定。完整目录布局和
validation 安装方式见运行时指南。

## 示例：账号与人员资料

<!-- example: core/quick-start -->
```rust
use qubit_id::Id;
use qubit_model_derive::Entity;
use qubit_model_metadata::metadata::TypeMetadata;

#[Entity(id = "guide.iam.User")]
struct User {
    #[identifier]
    id: Id,
    #[unique(ignore_case = true)]
    #[text(non_blank, min_chars = 3, max_chars = 64, allowed_chars = ascii)]
    username: String,
    #[indexed]
    #[text(max_chars = 128)]
    display_name: String,
}

#[Entity(id = "guide.person.PersonInfo")]
struct PersonInfo {
    #[identifier]
    id: Id,
    #[reference(entity = "guide.iam.User", property = id)]
    user_id: Option<Id>,
    #[text(non_blank, max_chars = 128)]
    name: String,
}

fn main() {
    let user_metadata = TypeMetadata::of::<User>();
    let username_field = user_metadata.field("username").unwrap();
    assert!(username_field.is_unique());
    assert_eq!(username_field.text_constraint().unwrap().max_chars(), Some(64));

    let person_metadata = TypeMetadata::of::<PersonInfo>();
    assert!(person_metadata.field("user_id").unwrap().reference().is_some());
}
```

标注把业务规则放在对应字段旁。校验消费者可检查文本长度，并查询数据库确认唯一性；约束感知的数据生成器
可为 `PersonInfo` 选取已有用户的 ID；REST 查询层可依据 `indexed` 判断允许哪些过滤条件。
宏记录声明，消费者决定如何执行。

## 声明速览

| 声明 | 含义 |
| --- | --- |
| `Entity`、`Projection`、`Model`、`Value`、`Enum` | 选择类型的领域角色，并生成反射和元数据。 |
| `identifier`、`key_part`、`unique`、`indexed`、`reference` | 描述身份、逻辑键、唯一性、查询能力和关系。 |
| `text`、`number`、`collection`、`validator`、`selector` | 描述内置约束与自定义规则绑定。 |
| `codec`、`redact`、Serde 选项 | 描述值转换、输出脱敏和序列化行为。 |
| `ModelImpl` | 将符合条件的 getter 和 setter 暴露为模型属性。 |

局部非法声明在宏展开时失败。跨模型目标与关系由 `StructureResolver` 从
`ModelRegistry` 构建模型图时检查；可执行规则另行绑定。语法与示例见[声明手册](doc/user_guide.zh_CN.md)，
元数据的消费方式见[运行时手册](../doc/user_guide.zh_CN.md)。

## 延伸阅读

- [English user guide](doc/user_guide.md)
- [中文用户指南](doc/user_guide.zh_CN.md)
- 本地 API 文档：在 crate 根目录运行 `cargo doc --open`
- [Design](doc/design.md) · [设计文档](doc/design.zh_CN.md)
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
