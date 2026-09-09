# 领域模型声明指南

[English](user_guide.md) · [README](../README.zh_CN.md) · [运行时指南](../../doc/user_guide.zh_CN.md)

本指南面向应用开发者，适用于工作区 0.1 契约。模型只需声明一次，框架便可读取其中的领域信息，
再由显式消费者完成验证、查询或持久化。项目要求 Rust 1.94、edition 2024，目前未发布，
请按 README 配置检出路径依赖。

## 声明用户模型并读取属性

```rust
use qubit_id::Id;
use qubit_model_derive::{Entity, ModelImpl};
use qubit_model_metadata::metadata::TypeMetadata;

#[Entity(id = "guide.User")]
struct User {
    #[identifier]
    id: Id,
    #[indexed]
    nickname: String,
    #[redact(level = "secret")]
    token: String,
    tags: Vec<String>,
}

#[ModelImpl]
impl User {
    pub fn display_name(&self) -> String { self.nickname.clone() }
    #[model_property(skip)]
    pub fn diagnostic(&self) -> usize { self.tags.len() }
}

let metadata = TypeMetadata::of::<User>();
assert!(metadata.field("nickname").unwrap().is_indexed());
assert!(metadata.try_property("display_name").unwrap().unwrap().is_computed());
assert!(metadata.try_property("diagnostic").unwrap().is_none());
```

存储字段仍然是 Property；没有同名字段的 getter 自动成为 computed Property，不再需要 computed 标记。
`diagnostic` 只退出 Property 集合，仍保留方法反射。

默认 Debug、Display、Serialize 委托 rs-redact。`redact(skip)` 在具名、位置字段、Enum payload 和透明
包装上都省略整个字段；关闭应用脱敏策略后恢复输出。Map 键脱敏后发生重名时，序列化返回错误。
观察原始字段值的脱敏模式不能与绕过它的自定义序列化适配器混用。

## 选择角色与 Rust 能力

| 角色 | 适用形状和用途 |
| --- | --- |
| Entity | 非泛型具名 struct，恰有一个直接使用 `qubit_id::Id` 的 identifier |
| Projection | 非泛型具名视图，具有 identifier，可指定固定 Entity 来源 |
| Model | 具名或 unit struct，可使用泛型 |
| Enum | unit、tuple、具名或混合 variant，可使用泛型 |
| Value | 具名值对象或单字段 tuple 包装，可指定 `transparent` |

稳定模型 `id` 可省略；它控制注册，不决定 metadata 是否存在。泛型定义与具体类型需要启用运行时
`generic` feature，不支持 lifetime 参数。

角色默认实现 Clone、Debug、Display、PartialEq、Eq、Hash、Redact、Serialize、Deserialize；
全部为 unit variant 的 Enum 还默认实现 Copy。`no_*` 关闭自动实现；`no_eq` 同时移除默认 Hash，
`no_partial_eq` 同时关闭相等、Hash 与排序能力。额外能力使用 `copy`、`default`、`partial_ord`、`ord`。
Enum 的 `default` 要求恰有一个标准 `#[default]` unit variant。可构造默认值不等于满足领域约束。

角色属性放在显式 derive 之前，宏才能识别并避免重复生成。手写实现使用对应的关闭开关。
`no_redact` 禁止当前字段或 selector 上存在脱敏规则，但嵌套类型保留自己的安全输出。
包含 HashMap 的模型通常需要 `no_hash`。泛型约束按实际存储字段生成，涵盖 PhantomData 和 const 数组。

具名 Option 与标准集合字段在缺失时使用默认值，空值默认不序列化。`keep_serializing` 只关闭自动省略，
显式 Serde 配置优先；位置字段不自动省略。

## 引用和查询声明

```rust,ignore
#[reference(entity = Country, property = id, path = "street/district/country")]
country: Id,
#[reference(entity = Order, property = id, path = "..")]
order: Id,
```

这是需要应用提供 Country、Order 等 Entity 的声明示例。`reference.path` 使用 `/` 和 `..` 导航对象绑定。
路径经过引用字段时，继续访问所绑定的完整 Entity，即使该字段只保存 ID 或 Projection。
省略 path 表示没有请求复用绑定；容器本身不增加领域父对象。独立的 `property` 仍使用点分隔路径。
引用可放在 Option、智能指针、序列、Set、数组及其组合中，不能直接放在 Map 中。
Enum payload 可以声明引用，但 Value 不能通过嵌套 Enum 隐藏引用。

identifier、unique、reference 都贡献隐含 indexed 原因，再添加 `#[indexed]` 会报冗余错误。
unique 通过 `respect_to(...)` 表达作用域。文本能力字段默认忽略大小写，真实类型别名同样有效；
非文本字段不能显式指定 ignore_case。具名 Model、Value 的 `key_part(order = n)` 表达逻辑键组成及顺序。

下游可以把 indexed nickname 设计为子串匹配，把 birthday、create_time、age 设计为上下界参数。
这些例子解释元数据的用途，不表示本库生成 filter 对象。本库公开直接 indexed 声明，
操作选择、物理索引和数据库匹配由消费者设计。

## validator、selector 与 codec

```rust,ignore
#[validator(id = "person.birthday",
    depends_on(gender, birthday(path = "..", property = birthday)),
    params(strict = true))]
#[validator(id = "person.birthday", params(strict = false))]
value: String,
```

同 ID 的多个 occurrence 独立保留参数、依赖和声明位置。裸依赖选择当前对象的 Property；
结构化依赖把对象 path 与属性 property 分开。声明不会执行 validator；可选运行时适配器显式绑定
validator registry，并由调用方另行提供父上下文。

text、decimal/money、time、sequence、map 记录声明的约束。
`element(...)`、`map_key(...)`、`map_value(...)` 保留各自作用位置。
Set 已有的唯一性和数组固定长度不能重复声明。opaque 截断内部遍历，但保留外层形状。
旧宏参数 target、on_none、validate_nested 已移除，执行策略由消费者掌握。

codec 声明保留稳定 ID 或 Rust codec 类型，具体实现必须存在于显式传入的 registry。
字段的显式 codec 优先于 Value 的 canonical codec；显式选择相同 canonical codec 也合法。

## 方法反射、错误与排查

ModelImpl 转发 reflect_impl 支持的选项。trait 方法和不能充当 Property 的方法仍然保留反射。
泛型运行时适配器沿用 rs-reflect 的显式 `specialize(T = String, ...)` 绑定。
只有公开、安全、同步且方法自身无泛型的固有访问器贡献 Property；借用 getter 保留生命周期，不强制 Send。

本地形状、冗余和能力冲突在编译期报告；跨 crate 目标与完整结构关系由 StructureResolver 检查。
父上下文需求会显式保留，不会因此判定声明非法。使用 `try_*` 查询处理能力和 Property 组装错误，
执行层错误处理见运行时指南。

完整契约见[需求](rs-model-derive-requirements.zh_CN.md)与[最终设计](rs-model-derive-final-design.zh_CN.md)。
运行 `cargo doc --workspace --all-features --no-deps` 可生成本地 API 文档。
