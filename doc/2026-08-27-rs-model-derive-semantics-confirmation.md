# `rs-model-derive` 语义确认记录

- 日期：2026-08-27
- 状态：确认中
- 当前仓库：`/home/starfish/working/qubit/rust-platform`
- 用途：逐项记录用户确认的最终公共语义；最终设计文档和用户手册只以本记录中“已确认”的条目为依据。

## 记录规则

1. 每个条目只确认一个清晰的公共语义或 API 决策。
2. 每轮提交 3～5 个条目供确认。
3. 用户确认或修订后，先更新本记录，再进入下一轮。
4. “待确认”内容不是设计结论，不能写入最终设计和用户手册。
5. 最终设计文档直接描述最终 API，不以旧设计、修改过程或迁移差异为正文结构。

## 状态说明

| 状态 | 含义 |
| --- | --- |
| 已确认 | 用户已经明确同意，可作为最终规范。 |
| 待确认 | 候选表述，尚不能作为最终规范。 |
| 已否决 | 已明确不采用，仅保留决策记录。 |

## 已确认的过程原则

### P001 最终文档的写法

- 状态：已确认
- 结论：最终设计文档直接定义最终设计是什么，包括语义、用途、参数、合法组合、默认行为、元数据和公开 API；
  不以“如何修改当前设计”为主体。

### P002 确认粒度

- 状态：已确认
- 结论：Entity、Projection、Model、Enum、Value 以及每个字段宏都必须逐项确认；每轮可以确认 3～5 个原子条目。

## 第一轮：字段标注的语义分类

### C001 身份、持久化与对象装配

- 状态：已确认
- 结论：`#[identifier]`、`#[indexed]`、`#[unique(...)]`、`#[reference(...)]` 属于同一类。它们描述字段在
  领域身份、数据库结构、持久化查询和 Entity 关联中的语义；自动化测试的随机对象生成器也使用这些信息决定 ID
  生成、唯一值生成、Entity 创建顺序、既有对象复用和关系字段装配。
- 边界：`lookup_relation`、`ownership` 不再需要，不进入最终设计。`computed` 的语义不同，移到 C006 单独确认。

### C002 声明式值约束

- 状态：已确认
- 结论：`#[text(...)]`、`#[decimal(...)]`、`#[money(...)]`、`#[time(...)]`、`#[sequence(...)]`、
  `#[map(...)]`、`#[element(...)]` 属于同一类。它们描述字段值或容器内容必须满足的不变量，可同时供数据库/schema
  定义、实例 validation、合法随机值生成和接口文档使用；它们不赋予字段领域身份或关系语义。

### C003 自定义处理策略

- 状态：已确认
- 结论：`#[validator(...)]` 与 `#[codec(...)]` 属于同一类扩展选择器。小写 `validator` 在字段 occurrence 上引用
  `ValidatorId`，并可携带该次使用的 `params` 与 `depends_on`；小写 `codec` 使用 `with = RustType` 或
  `id = ValueCodecId` 选择一个已经定义并注册的文本 codec，不携带实例参数。具体 validator 与 codec 分别由
  `rs-validator` 的大写 `#[Validator]` 和 `rs-codec` 的大写 `#[ValueCodec]` 定义并在链接期跨 crate 自动注册，
  `rs-model-derive` 只生成引用 metadata，不执行具体逻辑。最终设计不提供字段级 `#[generator(...)]`。

### C004 结构解析与递归边界

- 状态：已确认
- 结论：`#[opaque]` 单独属于结构解析类别。它不表示值合法、持久化或安全策略，而是声明框架只保留可见的
  Option/容器外层结构，不继续解析叶子内部 descriptor。校验、生成、schema 和访问器都必须遵守这个递归边界。

### C005 输出表示与敏感信息保护

- 状态：已确认
- 结论：`#[redact(...)]`、Serde 字段属性以及模型宏需要识别的 `#[keep_serializing]` 属于输出表示与安全类别。
  它们控制 Debug/Display/Serialize 等对外表示、嵌套脱敏和空值/空集合序列化，不改变字段的领域身份、数据库关系
  或值合法性。`redact` 与 `serde` 由各自专用 crate 定义，模型宏只负责正确联动和保留这些属性。

以上五类相互正交；同一个字段可以在不冲突时同时使用多类标注。例如一个 email relation 字段可以同时具有
`reference`、`text` 和 `redact`，三者分别表达对象装配、值约束和输出安全。

### C006 派生属性与依赖关系

- 状态：已被 R008-A/R008-B 修订
- 最终结论：删除 `#[computed]` 和 depends_on。改为“Field 与 Property”模型：是否 computed 由 property 是否存在
  同名真实 field 自动推导，不再由 annotation 重复声明。getter-only property 是 computed；field-backed property
  不是 computed；setter-only property 是无存储、只写的 virtual property，computed 对它没有意义。

### C007 键文本投影

- 状态：已确认
- 结论：`#[key_part(order = n)]` 描述具名 Model/Value 的部分存储字段如何按稳定顺序投影为比较或诊断用的
  key components。它服务于复杂 unique 字段（无论 ignore_case true/false）、respect_to 复杂属性、随机去重和
  DAO 重复键诊断，不表示数据库索引、对象身份、查询条件、ORM 映射或通用序列化。
- 规则：未标注字段不参与投影；已标注字段的 order 必须从 0 连续、无重复无缺号。运行时先形成结构化
  KeyComponentValue，再按 consumer 需要比较、大小写规范化或渲染诊断文本。

## C001 第一批：四个核心字段宏

### F001 `#[identifier]`

- 状态：已确认
- 已确认语义：标记对象的实例身份字段。对 Entity，它同时是领域身份和独立持久化记录的主键；对 Projection，它
  表示所投影 Entity 的身份，不产生 Projection 自己的主键。自动化生成器必须先准备 identifier，其他对象才能
  安全引用当前实例。
- 已确认限制：必须标注直接字段，准确类型为 `Id`，不能是 `Option<Id>`、容器、别名伪装或嵌套路径；不存在
  `ignore_case` 参数；业务自然键与 identifier 分开建模。它隐含 indexed 查询能力，但按 F002-B 不进入所属根对象
  自己的 list filter。
- 已确认角色规则：Entity 和 Projection 必须各有且仅有一个 identifier；Model、Enum、Value 禁止 identifier。
- 参数：使用 `#[identifier(assigned_by = application | database)]` 表示 ID 分配责任方，默认 `application`。
  需要保留“ID 是否由数据库在插入时分配”的 metadata。最终语法为
  `#[identifier(assigned_by = application | database)]`，默认 `application`。`database` 表示插入调用方无需提供权威
  ID；即使提供，数据库也可以覆盖，DAO 必须返回或回填数据库最终确定的 ID。metadata 使用
  `IdentifierAssignment::Application | Database`，不记录自增、序列、触发器等具体分配机制。
- 角色限制：`assigned_by = database` 只允许 Entity；Projection 的 identifier 来自来源 Entity，只允许默认
  `application`。
- 规则：不使用含义模糊的 `generated`；`assigned_by` 表达谁对最终 ID 负责，也覆盖数据库修改调用方暂时提供值的
  情况。已持久化 Entity 和所有 Projection 始终包含最终有效的 `Id`。

### F002 `#[indexed]`

- 状态：已确认

#### F002-A 核心语义与隐含规则

- 状态：已确认
- 结论：`indexed` 的准确含义是“当前字段路径可以参与该对象的查询过滤条件”，不是“为当前字段创建一个
  数据库单列索引”。框架把它记录为结构化查询路径 metadata，API、DAO 或代码生成器可据此产生 `UserFilter` 等
  过滤器。非 Option 叶子路径的基础过滤形式为 `Option<T>`：`None` 表示不使用该条件，`Some(value)` 表示按相等
  条件过滤；Option、容器以及范围等操作数的精确表示留到 F023 一并确认。
- 参数：只支持无参数形式 `#[indexed]`，不支持 `name` 或物理索引参数。
- 隐含规则：`identifier`、`unique`、`reference` 均赋予字段 indexed 查询能力。这里的“查询能力”表示该字段可被
  查询规划器使用，并不表示它必然出现在当前类型的 list filter 中；根对象唯一键和 reference 目标字段适用下面
  不同的投影规则。已经具有其中任一标注的字段再显式添加 `#[indexed]` 属于重复语义，编译时报错。

#### F002-B 根对象的 list filter 投影规则

- 状态：已确认
- 结论：生成某个类型自身的 list filter 时，直接包含它显式 `#[indexed]` 的普通字段，并按 F002-D 展开它的
  reference 字段；不包含根对象自己的 identifier，因为 identifier 应由专用的唯一查找 API 使用。
- 结论：根对象上没有 `respect_to(...)` 的全局 unique 字段也不进入 list filter，因为它同样应由专用唯一
  查找 API 使用。带 `respect_to(...)` 的 scoped unique 字段单独看不能唯一确定根对象，因此仍进入 list filter；
  完整 unique 字段组另外形成一个唯一查找键。`respect_to` 中列出的 scope 字段是否进入 list filter，仍由这些字段
  自己的标注决定。

#### F002-C 普通复杂对象的递归展开

- 状态：已确认
- 结论：非 reference 的复杂对象字段被显式 indexed 后，只沿其内部“有效 indexed”成员继续递归；内部有效
  indexed 包括显式 `indexed` 以及由 `identifier`、`unique`、`reference` 赋予的查询能力。未被 indexed 的中间
  节点会切断该路径，未被 indexed 的叶子不会成为查询条件。
- 结论：过滤条件的规范身份始终保存为结构化属性路径，例如 `category.id`、`category.name`；生成 Rust/REST
  平面字段名时默认用下划线连接为 `category_id`、`category_name`。若两个不同路径得到相同平面名称，派生时报错，
  不通过 `name` 参数绕过歧义。
- 结论：一个 complex indexed 字段如果递归后没有任何可查询叶子，派生时报错，避免产生看似可查询、实际
  无条件可用的空标注。

#### F002-D reference 只展开一跳

- 状态：已确认
- 结论：生成根对象的 list filter 时，每个 reference 最多沿 reference 图展开一跳。进入目标 Entity 后，
  包含目标 Entity 自己直接声明的 identifier、显式 indexed 字段和 unique 字段；不再进入目标 Entity 的 reference
  字段。目标 identifier 或全局 unique 虽然能唯一确定目标 Entity，却不能唯一确定根对象，因此必须保留为根对象
  的有效过滤条件。
- 结论：“一跳”限制针对 reference 图，而不是普通值对象的结构嵌套。目标 Entity 的直接 indexed 字段如果是
  非 reference 复杂值，仍按 F002-C 展开；任何路径一旦遇到目标 Entity 的 reference 就停止。
- 示例：Address 自己的 identifier 和全局 unique code 不进入 AddressFilter；Address.name 进入；Address.city
  展开 City.id、City.code、City.name，但 City.province 是第二跳 reference，不进入：

  ```rust
  pub struct AddressFilter {
      pub name: Option<String>,
      pub city_id: Option<Id>,
      pub city_code: Option<String>,
      pub city_name: Option<String>,
  }
  ```
- 结论：reference 通过 `property` 选择标量属性时，该字段直接成为叶子条件；选择 Projection、Value 等复杂
  属性时，将所选属性视为第一跳目标，按同样规则展开，但不跨越其中的 reference。

#### F002-E 组合条件与物理组合索引

- 状态：已确认
- 结论：`#[indexed]` 不增加组合索引或“组合查询参数”语法。调用方同时设置多个过滤字段，默认就是这些
  条件的 AND 组合，已经覆盖 `gender + create_time` 之类的组合查询，不需要额外模型语义。
- 结论：物理数据库组合索引涉及字段顺序、前缀、排序、部分索引以及具体数据库能力，是查询执行优化，不是
  领域模型的可查询性。当前设计不在 `rs-model-derive` 中表达它；将来若确有自动 DDL 需求，应在持久化层使用独立
  的类型级配置，而不是扩展或复用字段级 `#[indexed]`。

### F003 `#[unique(...)]`

- 状态：已确认
- 结论：声明当前字段在全局或给定 scope 字段组合下唯一。数据库/schema 用它建立唯一约束；带 repository 的
  validation 用它检查已有数据；随机对象生成器用它避开数据库和当前批次中的冲突。
- 参数：
  - `respect_to(field, ...)`：可选 scope 字段列表；省略表示当前字段全局唯一；
  - `ignore_case = bool`：文本比较是否忽略大小写，默认 `true`。因此通常只写 `#[unique]`；需要区分大小写时写
    `#[unique(ignore_case = false)]`。
- 规则：不支持 `name` 参数。`ignore_case` 只对文本能力字段有意义；当前字段与 `respect_to` 字段按声明顺序组成
  唯一约束。unique 隐含 indexed。

### F004 `#[reference(...)]`

- 状态：已确认
- 结论：声明当前字段保存另一个 Entity 本身或其某个属性/派生属性的值。数据库/schema 用它表达关联，随机
  对象生成器用它取得目标 Entity、按 `property` 提取值并装配当前字段。每个 reference 自动等价于 `#[indexed]`。
- 参数：
  - `entity = RustType` 与 `entity_id = "ModelId"` 必须二选一；前者在编译期验证目标类型，后者避免跨 crate
    循环依赖并在完整注册表阶段解析；二者效果等价；
  - `property = property_path`：可选；省略表示保存完整 Entity，`property = id` 表示 identifier，
    `property = info` 可以选择标有 `#[computed(...)]` 的 getter；
  - `existing = bool`：可选，默认 `true`；`true` 表示目标 Entity 必须预先存在，`false` 表示不要求预先持久化；
  - `path = "object/graph/path"`：可选；表示复用当前对象图中另一处 reference 所绑定的同一个 Entity。
- 规则：不支持 `name` 参数；不再提供 `select`、`bind` 或 `reference_key`。`property` 与 `path` 分别表示目标值选择和对象图复用，
  语义互不替代。

## C002 第一批：文本与十进制值约束

### F010 `#[text(...)]`

- 状态：已确认
- 已确认语义：声明文本能力叶子的内容约束，同时供实例校验、合法随机值生成、数据库/schema 和接口文档使用；它
  不负责 trim、大小写转换或其他会修改值的规范化。
- 已确认参数：
  - `min_chars = u32`、`max_chars = u32`：Unicode scalar value 数量上下界；
  - `min_bytes = u32`、`max_bytes = u32`：UTF-8 字节数上下界；
  - `non_blank`：拒绝空字符串以及全部由 Unicode whitespace 组成的字符串；
  - `format = email | cn_mobile | uri | uuid`：内置的语义格式约束；原先含义模糊的 `mobile` 改为 `cn_mobile`。
- 已确认规则：字符数与字节数分别校验；每组 min 不得大于 max；参数不可重复；字段必须具有 text capability。
  `#[text]` 没有实际约束时派生报错。Option 与容器传播稍后统一确认。
- `allowed_chars`：未显式声明时的有效默认值为 `unicode`，支持：
  - `unicode`：所有 Unicode scalar value，包括控制字符；
  - `printable_unicode`：Unicode Letter、Mark、Number、Punctuation、Symbol 和 Space Separator，排除控制字符、
    格式字符、私用区、未分配字符及行/段分隔符；
  - `ascii`：`U+0000..U+007F`，包括 ASCII 控制字符；
  - `printable_ascii`：`U+0020..U+007E`；
  - `code`：ASCII 字母、数字、下划线和连字符，即 `[A-Za-z0-9_-]`。
- 规则：使用 `code` 而不使用 `identifier`，避免和领域 identifier 混淆，也不谎称连字符是编程语言标识符字符。
  `code` 只限制每个字符，不限制首字符、不隐含 non_blank；完整 username/code 业务语法使用其他 text 约束和自定义
  validator。
- 消费语义：allowed_chars 同时供实例校验、前端校验、随机生成和数据库 schema/charset 使用。因此显式
  `#[text(allowed_chars = unicode)]` 合法；只有完全没有参数的 `#[text]` 报错。

### F011 `#[decimal(...)]`

- 状态：已确认
- 结论：声明普通十进制数的表示范围和定点精度，供数据库 decimal schema、实例校验、输入规范化和随机生成
  使用；只适用于具有 decimal capability 的精确十进制类型，不适用于 `f32`/`f64`。
- 参数：
  - `precision = u16`：定点表示允许的总位数上限；
  - `scale = u16`：允许的小数位数上限；与 precision 同时存在时不得大于 precision；
  - `min = "decimal"`、`max = "decimal"`：可选精确十进制边界，以字符串保存，避免浮点字面量损失；
  - `min_inclusive = bool`、`max_inclusive = bool`：边界是否包含，默认均为 `true`，只有对应边界存在时才允许声明；
  - `rounding = up | down | ceiling | floor | half_up | half_down | half_even | unnecessary`：超过 scale 时的规范化
    舍入策略；默认 `half_even`。
- 规则：`min` 不得大于 `max`，相同边界不能同时为排他；至少声明一个有效约束；不能与 `money` 同时出现。
  纯 validator 不修改对象，只验证当前值已经满足 precision/scale/range；codec、解析器和生成器可以按 rounding 先
  规范化，再交给 validator。`unnecessary` 表示需要舍入时返回错误。

### F012 `#[money(...)]`

- 状态：已确认
- 结论：声明字段是货币金额，而不只是普通十进制数。它沿用 F011 的 precision、scale、范围和 rounding
  规则，但 metadata 的 numeric semantic 为 Money，使 schema、序列化、显示和随机生成器能够选择货币策略。
- 参数：与 `decimal` 相同，但 `scale` 必须显式提供；没有跨币种通用的默认小数位数。rounding 默认仍为
  `unnecessary`，需要业务舍入时必须明确选择。
- 规则：只适用于 decimal-capable 精确十进制类型；不能与 `decimal` 同时出现；不包含 currency、分组显示或
  货币符号参数。币种是独立领域数据，分组和符号是输出表示，均不属于字段值约束。

## C002 第二批：时间与容器约束

### F013 `#[time(...)]`

- 状态：已确认
- 结论：声明时间能力叶子的有效精度，使实例校验、数据库/schema、序列化/输入适配和随机生成使用同一时间
  分辨率。它不表示时区，也不表达多个字段之间的先后关系。
- 参数：只支持必填的
  `precision = second | millisecond | microsecond | nanosecond`，没有默认值。字段必须是具有相应亚秒能力的 instant、
  datetime 或 time 类型；纯 date 类型不使用该约束。
- 规则：值必须能被所声明精度准确表示，例如 second 要求亚秒部分为零，millisecond 要求纳秒部分是
  1_000_000 的整数倍。validator 不截断或修改值；generator 直接生成对齐值；codec/DAO 若接收更高精度输入，必须
  在构造最终对象前规范化。`#[time]` 无参数时报错。
- 边界：固定最早/最晚时刻、必须为过去/未来以及 `start <= end` 等业务规则不进入 time；前两类使用 validator，
  跨字段关系使用能读取相关字段的 validator。

### F014 `#[sequence(...)]`

- 状态：已确认
- 结论：声明 sequence、set 或固定数组本身的元素数量约束；同时供实例校验、随机生成、schema 和接口文档
  使用，不约束单个元素内容。
- 参数：`min_items = u32`、`max_items = u32`、标记参数 `unique_items`。至少提供一个参数，min 不得大于 max。
- 规则：`unique_items` 按元素值相等性禁止重复，不等于数据库 unique。对 Set，唯一性由类型天然保证，再写
  `unique_items` 属于重复语义并报错。固定数组长度由 Rust 类型决定，因此不允许 min_items/max_items，但仍允许
  `unique_items`。generator 必须在约束可满足时生成相应大小且无重复的集合，否则返回明确的不可满足错误。

### F015 `#[map(...)]`

- 状态：已确认
- 已确认容器语义：`map` 只声明 Map 自身的 entry 数量约束；Map key 的唯一性由 Map 类型天然保证，不另设
  unique 参数。
- 已确认参数：`min_entries = u32`、`max_entries = u32`；至少提供一个，min 不得大于 max。
- 已确认规则：只用于具有 map capability 的类型。Map 不能作为 reference 的直接保存形状，这一角色组合限制
  稍后统一确认。

#### F015-B `#[map_key(...)]` 与 `#[map_value(...)]`

- 状态：已确认
- 结论：新增两个一层子值约束宏，与 sequence/set 的 `element` 对称：`map_key` 作用于每个实际 key，
  `map_value` 作用于每个实际 value；`map` 仍只负责 entry 数量，不混入 key/value DSL。
- 参数：二者都支持 `text(...)`、`decimal(...)`、`money(...)`、`time(...)`，参数与对应字段宏完全一致；每个
  Map 字段最多各出现一次 map_key 和 map_value，约束类型必须与 key/value capability 匹配。
- 递归规则：key/value 是 Option 时 None 跳过局部值约束；是具名复杂类型时由其 descriptor 递归处理；不在
  map_key/map_value 中继续嵌套 sequence、map、element、map_key 或 map_value，深层约束使用具名 Value。
- 生成规则：generator 同时遵守 map 数量、key 约束和 Map 天然键唯一性；有限 key 取值空间不足时返回约束
  不可满足错误，不能无限重试。value 根据 map_value 和自身 descriptor 生成。

### F016 `#[element(...)]`

- 状态：已确认
- 结论：把局部值约束应用到 sequence、set 或固定数组的第一层元素，不作用于容器本身；容器大小仍由
  sequence 描述。
- 参数：支持 `text(...)`、`decimal(...)`、`money(...)`、`time(...)`，参数和对应字段宏完全一致；
  元素类型必须具有匹配 capability。同一 element 中互斥的 decimal/money 组合报错。
- 规则：element 只下沉一层；元素是 `Option<T>` 时 None 跳过，Some 校验 T；元素是具名 Value/Model 等复杂
  类型时，其内部字段由自身 descriptor 递归处理。element 不支持 sequence/map 的再次嵌套，也不用于 Map key/value；
  需要深层局部约束时应定义有名字的 Value 类型，避免形成任意深度的属性 DSL。

## C003/C004：结构边界与自定义策略

### 策略宏的公共参数约定

- 状态：已确认
- 最终约定：小写 `#[validator(id = "ValidatorId", params(...), depends_on(...))]` 使用稳定 ValidatorId；
  `params` 的 literal 只支持 bool、整数、字符串及同类型数组，精确 decimal、时间等结构化配置使用字符串；
  `depends_on` 只接受当前对象内的字段路径。小写 `#[codec(with = RustType)]` 与
  `#[codec(id = "ValueCodecId")]` 二选一，不支持 `params` 或 `depends_on`。两种字段 helper 都只生成静态引用
  metadata，不实例化或执行实现。最终不存在小写 `#[generator(...)]`。

### F017 `#[opaque]`

- 状态：已确认
- 结论：把字段的最终叶子类型视为外部黑盒；descriptor 保留最外层 Option、sequence、set、array、map、box
  等结构，但不解析 opaque 叶子的内部字段，也不要求叶子实现 HasTypeDescriptor。
- 参数：无参数 marker，`#[opaque]`。
- 规则：默认 validator 不递归 opaque 叶子；默认对象生成器无法构造它，必须由调用方供值或在对象生成系统中
  另外注册类型生成能力；最终不存在字段级 `#[generator]`；
  标准 text/decimal/money/time 约束只有在 opaque 类型显式提供相应 capability adapter 时才允许。
- 边界：不得用 opaque 隐藏 Entity、Projection、Model 或 reference 目标来绕过角色和关系检查；不能与
  identifier/reference 组合。indexed/unique 只有在该 opaque 类型提供查询比较和持久化 adapter 时才允许。

### F018 `#[validator(...)]`

- 状态：已确认
- 用户修订：validator 只能验证由当前对象值本身确定的语法、格式、结构和字段间一致性；不得访问 repository、
  数据库、网络或其他外部业务状态，不验证唯一性、真实存在性、权限、库存等业务事实。
- 用户示例：username validator 可以验证长度和字符组成，不能验证用户名是否已被占用；身份证 validator 可以
  验证号码结构、校验位以及与同一对象 gender/birthday 的一致性，但不能验证身份证是否已登记、是否重复或现实中
  是否确有其人。
- 参考结论：JS `Validatable` 保存字段 validator 函数；validator 接收字段值和包含所属 instance 的 context；
  `ChinaIdentityCardRule` 负责号码结构/校验位，PersonInfo validator 进一步核对 credential、gender、birthday，均不
  访问外部系统。这验证了“字段 validator + 显式同对象依赖”的方向。
- 候选 attribute：继续使用
  `#[validator(id = "...", depends_on(field, ...), params(...))]`；允许同字段多个不同 id，按源码顺序聚合 violation。
- 候选 trait：具体实现必须注册为 `FieldValidator<T>`；注册 API 通过 Rust generic bound 在编译期保证实现类型符合
  trait，并把 ValidatorId 与准确字段类型绑定。attribute 使用稳定 id 以避免 model crate 直接依赖具体实现 crate；
  完整 validator registry 校验每个 id 已注册且字段类型兼容。
- 候选执行契约：`FieldValidator<T>` 是同步、确定性、无副作用的纯语法 validator。它接收 `&T` 和只包含字段路径、
  静态 params、显式依赖值的 ValidationContext；context 不提供 repository、数据库、网络或 service。返回结构化
  `ValidationResult`，violation 至少包含稳定 error code、字段路径和消息参数，显示文案由上层本地化。
- 候选 crate 边界：rs-model-metadata 只保存 ValidatorId、params、depends_on；rs-model-derive 只生成 metadata。
  trait、registry、ValidationContext/Result 应放在独立 `qubit-validator` crate，不塞入 proc-macro 或纯 metadata crate；
  可以先只实现这个最小契约，未来继续扩展该 crate。
- 用户补充：应提供 validator 类型宏，通过 `#[Validator(id = "...")]` 自动实现 FieldValidator 并自动注册。
- 候选类型宏语法：Rust struct 体不能包含方法，因此采用无状态 unit struct + inherent impl：

  ```rust
  #[Validator(id = "qubit.text.email", value = str)]
  pub struct EmailValidator;

  impl EmailValidator {
      fn validate(
          &self,
          value: &str,
          context: &ValidationContext<'_>,
      ) -> ValidationResult {
          // user implementation
      }
  }
  ```
- 候选宏参数：`id` 必填；`value = RustType` 必填，表示 validator 接收的视图类型。宏只支持无泛型 unit struct，
  自动生成 `FieldValidator<RustType>`、默认构造能力和链接期跨 crate 自动收集的 ValidatorRegistration。字段 occurrence 上的
  `depends_on`/`params` 仍写在小写 `#[validator(...)]` 中，因为它们属于这一次使用而非 validator 类型本身。
- 候选兼容规则：`value = str` 可用于 String 以及提供 text view adapter 的类型；其他 value 类型默认要求准确类型
  匹配。宏生成的 trait impl 调用同名 inherent `validate`，方法签名、返回值或 trait bound 不符时编译失败。
- 用户确认的 crate 归属：大写 `#[Validator]`、FieldValidator trait、ValidatorRegistry 和运行时校验 API 属于
  rs-validator；小写字段 helper `#[validator(...)]` 属于 rs-model-derive，仅生成模型中的 validator 引用 metadata。
- Rust crate 约束：若 rs-validator 主 crate 需要同时提供普通运行时 API 和过程宏，物理实现采用独立 proc-macro
  companion crate 并从 rs-validator 公开重导出；用户仍统一从 rs-validator 使用 `Validator`，不感知内部拆分。
- 状态更新：V003 已确认。Validator 类型宏最终只支持必填 id/value 参数和无泛型 unit struct，自动生成 Default、
  FieldValidator impl 与链接期 ValidatorRegistration；params/depends_on 只属于小写字段 occurrence。

### F019 `#[generator(...)]`

- 状态：已删除
- 结论：最终设计不提供 `#[generator(...)]`。当前用途和公共契约不够清晰；未来出现经过验证的需求后再独立设计，
  不预留含糊 API。

### F020 `#[codec(...)]`

- 状态：已确认
- 用户意见：codec 有实际价值，但公共语义和契约尚不成熟。需要先根据 Java annotation 与 model 中的实际应用重新
  归纳，再确认 attribute、trait、作用层级以及和 Serde/数据库表示的边界。
- 参考结论：Java `@TypeCodec` 只实际用于 Location、Phone、CredentialInfo，全部标注在类型上而非字段上，codec
  均实现 `Codec<DomainType, String>`；Rust `qubit-codec` 已提供 `ValueEncoder<Input>` 与 `ValueDecoder<Input>` 的
  owned whole-value 契约。证据共同表明 codec 是类型的规范外部表示能力，不应建模成每个字段 occurrence 的策略。
- 已被后续意见修订：不再删除小写字段级 `#[codec(...)]`。大写 ValueCodec 定义并注册 codec 实现；小写 codec
  只在一个字段 occurrence 上选择已注册实现，不在字段上定义编码逻辑。
- 候选 Rust 契约：当前只建模 Java 已验证的规范文本 codec。对领域类型 T，codec 类型 C 必须实现
  `Default + ValueEncoder<T, Output = String> + ValueDecoder<str, Output = T>`；不另造重复 encode/decode trait。
  codec error 必须可转为统一结构化错误。
- 候选边界：codec 实现及注册是类型级能力；字段级小写 codec 可以选择一个不同于领域类型 canonical codec 的
  已注册实现，但不能在字段上定义实现或携带实例配置。codec 是否驱动五种角色的默认 Display/FromStr/Serde，
  在角色默认派生阶段单独确认。
- 用户补充：增加 codec 注册表和 codec 类型宏，让 `#[ValueCodec(id = "...")]` 自动实现 ValueEncoder/ValueDecoder
  并自动注册。
- 已确认命名：使用 `#[ValueCodec(...)]`，避免与 qubit-codec 低层 `Codec` trait 混淆。
- 候选类型宏语法：

  ```rust
  #[ValueCodec(
      id = "qubit.contact.phone",
      value = Phone,
      encode_error = PhoneEncodeError,
      decode_error = PhoneDecodeError,
  )]
  pub struct PhoneCodec;

  impl PhoneCodec {
      fn encode(&mut self, value: &Phone) -> Result<String, PhoneEncodeError>;
      fn decode(&mut self, input: &str) -> Result<Phone, PhoneDecodeError>;
  }
  ```
- 候选宏参数：id、value、encode_error、decode_error 必填；若双向错误类型相同，可使用 `error = CodecError` 替代
  两个 error 参数。宏只支持无泛型 unit struct，自动实现 Default、ValueEncoder<T, Output=String>、
  ValueDecoder<str, Output=T>，并生成链接期跨 crate 自动收集的 ValueCodecRegistration。
- 候选 registry：ValueCodecRegistry 按 ValueCodecId 查询，注册项保存领域 TypeIdentity、外部表示 Text、codec 类型和 erased
  encode/decode 入口；重复 id、trait 类型不匹配是 registry 错误。同一领域类型允许注册多个不同 codec，但对应
  角色宏最多选择其中一个作为 canonical codec；领域类型需显式使用 `codec = RustType` 或 `codec_id = "CodecId"`
  二选一，使 canonical 表示声明可从领域类型定义处直接看到。
- 用户确认命名：类型宏最终命名为 `#[ValueCodec(...)]`，注册表命名为 `ValueCodecRegistry`，稳定 ID 类型命名为
  `ValueCodecId`。
- 用户确认的 crate 归属：大写 ValueCodec 宏、ValueCodecRegistry/Id 和 ValueEncoder/ValueDecoder 契约属于
  rs-codec；小写字段 helper `#[codec(...)]` 属于 rs-model-derive。与 Validator 相同，rs-codec 可通过内部 proc-macro
  companion crate 实现并从主 crate 重导出 `ValueCodec`。
- 候选小写语法：`#[codec(with = PhoneCodec)]` 与 `#[codec(id = "qubit.contact.phone")]` 二选一；前者进行 Rust
  类型和 trait 编译期检查，后者在完整 ValueCodecRegistry 阶段解析。字段最多一个 codec，不支持 params；需要
  不同配置时定义不同 ValueCodec 类型和 ID。
- 候选选择优先级：字段显式 codec 覆盖领域类型的 canonical codec；字段未声明时继承类型 canonical codec；两者
  都没有时不存在 codec。字段显式选择与类型 canonical codec 相同属于冗余声明并报错。
- 状态更新：K003～K005 已确认。ValueCodec 类型宏最终采用 id/value/error 或 encode_error+decode_error 参数，
  自动实现 Default、ValueEncoder<T, Output=String>、ValueDecoder<str, Output=T> 和链接期 ValueCodecRegistration。
  小写字段 codec 最终采用 `with = RustType` / `id = ValueCodecId` 二选一，并按“字段显式 > 类型 canonical > 无”
  的优先级解析。

### F019 决策对 S001 的修正

- 状态：已确认
- 结论：generator 已删除。小写 validator 使用 ValidatorId + params + depends_on；大写 ValueCodec 定义实现，
  小写 codec 使用 `with = RustType` 或 `id = ValueCodecId` 选择实现且不支持 params。C003 最终保留 field validator
  和 field codec selector，两者都不在 rs-model-derive 内执行具体逻辑。

## C005/C006 与递归边界：待确认批次

### F021 `#[redact(...)]`

- 状态：已确认
- 结论：最终设计直接复用 `qubit-redact` 的字段语义，不在 `rs-model-derive` 或 metadata 中发明第二套脱敏
  规则。字段必须且只能选择一种模式：`level = "low|medium|high|secret"`、`skip`、`nested`、`map`、
  `keyed_by = sibling_field` 或 `json`；空参数、重复模式和模式组合均报错。未标注字段保持 unmarked，不根据字段名
  猜测敏感性。
- 用途：`level` 对支持的标量叶子按敏感等级处理；`skip` 在启用脱敏时不输出字段；`nested` 委托给字段类型的
  `Redact` 实现，因此 `EmailAddress` 等包含自身脱敏策略的复杂值必须写 `#[redact(nested)]`；`map` 按文本 key 和
  策略处理 Map value；`keyed_by` 使用同一具名对象的兄弟文本字段为当前 payload 分类；`json` 解析 JSON 文本后按
  JSON 字段策略递归处理。
- 容器穿透：脱敏策略可以穿过 Option 和容器作用于其中的实际值。`Option::None` 保持为空，`Some(value)` 对 value
  应用策略；sequence/set/array 对每个元素应用策略。例如 `#[redact(level = "high")] Vec<String>` 对每个电话号码
  脱敏，`#[redact(level = "medium")] Option<String>` 对 Some 中的 email 脱敏。其他 redact 模式也遵循与其 capability
  相容的透明包装穿透规则；`skip` 等会改变输出结构的模式，其精确容器语义在 F023 中继续确认。
- 边界：属性及其执行实现归 `qubit-redact`；五种角色宏只负责自动派生/联动安全的 Redact、Debug、Display 和
  Serialize，精确的角色默认值留到角色阶段确认。`FieldMetadata` 保存规范化的 `RedactionMode`，供 schema、接口
  文档和安全审查发现字段分类，但真正输出仍必须交给 `qubit-redact`。

### F022 Serde 字段属性与 `#[keep_serializing]`

- 状态：已确认
- 结论：五种角色宏完整保留并支持标准 `#[serde(...)]` 类型、变体和字段属性；显式 Serde 配置优先于宏默认
  值。`rs-model-derive` 不重新定义 `rename`、`skip`、`with`、`flatten` 等 Serde 参数，只把最终序列化名称、方向性
  skip 等可发现事实规范化进 metadata。
- 默认：只对具名字段自动省略 `Option::None` 和直接声明的空标准集合，并在反序列化缺失时补默认值。标准集合
  包括 Vec、VecDeque、LinkedList、HashMap、BTreeMap、HashSet、BTreeSet、BinaryHeap；固定数组不做空值省略。
  newtype、tuple struct 和 enum tuple payload 不自动省略位置，避免改变序列化位置和形状。
- `keep_serializing`：无参数 marker，只允许用在上述具名 Option/集合字段；它仅关闭宏自动添加的
  `skip_serializing_if`，使 None 正常输出为 null、空集合正常输出为空集合，但不关闭反序列化的缺失默认值。它不
  覆盖用户显式写下的 `serde(skip...)`；与不可能被自动省略的字段组合时报冗余错误。

### F023 Option、Box、容器与递归传播

- 状态：已确认

#### F023-A 透明包装与普通递归

- Option、Box、Rc、Arc 是 descriptor 的透明包装。标准约束、validator 和 codec 遇到 None 时跳过；遇到 Some、
  Box、Rc、Arc 时作用于解包后的 payload；metadata 仍保留完整包装结构和 optionality。
- 未 opaque 的命名 Value、Model、Enum 等复杂类型按自身 descriptor 递归校验；它们位于 Option、集合元素或 Map
  key/value 中时同样递归。opaque 截断叶子递归。

#### F023-B selector 的多规则组合

- `element(...)`、`map_key(...)`、`map_value(...)` 允许在同一括号中组合 text、decimal、money、time、validator、
  codec、redact。例如：

  ```rust
  #[map_value(
      text(max_chars = 64),
      validator(id = "qubit.attribute.value"),
      redact(level = "high"),
  )]
  pub attributes: HashMap<String, String>;
  ```

- 同一 selector 可以有多个不同 Validator；最多一个 codec、一个 redact 和每种标准约束；decimal 与 money 互斥。
  selector 禁止 identifier、indexed、unique、reference、computed，也禁止 element/map_key/map_value 的再次嵌套。
  selector 只选择一层容器位置；更深的局部结构应定义为具名 Value 类型。

#### F023-C validator 与 codec 的作用位置

- 直接标在容器字段上的 validator/codec 在去掉外层 Option/所有权包装后作用于整个容器；写在 element、map_key、
  map_value 内的 validator/codec 分别逐元素、逐 key、逐 value 执行。成员自身是 Option 时 None 跳过、Some 解包。
- text、decimal、money、time 不从容器字段自动下沉；必须通过 element/map_key/map_value 选择成员。

#### F023-D redact 的递归能力与结构边界

- 字段级 redact 自动穿透 Option、Box/Rc/Arc、sequence、set、array，直到找到与模式 capability 相容的 payload；
  selector 内 redact 先选择 element、Map key 或 Map value 分支，然后仍可继续穿透该分支内的透明包装与容器。
- Map 上的字段级 redact 默认只递归 value，不修改 key；Map key 必须显式使用 `map_key(redact(...))`。同一路径同时
  声明字段级 redact 与 selector redact属于重复或歧义，编译报错。
- level 寻找支持标量脱敏的叶子，nested 寻找实现 Redact 的值，json 寻找支持的 JSON 文本，map 寻找 Map，
  keyed_by 使用兄弟字段为当前 payload 分类并递归处理。
- `#[redact(skip)]` 是唯一结构性例外：它始终省略整个字段，不递归到成员；禁止
  `element(redact(skip))`、`map_key(redact(skip))` 和 `map_value(redact(skip))`。Map key 脱敏若产生重复输出 key，
  安全序列化必须返回结构化错误，不得静默覆盖条目。

### F024 `#[computed(...)]` 与 `#[ModelProperties]`

- 状态：已删除（被 R008-A/R008-B 取代）
- 语法：由于标注 struct 的角色宏在 Rust 中看不到另一个 inherent impl 的方法，采用一个实现块宏收集 getter：

  ```rust
  #[ModelProperties]
  impl User {
      #[computed(depends_on(first_name, last_name, email))]
      pub fn info(&self) -> UserInfo {
          // ...
      }
  }
  ```

  `#[computed]` 表示未显式列依赖，运行时保守视为依赖全部存储字段；
  `#[computed(depends_on(...))]` 精确列出存储字段或其他 computed property 路径；允许空列表表示已知无依赖。
  不支持 name/codec/indexed/unique/reference 等参数，property 名就是方法名。
- getter 契约：必须是 public、同步、安全、非泛型、无额外参数的 `fn(&self) -> T`，返回 owned `'static` 值；
  Option 或复杂对象可以作为 T。无副作用和只由声明依赖决定是语义契约，Rust 类型系统不能完全证明。依赖路径必须
  存在，computed 依赖图不得成环。
- metadata：`ModelProperties` 为所属角色注册 computed property 的名称、返回 descriptor、依赖集合和只读
  erased getter。computed property 可被 metadata/schema、递归 validation 和 `reference(property = ...)` 使用，
  但不是持久化列、构造输入或随机生成槽位；生成器只在依赖就绪后调用 getter。返回复杂类型的约束和脱敏策略来自
  返回类型自身的 descriptor/Redact 实现。

## 字段/变体语义收尾批次：待确认

### F025 `#[variant(...)]`

- 状态：已确认
- 结论：`#[variant(name = "...")]` 只允许标在 Enum variant 上，唯一参数 name 用于覆盖规范变体名；省略时
  由 Rust variant 名转换为 SCREAMING_SNAKE_CASE。规范名必须非空且在同一 Enum 内唯一。
- 消费：规范名进入 EnumVariantMetadata，并供 Enum 的 name、Display 以及 unit Enum 的 from_name 使用。
  五种角色的默认 Serde 行为稍后确认；候选联动是 variant name 同时作为宏生成的默认 Serde wire name，但用户
  显式 serde rename 仍按 F022 优先，可以有意形成不同 wire name。metadata 同时暴露 canonical、serialize 和
  deserialize name，避免隐式混淆。
- 边界：不增加 code、ordinal、weight、default 等参数；ordinal 来自源码声明顺序，随机生成的 variant 权重是
  生成器运行配置，不进入领域模型标注。

### F026 不引入字段级 `modified` / `unmodified`

- 状态：已确认
- 参考结论：Java `@Modified` / `@Unmodified` 实际标在 DAO 方法上，描述某次 add/update/delete 操作应修改或保持的
  字段集合，不是字段自身永恒不变的属性。同一字段在不同操作中的期望不同。
- 结论：最终 `rs-model-derive` 不提供字段级 modified/unmodified。未来需要时应由 DAO/API 操作 metadata 在方法
  上按字段路径声明；模型 metadata 只提供字段路径供其引用。

### F027 不引入 `#[exclude]`

- 状态：已确认
- 参考结论：Java `@Exclude` 只在 common-random 的测试夹具中用于排除随机化，属于一个生成器 consumer 的选择，
  不是字段的 validation、持久化或输出不变量。
- 结论：最终模型字段不提供通用 exclude。无法默认生成的外部叶子使用已确认的 opaque + 调用方供值；仅在某次
  生成任务中排除普通字段，由生成请求按结构化字段路径配置，不把特定生成器策略固化进模型 metadata。

### F028 不引入 `#[key_index(...)]`

- 状态：已否决
- 参考结论：Java `@KeyIndex(n)` 用来规定复杂对象被展平为复合数据库索引或字符串 key 时的成员顺序。
- 候选结论：最终模型字段不提供 key_index。查询路径按已确认的结构化 path 表示，物理复合数据库索引不属于
  rs-model-derive；复杂值的规范字符串表示由 ValueCodec 决定；需要稳定遍历时 metadata 使用源码字段声明顺序。
  因此额外的整数顺序会制造第二套可漂移顺序，且没有独立领域语义。
- 否决理由：Java 实际消费者表明 KeyIndex 不建立数据库索引，而是复杂对象形成稳定复合键字符串时的分量排序
  metadata。DAO 自动化测试和 UniqueValueRandomizer 都依赖它避免反射字段遍历顺序不稳定，因此需要按该真实语义
  重新设计，而不是删除。

### F028-R `#[key_index(...)]` 修订方案

- 状态：已被新证据修订

#### F028-A 语义与分类

- `#[key_index(n)]` 是“规范复合键组成”类别的字段 metadata：n 从 0 开始，表示当前字段在所属对象的复合键分量
  中的位置。它只确定分量选择和顺序，不建立数据库索引，不影响 SQL/ORM 映射，也不自动赋予 indexed、unique 或
  identifier 语义。
- DAO 自动化测试、唯一值缓存和其他需要稳定对象键的 consumer 使用同一份有序 CompositeKeyMetadata，不再依赖
  Rust 字段声明顺序或各自实现反射式排序。

#### F028-B 语法、作用域与完整性

- 唯一语法为无名位置参数 `#[key_index(0)]`，参数类型为 u16；不支持 name/order/value 等别名。
- 只允许具名字段的 Model 和具名字段 Value 使用。Entity/Projection 已由 identifier 形成对象键，Enum 由规范
  variant name + payload 形成自身表示，newtype 只有一个隐含分量，因此这些位置禁止 key_index 以避免无效 metadata。
- 一个类型只要出现任意 key_index，它的所有存储字段就必须全部标注；索引必须恰好构成连续集合 0..field_count，
  不允许重复、缺号或越界。computed property 不是存储字段，不参与也禁止标注。这样保证复合键不退回源码字段顺序。

#### F028-C 分量 capability 与递归

- 每个 key component 必须具有 canonical key representation capability：内建 scalar、Enum、identifier-bearing
  Entity/Projection、具有 canonical ValueCodec 的类型，或自身具有完整 CompositeKeyMetadata 的 Model/Value。
- Option 必须区分 None 与 Some(empty)；sequence/array 保留元素顺序；Set 和 Map 必须按各成员的 canonical encoded
  bytes 排序后编码，不能依赖 Hash 迭代顺序；opaque 只有提供 key capability adapter 或 canonical codec 时才允许。
- key_index 只声明顺序，不自行规定使用连字符等容易碰撞的文本格式。公共 key encoder 使用无歧义的长度/类型边界
  编码，并可另行提供稳定文本封装；其精确 API 留到 Runtime 公共 API 阶段确认。

#### F028-D 选择优先级与安全边界

- 候选对象键规则：Entity/Projection 始终使用 identifier；内建 scalar 与 Enum 使用各自规范值；具名 Model/Value
  可以使用完整 CompositeKeyMetadata。类型的 canonical ValueCodec 是另一种规范外部表示；同一个 Model/Value
  若同时声明 canonical ValueCodec 与 key_index，会形成两个竞争的规范键来源，因此编译报错，必须明确二选一。
- CompositeKeyMetadata 与 canonical key 只用于等值比较、缓存、测试装配和稳定协议键，不等价于 Display/Serde，
  也不得绕过 redact 直接写日志。key component 即使带 redact，仍可参与内部 key 编码；输出该键时由调用方承担敏感
  信息保护。

### F028-R2 `#[key_part(...)]` 精确收敛方案

- 状态：已确认
- 新证据：Java 当前只有两个消费者：DAO 集成测试把 respectTo 复杂值按字段顺序格式化为重复键诊断文本；
  UniqueValueRandomizer 在 unique(ignore_case) 的字段本身是对象时，用该字段的字符串投影做去重比较。它不用于
  数据库索引、ORM、查询、对象装配、业务身份或 respectTo 随机去重键。
- 命名：删除含义易混淆的 `key_index`，改为 `#[key_part(order = 0)]`。key part 表示当前字段是对象 key-text
  projection 的一个分量，order 只表示分量顺序。
- 语义：该标注仅生成 `KeyTextMetadata`，供需要把复杂对象投影为有序 key components 的消费者使用；它不
  自动生成业务 key，不改变 Eq/Hash/Display/Serde，不隐含 identifier/indexed/unique，也不规定数据库行为。
- 使用场景：key part 投影可用于 `unique(ignore_case = true)` 和 `unique(ignore_case = false)` 的复杂字段比较与去重；
  也可用于把 `respect_to(...)` 中的复杂属性值投影为复合键。当前实现即使暂不消费后一能力，metadata 和公共契约也
  必须保留这种可能。
- 完整性：只允许具名 Model/Value。未标注字段不属于 key-text projection；所有已标注字段的 order 必须从 0 开始
  连续、无重复无缺号。Entity/Projection 直接使用 identifier，Enum/newtype 有自身规范表示，computed 不是存储
  字段，均禁止 key_part。
- 运行时边界：metadata 暴露按 order 排列的字段访问器。底层比较应使用结构化 `KeyComponentValue`，以区分
  None、空字符串和容器边界；DAO 异常诊断可再选择 `-` 等分隔符渲染为文本。ValueCodec 是外部编码能力，可以与
  key_part 共存，不再设置隐式优先级；consumer 必须明确请求 key-text projection 或 codec。
- 安全边界：key-text projection 可能包含敏感值，只供内部比较和受控诊断；它不绕过 redact，也不应自动进入
  日志、Display 或 Serialize。

## 延后确认：五种角色的语义与用途

以下条目按用户要求延后到全部字段标注确认完成后再确认。

### R001 `Entity` 的语义与用途

- 状态：已确认
- 结论：Entity 是具有稳定种类身份、实例身份、独立领域生命周期和独立持久化生命周期的领域对象。它是唯一
  可以作为 reference 目标、拥有 `ModelId`、identifier、unique 和显式 index 的角色。典型用途是 User、Order、
  Tenant、Organization 等可独立创建、查找、更新和删除的对象。

### R006 `#[Entity]` 的形状与角色参数

- 状态：已确认（按用户修订）
- 形状：只支持无泛型、无 lifetime、无 where 子句的具名字段 struct；字段可为任意可见性。禁止 unit/tuple struct、
  enum 和 union。
- 参数：`#[Entity(id = "ModelId")]`，id 必填；不使用 model_id。Entity 角色参数不包含 index、unique、projection、
  primary_key，这些事实由字段 annotation、identifier 和 computed/property 协议表达；是否新增 singular
  projection/projection_id 因新意见重新评估，尚未确认。
- ModelId：全局唯一、区分大小写，是一个 Java fully-qualified class name 风格的字符串。它可以包含句点，句点分隔
  出的每一段都必须符合项目约定的 Java namespace/name segment 要求；精确 ASCII 语法为
  `Segment ("." Segment)*`，其中 `Segment = [A-Za-z][A-Za-z0-9_]*`。因此不含句点的单段名称同样合法；不得有空段、
  前导句点或尾随句点。推荐 namespace 段使用 lower_snake_case、类型段使用 UpperCamelCase，但不强制最后一段等于
  Rust 类型名。

### R007 `#[Projection]` 的形状与来源参数

- 状态：已确认
- 形状：只支持无泛型、无 lifetime、无 where 子句的具名字段 struct，必须具有唯一的直接 Id identifier。
- 参数：`#[Projection]`、`#[Projection(source = EntityType)]`、
  `#[Projection(source_id = "EntityModelId")]`。source/source_id 最多一个且效果等价；前者编译期验证 Entity 协议，
  后者在完整注册表解析。省略表示开放 Projection，可由多个 Entity 产生；指定则限制来源。Projection 自身没有
  ModelId，source 只表达来源约束与数据血缘，不是注册 ID，也不提供 projector 实现。

### R006-P Entity 是否声明 `projection` / `projection_id`

- 状态：已确认
- 候选结论：不增加 singular projection/projection_id。Projection 与 Entity 在领域上是多对多来源关系：同一 Entity
  可以按公开摘要、管理视图、列表摘要、带状态摘要等用途产生多个 Projection；开放 Projection 也可以由多个 Entity
  产生。Java 当前常见单一 info() 主要受统一接口方法名及不能按返回类型重载的限制，不应上升为领域基数约束。
- 证据：Java/Rust 同时存在通用 Info、StatefulInfo、InfoWithEntity 和 UserInfo/PersonInfo/EmployeeInfo 等具体投影族；
  Product.getInfo(specification) 还表明 projector 可能带上下文参数。未来出现更多 API/权限场景时，多 Projection 很
  自然。
- 关系发现：fixed Projection 通过 source/source_id 声明来源；开放 Projection 通过 ModelProperties 中返回它的
  property getter 建立 producer。Registry 可反向计算 Entity 的 projections，不需要 Entity 重复列举。
- 未来若确有一个“默认 info Projection”用于自动实现 HasSpecificInfo，应新增语义明确的 default_projection，而非
  把所有 Projection 关系错误限制为一个；当前没有上移 trait 的需求，因此不预留该参数。

### R008-R Field、Property 与 computed 推导

- 状态：已确认
- 结论：删除 `#[computed]` 及 depends_on。ModelMetadata 明确分开 fields 与 properties：field 是 struct 的
  真实存储字段，不受可见性影响；property 是按名称合并后的可读属性集合。
- 每个 field 自动形成同名 property。`#[ModelProperties] impl Type` 中符合 getter 形状的公开方法登记为 property
  getter；若存在同名 field，则合并为一个 field-backed property；若不存在同名 field，则自然推导为 computed
  property。computed 因而是 `property.field().is_none()` 的派生事实，不需要由人重复标注。
- reference(property = name) 直接解析统一 PropertyMetadata；可选择 field-only、field+getter 或 getter-only
  property，不依赖 computed 标记。读取时有显式 getter则使用 getter，否则使用生成的 field accessor。
- `#[ModelProperties]` 是有意暴露 property 的 impl 边界。getter 采用 Rust 风格
  `pub fn xxxx(&self) -> T`；setter 采用 Java Bean 对应规则 `pub fn set_xxxx(&mut self, value: T)`，属性名均为 xxxx。
  两者必须同步、安全、非泛型；getter 无额外参数且返回非 unit，setter 恰好一个值参数且返回 unit。
- property 名称集合是 field、getter、setter 名称的并集并按名称合并。field 或 getter 使 property readable；field 或
  setter 使 property writable。因此允许 field-only、field+getter/setter、getter-only（computed、只读）和
  setter-only（virtual、只写）。有显式 getter/setter 时动态访问优先调用方法，否则使用宏生成的 field accessor。
- 所有普通 owned field 在获得 &self/&mut self 时都可读写，Rust 字段没有 Java final 的等价声明；字段可见性不影响
  宏在定义作用域内生成访问器。带 lifetime 的借用字段因角色类型不支持 lifetime 参数而不进入当前模型；
  `&'static str` 技术上仍可重新赋另一个 static str，并非天然只读。
- PropertyMetadata 至少暴露 name、value descriptor、可选 FieldMetadata、可选 getter/setter、readable、writable 和
  storage kind。getter/setter 与 field 类型兼容性、borrowed/owned erased access ABI 在 Runtime API 阶段确认。

### R008-C Property getter/setter 精确协议

- 状态：已确认
- `#[ModelProperties]` 自动识别 public、同步、safe、非泛型的 `fn xxxx(&self) -> T` getter，以及 public、同步、
  safe、非泛型、返回 unit 的 `fn set_xxxx(&mut self, value: T)` setter；属性名均为 xxxx。
- field/getter/setter 按属性名合并。field 或 getter 使属性 readable；field 或 setter 使属性 writable；允许
  getter-only computed property 和 setter-only virtual property。显式方法优先于生成的 field accessor。
- getter 与 field/setter 通过 property type adapter 检查语义兼容而非要求语法类型完全相同；首批至少支持 T/&T、
  String/str、Vec<T>/[T]、Option<T>/Option<&T> 等 owned/borrowed 视图。精确 erased ABI 延后到 Runtime API。

### R009-G 泛型 `#[Model]`

- 状态：已确认
- 结论：支持具有类型参数的具名字段或 unit Model；禁止 lifetime 参数，const generic 是否首版支持另行确认。
  字段使用到的类型参数必须在具体实例化时满足 HasTypeDescriptor 及相应字段 capability。
- 注册必须分两层：链接期注册一次 GenericModelTemplate，字段 shape 可含 TypeParameter(index)；具体
  `Page<UserInfo>` 通过模板替换参数得到 ConcreteTypeDescriptor，并按具体 Rust TypeIdentity 缓存。链接期不可能
  枚举所有未来单态化，因此 concrete descriptor 不作为预先分布式注册项。
- ModelRegistry 可枚举普通 Model 和 generic templates；调用 `TypeMetadata::of::<Page<UserInfo>>()` 或
  `template.instantiate(...)` 获得具体 descriptor。validation/schema/random generation 始终在具体 descriptor 上运行；
  只有模板可用时只能检查与类型参数无关的结构规则。
- 泛型 definition 仍算“参与注册”，但公共 API 必须明确 template registration 与 concrete descriptor 的区别，不能
  声称每个潜在实例都已链接。reference、角色组合和 capability 在模板实例化后使用具体参数角色完成校验。

### R009-H 泛型参数范围

- 状态：已确认
- Model 支持类型参数、`const N: usize` 和 where 子句；禁止 lifetime 参数。GenericModelTemplate 同时记录类型参数
  和 usize const 参数，具体实例通过参数替换产生并缓存 descriptor。

### R010 `#[Enum]` 形状与角色参数

- 状态：已确认
- 支持 unit、tuple、struct 和混合 variant；支持类型参数、`const N: usize` 与 where 子句；禁止 lifetime 和 union。
  泛型 Enum 注册 template，具体实例 descriptor 按需产生。Enum 没有角色专属参数，variant 规范名继续使用已确认的
  `#[variant(name = "...")]`。

### R011 `#[Value]` 形状与角色参数

- 状态：已确认
- 已确认形状：具名字段 struct 或单字段 tuple newtype；支持类型参数、`const N: usize`、where 子句；禁止 unit、
  多字段 tuple、enum、lifetime 和 union。
- 已确认方向：删除 textual；具名 Value 的文本能力来自明确 capability trait 或 canonical ValueCodec，不能只靠一个
  无可执行协议的 marker。

### R011-T `#[Value(transparent)]`

- 状态：已确认
- `transparent` 只用于恰好包含一个存储字段的 Value；允许单字段 tuple struct 和只有一个字段的 named struct，
  property 不计入存储字段数量。多字段 Value 禁止使用。
- 透明 Value 保持独立的 Rust 名义类型和 Value descriptor；metadata 记录 `transparent = true`、唯一内部字段及其类型、
  约束和策略。消费者不能把它与内部类型视为同一个类型。
- Serialize、Deserialize 和 Display 使用内部值的外部表示，不增加对象层或 newtype 标签；Debug 保留 Value 类型名以便
  诊断。Redact 保持透明表示，但仍执行唯一字段声明的脱敏规则。
- transparent 不删除内部字段的约束、validator、codec、key_part 等 metadata，也不自动生成 Deref、From、Into 或
  TryFrom。表示透明不代表从内部值构造该 Value 必然无条件合法。

### R012-A 五种角色的默认能力

- 状态：已确认
- Entity、Projection、Model、Enum、Value 默认实现 Clone、Debug、Display、PartialEq、Eq、Hash、Redact、Serialize、
  Deserialize。
- Copy 默认只用于全部 variant 都是 unit 的 Enum；其他类型不默认 Copy。Default、PartialOrd、Ord 均不默认实现。
- 默认 PartialEq/Eq/Hash 使用标准结构化语义；Entity 不擅自改成只比较 identifier。需要 identifier-only equality 时应
  关闭默认实现并自行实现。
- Default 不默认实现，因为字段默认值不保证满足模型约束；Ord 不默认实现，因为字段或 variant 声明顺序不天然代表
  领域顺序。泛型类型的能力实现带相应条件约束。

### R012-B 移除默认能力

- 状态：已确认
- 五种角色统一支持 no_clone、no_debug、no_display、no_partial_eq、no_eq、no_hash、no_redact、no_serialize、
  no_deserialize；全 unit Enum 另支持 no_copy。
- no_clone 与 copy 冲突；no_partial_eq 同时移除 Eq、Hash、PartialOrd、Ord；no_eq 保留 PartialEq，但同时移除默认 Hash
  并禁止 Ord；no_hash 只移除 Hash。
- no_debug/no_display 只关闭对应格式化接口，不移除 Redact。no_serialize/no_deserialize 相互独立。
- no_redact 仅当类型和 element/map_key/map_value selector 中完全没有 redact 规则时允许；否则编译错误。关闭后仍保留
  的 Debug、Display、Serialize 使用普通非脱敏实现。
- 不提供 no_default/no_ord 等关闭参数，因为这些能力本来不默认启用。

### R012-C 增加非默认能力

- 状态：已确认
- 五种角色统一支持 copy、default、partial_ord、ord。copy 要求 Clone 未关闭且所有存储字段实现 Copy；全 unit Enum
  已默认 Copy。
- default 对 struct 使用字段级 Default；Enum 要求恰好一个标有标准 `#[default]` 的 unit variant。该能力只保证产生
  Rust 默认值，不保证通过约束与 validator。
- partial_ord 要求保留 PartialEq；ord 同时启用 PartialEq、Eq、PartialOrd、Ord，并与 no_eq/no_partial_eq 冲突。
- 用户显式 `#[derive(...)]` 已包含相同 trait 时，角色宏将其视为已启用，不重复产生实现。

### R014 默认能力与脱敏联动

- 状态：已确认
- 五种角色默认实现 Redact，字段 redact 规则不再要求类型宏添加 redact 参数；删除旧类型级 redact 参数。
- 默认 Debug、Display 和 Serialize 执行字段声明的脱敏策略；Deserialize 只负责输入，不应用脱敏。未声明 redact 的字段
  正常输出；nested、level、map、keyed_by、json 及容器 selector 按 F021～F023 的规则执行。
- no_debug/no_display/no_serialize 只关闭对应接口，不关闭 Redact。no_redact 只允许无任何脱敏规则的类型，并使保留的
  Debug、Display、Serialize 恢复普通实现。

### R015-A `ModelId` 的唯一职责

- 状态：部分结论已被 R015-F 替换
- 保留结论：ModelId 用于在 Rust 类型系统之外稳定标识一个可动态发现的模型类型或泛型模板；
  `#[Entity(id = "...")]` 必填。ModelId 在最终注册表中全局唯一，并可跨进程、服务、语言和版本使用。
- 替换结论：ModelId 不再是 Entity 专属；Projection、Model、Enum、Value 可通过可选 id 获得 ModelId 并注册。
- `reference(entity_id = "...")` 与 `Projection(source_id = "...")` 引用的是 Entity ModelId。Entity 宏参数 id 标识
  实体类型，`#[identifier]` 字段标识实体实例，二者语义不同。

### R015-B 五种角色的注册规则

- 状态：已由 R015-B2/F/G/H/E2 完成修订
- 不再存在“没有稳定 ID 但仍自动注册”的类型；具体规则见后续条目。

### R015-C Value 不注册但拥有完整 metadata

- 状态：metadata 结论保留，注册结论已被 R015-F 替换
- Value 不进入全局注册表不等于没有 metadata。每个 Value 都实现统一静态类型描述接口，既可通过具体 Rust 类型直接
  获取，也可由外层 descriptor 递归进入其完整约束、validator、codec、redact 和透明包装信息。
- 未声明 id 的 Value 不注册；声明 id 的 Value 进入注册表。无论是否注册，Value 都不承担实体关系目标身份。

### R015-D 泛型类型的注册规则

- 状态：已确认，并由 R015-H 补充
- 对声明 id 的泛型 Model、Enum、Value，链接期只注册泛型模板，不枚举潜在 concrete 实例；具体实例由
  `TypeMetadata::of::<ConcreteType>()` 按需构造并以当前进程 Rust 类型身份缓存。
- 模板的 id 标识模板自身；具体实例不自动合成 ModelId，也不自动成为新的注册项。未声明 id 的泛型类型不注册模板。

### R015-E 五种类型宏的最终参数集合

- 状态：已由 R015-E2 完成修订
- Value 明确支持 `copy`，典型形式为 `#[Value(transparent, copy)]`；要求唯一内部值或所有存储字段实现 Copy，且 Clone
  未关闭。以 String、Vec 等非 Copy 类型为字段的 Value 不能启用 copy。

### R015-B2 注册表的第一性原理

- 状态：已确认
- 注册表服务于调用方仅持有稳定 ModelId、并不知道具体 Rust 类型时的动态发现。只有稳定 ID 的类型才进入注册表；
  已知 Rust 类型时的 metadata 获取、外层字段递归和“证明类型存在”不构成注册理由。
- 不再保留只可枚举、不可按稳定 ID 查询的匿名注册项。

### R015-F 所有角色的可注册性

- 状态：已确认
- Entity 的 id 必填并始终注册；Projection、Model、Enum、Value 的 id 可选，提供 id 时注册，省略时不注册。
- ModelId 是模型系统中可动态发现的类型或泛型模板的稳定标识，所有角色共享同一个全局唯一命名空间。
- reference.entity_id 和 Projection.source_id 仍只能解析到 Entity；ModelId 与实例 `#[identifier]` 完全不同。

### R015-G 注册与角色语义正交

- 状态：已确认
- id/注册只增加动态发现能力，不改变 identifier、relation、持久化、生命周期、组合限制等角色语义。注册后的 Value
  仍是 Value，注册后的 Enum/Model/Projection 也不会获得 Entity 身份。

### R015-H 泛型类型的稳定 ID

- 状态：已确认
- 泛型 Model、Enum、Value 的 id 标识泛型模板；注册表按 id 返回模板 metadata。具体实例通过 Rust concrete type
  按需实例化并缓存，但首版不拼接或合成具体实例的稳定 ModelId。
- 将来若需要字符串表示 concrete 泛型实例，应单独设计规范化 TypeExpression，不能临时拼接 Rust 风格字符串。

### R015-E2 类型宏参数修订

- 状态：已确认
- Entity 的 id 必填；Projection、Model、Enum、Value 的 id 可选并决定是否注册。Projection 的 source/source_id
  规则不变，Value 明确支持 transparent 与 copy；其余已确认能力增删参数保持不变。

### A001-S 静态 metadata 获取入口

- 状态：已确认
- 统一使用 `TypeMetadata::of::<T>() -> &'static TypeMetadata` 获取已知 Rust 类型的 metadata；要求
  `T: HasTypeMetadata + 'static`。适用于 Entity、Projection、Model、Enum、Value，无论该类型是否注册。
- 删除自由函数 `metadata_of::<T>()`，不保留两套同义入口。保留公开 HasTypeMetadata trait 供泛型约束和用户自定义
  实现；不向用户类型注入可能发生命名冲突的固有 `metadata()` 方法。
- 静态类型查询不返回 Option；不满足 HasTypeMetadata 时编译失败。只持有稳定 ModelId 时使用
  `ModelRegistry::global().get(model_id)` 动态查询。

### A001-T `TypeMetadata` 与 `TypeDescriptor` 的边界

- 状态：已确认
- TypeMetadata 只描述 Entity、Projection、Model、Enum、Value 五种领域声明类型；静态入口为
  `TypeMetadata::of::<T>()`。TypeDescriptor 描述模型系统可理解的任意 Rust 类型，包括标量、容器、包装、tuple、
  opaque、领域声明类型、泛型参数和具体泛型实例；静态入口为 `TypeDescriptor::of::<T>()`。
- 字段和 property 的类型统一返回 TypeDescriptor；TypeDescriptor 可通过 metadata() 查询其是否对应领域声明类型。
  普通 String 具有 TypeDescriptor，但不具有 TypeMetadata。

### A001-I Rust 类型身份、ModelId 与泛型来源

- 状态：已确认
- 语义方向确认：公开当前编译产物内的 Rust 类型 ID、诊断用 Rust 类型名、可选 ModelId、具体泛型实例的泛型定义来源，
  以及当前具体 metadata 是否直接注册。
- 已确认命名：`TypeMetadata::type_id() -> std::any::TypeId`，直接返回 Rust 标准库 TypeId；
  `TypeMetadata::type_name() -> &'static str` 返回诊断用完整 Rust 类型名。不定义 RustTypeIdentity、RustTypeIdentifier、
  RustTypeId 或模型库自有 TypeId 包装，避免与标准类型重名。
- `TypeMetadata::generic_definition() -> Option<&'static GenericTypeMetadata>` 返回具体泛型实例来源的泛型定义；
  非泛型类型返回 None。采用 generic_definition 而非 template/raw_type，避免含糊及 Java raw type 擦除语义。
- `model_id() -> Option<ModelId>` 与 `is_registered()` 保持：ModelId 是稳定注册身份；标准 TypeId 只表示当前程序内
  具体 Rust 类型身份。

### A001-R 角色判断与角色专属 metadata

- 状态：已确认
- ModelRole 包含 Entity、Projection、Model、Enum、Value。TypeMetadata 提供 role()、role_metadata() 以及
  as_entity/as_projection/as_model/as_enum/as_value；角色不匹配返回 None，不提供 panic 型便利接口。
- ModelMetadata 专指 `#[Model]` 角色 metadata；五种角色的统一入口保持 TypeMetadata。

### A002-F 存储字段 metadata

- 状态：已确认，并由 A002-F2/I/V 固化
- TypeMetadata 提供 fields、field(name)、field_at(index)。真实存储字段不受可见性影响；tuple Value 的唯一字段 name
  为 None/index 为 0；Enum 顶层 fields 为空，payload 字段属于 variant metadata。
- FieldMetadata 的类型访问器由 ty() 改为 descriptor()；增加 is_unique() 与 is_reference()，分别等价于
  unique().is_some() 和 reference().is_some()。其他已提出的字段索引、名称、可见性、attributes、identifier、indexed、
  constraints、validators、codec、redact 查询方向确认。

### A002-P Property metadata

- 状态：已确认，并由 A002-P2 固化
- TypeMetadata 提供 properties() 与 property(name)。PropertyMetadata 的类型访问器由 ty() 改为 descriptor()；增加
  is_field()、is_getter()、is_setter()。field/getter/setter、readable/writable/computed 和 storage_kind 的主体方向确认。
- is_field/is_getter/is_setter 分别表示存在同名 field、显式 getter、显式 setter；其与 readable/writable 的精确逻辑见
  A002-P2。

### A002-F2 `FieldMetadata` 基础接口

- 状态：已确认
- 类型访问器统一为 descriptor()。提供 index、可选 name、visibility、attributes、identifier/indexed/unique/reference、
  constraints、validators、codec、redact 等查询；is_identifier/is_unique/is_reference 分别等价于对应 metadata 是否存在。

### A002-I 多重索引原因

- 状态：已确认
- 使用 `indexing_reasons() -> IndexingReasons` 而非单数 source；原因集合包含 EXPLICIT、IDENTIFIER、UNIQUE、REFERENCE。
  is_indexed 等价于原因集合非空。显式 indexed 与任何隐含 indexed 原因重复时仍编译报错，但 unique 与 reference 等多个
  隐含原因可以同时存在。

### A002-P2 `PropertyMetadata` 便利方法

- 状态：已确认，并修正 readable/writable 措辞
- descriptor() 返回 property 类型描述。is_field/is_getter/is_setter 分别等价于 field/getter/setter metadata 是否存在；
  一个 property 可同时具备三者。
- 存在 getter 必然 readable，存在 setter 必然 writable；但反方向不成立：没有显式 getter 的 field-backed property 仍然
  readable，没有显式 setter 的 field-backed property 仍然 writable。
- is_readable 等价于 is_field || is_getter；is_writable 等价于 is_field || is_setter。显式 getter/setter 优先于生成的
  field accessor。

### A002-V 字段可见性 metadata

- 状态：已确认并按意见细分
- `FieldVisibility` 定义为 Public、Crate、Super、Path(&'static str)、Private。pub 映射 Public，pub(crate) 映射
  Crate，pub(super) 映射 Super，pub(in path) 映射 Path。pub(self) 及无 pub 归一化为 Private；等价的
  pub(in crate)/pub(in super)/pub(in self) 分别归一化为 Crate/Super/Private。
- visibility 只描述 Rust 源声明，不决定 metadata 访问能力或 property readable/writable。

### R002 `Projection` 的语义与用途

- 状态：已确认
- 结论：Projection 是某个 Entity 实例的派生表示，借用来源 Entity 的实例 identifier，但没有自己的
  `ModelId`、独立生命周期或独立持久化记录。典型用途是 UserInfo、OrderSummary 等关联值、列表摘要和传输视图。
  通用 Projection 可以表示多种 Entity，固定来源 Projection 只表示一种 Entity。

### R003 `Model` 的语义与用途

- 状态：已确认
- 结论：Model 是可被框架全局发现、校验和生成的数据契约，但没有 `ModelId`、identifier 或独立持久化
  生命周期。它可以声明 Entity relation。典型用途是请求/响应 DTO、查询参数、配置、命令、组合结果和对象图根节点。

### R004 `Enum` 的语义与用途

- 状态：已确认
- 结论：Enum 是封闭值域或代数和类型，支持 unit、tuple、struct 和混合 variant；它参与全局注册，但没有
  `ModelId`、identifier、独立持久化或 direct relation。典型用途是状态、分类和带数据的互斥结果。

### R005 `Value` 的语义与用途

- 状态：已确认
- 结论：Value 是按内容定义相等性、没有身份和独立生命周期的值对象。它拥有完整静态 descriptor，但不参与
  全局注册；外层类型仍能递归校验和生成它。Value 不能包含 Entity、Projection、Model 或 relation。典型用途是
  EmailAddress、Phone、Money、Revision 等可复用领域值。

## 后续确认清单

### 身份、持久化、关系与投影字段宏

- F001 `#[identifier]`
- F002 `#[unique(...)]`
- F003 `#[indexed]`
- F004 `#[reference(...)]` 的 `entity` / `entity_id`
- F005 `#[reference(...)]` 的 `property`
- F006 `#[reference(...)]` 的 `existing`
- F007 `#[reference(...)]` 的 `path`
- F008 reference 自动索引及重复 `#[indexed]` 诊断
- F009 Projection 的来源约束与 relation 中 Projection 的类型校验

### 局部约束与字段策略

- F010 `#[text(...)]`
- F011 `#[decimal(...)]`
- F012 `#[money(...)]`
- F013 `#[time(...)]`
- F014 `#[sequence(...)]`
- F015 `#[map(...)]`
- F016 `#[element(...)]`
- F017 `#[opaque]`
- F018 `#[validator(...)]`
- F019 `#[generator(...)]`
- F020 `#[codec(...)]`
- F021 `#[redact(...)]` 与 nested/map/level 等用法边界
- F022 `#[variant(...)]`
- F023 Option、容器、Box 与递归约束语义

### 角色参数与默认行为（全部字段标注确认后开始）

- R006 Entity 支持的 Rust 形状与宏参数
- R007 Projection 支持的 Rust 形状与 `source` / `source_id`
- R008 Projection 转换与 `property = info` 的可执行协议
- R009 Model 支持的 Rust 形状与宏参数
- R010 Enum 支持的形状、宏参数与 variant 规范名
- R011 Value 支持的形状、`transparent` 与 `textual`
- R012 五种宏的默认派生集合
- R013 `no_*` 控制参数及能力依赖规则
- R014 类型级与字段级 redaction 的默认联动
- R015 五种角色的注册、`ModelId` 和 identifier 规则

### Runtime 公共 API

- A001 `TypeDescriptor`、`TypeMetadata`、`TypeKind`、`TypeShape`
- A002 `FieldMetadata`、typed constraints 与策略查询
- A003 五种角色 metadata 与角色 traits
- A004 registration、registry 与 `ModelId` / `TypeIdentity` 查询
- A005 reference、projection、index 与路径 metadata
- A006 安全字段访问、构造与 Enum variant API
- A007 schema 校验、实例校验和错误 API
- A008 随机生成、DAO 测试与 provenance/依赖规划 API

## 确认历史

| 轮次 | 条目 | 结果 |
| --- | --- | --- |
| 0 | P001、P002 | 已确认 |
| 1 | R001～R005 | 调整顺序，延后确认 |
| 2 | C001～C005 | 已确认；computed 从 C001 移出，lookup_relation/ownership 删除 |
| 3 | C006 | 已确认，computed 独立成类 |
| 4 | F001～F004 | F003、F004 已确认；F001 待确认；F002 按查询过滤语义重拟 |
| 5 | F002-A～F002-E | 全部已确认；确定查询能力来源、根对象唯一键排除、普通复杂值展开、reference 一跳边界及不表达物理组合索引 |
| 6 | F001、F010～F012 | F011、F012 已确认；F001 保留数据库分配语义并待定准确名称；F010 主体确认并扩展 allowed_chars 设计 |
| 7 | F001-B、F010-B～F010-D | 全部已确认；ID 使用 assigned_by；allowed_chars 默认 unicode，并增加 printable/ascii/code 精确定义 |
| 8 | F013～F016 | F013、F014、F016 已确认；F015 容器部分确认，补充对称的 map_key/map_value 方案待确认 |
| 9 | F015-B～F015-D | 全部已确认；Map 使用独立 map_key/map_value 一层约束，并定义生成失败语义 |
| 10 | S001、F017～F020 | S001、F017 已确认；F019 删除；F018 按纯语法/结构校验重拟；F020 根据 Java 实际用法重拟 |
| 11 | F018/F020 类型宏与 crate 边界 | 大体同意；Validator 归 rs-validator，ValueCodec/Registry/Id 归 rs-codec；小写字段 helper 归 rs-model-derive；精确选择规则待确认 |
| 12 | V003、K003～K005、F017 修订 | 全部已确认；固化 Validator/ValueCodec 自动宏、字段 codec 选择优先级及 opaque 外部供值规则 |
| 13 | F021～F024 | F021、F022、F024 已确认；F021 增加 Option/容器穿透；F023 按通用子 selector 组合方案重拟 |
| 14 | F023-A～F023-D | 全部已确认；统一透明包装、selector 多规则、容器整体/成员策略作用域和 redact 特殊递归边界 |
| 15 | F025～F028 | F025～F027 已确认；删除 F028 的结论被否决，KeyIndex 按稳定复合键分量顺序重新设计 |
| 16 | F028-R2 | 按修订确认；最终命名 key_part，允许选择部分字段，同时支持大小写敏感/不敏感 unique 与 respect_to 复合属性投影 |
| 17 | R001～R005 | 全部确认；固化 Entity、Projection、Model、Enum、Value 的基础身份、注册、生命周期和组合语义 |
| 18 | R006-P、R008-A/B、R009-G | 全部确认；Entity 不声明单数 Projection；删除 computed，建立 field/getter/setter 合并的 Property；Model 支持泛型模板 |
| 19 | R008-C、R009-H、R010、R011 | 前三项确认；Value 形状与删除 textual 确认，transparent 因实际 Value 场景重新设计 |
| 20 | R011-T、R012-A～C、R014 | 全部确认；固化透明 Value、五种角色默认能力、能力增删参数及默认脱敏联动 |
| 21 | R015-A～E | A、C、D 确认；E 除 id/注册问题外确认并明确 Value 支持 copy；B 因无 id 类型的注册价值问题重拟 |
| 22 | R015-B2、R015-F～H、R015-E2 | 全部确认；只有稳定 ID 的类型注册，Entity 强制 id，其余角色可选 id，泛型 id 标识模板 |
| 23 | A001-S | 已确认；静态查询统一为 TypeMetadata::of::<T>()，删除 metadata_of 自由函数 |
| 24 | A001-T/I/R、A002-F/P | T、R 确认；I 按标准 TypeId 与泛型定义命名重拟；F/P 主体确认并修订 descriptor 与便利方法命名 |
| 25 | A001-I 修订 | 已确认；使用 type_id/type_name、标准 std::any::TypeId 与 generic_definition() |
| 26 | A002-F2/I/P2/V | 全部确认；字段类型统一 descriptor，多重 indexing reasons，property 便利判断及细分 Rust visibility |
