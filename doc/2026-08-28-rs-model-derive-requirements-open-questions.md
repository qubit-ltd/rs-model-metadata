# `rs-model-derive` Runtime Metadata API 待确认清单

- 对应需求规范：[`2026-08-28-rs-model-derive-requirements.md`](2026-08-28-rs-model-derive-requirements.md)
- 对应用户手册：[`2026-08-28-rs-model-derive-target-api-guide.zh_CN.md`](2026-08-28-rs-model-derive-target-api-guide.zh_CN.md)
- 状态：自顶向下整理中，等待分批确认
- 范围：所有会被普通用户直接使用，或从稳定公共接口返回的 runtime metadata 类型

## 1. 使用方式

本清单只记录尚未闭合的 API，不重复询问已经确认的 `TypeMetadata`、`FieldMetadata`、`PropertyMetadata` 基础接口。

每个 TODO 在用户手册和需求规范中都有同编号占位符。后续按本文分组，每次确认四至五项；一组确认后，先把结论同步回
两份主文档，再进入下一组，避免讨论结论只停留在对话里。

在 TODO 关闭前：

- 主文档可以说明类型职责、使用位置和已确认行为；
- 主文档不得把未经确认的方法名、参数类型或返回类型写成最终承诺；
- 示例中若必须表达未确认调用，应使用 `rust,ignore` 并注明 TODO 编号。

## 2. 第一组：类型描述基础（5 项）

- [ ] **META-API-TODO-001 — `TypeDescriptor` 完整查询 API**

  已确认：`TypeDescriptor::of::<T>() -> &'static TypeDescriptor`、`metadata()`，以及它必须覆盖 scalar、透明包装、
  容器、tuple、五种角色、opaque、泛型参数和 concrete 泛型实例。

  待确认：公开结构表示、类型身份、容器导航、wrapper 剥离、opaque 查询及便利判断的精确 Rust API。

- [ ] **META-API-TODO-002 — `TypeCapabilities`**

  已确认：Value 的 copy/default/serialize 等公共能力不能塞进 `ValueMetadata`；字段约束合法性也需要类型 capability。

  待确认：能力集合、flag 类型、查询入口，以及 Rust trait 实现能力与约束 capability 是否分成两个类型。

- [ ] **META-API-TODO-003 — `HasTypeMetadata` 与 `HasTypeDescriptor`**

  已确认：两个 trait 必须公开，供 `of::<T>()` 和泛型约束使用。

  待确认：trait 继承关系、关联项/方法、用户手工实现是否属于受支持用法，以及 opaque adapter 如何进入该体系。

- [ ] **META-API-TODO-004 — `GetterMetadata` 与 `SetterMetadata`**

  已确认：它们分别由 `PropertyMetadata::getter()` 和 `setter()` 返回；Property 可以同时具有 field/getter/setter。

  待确认：类型信息、Rust 方法来源信息、borrowed/owned getter、setter 入参所有权、erased accessor、失败类型和线程安全
  边界。

- [ ] **META-API-TODO-005 — `FieldAttributeMetadata`**

  已确认：`FieldMetadata::attributes() -> &[FieldAttributeMetadata]` 保留，同时已有 identifier、unique、reference、
  constraint、validator、codec、redact 强类型 getter。

  待确认：`FieldAttributeMetadata` 是归一化枚举还是另一种只读视图、包含哪些 variant，以及如何避免与强类型 getter
  产生两套不一致事实。

## 3. 第二组：字段语义 metadata（5 项）

- [ ] **META-API-TODO-006 — `IdentifierMetadata`**

  待确认：除了“该字段是 identifier”这一存在性事实外是否还需要公开任何信息，以及 `IdentifierMetadata` 的完整接口。

- [ ] **META-API-TODO-007 — `UniqueMetadata`**

  已确认：字段级 unique 不接受物理索引名和数据库参数，并会隐含 indexed。

  待确认：比较方式、scope/path 等最终保留事实及完整 getter。

- [ ] **META-API-TODO-008 — `ReferenceMetadata`**

  已确认：reference 指向 Entity，selection 与 binding 均无默认值，ID 解析必须显式进行。

  待确认：目标类型/目标 ID、selection、binding、路径、容器叶子信息和解析后目标的完整接口。

- [ ] **META-API-TODO-009 — `ConstraintMetadata` 体系**

  已确认：text、decimal、money、time、sequence、element、map、map_key、map_value 都必须保留强类型语义和作用位置。

  待确认：顶层枚举、各 constraint 类型、selector/嵌套视图及每个 getter 的精确 API。

- [ ] **META-API-TODO-010 — `ValidatorMetadata`**

  已确认：保存稳定 validator ID、静态 params、显式 depends_on，按源码顺序确定性执行。

  待确认：`StrategyId`/`ValidatorId` 类型、参数值枚举、依赖路径、值类型 descriptor 和 registry 解析结果的完整接口。

## 4. 第三组：输出策略、角色和查询（4 项）

- [ ] **META-API-TODO-011 — `CodecMetadata`**

  已确认：codec 是 Value 与规范文本之间的双向领域语义，使用稳定 ID 和静态参数，不替代通用 Serde。

  待确认：`ValueCodecId`、方向、参数、值类型 descriptor 和解析后 codec 入口的完整 API。

- [ ] **META-API-TODO-012 — `RedactMetadata`**

  已确认：redact 可按已定义规则传播到 Option、sequence 和 Map key/value 位置，并影响安全输出。

  待确认：策略表示、selector 作用位置、参数、默认行为以及与 `qubit-redact` 公共类型的复用边界。

- [ ] **META-API-TODO-013 — 五种角色专属 metadata（原 A003）**

  待确认候选范围：

  - `EntityMetadata`：必有 `model_id` 和 identifier；
  - `ProjectionMetadata`：identifier、可选 source、open/fixed；
  - `ModelMetadata`：首版保留空的角色 payload；
  - `EnumMetadata`/`EnumVariantMetadata`：variant 集合、三种名称、payload 字段和 default；
  - `ValueMetadata`：transparent 状态和可选 transparent field。

  还需确认候选接口是否完整，以及 `ProjectionSource` 和 Enum variant 查询的精确返回形式。

- [ ] **META-API-TODO-014 — `QueryMetadata`**

  已确认：可查询路径不应直接塞进 `EntityMetadata`；indexed 原因保留在字段级。

  待确认：`QueryMetadata` 的拥有者和取得入口、平面/路径查询条目、filter 类型投影、字段冲突及角色适用范围。

## 5. 第四组：泛型、注册和完整解析（4 项）

- [ ] **META-API-TODO-015 — 泛型 metadata**

  已确认：带 ID 的泛型 Model/Enum/Value 只注册泛型定义；concrete 实例按需产生 metadata，`model_id() == None` 且
  `is_registered() == false`；首版不拼接 concrete ModelId。

  待确认：`GenericTypeMetadata`、类型参数、const 参数、where 约束、concrete 实参、实例化和缓存 API；同时统一首版
  const generic 支持范围。

- [ ] **META-API-TODO-016 — `ModelId` 与 `ModelRegistry` 完整 API**

  已确认：Entity 必注册；其他角色有 ID 才注册；注册表不可变；提供 fallible 全局入口和 panic 便利入口；支持稳定 ID
  查询和确定性枚举。

  待确认：`ModelId`/owned ID 构造接口、`get()` 参数类型、registration 视图、迭代器、按 `TypeId` 查询、泛型定义枚举和
  concrete cache 公开程度。

- [ ] **META-API-TODO-017 — Resolver 完整 API**

  已确认：显式解析 Entity ID、Projection source、validator ID、codec ID，并检查角色和 descriptor 兼容性；metadata
  getter 不隐式读取全局 registry。

  待确认：resolver 是 trait、构建器还是 registry 方法；输入、解析后只读视图、增量/完整解析和返回类型。

- [ ] **META-API-TODO-018 — Registry/Resolver 错误 API**

  已确认：错误必须确定性排序，包含稳定分类、完整路径、相关 ID、期望/实际角色或类型和源码位置。

  待确认：错误枚举、单错误/多错误集合、字段 getter、source 链和展示文案边界。

## 6. 第五组：派生宏隐藏生产 ABI（1 项）

- [ ] **META-API-TODO-019 — 隐藏生产接口**

  已确认：普通查询 API 与宏生产 API 分层；宏生产入口放入类似 `__private` 的隐藏模块，跨 crate 可达但不作为业务代码
  应使用的稳定 API。

  待确认：模块名、构造器/descriptor builder、静态注册提交、校验失败方式、宏与 runtime crate 的兼容策略，以及哪些
  内存安全不变量必须在构造时重复检查。

## 7. 完成条件

全部 TODO 关闭后，必须同时满足：

- 用户手册中有一份从顶层入口开始、可沿返回类型逐层导航的完整 API 参考；
- 需求规范中每个公开方法都有需求编码、返回语义和示例；
- 不存在“公开方法返回某 metadata 类型，但该类型没有接口章节”的断链；
- 不存在同一方法在用户手册、需求规范和 Rustdoc 中名称或签名不同的情况；
- 所有临时 `rust,ignore` API 示例已被替换为可编译示例，或被明确保留为非代码概念示例。
