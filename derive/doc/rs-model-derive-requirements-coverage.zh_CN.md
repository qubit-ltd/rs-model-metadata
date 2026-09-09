# 需求归属、实现评估与验收台账

- 基准：用户已确认的 2026-09-10 [冻结需求](rs-model-derive-requirements.zh_CN.md)。
- 本轮完成全部 338 个编号的归属、实施必要性与任务映射；源码证据按[缺口报告](rs-model-implementation-gaps.zh_CN.md)中的 G01～G15 分组。
- 已运行报告中的限定基线，38 项通过；不等于新需求全部通过。未运行全量 trybuild、Clippy、coverage 和真实下游编译。
- “已有基础；完整边界需专项验收”明确保留证据不足，不宣称实现完整；“源码证实缺口/冲突”指报告中的明确代码分支。
- 必做/复用表示满足冻结契约，优先复用已有实现；D 是下游责任，R 是参考或废弃，不是漏排的本库任务。
- 本表不是通过率。每项的实际用例、命令和交付结果由 T8 汇总；不能以任务映射替代测试证据。

## 归属统计

| 归属 | 条目数 |
| --- | ---: |
| 本库（含反射/输出集成） | 283 |
| 本库声明/结构与执行组件协作 | 16 |
| 下游组件 | 20 |
| 下游设计参考 | 10 |
| 废弃 | 1 |
| 文档与交付验收 | 8 |
| 合计 | 338 |

## 逐项评估

摘要仅供定位，完整语义以规范为准。T1～T8 的职责和验收命令见[实施计划](../../doc/plans/2026-09-10-model-metadata-plan.md)。

| 需求 ID | 定位摘要 | 验收归属 | 实现评估 | 必要性与任务 | 证据入口 |
| --- | --- | --- | --- | --- | --- |
| `REQ-SYS-001` | 系统必须以强类型、不可变、可静态共享的 metadata 表达模型语义，不得使用任意字符串键值表作为 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-SYS-002` | qubit-model-metadata 不得依赖 qubit-model-derive；过程宏展开后的代码可以引用 metadata crate。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-SYS-003` | 过程宏必须只根据 token、Rust 类型约束和生成的链接期注册项工作，不得在宏执行时读取数据库、 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-SYS-004` | schema、validation、随机生成、DAO 测试和查询生成器必须消费同一规范化 metadata；宏输入简写不得 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-SYS-005` | 模型 metadata 必须与对象实例分离；读取 metadata 不应要求构造对象。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-SYS-006` | 同一模型事实在不同组件中的解释必须一致。例如 reference 隐含 indexed、opaque 截断递归、 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-SYS-007` | 可以在当前声明内判定的问题必须在编译期报告；只有跨 crate、稳定 ID 或完整图依赖的问题才可延后到 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-SYS-008` | 所有公开 metadata API 必须只读；全局注册表必须在初始化完成后不可变并可安全并发读取。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-SYS-009` | 系统必须允许循环 Entity reference 图，但所有 descriptor、查询路径和图校验算法必须有明确递归边界， | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-SYS-010` | 公共类型、宏参数、错误类型和行为必须具备 Rustdoc；用户手册示例和本文规范必须作为 API 验收输入。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-SYS-011` | 每个 concrete Rust 类型必须只有一个由 qubit-reflect 提供的 TypeDescriptor 根；模型 metadata | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ROLE-001` | 同一 Rust 声明必须且只能使用 Entity、Projection、Model、Enum、Value 中一个角色宏。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T2 | [报告 G01](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ROLE-002` | 五种角色必须共享 Field、Property、TypeDescriptor、约束、输出策略和静态查询基础设施，不得为每个 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T2 | [报告 G01](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ROLE-003` | 角色必须是 metadata 中可查询的一等值，至少包含 Entity、Projection、Model、Enum、Value。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T2 | [报告 G01](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ROLE-004` | 注册与角色必须正交；为非 Entity 类型声明 id 只增加动态发现能力，不得赋予 identifier、 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T2 | [报告 G01](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ROLE-005` | 五种角色默认实现 Clone、Debug、Display、PartialEq、Eq、Hash、Redact、Serialize、Deserialize。 | 本库（含反射/输出集成） | 源码证实缺口/冲突 | 必做/复用；T2 | [报告 G01](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ROLE-006` | 只有全部 variant 都是 unit 的 Enum 默认实现 Copy；其他角色默认不得实现 Copy。 | 本库（含反射/输出集成） | 源码证实缺口/冲突 | 必做/复用；T2 | [报告 G01](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ROLE-007` | 五种角色默认不得实现 Default、PartialOrd、Ord。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T2 | [报告 G01](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ROLE-008` | 角色宏必须识别当前类型声明上可见的显式 derive，避免重复生成同一能力；角色 attribute 必须位于 | 本库（含反射/输出集成） | 源码证实缺口/冲突 | 必做/复用；T2 | [报告 G01](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ROLE-009` | 泛型类型的自动能力实现必须带准确 trait bound，不得要求所有潜在泛型实参无条件支持该能力。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T2 | [报告 G01](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ENT-001` | #[Entity] 必须只接受非泛型、无 lifetime、无 where 子句的具名字段 struct。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T5 | [报告 G07](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ENT-002` | Entity 的 id = "ModelId" 参数必须存在，并使该类型进入全局注册表。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T5 | [报告 G07](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ENT-003` | Entity 必须有且仅有一个符合身份约束的直接 identifier 字段。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T5 | [报告 G07](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ENT-004` | Entity 可以声明 reference、indexed、unique 和全部允许的值约束及输出策略。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T5 | [报告 G07](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ENT-005` | Entity 不得直接嵌入另一个 Entity 或 Projection 作为普通值；出现这两种角色必须通过 reference | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T5 | [报告 G07](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ENT-006` | Entity 不得声明单数 projection 或 projection_id 参数。一个 Entity 可以产生零个、一个或多个 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T5 | [报告 G07](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ENT-007` | Entity 的默认 PartialEq、Eq、Hash 必须采用标准结构化字段语义，不得擅自改为只比较 identifier。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T5 | [报告 G07](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-PRJ-001` | #[Projection] 必须只接受非泛型、无 lifetime 的具名字段 struct。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-PRJ-002` | Projection 必须有且仅有一个直接 Id identifier；它表示来源 Entity 实例 ID，不产生 Projection | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-PRJ-003` | Projection 可以不声明 source，表示开放 Projection；也可以使用 source = EntityType 或 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-PRJ-004` | source 与 source_id 的业务效果必须相同；前者通过 Rust 类型约束校验，后者通过完整 registry 解析。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-PRJ-005` | source 只表达来源约束和数据血缘，不得被解释为自动转换函数。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-PRJ-006` | 没有 projector 的 Projection 仍可以由 DAO/SQL mapper 或反序列化过程构造；需要默认生成但无 | 本库声明/结构与执行组件协作 | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-PRJ-007` | 固定来源 Projection 的 producer getter 所属 Entity 必须与 source 一致，生成结果的 identifier | 本库声明/结构与执行组件协作 | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-PRJ-008` | Projection 的 id 可选；提供时进入注册表，省略时仍必须可以按 Rust 类型取得完整 metadata。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-MDL-001` | #[Model] 必须接受具名字段 struct 和 unit struct，不得接受 tuple struct、enum 或 union。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-MDL-002` | Model 必须支持类型参数、const N: usize 和 where 子句。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-MDL-003` | Model 不得支持 lifetime 参数；可形成静态 metadata 的 concrete 实例必须满足 'static。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-MDL-004` | Model 禁止 identifier 和独立持久化语义，但可以声明 Entity reference。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-MDL-005` | Model 的 id 可选；有 id 时通过反射 capability 注册泛型定义或非泛型类型，无 id 时仍提供静态 metadata。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-MDL-006` | 单字段领域包装应使用 Value，而不是 tuple Model。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ENUM-001` | #[Enum] 必须接受 unit、tuple、struct 和混合 variant。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T5 | [报告 G07](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ENUM-002` | Enum 必须支持类型参数、const N: usize 和 where 子句，不得支持 lifetime 或 union。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T5 | [报告 G07](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ENUM-003` | Enum 禁止 identifier 和独立持久化；payload 字段可以显式声明 reference， | 本库（含反射/输出集成） | 源码证实缺口/冲突 | 必做/复用；T4、T5 | [报告 G07](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ENUM-004` | 每个 payload 字段必须拥有完整 TypeDescriptor、约束与输出 metadata。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T5 | [报告 G07](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ENUM-005` | Enum 的 id 可选；声明 id 时注册具体类型或泛型定义 capability。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T5 | [报告 G07](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ENUM-006` | 全部 unit variant 的 Enum 必须默认 Copy，且可用 no_copy 关闭。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T5 | [报告 G07](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-VAL-001` | #[Value] 必须接受具名字段 struct 和单字段 tuple newtype，不得接受 unit struct、多字段 tuple、 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T5 | [报告 G07](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-VAL-002` | Value 必须支持类型参数、const N: usize 和 where 子句，不得支持 lifetime。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T5 | [报告 G07](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-VAL-003` | Value 禁止 identifier、reference 和独立持久化生命周期。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T5 | [报告 G07](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-VAL-004` | Value 的传递字段闭包不得包含 Entity、Projection 或 Model；它可以包含 scalar、Enum、其他 Value、 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T5 | [报告 G07](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-VAL-005` | Value 的 id 可选；有 id 时可注册，但注册不得改变纯值角色。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T5 | [报告 G07](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-VAL-006` | transparent 只允许恰好一个存储字段的 Value，包括单字段 tuple 和单字段 named struct。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T5 | [报告 G07](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-VAL-007` | 透明 Value 必须保留独立名义类型和完整 metadata；Serialize、Deserialize、Display 使用内部值表示， | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T2 | [报告 G02](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-VAL-008` | transparent 不得自动生成 Deref、From、Into、TryFrom；表示透明不得绕过值约束。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T5 | [报告 G07](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-CAP-001` | 五种角色必须支持 no_clone、no_debug、no_display、no_partial_eq、no_eq、no_hash、 | 本库（含反射/输出集成） | 源码证实缺口/冲突 | 必做/复用；T2 | [报告 G01](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-CAP-002` | 五种角色必须支持 opt-in copy、default、partial_ord、ord。 | 本库（含反射/输出集成） | 源码证实缺口/冲突 | 必做/复用；T2 | [报告 G01](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-CAP-003` | copy 必须要求 Clone 未关闭且所有存储字段实现 Copy；冲突必须编译报错。 | 本库（含反射/输出集成） | 源码证实缺口/冲突 | 必做/复用；T2 | [报告 G01](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-CAP-004` | no_partial_eq 必须同时移除 Eq、Hash、PartialOrd、Ord。 | 本库（含反射/输出集成） | 源码证实缺口/冲突 | 必做/复用；T2 | [报告 G01](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-CAP-005` | no_eq 必须保留 PartialEq，但移除默认 Hash 并禁止 Ord。 | 本库（含反射/输出集成） | 源码证实缺口/冲突 | 必做/复用；T2 | [报告 G01](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-CAP-006` | ord 必须同时启用 PartialEq、Eq、PartialOrd、Ord，并与 no_eq/no_partial_eq 冲突。 | 本库（含反射/输出集成） | 源码证实缺口/冲突 | 必做/复用；T2 | [报告 G01](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-CAP-007` | struct 的 default 必须使用字段 Default；Enum 的 default 必须要求恰有一个标准 #[default] | 本库（含反射/输出集成） | 源码证实缺口/冲突 | 必做/复用；T2 | [报告 G01](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-CAP-008` | 自动 Default 只保证 Rust 值可构造，不得声明其一定满足模型约束或 validator。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T2 | [报告 G01](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-CAP-009` | no_redact 只允许类型及所有 selector 中不存在任何 redact 规则；关闭后保留的 Debug、Display、 | 本库（含反射/输出集成） | 源码证实缺口/冲突 | 必做/复用；T2 | [报告 G01](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-CAP-010` | no_debug/no_display/no_serialize 只关闭对应接口，不得关闭 Redact。 | 本库（含反射/输出集成） | 源码证实缺口/冲突 | 必做/复用；T2 | [报告 G01](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-FLD-001` | 所有真实 struct 字段必须进入 TypeMetadata.fields，不受 public/private 等 Rust visibility 影响。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-FLD-002` | FieldMetadata 必须提供零基 index、可选 name、TypeDescriptor、FieldVisibility、规范化属性和专用 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-FLD-003` | 具名字段 name 必须为 Some；tuple Value 或 Enum tuple payload 字段 name 必须为 None，并保留 index。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-FLD-004` | Enum 顶层 fields 必须为空；payload fields 必须属于对应 EnumVariantMetadata。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-FLD-005` | FieldMetadata 类型访问器必须命名为 descriptor()，不得使用含义不明确的 ty() 或旧 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-FLD-006` | FieldVisibility 必须包含 Public、Crate、Super、Path(&'static str)、Private。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-FLD-007` | pub(in crate)、pub(in super)、pub(in self) 必须分别归一化为 Crate、Super、Private；普通 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-FLD-008` | visibility 只描述源代码声明，不得决定 metadata 是否可查询，也不得改变 Property readable/writable。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-PROP-001` | #[ModelImpl] 必须作为 impl 反射上的模型扩展入口，提供 #[reflect_impl] 的反射能力， | 本库（含反射/输出集成） | 源码证实缺口/冲突 | 必做/复用；T3 | [报告 G03](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-PROP-002` | getter 形状必须为 pub fn name(&self) -> T，不得有额外参数，返回值不得为 ()。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T3 | [报告 G03](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-PROP-003` | setter 形状必须为 pub fn set_name(&mut self, value: T) -> ()，并且只能有一个值参数。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T3 | [报告 G03](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-PROP-004` | 同名 field、getter、setter 必须合并为一个 Property；显式 getter/setter 必须优先于生成的 field | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T3 | [报告 G03](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-PROP-005` | is_readable() 必须等价于 is_field() ／／ is_getter()。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T3 | [报告 G03](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-PROP-006` | is_writable() 必须等价于 is_field() ／／ is_setter()。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T3 | [报告 G03](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-PROP-007` | 存在同名 field 的 Property 必须是 FieldBacked；无 field 有 getter 必须是 Computed；无 field、 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T3 | [报告 G03](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-PROP-008` | is_computed() 必须等价于 storage_kind == Computed；不得通过字段或方法 attribute 重复声明。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T3 | [报告 G03](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-PROP-009` | PropertyMetadata 必须提供 name、descriptor、field/getter/setter、is_field/is_getter/is_set… | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T3 | [报告 G03](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-PROP-010` | getter 与 field/setter 的兼容检查至少必须支持 T ↔ &T、String ↔ str/&str、 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T3 | [报告 G03](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-PROP-011` | getter/setter 的 erased 访问协议必须遵守 Rust ownership、aliasing 和 lifetime 规则；不得将借用结果 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T3 | [报告 G03](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-PROP-012` | tuple Value 的无名字段不得自动形成具名 Property；Enum payload field 不得进入类型级 properties。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T3 | [报告 G03](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-PROP-013` | ModelImpl 必须支持方法级标记，显式排除该方法对 Property 的自动贡献，但保留其方法反射信息。 | 本库（含反射/输出集成） | 源码证实缺口/冲突 | 必做/复用；T3 | [报告 G03](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ID-001` | #[identifier] 只允许标在 Entity 或 Projection 的直接字段。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ID-002` | identifier 必须是准确类型为 qubit_id::Id 的直接字段，按 Rust 类型身份判断；真正指向该类型的 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ID-003` | 语法必须支持 #[identifier] 和 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ID-004` | database assignment 只允许 Entity；Projection 必须使用默认 application。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ID-005` | database assignment 表示数据库对最终 ID 负责。DAO 必须返回或回填数据库最终 ID，不得假设调用方 | 下游组件 | 外部执行职责 | D；本库保留声明接口 | 需求第 10 章 |
| `REQ-ID-006` | identifier metadata 只记录分配责任方，不得记录序列、自增、触发器等数据库机制。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ID-007` | identifier 必须隐含 indexed 查询能力；是否进入某种 list filter 由下游查询方案决定。 | 本库（含反射/输出集成） | 源码证实缺口/冲突 | 必做/复用；T5 | [报告 G10](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-QRY-001` | #[indexed] 只支持无参数形式，语义是字段路径可参与查询过滤，不是创建物理数据库索引。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T5 | [报告 G10](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-QRY-002` | identifier、unique、reference 必须分别增加 IDENTIFIER、UNIQUE、REFERENCE 索引原因。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T5 | [报告 G10](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-QRY-003` | 字段已有任一隐含 indexed 原因时，再显式添加 #[indexed] 必须编译报冗余错误。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T5 | [报告 G10](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-QRY-004` | IndexingReasons 必须是集合并支持 EXPLICIT、IDENTIFIER、UNIQUE、REFERENCE；is_indexed() 等价于 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T5 | [报告 G10](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-QRY-015` | 物理组合索引、字段顺序、排序、前缀、部分索引不得进入字段 indexed 语义。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T5 | [报告 G10](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-QRY-016` | 本库必须记录并公开有意义的 indexed 声明、索引原因、来源字段或 Property 路径及其类型， | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T5 | [报告 G10](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-UNQ-001` | #[unique] 必须声明当前字段在全局或 respect_to scope 内唯一。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T5 | [报告 G08](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-UNQ-002` | respect_to(field, ...) 可选；当前字段与 scope 字段按声明顺序构成唯一约束。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T5 | [报告 G08](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-UNQ-003` | ignore_case 只对 text-capable 当前字段有效，默认 true；显式 false 表示大小写敏感。 | 本库声明/结构与执行组件协作 | 源码证实缺口/冲突 | 必做/复用；T4、T5 | [报告 G08](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-UNQ-004` | unique 不得支持逻辑 name 参数。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T5 | [报告 G08](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-UNQ-005` | schema 必须能消费 unique metadata 建立约束；外部状态唯一性检查不属于纯字段 validator。 | 下游组件 | 外部执行职责 | D；本库保留声明接口 | 需求第 10 章 |
| `REQ-UNQ-006` | 随机生成器必须同时避开已有数据和当前批次的唯一冲突，并对不可满足情况返回明确错误。 | 下游组件 | 外部执行职责 | D；本库保留声明接口 | 需求第 10 章 |
| `REQ-REF-001` | entity = RustType 与 entity_id = "ModelId" 必须二选一。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T5 | [报告 G07](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-REF-002` | RustType 必须通过编译期 trait/role 约束验证为 Entity；entity_id 必须在完整 registry 中解析为 Entity。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T5 | [报告 G07](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-REF-003` | 省略 property 表示保存完整 Entity；property = id 表示 identifier；其他路径必须通过统一 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T5 | [报告 G07](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-REF-004` | reference property 必须存在、可读，并且 descriptor 与 reference 字段兼容；它是否 computed 不影响 | 本库（含反射/输出集成） | 源码证实缺口/冲突 | 必做/复用；T4、T5 | [报告 G07](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-REF-005` | existing 默认 true；false 表示目标无需预先持久化。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T5 | [报告 G07](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-REF-006` | path 必须表示在当前对象实例上下文中定位并复用同一 Entity 的绑定。路径以 / 分隔， | 本库（含反射/输出集成） | 源码证实缺口/冲突 | 必做/复用；T4、T5 | [报告 G04](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-REF-007` | reference 不得支持 name、select、bind、reference_key 等替代参数。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T5 | [报告 G07](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-REF-008` | reference 必须隐含 indexed；Map 不得作为 reference 的直接保存形状。 | 本库（含反射/输出集成） | 源码证实缺口/冲突 | 必做/复用；T4、T5 | [报告 G07](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-REF-009` | 对象生成器必须根据 existing、path 和 property 规划目标 Entity 创建顺序、既有对象复用和字段装配。 | 下游组件 | 外部执行职责 | D；本库保留声明接口 | 需求第 10 章 |
| `REQ-KEY-001` | #[key_part(order = n)] 只允许标在具名 Model 或 Value 的真实存储字段。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-KEY-002` | 未标注字段不得参与键投影；允许只选择部分字段。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-KEY-003` | order 必须从 0 连续、无重复、无缺号。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-KEY-004` | key_part 必须服务于复杂 unique、respect_to、随机去重和 DAO 重复键诊断。 | 下游组件 | 外部执行职责 | D；本库保留声明接口 | 需求第 10 章 |
| `REQ-KEY-005` | 下游键提取消费者必须先产生结构化键分量值，再进行比较、大小写规范化或诊断渲染。 | 下游组件 | 外部执行职责 | D；本库保留声明接口 | 需求第 10 章 |
| `REQ-KEY-006` | key_part 不得创建物理数据库索引，不得成为通用序列化协议或安全边界。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-TXT-001` | text 只允许 text-capable 叶子，不得负责 trim、大小写转换等值修改。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-TXT-002` | 必须支持 min_chars/max_chars，并按 Unicode scalar value 数量计算。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-TXT-003` | 必须支持 min_bytes/max_bytes，并按 UTF-8 字节长度计算；字符和字节约束必须分别验证。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-TXT-004` | non_blank 必须拒绝空串和完全由 Unicode whitespace 组成的字符串。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-TXT-005` | format 必须支持 email、cn_mobile、uri、uuid；不得使用含义不明确的 mobile。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-TXT-006` | allowed_chars 必须支持 unicode、printable_unicode、ascii、printable_ascii、code，默认 unicode。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-TXT-007` | unicode 表示所有 Unicode scalar value，包括控制字符；ascii 表示 U+0000..U+007F，包括控制字符。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-TXT-008` | printable_ascii 必须限制 U+0020..U+007E；printable_unicode 必须排除控制、格式、私用、未分配、 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-TXT-009` | code 必须等价 [A-Za-z0-9_-]，不限制首字符且不隐含 non_blank。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-TXT-010` | 每组 min 不得大于 max；参数不得重复。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-TXT-011` | 完全无约束的 #[text] 必须报错；显式 allowed_chars = unicode 必须合法。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-TXT-012` | allowed_chars 必须同时可供 validation、前端、随机生成和 schema/charset 消费。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-DEC-001` | decimal 和 money 只允许精确 decimal-capable 类型，不得允许 f32/f64。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-DEC-002` | 必须支持 precision、scale、字符串 min/max、min_inclusive/max_inclusive、rounding。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-DEC-003` | 同时声明 scale 与 precision 时，scale 不得大于 precision；min 不得大于 max； | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-DEC-004` | min/max 必须以字符串保存，避免浮点字面量精度损失。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-DEC-005` | rounding 必须支持 up、down、ceiling、floor、half_up、half_down、half_even、unnecessary。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-DEC-006` | decimal 默认 rounding 为 half_even，并且至少包含一个有效约束。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-DEC-007` | money 必须要求显式 scale，默认 rounding 为 unnecessary，metadata numeric semantic 为 Money。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-DEC-008` | money 不得包含 currency、货币符号或分组显示参数。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-DEC-009` | 同一作用位置的 decimal 与 money 必须互斥。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-DEC-010` | validator 只验证当前值；超过 scale 的规范化必须在构造最终对象前由 codec/解析器完成。 | 本库声明/结构与执行组件协作 | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-TIME-001` | time 必须要求 precision，支持 second、millisecond、microsecond、nanosecond，不得提供默认值。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-TIME-002` | time 只允许有相应亚秒能力的 instant/datetime/time 类型，纯 date 不使用该约束。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-TIME-003` | 值必须能被声明精度准确表示；validator 不得截断，生成器必须直接生成对齐值。 | 本库声明/结构与执行组件协作 | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-TIME-004` | 时区、过去/未来和跨字段先后关系不得进入 time；它们由类型或 validator 表达。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-SEQ-001` | sequence 必须支持 min_items、max_items、unique_items，并至少提供一个参数。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-SEQ-002` | min_items 不得大于 max_items。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-SEQ-003` | unique_items 按元素值相等性禁止重复，不等于数据库 unique。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-SEQ-004` | Set 天然唯一，再声明 unique_items 必须报冗余错误。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-SEQ-005` | 固定数组不得声明 min_items/max_items，但可以声明 unique_items。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-SEQ-006` | element 只选择 sequence、set、array 的第一层元素，不作用于容器本身。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-SEQ-007` | 生成器无法满足容量或唯一性时必须返回约束不可满足错误，不得无限重试。 | 下游组件 | 外部执行职责 | D；本库保留声明接口 | 需求第 10 章 |
| `REQ-MAP-001` | map 只约束 entry 数，支持 min_entries/max_entries，至少一个且 min 不得大于 max。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-MAP-002` | Map key 唯一由类型保证，不得提供 unique_entries 参数。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-MAP-003` | map_key 与 map_value 分别作用于每个实际 key/value，每个 Map 字段最多各一个。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-MAP-004` | key/value 是 Option 时，None 必须跳过局部值约束；具名复杂类型必须按 descriptor 递归。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-MAP-005` | map_key/map_value 不得继续嵌套 sequence、map、element、map_key、map_value；深层局部结构必须使用 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-MAP-006` | 生成器必须同时满足 entry 数、key 规则和 Map 天然键唯一性；有限 key 空间不足时必须返回不可满足。 | 下游组件 | 外部执行职责 | D；本库保留声明接口 | 需求第 10 章 |
| `REQ-SEL-001` | element、map_key、map_value 必须允许组合 text、decimal、money、time、validator、codec、redact。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-SEL-002` | 同一 selector 内每种标准约束最多一个；decimal/money 互斥；允许多个 validator occurrence，包括相同 ID；codec/re… | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-SEL-003` | selector 不得包含 identifier、indexed、unique、reference、key_part 或任意角色身份语义。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-SEL-004` | Option、Box、Rc、Arc 必须是透明包装；None 跳过标准约束、validator、codec，其他情况解包处理， | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-SEL-005` | sequence、set、array、map 不得被视为透明包装。直接字段 validator/codec 作用于整个容器， | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-SEL-006` | 标准 text/decimal 等不得从容器字段自动下沉；必须使用 element/map_key/map_value。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-SEL-007` | 未 opaque 的命名 Value、Model、Enum 必须按自身 descriptor 递归，无论位于字段、Option、元素或 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-SEL-008` | opaque 必须截断叶子内部递归，但不得删除外层 Option/容器 shape。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-SEL-009` | 类型自身约束与字段或 selector 使用位置的附加约束必须叠加，不能以使用位置声明覆盖或取消类型约束。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-VLD-001` | validator 必须同步、确定、无副作用，只验证当前值及显式提供的对象图依赖上下文可决定的事实。 | 下游组件 | 外部执行职责 | D；本库保留声明接口 | 需求第 10 章 |
| `REQ-VLD-002` | validator 不得访问 repository、数据库、网络、权限、库存或其他外部业务状态。 | 下游组件 | 外部执行职责 | D；本库保留声明接口 | 需求第 10 章 |
| `REQ-VLD-003` | 字段 occurrence 必须使用稳定 ValidatorId，并可以携带 params 和 depends_on。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G09](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-VLD-004` | params 只允许 bool、整数、字符串及同类型数组；精确 decimal、时间等结构化值使用字符串。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G09](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-VLD-005` | validator 依赖声明必须支持 path 和父对象导航，不得限制为当前对象的 Field/Property。 | 本库声明/结构与执行组件协作 | 源码证实缺口/冲突 | 必做/复用；T4、T6 | [报告 G09](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-VLD-006` | 同一字段或 selector 可以多次声明同一 validator ID，参数和依赖可以不同。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G09](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-VLD-007` | validator 必须满足 qubit-validator 的执行与注册契约；注册项必须关联稳定 ID、执行实现与 | 本库声明/结构与执行组件协作 | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G09](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-VLD-008` | 结构解析必须检查在显式类型上下文中可确定的 validator 依赖路径的存在性与可读性； | 本库声明/结构与执行组件协作 | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G09](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-VLD-009` | ValidationResult 必须结构化，violation 至少包含稳定 code、字段路径和消息参数；本地化展示不属于 | 下游组件 | 外部执行职责 | D；本库保留声明接口 | 需求第 10 章 |
| `REQ-VLD-010` | Validator trait、registration、registry 与 context 属于 qubit-validator；小写字段 helper 属于 | 本库声明/结构与执行组件协作 | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G09](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-CODEC-001` | codec 类型必须实现 ValueEncoder<T, Output = String>、 | 下游组件 | 外部执行职责 | D；本库保留声明接口 | 需求第 10 章 |
| `REQ-CODEC-002` | register_value_codec! 必须用稳定 ID、codec 类型和值类型提交链接期 registration，并形成 | 本库声明/结构与执行组件协作 | 已有基础；完整边界需专项验收 | 必做/复用；T6 | [报告 G11](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-CODEC-003` | ValueCodecRegistry 必须按 ValueCodecId 查询，保存领域类型身份、文本外部表示和 erased 双向入口。 | 下游组件 | 外部执行职责 | D；本库保留声明接口 | 需求第 10 章 |
| `REQ-CODEC-004` | 同一领域类型允许注册多个不同 codec；重复 ID 和类型不匹配必须成为 registry 错误。 | 本库声明/结构与执行组件协作 | 已有基础；完整边界需专项验收 | 必做/复用；T6 | [报告 G11](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-CODEC-005` | Value 可以使用 codec = RustType 声明 canonical codec。模型宏只记录声明中的 Rust 类型身份； | 本库声明/结构与执行组件协作 | 已有基础；完整边界需专项验收 | 必做/复用；T6 | [报告 G11](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-CODEC-006` | 字段 #[codec(RustType)] 与 #[codec(id = "ValueCodecId")] 必须二选一，最多一个， | 本库声明/结构与执行组件协作 | 已有基础；完整边界需专项验收 | 必做/复用；T6 | [报告 G11](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-CODEC-007` | codec 解析优先级必须为字段显式、类型 canonical、无 codec；字段显式选择与类型 canonical | 本库声明/结构与执行组件协作 | 已有基础；完整边界需专项验收 | 必做/复用；T6 | [报告 G11](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-CODEC-008` | codec trait、descriptor、ID、registration 与 registry 属于 qubit-codec；字段 helper 属于 derive。 | 本库声明/结构与执行组件协作 | 已有基础；完整边界需专项验收 | 必做/复用；T6 | [报告 G11](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-OPAQUE-001` | opaque 必须是无参数 marker，并把最终叶子视为外部黑盒。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-OPAQUE-002` | opaque 叶子不得要求 Reflect；默认 validation 不进入叶子，默认生成器不能自行构造。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-OPAQUE-003` | opaque 值必须由调用方提供或通过模型系统之外的类型生成 adapter 提供。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-OPAQUE-004` | opaque 不得与 identifier 或 reference 组合，不得隐藏 Entity、Projection、Model 以绕过角色检查。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-OPAQUE-005` | opaque 与 indexed/unique 同时声明时，本库必须保留 opaque 类型身份及查询、唯一性声明。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-OPAQUE-006` | 系统不得提供字段级 generator attribute；未来生成策略如有需求必须单独设计。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T4、T6 | [报告 G12](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-RED-001` | 字段只能选择 level=low/medium/high/secret、skip、nested、map、keyed_by、json 中一种模式。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T2 | [报告 G02](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-RED-002` | 空参数、重复模式或多个模式组合必须报错；未标字段不得根据字段名猜测敏感性。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T2 | [报告 G02](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-RED-003` | nested 必须委托给字段值的 Redact；map、keyed_by、json 必须遵循 qubit-redact 对应能力契约。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T2 | [报告 G02](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-RED-004` | FieldMetadata 必须保存规范化 RedactionMode，实际输出必须由 qubit-redact 执行。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T2 | [报告 G02](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-RED-005` | 字段级 redact 必须穿透 Option、Box/Rc/Arc、sequence、set、array 到实际值。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T2 | [报告 G02](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-RED-006` | Map 字段级 redact 默认只进入 value；Map key 必须用 map_key(redact(...)) 显式选择。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T2 | [报告 G02](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-RED-007` | 字段级与 selector redact 不得同时作用同一路径；重复或歧义必须报错。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T2 | [报告 G02](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-RED-008` | redact(skip) 表示启用脱敏时，在 Debug、Display/文本输出和 JSON/Serde 输出中省略整个字段， | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T2 | [报告 G02](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-RED-009` | Map key 脱敏产生重复输出 key 时不得静默覆盖，必须返回结构化序列化错误。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T2 | [报告 G02](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-RED-010` | 五种角色默认 Debug、Display、Serialize 必须执行字段脱敏；Deserialize 只负责输入，不应用脱敏。 | 本库（含反射/输出集成） | 源码证实缺口/冲突 | 必做/复用；T2 | [报告 G02](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-SER-001` | 五种角色必须完整保留标准 Serde 类型、variant 和字段属性；显式 Serde 配置优先。 | 本库（含反射/输出集成） | 源码证实缺口/冲突 | 必做/复用；T2 | [报告 G02](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-SER-002` | metadata 必须规范化最终序列化名称、反序列化名称、方向性 skip 等可发现事实，不得重新定义 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T2 | [报告 G02](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-SER-003` | 宏默认只对具名字段省略 Option::None 和空标准集合，并在反序列化缺失时补默认。 | 本库（含反射/输出集成） | 源码证实缺口/冲突 | 必做/复用；T2 | [报告 G02](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-SER-004` | 标准集合至少包含 Vec、VecDeque、LinkedList、HashMap、BTreeMap、HashSet、BTreeSet、BinaryHeap。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T2 | [报告 G02](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-SER-005` | 固定数组、newtype、tuple struct、Enum tuple payload 不得自动省略位置。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T2 | [报告 G02](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-SER-006` | keep_serializing 必须是无参数 marker，只允许可被默认省略的具名 Option/集合字段。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T2 | [报告 G02](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-SER-007` | keep_serializing 只关闭自动 skip_serializing_if，不关闭反序列化缺失默认，也不覆盖用户显式 serde skip。 | 本库（含反射/输出集成） | 源码证实缺口/冲突 | 必做/复用；T2 | [报告 G02](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-SER-008` | 在不可能被默认省略的字段上使用 keep_serializing 必须报冗余错误。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T2 | [报告 G02](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-VAR-001` | variant helper 只允许 name = "CANONICAL_NAME"。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T2 | [报告 G02](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-VAR-002` | 省略 name 时，canonical name 必须由 Rust variant 名转换为 SCREAMING_SNAKE_CASE。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T2 | [报告 G02](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-VAR-003` | canonical name 不得为空，同一 Enum 内不得重复；index/ordinal 按当前声明顺序确定， | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T2 | [报告 G02](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-VAR-004` | Rust name、canonical name、serialized name 必须分别保存；Serde rename 可以使 wire name 与 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T2 | [报告 G02](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-VAR-005` | 按 canonical name 查询的 API 不得同时模糊匹配 Rust/serialized name；其他名称必须使用独立查询。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T2 | [报告 G02](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-VAR-006` | variant 不得增加 code、weight 或随机生成概率参数；Default 使用标准 #[default]。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T2 | [报告 G02](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-001` | runtime metadata 必须由类型描述、成员描述、角色描述、字段语义、泛型描述和动态发现六组公共组件 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-002` | 任何从稳定公共接口返回的公开 metadata 类型，都必须定义查询能力、返回信息、缺失语义、错误 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-003` | 所有普通用户查询 API 必须只读；metadata 对象必须可静态共享，查询不得要求构造模型实例。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-010` | 普通用户查询 API 与派生宏生产 API 必须分层；用户手册不得要求业务代码手工构造 metadata。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-011` | 派生宏生产 API 必须公开可达，以允许下游 crate 中的宏展开代码调用，但必须放入明确的隐藏模块并 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-012` | 生产 API 的构造器必须重复验证角色与结构组合、descriptor 与 accessor 对齐等内存安全不变量， | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-013` | 隐藏生产 ABI 必须位于 qubit_model_metadata::__private 的版本化子模块，提供 checked 构造器、 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-020` | TypeMetadata 只能描述 Entity、Projection、Model、Enum、Value 五种领域声明类型。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-021` | TypeDescriptor 必须描述任意模型系统可理解的 Rust 类型，包括 scalar、透明包装、容器、tuple、 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-022` | 五种角色类型的静态入口必须为 TypeMetadata::try_of::<T>() 与 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-023` | 任意可描述类型的唯一静态入口必须为 TypeDescriptor::of::<T>()；显式 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-024` | 系统不得同时公开 metadata_of::<T>() 自由函数，也不得向用户类型注入 User::metadata() 固有 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-025` | HasTypeMetadata 必须是 sealed 的公共泛型约束并继承 Reflect；业务代码不得手工实现内部 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-030` | type_id() 必须直接返回 std::any::TypeId；系统不得定义 RustTypeIdentity、RustTypeId 或自有 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-031` | type_name() 必须返回诊断用完整 Rust 类型名；其字符串不得作为稳定协议、持久化键或类型相等依据。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-032` | model_id() 必须表示稳定动态身份；未声明 ID 的非泛型类型和 concrete 泛型实例必须返回 None。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-033` | generic_definition() 必须让 concrete 泛型实例返回所属 GenericModelMetadata；非泛型类型返回 | 本库（含反射/输出集成） | 源码证实缺口/冲突 | 必做/复用；T5 | [报告 G05](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-034` | is_registered() 只表示当前 metadata 本身是否直接存在于 registry；不得等价于“可以静态查询”， | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-035` | fields() 必须提供只读字段集合；field(name) 只查具名字段， | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-036` | Entity、Projection、具名 Model 和具名 Value 的 fields() 必须返回全部存储字段；unit Model 返回 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-037` | try_properties() 与 try_property(name) 必须提供可失败的只读 Property 查询；每个具名存储字段形成同名 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-038` | role()、role_metadata() 和五个 as_*() 方法必须提供角色标签、只读角色 payload 和角色导航；角色不匹配返回 None， | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-040` | TypeDescriptor::of() 和 ModelRegistry::metadata_for() 必须具有第 8.3 节给出的精确语义。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-041` | TypeDescriptor 必须能够区分并导航 scalar、Option、sequence、set、array、map、tuple、 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-042` | 公开结构表示、容器导航、descriptor 类型身份、能力查询和 opaque | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-043` | 类型能力查询必须区分 Rust trait 实现能力与字段约束适用能力；能力描述由反射层拥有，模型层复用， | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-050` | FieldMetadata 必须提供上述完整基础接口；字段类型方法必须命名为 descriptor()，不得退回含义 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-051` | is_identifier()、is_unique()、is_reference() 必须严格等价于对应 metadata 是否存在。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-052` | IndexingReasons 必须具有上述四个 flag；一个字段可以同时具有多个隐含原因，metadata 不得压缩成 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-053` | is_indexed() 必须严格等价于 !indexing_reasons().is_empty()；合法输入不得同时存在 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-054` | FieldVisibility 必须精确区分 Public、Crate、Super、Path、Private；可见性只记录源码 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-055` | FieldAttributeMetadata 必须以可枚举的强类型声明表示字段语义，区分不同属性并保留规范化后的参数； | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-060` | PropertyMetadata 必须提供上述完整基础接口；descriptor() 表示 field/getter/setter 合并后的 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-061` | is_field()、is_getter()、is_setter() 必须严格等价于相应 metadata 是否存在；它们不得与 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-062` | 一个 Property 可以同时具有 field、getter 和 setter；API 不得把三者建模成互斥 variant。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-063` | field-backed Property 必须可读、可写；getter 使 Property 可读；setter 使 Property 可写；只有 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-064` | Computed 表示无同名 field 且有 getter；Virtual 表示无同名 field 且只有 setter；不得要求用户 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-065` | GetterMetadata、SetterMetadata 必须公开方法来源、输入/输出类型、借用或所有权方式与可调用性； | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-070` | ModelRole 与 RoleMetadata 必须具有上述五个角色；角色公共信息放在 TypeMetadata，不得在每个 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-071` | 角色专属 metadata 必须提供最小角色信息：Entity 暴露 identifier；Projection 暴露 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-080` | IdentifierMetadata 必须提供 ID 分配责任方，语义遵循第 4.2 节。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-081` | UniqueMetadata 必须提供有序 scope 路径、全局或 scoped 分类及有效大小写比较规则，语义遵循第 4.4 节。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-082` | ReferenceMetadata 必须提供声明目标、Entity 或 Property 选择、existing 与复用路径；声明事实与解析结果分离。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-083` | ConstraintMetadata 必须区分第 5 章的约束种类，提供其完整规范化参数和 selector；未声明的边界必须可区分。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-084` | ValidatorMetadata 必须提供稳定策略 ID、有序参数和依赖路径，保留同一作用位置 occurrence 的源码顺序。 | 本库（含反射/输出集成） | 源码证实缺口/冲突 | 必做/复用；T4、T6 | [报告 G09](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-085` | CodecMetadata 必须区分 Rust 类型引用与稳定 ID 引用，并标明字段、类型 canonical 或 selector 声明来源； | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-086` | RedactMetadata 必须提供规范化模式及字段或 selector 作用位置，语义遵循第 7.1 节。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-META-087` | QueryMetadata 必须由 ModelGraph 拥有并只为 Entity 构造；它不得塞入静态 | 本库（含反射/输出集成） | 源码证实缺口/冲突 | 必做/复用；T5 | [报告 G10](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-REG-001` | ModelId 必须是 Java fully-qualified-class-name 风格的稳定字符串，精确语法为 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-REG-002` | 单段 ModelId 必须合法；空段、前导点、尾随点、连字符、Unicode 非 ASCII 字符必须非法。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-REG-003` | 命名空间 lower_snake_case、末段 UpperCamelCase 只能作为推荐，不得成为强制规则；末段不要求等于 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-REG-004` | ModelId 必须在所有角色共享的全局命名空间内唯一。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-REG-005` | reference.entity_id 和 Projection.source_id 必须只解析到 Entity；ModelId 自身不改变角色。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-REG-006` | ModelId::new() 接受宏已验证的静态字面量；动态输入必须使用 ModelIdBuf::parse() 返回 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-REG-010` | Entity id 必填并始终注册；Projection、Model、Enum、Value 只有声明 id 才注册。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-REG-011` | 无 id 类型不得产生 ModelRegistry 的匿名稳定 ID 注册项；这不禁止反射层注册无 ID 类型或泛型定义， | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-REG-012` | 无论是否注册，五种角色都必须可以通过已知 Rust 类型取得 TypeMetadata。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-REG-013` | registry 必须检测重复 ModelId，并返回包含两个注册来源位置的结构化错误。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-REG-014` | registry 必须能够按 ModelId 查询注册 metadata，并能够使用标准 TypeId 管理当前进程 concrete 类型缓存。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-REG-015` | registry 初始化完成后必须不可变；全局入口必须提供可处理错误和 panic 便利两种形式。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-REG-016` | ModelRegistry 必须提供 fallible/panic 全局入口、按稳定 ID 查询 registration/concrete/generic、 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T3、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-GEN-001` | 带 id 的泛型 Model、Enum、Value 在链接期只注册一等泛型定义，不得枚举 concrete 实例。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T5 | [报告 G05](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-GEN-002` | 定义必须描述类型参数、const 参数、where 约束和使用参数的字段 descriptor shape。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T5 | [报告 G05](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-GEN-003` | TypeMetadata::of::<Concrete>() 必须按需实例化并按当前进程标准 TypeId 缓存 concrete metadata。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T5 | [报告 G05](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-GEN-004` | 定义 ID 标识泛型声明；首版不得为 concrete 实例拼接或合成新的 ModelId。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T5 | [报告 G05](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-GEN-005` | 未声明模型 ID 的泛型定义仍由反射注册，并保留 generic-model capability；concrete 类型可静态查询， | 本库（含反射/输出集成） | 源码证实缺口/冲突 | 必做/复用；T1、T5 | [报告 G05](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-GEN-006` | 未来若需要字符串 concrete 泛型身份，必须另行设计 TypeExpression，不得使用 Rust type_name 作为协议。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T5 | [报告 G05](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-GEN-007` | GenericModelMetadata、类型参数、const 参数、where 约束、concrete | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T5 | [报告 G05](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-GEN-008` | const generic 必须支持 Rust 基础整数类型、bool、char 参数，常量实参和直接参数引用， | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T5 | [报告 G05](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-RES-001` | StructureResolver 必须解析 entity_id 与 source_id，并验证目标存在；validator 和 codec 的稳定 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T5 | [报告 G06](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-RES-002` | StructureResolver 必须验证 ID 目标角色、字段/property descriptor 兼容性和 validator 依赖 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T5 | [报告 G06](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-RES-003` | resolver 必须验证 fixed Projection source 与 producer 一致，并验证 Projection identifier 契约。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T5 | [报告 G06](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-RES-004` | resolver 必须检测跨 crate Value 传递闭包中的非法 Entity/Projection/Model/reference。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T5 | [报告 G06](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-RES-005` | resolver 错误必须确定性排序，并包含可用的稳定 ID、完整路径、期望/实际角色或类型及源码位置。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T5 | [报告 G06](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-RES-006` | ProjectionMetadata::source()、ReferenceMetadata getter、validator/codec metadata gett… | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T5 | [报告 G06](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-RES-007` | resolver 必须接受显式结构解析输入，成功时返回只读结构图，失败时返回结构化多错误集合； | 本库（含反射/输出集成） | 源码证实缺口/冲突 | 必做/复用；T5 | [报告 G06](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-RES-008` | registry/resolver 错误必须提供稳定类别、结构化路径、相关 ID、源码位置与适用的期望/实际类型或角色； | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T5 | [报告 G06](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-CONS-001` | 实例 validation 必须递归遵守 TypeDescriptor、Option、容器 selector、opaque 和 Value 边界。 | 下游组件 | 外部执行职责 | D；本库保留声明接口 | 需求第 10 章 |
| `REQ-CONS-002` | schema 消费者可以将领域约束映射到具体数据库/API schema，但不得把数据库专用配置回写为字段语义。 | 下游组件 | 外部执行职责 | D；本库保留声明接口 | 需求第 10 章 |
| `REQ-CONS-003` | 随机生成器必须生成满足声明式约束的值，并对有限空间、unique、sequence/map 容量等不可满足条件 | 下游组件 | 外部执行职责 | D；本库保留声明接口 | 需求第 10 章 |
| `REQ-CONS-004` | 对象图生成必须使用 identifier/reference/existing/path/property 规划依赖，不得将 Value/Model | 下游组件 | 外部执行职责 | D；本库保留声明接口 | 需求第 10 章 |
| `REQ-CONS-005` | 查询消费者必须依据 indexed 原因、类型、结构化属性路径、unique scope 和 reference 声明设计查询能力， | 下游组件 | 外部执行职责 | D；本库保留声明接口 | 需求第 10 章 |
| `REQ-CONS-006` | 接口文档必须能发现类型角色、字段约束、最终 Serde 名称、optional/container shape 和 redaction 分类， | 下游组件 | 外部执行职责 | D；本库保留声明接口 | 需求第 10 章 |
| `REQ-CONS-007` | DAO 重复键诊断和随机唯一缓存必须基于结构化 key components，不得依赖不稳定 Debug/Display 文本。 | 下游组件 | 外部执行职责 | D；本库保留声明接口 | 需求第 10 章 |
| `REQ-QRY-005` | 可从显式 indexed 字段和 reference 关联路径构造 list filter。 | 下游设计参考 | 参考，不实施 | R；不纳入本库 | 需求第 10.3 节 |
| `REQ-QRY-006` | 可为 identifier 和全局 unique 提供专用唯一查找；是否也进入 list filter 由查询组件决定。 | 下游设计参考 | 参考，不实施 | R；不纳入本库 | 需求第 10.3 节 |
| `REQ-QRY-007` | 可为 scoped unique 同时提供字段过滤和完整唯一键查找。 | 下游设计参考 | 参考，不实施 | R；不纳入本库 | 需求第 10.3 节 |
| `REQ-QRY-008` | 复杂字段的一种展开方案是只沿内部 indexed 成员递归，并在未标记节点停止。 | 下游设计参考 | 参考，不实施 | R；不纳入本库 | 需求第 10.3 节 |
| `REQ-QRY-009` | 采用叶子展开方案的消费者负责识别无法生成查询条件的情况，并提供路径、声明来源和明确诊断； | 下游设计参考 | 参考，不实施 | R；不纳入本库 | 需求第 10.3 节 |
| `REQ-QRY-010` | 查询身份应保留结构化路径；category.id 映射为 category_id 只是可选平面命名方案。 | 下游设计参考 | 参考，不实施 | R；不纳入本库 | 需求第 10.3 节 |
| `REQ-QRY-011` | 采用平面命名的消费者负责检测并处理冲突，诊断应标明原路径和生成名称；不得丢失或静默覆盖条件。 | 下游设计参考 | 参考，不实施 | R；不纳入本库 | 需求第 10.3 节 |
| `REQ-QRY-012` | reference 只展开一跳是一种控制复杂度的候选方案，不是模型关系图的固定限制。 | 下游设计参考 | 参考，不实施 | R；不纳入本库 | 需求第 10.3 节 |
| `REQ-QRY-013` | 采用 reference 深度限制时，可分别设计普通值对象嵌套的展开规则。 | 下游设计参考 | 参考，不实施 | R；不纳入本库 | 需求第 10.3 节 |
| `REQ-QRY-014` | 多个条件按 AND 组合是一种默认方案；组合表达能力由查询组件设计。 | 下游设计参考 | 参考，不实施 | R；不纳入本库 | 需求第 10.3 节 |
| `REQ-ERR-001` | 角色与 Rust shape 不匹配、identifier 数量/类型错误、无效参数、约束范围、重复属性、互斥组合必须 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T4、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ERR-002` | 错误必须定位到导致问题的用户 token；涉及两处声明时应同时保留主错误和相关位置。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T4、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ERR-003` | parser 应聚合互相独立的错误，使一次编译可以报告多个问题；不得在首个无关错误处停止。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T4、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ERR-004` | 重复显式/隐含 indexed、key_part 缺号、selector 非法嵌套必须有专用编译期诊断，不得退化为泛化的 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T4、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ERR-005` | 与类型 capability 不匹配的 text/decimal/time/container 约束必须通过清晰编译错误说明期望能力。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T4、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ERR-006` | 已废弃。原条款中的 validator/codec with = RustType 路径不属于模型声明契约；执行实现的注册 | 废弃 | 废弃 | R；保留编号 | REQ-ERR-006 |
| `REQ-ERR-010` | 跨 crate 缺失 ID、重复 ID、错误角色、property 不存在/不可读/类型不兼容、策略未注册必须返回 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T4、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ERR-011` | 错误类型必须提供稳定错误类别和机器可读数据；展示文案和本地化不属于核心 metadata。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T4、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ERR-012` | 错误路径必须使用结构化 Field/Property 路径，并能渲染为清晰诊断文本。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T1、T4、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ERR-013` | Map key 脱敏冲突、随机生成不可满足、缺少 Projection projector 等执行期错误必须明确区分， | 本库声明/结构与执行组件协作 | 已有基础；完整边界需专项验收 | 必做/复用；T1、T4、T5、T8 | [报告 G13](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-OUT-001` | 不得提供 lookup_relation 或 ownership 字段/类型属性。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T8 | [报告 G14](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-OUT-002` | 不得提供 #[computed] 或 computed depends_on；computed 必须由 Property 是否有同名 field 推导。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T8 | [报告 G14](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-OUT-003` | 不得提供字段级 #[generator]。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T8 | [报告 G14](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-OUT-004` | 不得提供字段级 modified/unmodified；它们属于具体 DAO 操作 metadata。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T8 | [报告 G14](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-OUT-005` | 不得提供通用 #[exclude]；一次生成任务的排除必须在生成请求中按结构化路径配置。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T8 | [报告 G14](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-OUT-006` | 不得提供 #[key_index]；最终只使用 #[key_part(order = n)]。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T8 | [报告 G14](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-OUT-007` | indexed、unique、reference 不得支持逻辑 name 参数。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T8 | [报告 G14](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-OUT-008` | 不得在字段宏中表达物理数据库表名、列名、组合索引、排序、前缀或部分索引。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T8 | [报告 G14](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-OUT-009` | 不得提供两套同义 metadata 静态查询入口。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T8 | [报告 G14](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-OUT-010` | 不得根据 Rust type_name 字符串判断类型相等或生成稳定跨进程 ID。 | 本库（含反射/输出集成） | 已有基础；完整边界需专项验收 | 必做/复用；T8 | [报告 G14](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ACC-001` | 每个本库实现需求编码必须映射到自动化测试、可执行 doctest 或适用的文档边界审查清单。 | 文档与交付验收 | 文档已核查；实现交付验收未完成 | 必做/复用；T8 | [报告 G14](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ACC-002` | 每个合法宏示例必须有 compile-pass 或 runtime metadata 测试；每个明确非法组合必须有 compile-fail | 文档与交付验收 | 文档已核查；实现交付验收未完成 | 必做/复用；T8 | [报告 G14](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ACC-003` | 跨 crate ID、注册、source、reference、validator、codec 必须使用真实多 crate fixture 验证。 | 文档与交付验收 | 文档已核查；实现交付验收未完成 | 必做/复用；T8 | [报告 G14](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ACC-004` | Field/Property erased accessor 必须有内存安全测试；借用 getter 和可写 setter 是高风险必测路径。 | 文档与交付验收 | 文档已核查；实现交付验收未完成 | 必做/复用；T8 | [报告 G14](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ACC-005` | 默认能力矩阵、transparent Value、Serde 省略、keep_serializing、Redact 容器传播必须有行为测试。 | 文档与交付验收 | 文档已核查；实现交付验收未完成 | 必做/复用；T8 | [报告 G14](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ACC-006` | 泛型定义注册、concrete descriptor 实例化和缓存必须有并发与重复查询测试。 | 文档与交付验收 | 文档已核查；实现交付验收未完成 | 必做/复用；T8 | [报告 G14](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ACC-007` | 用户手册中的 API 名称、参数、代码示例和限制必须与本文需求编码一致；修改公共语义时必须同时更新 | 文档与交付验收 | 文档已核查；实现交付验收未完成 | 必做/复用；T8 | [报告 G14](rs-model-implementation-gaps.zh_CN.md) |
| `REQ-ACC-008` | 需求规范、最终设计、公开签名、用户手册、Rustdoc 和测试必须同步，不得保留未决 API 占位符。 | 文档与交付验收 | 文档已核查；实现交付验收未完成 | 必做/复用；T8 | [报告 G14](rs-model-implementation-gaps.zh_CN.md) |
