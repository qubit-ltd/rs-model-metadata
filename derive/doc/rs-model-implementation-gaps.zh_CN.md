# 模型元数据实现缺口与必要性评估

- 审计日期：2026-09-10；基准为用户已确认的冻结需求。
- 范围：rs-model-metadata、derive、相关测试及 rs-platform 声明；只读审计，不修改 Rust 实现。
- 方法：CodeGraph 定位后查阅源码；对明确分支给出证据，运行限定的现有测试作为回归基线。
- [逐项台账](rs-model-derive-requirements-coverage.zh_CN.md)将全部 338 个编号关联到本报告的评估组与重构任务。

## 结论

现有代码已经具备反射 overlay、角色/字段声明、Property 合并、稳定 ID 注册、泛型实例缓存和可选执行绑定基础。
缺口集中在默认能力、impl 反射范围、路径/匿名类型图和输出集成，同时存在必须移出的查询产品策略。
不能将本次重构描述为“全部重新实现”，也不能以旧测试通过判定冻结需求已经满足。

| 组 | 现状证据与缺口 | 是否应做 / 处理方式 | 任务 |
| --- | --- | --- | --- |
| G01 默认能力 | [parse/options.rs](../src/parse/options.rs:92)明确拒绝 no_*、copy/default/ord；[expand/pipeline.rs](../src/expand/pipeline.rs:69)只增加反射及 metadata | 必做；补齐 ROLE-005、CAP，保留用户确认的 Eq/Hash 联动，不用手写 derive 替代目标需求 | T2 |
| G02 输出语义 | 同一 pipeline 未安装模型默认 Debug/Display/Serialize；[expand/fields.rs](../src/expand/fields.rs:760)只规范化脱敏声明；[parse/fields.rs](../src/parse/fields.rs:474)记录部分 Serde 参数 | 必做；接入 rs-redact 投影、标准 Serde 委托和自动 omit/default；验收必须检查实际输出 | T2 |
| G03 ModelImpl | [expand/model_impl.rs](../src/expand/model_impl.rs:42)拒绝 trait/generic；parse_property_method 对部分普通方法报错；暂无退出 Property 标记 | 必做；完整委托 reflect_impl，增加 model_property(skip)，保留现有兼容类型/借用访问基础 | T3 |
| G04 对象路径 | [parse/fields.rs](../src/parse/fields.rs:301)将 reference.path 交给按点分割的 parse_path_value；[metadata_vocabulary.rs](../../src/metadata_vocabulary.rs:389)用 PropertyPath 保存 same_as | 必做；对象导航与属性选择分离，结构化 Parent 步骤；不实现实例绑定或回退 | T1、T4 |
| G05 泛型定义 | [expand/metadata.rs](../src/expand/metadata.rs:75)只在有 id 时连接 generic_definition；[generic_model_metadata.rs](../../src/generic/generic_model_metadata.rs:22)要求非空 ModelId | 必做；ID 改可选，所有泛型连接定义，稳定 ID 枚举仍只收有 ID 项 | T1、T5 |
| G06 匿名解析与上下文 | [resolve/resolver.rs](../../src/resolve/resolver.rs:42)输入只有 registry，并只遍历 concrete_entries；[validation_plan.rs](../../src/validation/validation_plan.rs:134)expect 根有 ModelId | 必做；显式 roots + 可达闭包；无 ID 诊断和计划身份；父路径保留上下文需求 | T1、T5、T6 |
| G07 引用形状与 Enum | [normalize/declaration.rs](../src/normalize/declaration.rs:190)禁止 Enum reference；[resolver.rs](../../src/resolve/resolver.rs:269)直接比较选中值与外层字段 TypeId | 必做；允许 Enum payload 引用、包装兼容，Value 传递闭包继续禁止 reference；补无名 variant 字段定位 | T4、T5 |
| G08 Unique 类型能力 | [parse/fields.rs](../src/parse/fields.rs:230)默认 ignore_case=true；[normalize/declaration.rs](../src/normalize/declaration.rs:224)依赖 is_text_type 语法分类 | 必做；保留显式参数来源，以能力确定有效值；非文本普通 unique 可用。Unicode 比较由下游实现，不在宏执行 | T4 |
| G09 Validator 依赖 | [parse/validator.rs](../src/parse/validator.rs:33)已有稳定 ID、有序 params 和裸/命名依赖；依赖使用 syn::Path，没有独立对象导航；[validation_plan.rs](../../src/validation/validation_plan.rs:314)编译局部属性依赖 | 必做；每项 path/property、声明起点与上下文要求；保留重复 occurrence 顺序并新增针对同 ID 的验收。实例执行属于 adapter/下游 | T4、T6 |
| G10 查询越界 | [resolve/queries.rs](../../src/resolve/queries.rs:34)生成 filters，排除根 ID/全局 unique 并处理平面名；[graph.rs](../../src/resolve/graph.rs:273)公开 filter_by_flat_name | 必须移出；只保留查询声明视图。unique scope 的结构校验必须迁到 relations，不能随 build_query 删除。下游候选方案不在本库补实现 | T5 |
| G11 Codec | [codec/binder.rs](../../src/codec/binder.rs)及 tests/codec_binding_tests.rs 已覆盖 Rust 类型/ID 绑定与缺失、歧义、类型错误 | 保留并扩展；相同显式/canonical 的优先级、匿名可达模型和 selector occurrence 需专项验证。编码算法仍归 rs-codec | T6 |
| G12 约束与 selector | [parse/constraints.rs](../src/parse/constraints.rs)、[expand/fields.rs](../src/expand/fields.rs)已有标准约束；现有 validation 测试还明确覆盖“不支持 map selector”的报告 | 必做声明与来源补齐，执行适配按能力覆盖；保留 Set/数组冗余禁令，不将校验不足改成放宽需求。跨位置叠加、opaque 和 text-capable Value 需要针对性验证 | T4、T6 |
| G13 反射、字段、注册、身份 | [type_metadata.rs](../../src/type_metadata.rs)、[registry/model_registry.rs](../../src/registry/model_registry.rs)、[property.rs](../../src/property.rs)已有主要公共基础；[expand/fields.rs](../src/expand/fields.rs:225)已用 IdentifierType bound | 保留并回归；不把真实 Id 别名支持误列为“按名字判定”的缺口。所有借用/可见性/循环/错误分支未由本次 38 项测试全部证实，台账不能标整组完成 | T1、T3、T5、T8 |
| G14 文档与实际调用方 | 原指南说能力开关非法；历史设计仍要求查询展开；rs-platform 当前有 131 个角色声明，5 个文件出现点路径字符串 | 必做迁移和同步；统计只是文本扫描，不等同全部正确展开。设计阶段给出范围，实施时重写指南并编译真实调用方 | T7、T8 |
| G15 未采纳扩展与 ABI | [parse/validator.rs](../src/parse/validator.rs)还接收 target/on_none；[parse/fields.rs](../src/parse/fields.rs)接收 validate_nested；当前隐藏 ABI v5 | 不以现状扩充需求；迁移为既定 selector/Option 或下游配置。新字段布局使用同步 v6 协议并保留 checked 构造 | T1、T4、T6、T7 |

以上“必做”表示冻结能力必须交付；已经存在的部分优先复用。新增测试先验证缺口再实现，不能删除原测试后只检查新名称。

## 已运行的基线

工作目录：`rs-model-metadata`。

```sh
cargo test --locked --offline --workspace --all-features --lib --test role_metadata_runtime_tests --test contract_runtime_tests --test projection_producer_tests --test validation_capabilities_tests --test model_validation_tests --test codec_binding_tests
```

结果：38 passed，0 failed。包括 derive 单元测试 16、metadata 单元测试 3、指定集成测试 19。
其中 test_reject_behavior_options、test_resolver_builds_scoped_unique_and_reference_queries 验证的是旧行为；它们通过反而说明新需求需要迁移。
本次未运行全量 trybuild、多 crate 集成、Clippy 或 coverage，不给出新需求覆盖率。

## rs-platform 实际声明基线

2026-09-10 对 `rs-platform/modules/**/*.rs` 中角色 attribute 作文本扫描：Entity 37、Projection 11、Model 35、Enum 46、Value 2，共 131。
统计未做 cfg 展开或 Rust AST 语义分析，只作为迁移范围。点路径候选文件为：

- `rs-platform/modules/address/src/model/address.rs`
- `rs-platform/modules/iam/src/model/session.rs`
- `rs-platform/modules/feedback/src/model/feedback.rs`
- `rs-platform/modules/tenant/src/model/department.rs`
- `rs-platform/modules/notification/src/model/verify_code.rs`

迁移必须定位到 reference.path 属性再转换，不能批量替换源码中的全部点号。普通 property、validator 的属性选择及 ModelId 保留点分隔。

## 优先级与必要性

P0：T1 公共表示和 ABI；T2 默认能力/输出；T3 ModelImpl；T4 领域声明语义；T5 完整结构解析。它们直接影响领域声明是否成立和 metadata 是否可信。
P1：T6 可选执行绑定、T7 实际调用方与指南迁移、T8 集成验收。P1 不等于可省略，只是依赖前面的稳定接口。

不纳入本库重构：filter 对象和 SQL 生成、字符串/范围操作实现、父对象业务回退、数据库物理索引、随机对象算法。
这些已记录在需求第 10 章；没有这些产品实现不构成本库交付缺口。

## 证据边界

逐项台账中的“基础存在/需专项验收”表示源码和抽样测试支持该实现方向，但尚不足以证明条款所有边界；不是未经检查地声称完全实现。
“缺失或冲突”只用于本报告列出的明确代码分支。“下游负责”“设计参考”“废弃”不视作本库待开发项。
后续执行必须逐项填写新测试及结果，不能用本报告的任务映射替代验收。
