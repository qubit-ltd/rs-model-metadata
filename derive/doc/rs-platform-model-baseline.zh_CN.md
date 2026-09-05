# `rs-platform` 模型声明基线

本文记录本次重构完成时 `rs-platform/modules` 中的真实模型声明分布，用于说明兼容性验证覆盖的边界。统计对象为 Rust
源码中的角色 attribute，不包含测试 fixture、生成代码和文档示例。

| 角色 | 声明数 |
| --- | ---: |
| Entity | 37 |
| Projection | 11 |
| Model | 35 |
| Enum | 46 |
| Value | 2 |
| 合计 | 131 |

这些声明已通过 `cargo check --workspace --all-targets`，因此能够证明五种角色、现有字段规则、注册和主要下游依赖迁移闭环。
它不能单独证明所有高级能力均已成熟：当前业务模块没有 `ModelImpl`、validator attribute、codec、key_part、money、
map_key 或 map_value 的真实声明。这些能力必须由 `rs-model-derive` 与 `rs-model-metadata` 的专门 runtime、resolver 和
trybuild 测试验证，不能用下游编译通过替代。

Entity 的默认 `Eq`/`Hash` 是结构语义：全部字段共同参与比较和哈希，不代表按 identifier 比较。可变 Entity 放入
`HashSet` 或作为 `HashMap` key 后，如果任何参与哈希的字段发生变化，将破坏集合不变量。业务代码应避免这种用法；若领域
对象需要身份相等，应显式关闭默认实现并自行定义，或使用稳定 identifier 作为集合 key。
