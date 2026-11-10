# Validation plan 构建规模分析

[English](validation-plan-build-analysis.md)

> **历史实验：** 本次有界性能测量早于当前固定校验计划构建上限。下文数据只反映当时的实现，不是当前 root-scoped walker 的性能测量，也不能证明当前构建没有上限。当前限制和诊断见[用户指南](user_guide.zh_CN.md#进阶用法停止条件与执行预算)。

## 核查问题与方法

本次有界实验曾用于核查计划构建成本是否超出计划必须保留的 occurrence 数量。实验比较共享二叉模型树与单链：叶节点有一条 `non_blank` 文本声明；深度 1、4、8、10 的二叉模型分别产生 2、16、256、1,024 个 occurrence，单链各深度均产生 1 个。计时循环外构建模型图和 validator registry，计时操作只有 `ValidationPlan::build`。

基准曾在计时前断言解析出的唯一模型类型数和计划 binding 数。`tests/validation/validation_plan_tests.rs` 另用两个字段共享同一受约束子类型的 root 构建计划，断言有两个 binding、违例顺序稳定为 `left.value`、`right.value`，并验证左侧合法时只报告 `right.value`。

## 复现命令与环境

```text
cargo bench --locked --features validation --bench model_pipeline -- plan_build_scaling
```

测量使用 release profile、Rust `1.94.0`（`4a4ef493e`）、`x86_64-unknown-linux-gnu`，CPU 为 Intel Xeon E5-2680 v4 2.40 GHz。Criterion 每项采样 10 次，预热 3 秒，测量 1 秒。测量期间主机上还有其他 Cargo 任务，因此这些耗时仅作估算，不作为稳定性能门槛。未测量峰值 RSS。

## 测量数据

下表列出 Criterion 报告的估计值及置信区间：

| 形状 | 深度 | 唯一类型数 | Bindings | 计划构建耗时 |
| --- | ---: | ---: | ---: | ---: |
| 二叉树 | 1 | 2 | 2 | 7.78–8.05 µs（估算 7.85 µs） |
| 二叉树 | 4 | 5 | 16 | 72.53–77.00 µs（估算 74.57 µs） |
| 二叉树 | 8 | 9 | 256 | 1.381–1.394 ms（估算 1.389 ms） |
| 二叉树 | 10 | 11 | 1,024 | 6.090–8.438 ms（估算 7.096 ms） |
| 单链 | 1 | 2 | 1 | 5.03–5.07 µs（估算 5.05 µs） |
| 单链 | 4 | 5 | 1 | 10.32–10.45 µs（估算 10.38 µs） |
| 单链 | 8 | 9 | 1 | 18.30–18.35 µs（估算 18.33 µs） |
| 单链 | 10 | 11 | 1 | 22.37–22.78 µs（估算 22.54 µs） |

二叉模型的耗时随输出保留的 2^depth 个 occurrence 增长；测量没有显示相对于 binding 数的超线性成本。单链的 binding 数保持为 1，但随唯一模型类型数增加而耗时较高，这与遍历更深的模型图相符。该实验没有单独测量分配，也不能证明每次重复读取元数据都不可避免；不过没有发现既可测得收益、又能保留现有 occurrence 契约的可行动重复工作。

## 下游核对与结论

`rs-platform/modules/model-runtime/src/lib.rs` 在首次成功验证时构建 root 图并创建计划，随后按 root `TypeId` 缓存计划，后续请求复用缓存。平台 `Person` 模型有三个可选 `CredentialInfo` 字段，其 `number` 字段带 validator，因此计划会为三个不同路径保留三个 occurrence。这是有限的分支展开，不是深层共享二叉结构。更宽的合成树属于压力形状，不代表已发现生产故障。

历史结论：需要为不同路径保留不同 binding 时，路径展开本身会产生相应成本。本实验在当时实现中没有发现可行动的冗余工作；其中耗时不代表当前 walker 的性能。当前实现已有固定构建上限，且它与 `ValidationOptions` 的实例执行预算相互独立，详见[当前用户指南](user_guide.zh_CN.md#进阶用法停止条件与执行预算)。应用应像 `ModelRuntime` 一样构建一次并复用计划。
