# 带数据枚举支持设计

本文记录 `qubit_model_derive::Enum` 与 `qubit-model-metadata` 对带数据枚举的长期设计约束。

## 目标与边界

`#[Enum]` 支持以下变体并允许混合声明：

```rust
enum Event {
    Started,
    Progress(u8),
    Failed { message: String },
}
```

支持范围包括静态元数据、字段类型与局部约束、Serde、规范名称、`Display`、默认 trait、脱敏和编译期诊断。泛型枚举、跨变体字段路径和枚举级记录约束不在范围内。

## Runtime 元数据

`EnumVariantMetadata` 保留序号和规范名称，并增加形状：

```rust
#[non_exhaustive]
pub enum EnumVariantKind {
    Unit,
    Tuple(&'static [FieldMetadata]),
    Struct(&'static [FieldMetadata]),
}
```

载荷复用 `FieldMetadata`，因此字段类型继续由 `TypeRef` 表达，字段约束和能力校验不需要第二套实现。tuple 字段名称按声明序号写成 `"0"`、`"1"`；struct 字段保留 Rust 字段名。Serde 字段重命名不改变元数据名称。

兼容性构造器 `EnumVariantMetadata::new(ordinal, name)` 继续创建 unit variant。`tuple` 和 `structure` 构造器接收静态字段切片，并验证字段序号连续、名称非空且不重复。`kind()` 返回形状，`fields()` 对 unit 返回空切片。

`EnumMetadata`、`TypeKind::Enum`、`variant()` 和 `variant_at()` 的签名保持不变。

## Derive 数据流

解析层把每个变体规范化为 unit、tuple 或 struct 形状。tuple 字段在解析时生成数字名称，struct 字段沿用具名字段解析。所有允许的字段属性继续进入现有 `ModelField` → `FieldIr` 管线。

展开层为每个带数据变体生成独立静态 `FieldMetadata` 数组，再调用 `EnumVariantMetadata::tuple` 或 `structure`。独立字段能力断言遍历所有变体载荷，但模型级约束不得把这些字段展平成一个共同字段集合。

## 名称与构造

所有枚举生成：

```rust
pub const fn name(&self) -> &'static str;
```

载荷模式使用 `(..)` 或 `{ .. }`，因此返回值只取决于变体。只有全部变体都是 unit variant 时才生成：

```rust
pub fn from_name(name: &str) -> Option<Self>;
```

带数据枚举不生成部分构造 API；仅需查询名称时，调用方使用 `EnumMetadata::variant(name)`。

## `Display`、Serde 与脱敏

unit variant 只输出规范名称。tuple 与 struct variant 使用 `Formatter::debug_tuple` 和 `debug_struct` 输出 Debug-shaped 载荷，例如：

```text
STARTED
PROGRESS(42)
FAILED { message: "timeout" }
```

该格式供人阅读，不承诺可反序列化。普通 `Display` 要求载荷字段实现 `Debug`；调用方可用 `no_display` 关闭。

Serde 继续使用 `SCREAMING_SNAKE_CASE` 变体名。载荷字段复用结构体字段的默认规则：`Option<T>` 和支持的空集合在反序列化缺失时使用 `default`。struct variant 的每个此类字段都可在序列化时省略；为避免位置左移，至少有两个字段的 tuple variant 只有最后一个此类字段可自动省略。单字段 newtype variant 受 Serde 表示限制，会保留 `null` 或空集合。这保证支持省略的 tuple variant 在省略尾部字段后仍可往返反序列化。

出现 `redact` 或载荷字段 `#[redact(...)]` 时，普通 `Display`、`Debug` 和 Serde 实现不再生成，由 `qubit-redact` 生成安全实现，避免原始载荷泄露。

## 默认 trait

纯 unit enum 保持原有默认 trait，包括 `Copy`。只要存在一个带数据变体，就不默认生成 `Copy`，即使当前载荷恰好全部为 `Copy`。其他默认 trait 继续遵循现有 `no_*` 开关和字段 trait bound。

## 属性作用域

载荷字段允许只描述字段自身的规则：文本、序列、Map、时间、十进制、元素约束、codec、generator、`opaque`、`keep_serializing`、Serde 和脱敏属性。

以下字段 helper 会产生记录级语义，因此在枚举载荷上拒绝：

- `identifier`
- `unique`
- `indexed`
- `reference`
- `lookup_relation`

枚举上的 `textual`、`primary_key`、`index`、`key`、`ownership` 等模型级能力同样拒绝。不同变体没有共同且必然存在的字段集合，本设计不定义跨变体字段路径。

## 兼容性与验证

已有纯 unit enum 的 `new`、`name`、`from_name`、`Display`、Serde、默认 `Copy` 和元数据查询保持兼容。带数据 enum 过去无法编译，因此新增行为不会改变已有成功编译程序。

验证必须覆盖 runtime 构造与校验、三种变体元数据、名称和 Display、Serde tuple 往返、字段约束、脱敏、默认 `Copy` 差异、条件 `from_name`、泛型拒绝及全部属性作用域诊断。每个 crate 在提交前运行 `align-ci.sh` 和 `ci-check.sh`。
