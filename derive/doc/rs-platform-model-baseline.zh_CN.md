# `rs-platform` 模型声明基线

本表于 2026-09-10 按 `rs-platform/modules/**/*.rs` 中行首完整角色属性重新采集，
不把 ModelImpl 计为 Model，也不计文档中的示例。迁移前后实际角色数量均为 131；
迁移前宽松匹配得到的 133 包含另外 2 个 ModelImpl，并不是新增了两个模型。

| 角色 | 声明数 |
| --- | ---: |
| Entity | 37 |
| Projection | 11 |
| Model | 35 |
| Enum | 46 |
| Value | 2 |
| 合计 | 131 |

已验证 `cargo check --offline --workspace --all-features --all-targets`、完整关系图解析和一轮完整 `ci-check.sh`。
最终 CI 与完整关系图结果见需求验收台账，编译结果不代替执行语义验收。
迁移后另有 20 个 ModelImpl 和 4 个 validator 声明；没有 codec、key_part、money、map_key 或 map_value
的真实声明。高级能力仍由模型库的 runtime、resolver、trybuild 测试独立覆盖。

迁移覆盖全部 131 个模型文件：使用角色默认能力，保留 Default/必要 Copy 和手写实现的关闭开关；
5 个文件中的 reference.path 改为斜线；4 处 validator 依赖改为独立对象导航与属性选择语法；
移除 validate_nested 标记，以 metadata 和消费者遍历边界表达嵌套验证。
非文本 Phone、CredentialInfo 的 unique 保留值唯一性，删除不适用的 ignore_case 参数。

Entity 的默认 `Eq`/`Hash` 是结构语义：全部字段共同参与比较和哈希，不代表按 identifier 比较。可变 Entity 放入
`HashSet` 或作为 `HashMap` key 后，如果任何参与哈希的字段发生变化，将破坏集合不变量。业务代码应避免这种用法；若领域
对象需要身份相等，应显式关闭默认实现并自行定义，或使用稳定 identifier 作为集合 key。

新增的 18 个 ModelImpl 暴露现有或补齐的 info getter，供跨模型 reference.path 使用；
补齐 Device、Hardware、Person、Organization 中 Entity/Projection 存储字段的 reference 声明。
