# `rs-model-derive` 最终设计（中文版）

- 状态：最终目标设计
- 适用仓库：`rs-model-derive`、`rs-model-metadata`、`rs-reflect`
- 需求规范：[最终需求规范](rs-model-derive-requirements.zh_CN.md)
- 决策记录：[完整讨论记录](2026-08-28-discuss-session.md)
- 验证映射：[需求覆盖台账](rs-model-derive-requirements-coverage.zh_CN.md)
- 核心前提：`qubit-reflect` 提供唯一的 Rust 结构反射、动态访问、泛型描述、typed capability 和链接期注册机制

## 结论

`rs-model-derive` 的领域需求与讨论记录确认的语义保持一致；重构只改变结构反射的承载方式。最终方案是：

1. `qubit-reflect` 是唯一的 Rust 结构事实源，负责 `TypeDescriptor`、`FieldDescriptor`、
   `VariantDescriptor`、`TypeRef`、泛型表达式、动态值、字段访问和反射注册表。
2. `qubit-model-metadata` 是领域语义覆盖层，负责五种模型角色、字段约束、关系、查询、策略引用、
   Serde/Redact 语义、稳定 `ModelId` 和模型图解析。
3. `qubit-model-derive` 是领域声明编译器。六个属性宏共享一条
   `parse -> normalize -> validate -> expand` 流水线，并委托 `qubit-reflect` 生成结构反射。
4. 模型 metadata 通过 `qubit-reflect` 的 typed capability 挂到唯一的
   `TypeDescriptor` 根上，不修改、包装或复制该根。
5. `ModelRegistry` 是 `ReflectRegistry` 之上的稳定 ID 索引和领域解析层；它不再保存另一份 Rust 结构注册表。
6. 需求规范中的角色、字段、约束、关系、Property、注册与解析语义全部保留；结构描述、泛型表达和安全动态访问
   统一复用 `qubit-reflect`，模型层只增加领域 overlay。

一句话概括最终形态：

```text
Rust declaration
    │
    ├─ qubit-reflect descriptor（唯一结构事实）
    │       ├─ fields / variants / generics / access / construct
    │       └─ typed capability: qubit.model.metadata
    │                              │
    │                              └─ TypeMetadata（领域语义覆盖层）
    │
    └─ 具体类型与泛型定义都由 ReflectRegistry 投影
             └─ ModelRegistry -> ModelResolver -> ResolvedModelGraph
```

## 设计依据：`rs-reflect` 已经解决的能力

本方案基于当前 `rs-reflect` 代码中的以下稳定设计，而不是仅根据 README 推断：

- `Reflect` 是唯一静态反射 trait，`TypeDescriptor::of::<T>()` 返回唯一根描述符。
- `TypeDescriptor` 已直接提供标准 `TypeId`、`type_name()`、结构种类、字段、variant、泛型实例和能力集合。
- `TypeRef::{Resolved, Opaque, Symbolic}` 已准确覆盖 concrete 字段、显式 opaque 字段和泛型定义中的符号类型。
- `FieldDescriptor` 已保存字段 index、Rust/query name、可见性、所属 variant、字段类型和安全访问适配器。
- 字段读取/写入使用 `ReflectedRef`、`ReflectedMut`、`ReflectedOwned`，并在调用前检查目标和值的准确
  `TypeId`；失败不会伪造生命周期或绕过 Rust aliasing。
- `ConcreteGenericDescriptor`、`GenericDefinitionDescriptor`、`GenericArgument` 和 `TypeExpression` 已覆盖泛型定义、
  concrete 实参及结构化 const 表达式。
- `TypeCapabilities` 通过 `CapabilityKey<A>` 同时校验稳定 ID 和 Rust adapter 类型；第三方可以注册自定义 typed
  capability。
- `ReflectRegistry` 使用 immutable snapshot、`OnceLock<Result<...>>`、确定性 fragment 排序、源位置 identity 和
  冲突错误。
- `qubit-reflect` 已有 downstream facade 测试：领域 runtime crate 可以重导出反射 API，领域 derive crate 通过
  `#[reflect(crate = facade)]` 委托生成代码，最终业务 crate 不必直接依赖 `qubit-reflect`。
- `__private::codegen_v2` 已提供宏生产 ABI、lazy type reference 和统一 registration fragment；普通用户 API 与
  生成代码 API 已有清楚边界。模型生成 facade 固定为 model ABI v4，不再保留旧 ABI。

因此，若 `rs-model-metadata` 再定义 `TypeDescriptor`、`TypeIdentity`、`TypeRef`、字段访问器或 generic expression，
会产生两个不可避免的问题：同一 Rust 类型出现两棵可能不一致的图；修复递归、泛型和安全访问时必须维护两份实现。

## crate 边界和依赖方向

最终依赖关系为：

```text
qubit-reflect
      ▲
      │
qubit-model-metadata ─────► qubit-codec
      │                    qubit-redact
      ▲
      │ generated code only
qubit-model-derive
      ▲
      │
domain crates
```

具体规则：

- `qubit-model-metadata` 直接依赖 `qubit-reflect`，并重导出普通模型用户需要的反射类型。
- `qubit-model-derive` 不在宏进程中读取 runtime registry，不加载领域类型，不读取数据库或网络。
- `qubit-model-derive` 使用 `proc-macro-crate` 解析 `qubit-model-metadata` 的实际依赖名；生成代码只引用该 facade，
  不要求业务 crate 直接依赖 `qubit-reflect`。
- `qubit-model-metadata::__private` 重导出 `qubit_reflect::__private`，保持 facade 委托链。
- codec 直接复用 `qubit-codec` 的 `ValueEncoder`、`ValueDecoder`、`ValueCodecDescriptor`、
  `register_value_codec!` 和注册表契约，不定义平行
  contract。validator 直接复用 `qubit-validator` 的 `Validator`、注册表和解析契约；模型层保存 occurrence，并由显式 resolver 完成绑定与类型检查。
- 当前内部仓库形态下，业务代码通过路径分别依赖 `qubit-model-metadata` 与 `qubit-model-derive`；runtime facade
  负责生成代码协议，但不通过虚构的默认 `derive` feature 重导出宏。

## 用户可见的宏 API

最终只公开以下六个领域入口：

```rust
#[proc_macro_attribute]
pub fn Entity(args: TokenStream, input: TokenStream) -> TokenStream;

#[proc_macro_attribute]
pub fn Projection(args: TokenStream, input: TokenStream) -> TokenStream;

#[proc_macro_attribute]
pub fn Model(args: TokenStream, input: TokenStream) -> TokenStream;

#[proc_macro_attribute]
pub fn Enum(args: TokenStream, input: TokenStream) -> TokenStream;

#[proc_macro_attribute]
pub fn Value(args: TokenStream, input: TokenStream) -> TokenStream;

#[proc_macro_attribute]
pub fn ModelImpl(args: TokenStream, input: TokenStream) -> TokenStream;
```

业务用户不需要同时书写 `#[derive(Reflect)]`。五种类型宏自动委托反射宏，并生成领域 metadata：

```rust
use qubit_model_metadata::{Entity, TypeMetadata};

#[Entity(id = "qubit.platform.iam.User")]
pub struct User {
    #[identifier]
    pub id: Id,
    #[unique]
    #[text(min_chars = 3, max_chars = 32, allowed_chars = code)]
    pub username: String,
}

let metadata = TypeMetadata::of::<User>();
assert!(metadata.as_entity().is_some());
```

类型宏内部展开等价于：

```rust,ignore
#[derive(qubit_model_metadata::Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct User { /* retained source */ }

impl qubit_model_metadata::HasTypeMetadata for User { /* generated */ }

qubit_model_metadata::__private::register_model_capability!(User, /* provider */);
qubit_model_metadata::__private::register_model!(/* only when ModelId exists */);
```

用户在同一声明上显式添加 `Reflect` 派生属于重复配置，宏应在原 derive token 上给出专用诊断，避免产生冲突实现。
`#[reflect(...)]` 的领域无关参数首版不透传；若模型系统以后确实需要公开某项反射控制，应增加明确的模型参数，不能把
整个反射 helper 语法泄漏到领域 API。

五种角色、shape、泛型、默认能力、identifier、reference、约束、selector、Serde/Redact 的最终业务语义继续采用
最终需求规范。明确删除项继续删除：lookup_relation、ownership、field generator、computed、exclude、
modified/unmodified、key_index、物理数据库索引参数和基于 `type_name()` 的稳定身份。

## 静态查询与唯一 descriptor

### 公共 trait

模型层不再定义 `HasTypeDescriptor`。通用类型约束直接使用 `qubit_reflect::Reflect`：

```rust
pub use qubit_reflect::Reflect;
pub use qubit_reflect::TypeDescriptor;

pub trait HasTypeMetadata: Reflect + __private::ModelTypeSeal {
    fn type_metadata() -> &'static TypeMetadata;
}
```

`HasTypeMetadata` 是公开泛型约束，但手工实现不是受支持扩展点。隐藏的 seal 只能由五种角色宏实现，确保模型 metadata、
反射 descriptor 和注册 fragment 同时生成。需要手写反射的普通类型仍可实现 `Reflect`，但不会自动成为模型角色。

### `TypeMetadata` 入口

```rust
impl TypeMetadata {
    pub fn of<T: HasTypeMetadata>() -> &'static Self;
    pub fn try_of<T: HasTypeMetadata>() -> Result<&'static Self, AbiViolation>;

    pub fn descriptor(&self) -> &'static TypeDescriptor;
    pub fn type_id(&self) -> std::any::TypeId;
    pub fn type_name(&self) -> &'static str;

    pub fn model_id(&self) -> Option<ModelId>;
    pub fn is_registered(&self) -> bool;
    pub fn generic_definition(&self) -> Option<&'static GenericModelMetadata>;

    pub fn fields(&self) -> &'static [FieldMetadata];
    pub fn field(&self, name: &str) -> Option<&'static FieldMetadata>;
    pub fn field_at(&self, index: usize) -> Option<&'static FieldMetadata>;

    pub fn try_properties(&'static self) -> Result<&'static LocalPropertySet, &'static PropertyBuildErrors>;
    pub fn try_property(&'static self, name: &str) -> Result<Option<&'static PropertyMetadata>, &'static PropertyBuildErrors>;

    pub fn role(&self) -> ModelRole;
    pub fn role_metadata(&self) -> &'static RoleMetadata;
    pub fn as_entity(&self) -> Option<&'static EntityMetadata>;
    pub fn as_projection(&self) -> Option<&'static ProjectionMetadata>;
    pub fn as_model(&self) -> Option<&'static ModelMetadata>;
    pub fn as_enum(&self) -> Option<&'static EnumMetadata>;
    pub fn as_value(&self) -> Option<&'static ValueMetadata>;
}
```

`type_id()`、`type_name()` 和所有结构信息均委托给 `descriptor()`；`TypeMetadata` 不保存第二份身份。

### 从显式注册表查询任意 descriptor 的模型 metadata

effective capability 依赖冻结的链接上下文，因此 descriptor 不提供隐式全局查询扩展。调用方显式使用
`ModelRegistry::metadata_for(descriptor)`；未命中返回 `None`，注册表初始化失败通过 `Result` 报告。

内部 capability 契约为：

```rust
#[doc(hidden)]
pub type ModelMetadataProvider = fn() -> &'static TypeMetadata;

#[doc(hidden)]
pub fn model_metadata_key() -> CapabilityKey<ModelMetadataProvider>;
```

稳定 capability ID 固定为 `qubit.model.metadata.v1`，并由 `ReflectRegistry` 统一解析。

### `TypeCapabilities` 的最终边界

模型层不再定义另一个 `TypeCapabilities` bitflag：

- Rust 可执行能力使用 `qubit_reflect::TypeCapabilities`，例如 Clone、Default 或类型适配器。
- text/decimal/container 等约束适用性优先根据 `TypeDescriptor::kind()` 和 typed views 判定。
- 仅当结构种类不足以证明能力时，使用独立、具名的 reflect capability key，例如 text view、decimal view、query
  comparison adapter；每个 key 都有准确 Rust adapter 类型。
- “字段声明了某约束”属于 `FieldMetadata`，不得混入类型 capability。

这关闭了 结构能力边界，并避免同一个 `Copy`/`Serialize` 事实在模型与反射两处不一致。

## Field：结构事实与领域语义分层

`FieldMetadata` 是 `FieldDescriptor` 的领域覆盖层，不复制 index、name、visibility、类型或访问器：

```rust
impl FieldMetadata {
    pub fn reflect(&self) -> &'static qubit_reflect::FieldDescriptor;

    pub fn index(&self) -> usize;
    pub fn name(&self) -> Option<&'static str>;
    pub fn type_ref(&self) -> &'static qubit_reflect::descriptor::TypeRef;
    pub fn descriptor(&self) -> Option<&'static TypeDescriptor>;
    pub fn visibility(&self) -> qubit_reflect::access::FieldVisibility<'static>;

    pub fn attributes(&self) -> &'static [FieldAttributeMetadata];
    pub fn identifier(&self) -> Option<&'static IdentifierMetadata>;
    pub fn is_identifier(&self) -> bool;
    pub fn indexing_reasons(&self) -> IndexingReasons;
    pub fn is_indexed(&self) -> bool;
    pub fn unique(&self) -> Option<&'static UniqueMetadata>;
    pub fn is_unique(&self) -> bool;
    pub fn reference(&self) -> Option<&'static ReferenceMetadata>;
    pub fn is_reference(&self) -> bool;
    pub fn key_part(&self) -> Option<&'static KeyPartMetadata>;

    pub fn constraints(&self) -> &'static [ConstraintMetadata];
    pub fn validators(&self) -> &'static [ValidatorMetadata];
    pub fn codec(&self) -> Option<&'static CodecMetadata>;
    pub fn redact(&self) -> Option<&'static RedactMetadata>;
    pub fn serde(&self) -> &'static SerdeFieldMetadata;
    pub fn is_opaque(&self) -> bool;
}
```

这里对需求规范 `descriptor() -> &'static TypeDescriptor` 做必要修正：最终返回 `Option`。原因不是放宽实现，而是
准确遵守 `qubit-reflect::TypeRef`：resolved 字段返回 `Some`，显式 opaque 和泛型定义中的 symbolic 字段返回
`None`。调用者需要完整分支时使用 `type_ref()`，不得把 opaque 或 symbolic 伪造成根 descriptor。

`name()` 使用反射字段的 query name；需要源代码名时调用 `reflect().rust_name()`。tuple Value 和 Enum tuple payload 的
字段名为 `None`。Enum payload 字段仍属于对应 variant，不进入类型顶层 `fields()`。

高频便利方法保持严格等价：

```rust
field.is_identifier() == field.identifier().is_some()
field.is_unique() == field.unique().is_some()
field.is_reference() == field.reference().is_some()
field.is_indexed() == !field.indexing_reasons().is_empty()
```

### 统一属性迭代

`FieldAttributeMetadata` 是同一批强类型对象的源码顺序视图，不保存第二份值：

```rust
pub enum FieldAttributeMetadata {
    Identifier(&'static IdentifierMetadata),
    Indexed(IndexingReasons),
    Unique(&'static UniqueMetadata),
    Reference(&'static ReferenceMetadata),
    KeyPart(&'static KeyPartMetadata),
    Constraint(&'static ConstraintMetadata),
    Validator(&'static ValidatorMetadata),
    Codec(&'static CodecMetadata),
    Redact(&'static RedactMetadata),
    Serde(&'static SerdeFieldMetadata),
    Opaque,
}
```

强类型 getter 与 `attributes()` 中的引用必须指向同一个静态对象。`Indexed` 保存最终 reason 集合，但只在源码中确实
存在 indexed/identifier/unique/reference 事实时出现一次，避免按每个隐含原因重复列项。

## Property 与 `#[ModelImpl]`

field-backed Property 直接复用 `FieldDescriptor` 的安全访问器；显式 getter/setter 使用同一组动态值类型和相同的
TypeId 预检规则，不再发明基于裸 `Any` 的接口。

```rust
pub enum PropertyStorageKind {
    FieldBacked,
    Computed,
    Virtual,
}

pub enum PropertyValue<'a> {
    Borrowed(qubit_reflect::ReflectedRef<'a>),
    OptionalBorrowed(Option<qubit_reflect::ReflectedRef<'a>>),
    BorrowedSlice(BorrowedPropertySlice<'a>),
    Owned(qubit_reflect::ReflectedOwned),
}

impl PropertyMetadata {
    pub fn name(&self) -> &'static str;
    pub fn type_ref(&self) -> &'static qubit_reflect::descriptor::TypeRef;
    pub fn descriptor(&self) -> Option<&'static TypeDescriptor>;

    pub fn field(&self) -> Option<&'static FieldMetadata>;
    pub fn getter(&self) -> Option<&'static GetterMetadata>;
    pub fn setter(&self) -> Option<&'static SetterMetadata>;

    pub fn is_field(&self) -> bool;
    pub fn is_getter(&self) -> bool;
    pub fn is_setter(&self) -> bool;
    pub fn is_readable(&self) -> bool;
    pub fn is_writable(&self) -> bool;
    pub fn is_computed(&self) -> bool;
    pub fn storage_kind(&self) -> PropertyStorageKind;

    pub fn get<'a>(
        &self,
        target: qubit_reflect::ReflectedRef<'a>,
    ) -> Result<PropertyValue<'a>, PropertyAccessError>;

    pub fn set(
        &self,
        target: qubit_reflect::ReflectedMut<'_>,
        value: qubit_reflect::ReflectedOwned,
    ) -> Result<(), PropertySetFailure>;
}
```

```rust
pub enum GetterOutputKind {
    Borrowed,
    Owned,
}

impl GetterMetadata {
    pub fn rust_method_name(&self) -> &'static str;
    pub fn output_type(&self) -> &'static qubit_reflect::descriptor::TypeRef;
    pub fn output_kind(&self) -> GetterOutputKind;
    pub fn get<'a>(
        &self,
        target: qubit_reflect::ReflectedRef<'a>,
    ) -> Result<PropertyValue<'a>, PropertyAccessError>;
}

impl SetterMetadata {
    pub fn rust_method_name(&self) -> &'static str;
    pub fn input_type(&self) -> &'static qubit_reflect::descriptor::TypeRef;
    pub fn set(
        &self,
        target: qubit_reflect::ReflectedMut<'_>,
        value: qubit_reflect::ReflectedOwned,
    ) -> Result<(), PropertySetFailure>;
}
```

`PropertyAccessError` 区分 target type mismatch、value type mismatch、不可读、不可写、adapter unavailable 和用户方法
执行错误。`PropertySetFailure` 像 `FieldSetFailure` 一样在执行前失败时归还 replacement value；setter 已开始执行后的错误
不承诺回滚对象状态。

首版只提供 local erased mode。线程安全 Property adapter 必须在单独需求中显式加入，不能根据类型实现 Send/Sync 就自动
升级动态值模式。

合并规则保持不变：

- field/getter/setter 是三个可同时存在的槽位，不是互斥 enum；
- field 存在即 `FieldBacked`；无 field 有 getter 即 `Computed`；只有 setter 即 `Virtual`；
- `is_readable() == is_field() || is_getter()`；
- `is_writable() == is_field() || is_setter()`；
- 显式 getter/setter 优先于默认 field adapter；
- 借用输出的生命周期由 `for<'a>` adapter 绑定到输入 target，不得转成 `'static`；
- getter/setter 兼容规则按 需求规范支持 `T <-> &T`、`String <-> str/&str`、
  `Vec<T> <-> [T]/&[T]`、`Option<T> <-> Option<&T>`，所有转换必须由具名安全 adapter 完成。

Property 采用三层模型。类型宏与 `#[ModelImpl]` 只生成来源事实：

```text
PropertyFragment(Field | Getter | Setter)
    -> LocalPropertySet::try_merge(...)
    -> Result<LocalPropertySet, PropertyBuildErrors>
    -> ModelResolver
    -> ResolvedPropertySet
```

字段与独立 impl 宏之间的类型冲突不能由单次宏展开完整证明，因此由 `try_properties()` 返回确定排序的结构化错误；
普通 metadata 查询不得因此 panic。相同 impl 内同名 getter/setter 的 canonical 类型兼容则由 `__private::v4` trait assertion
在编译期证明。

`#[ModelImpl]` 每个目标类型只允许一个 impl。宏通过实现隐藏的 `ModelImplProvider` seal 让重复 impl 产生稳定的
conflicting implementation 编译错误；同一 impl 内的重复 property、非法方法形状和不兼容类型由宏给出聚合诊断。
不符合 getter/setter 契约的普通业务方法被保留但不进入 metadata；`set_` 前缀显式表达 setter 意图，因此非法
`set_` 方法必须诊断。

### Projection producer 与 projector

Entity 上返回 Projection 的 readable property 形成 producer edge。`ModelResolver` 校验固定 source、双方角色与
identifier 类型，并把 getter 的安全 erased adapter 挂到 `ResolvedProjectionProducer`。执行 projector 后必须读取
两侧精确的 `qubit_id::Id`；不一致返回 `ProjectionExecutionError::IdentifierMismatch`。缺少 projector 只影响显式自动
投影操作，不影响 Projection 声明、DAO/SQL mapper 构造或反序列化。

## 五种角色的最终 runtime API

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ModelRole {
    Entity,
    Projection,
    Model,
    Enum,
    Value,
}

pub enum RoleMetadata {
    Entity(EntityMetadata),
    Projection(ProjectionMetadata),
    Model(ModelMetadata),
    Enum(EnumMetadata),
    Value(ValueMetadata),
}
```

角色公共信息只存在于 `TypeMetadata`。角色 payload 只保存角色专属事实。

### Entity

```rust
impl EntityMetadata {
    pub fn identifier(&self) -> &'static FieldMetadata;
}
```

`model_id()` 不在这里重复；Entity 的必填 ID 从拥有它的 `TypeMetadata::model_id()` 取得。

### Projection

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeclaredEntityTargetKind {
    RustType,
    ModelId,
}

impl DeclaredEntityTarget {
    pub fn kind(&self) -> DeclaredEntityTargetKind;
    pub fn metadata(&self) -> Option<&'static TypeMetadata>;
    pub fn model_id(&self) -> Option<ModelId>;
}

impl ProjectionMetadata {
    pub fn identifier(&self) -> &'static FieldMetadata;
    pub fn source(&self) -> Option<&'static DeclaredEntityTarget>;
    pub fn is_open(&self) -> bool;
    pub fn is_fixed(&self) -> bool;
}
```

Rust type source 可以通过静态 provider 返回 metadata；ID source 只返回声明的 ID。`source()` 从不隐式读取 registry。

### Model

```rust
#[derive(Clone, Copy, Debug, Default)]
pub struct ModelMetadata;
```

保留空 payload，使 `RoleMetadata` 五个分支稳定且未来可向非穷举结构增加 Model 专属事实。首版不向其中塞 query、fields
或 properties。

### Enum

```rust
impl EnumMetadata {
    pub fn variants(&self) -> &'static [EnumVariantMetadata];
    pub fn variant(&self, canonical_name: &str) -> Option<&'static EnumVariantMetadata>;
    pub fn variant_by_rust_name(&self, name: &str) -> Option<&'static EnumVariantMetadata>;
    pub fn variant_by_serialized_name(&self, name: &str) -> Option<&'static EnumVariantMetadata>;
}

impl EnumVariantMetadata {
    pub fn reflect(&self) -> &'static qubit_reflect::VariantDescriptor;
    pub fn index(&self) -> usize;
    pub fn rust_name(&self) -> &'static str;
    pub fn canonical_name(&self) -> &'static str;
    pub fn serialized_name(&self) -> &'static str;
    pub fn deserialized_name(&self) -> &'static str;
    pub fn fields(&self) -> &'static [FieldMetadata];
    pub fn field(&self, name: &str) -> Option<&'static FieldMetadata>;
    pub fn field_at(&self, index: usize) -> Option<&'static FieldMetadata>;
    pub fn is_default(&self) -> bool;
}
```

variant 形状直接使用 `reflect().kind()`，不再定义重复的 `EnumVariantKind`。序列化和反序列化名称分开，正确覆盖 Serde
directional rename。

### Value

```rust
impl ValueMetadata {
    pub fn is_transparent(&self) -> bool;
    pub fn transparent_field(&self) -> Option<&'static FieldMetadata>;
    pub fn canonical_codec(&self) -> Option<&'static CodecMetadata>;
}
```

`transparent_field().is_some()` 与 `is_transparent()` 严格等价。Copy/Default/Serialize 等能力不放入 Value payload，统一从
反射 capability 或 Rust trait 行为取得。

## 字段语义类型

### 身份、索引、唯一和键分量

```rust
pub enum IdentifierAssignment {
    Application,
    Database,
}

impl IdentifierMetadata {
    pub fn assigned_by(&self) -> IdentifierAssignment;
}

bitflags::bitflags! {
    pub struct IndexingReasons: u8 {
        const EXPLICIT   = 0b0001;
        const IDENTIFIER = 0b0010;
        const UNIQUE     = 0b0100;
        const REFERENCE  = 0b1000;
    }
}

impl UniqueMetadata {
    pub fn respect_to(&self) -> &'static [PropertyPath];
    pub fn ignore_case(&self) -> bool;
    pub fn is_scoped(&self) -> bool;
}

impl KeyPartMetadata {
    pub fn order(&self) -> usize;
}
```

`key_part` 表达值语义上的逻辑复合键，而不是持久化身份。它只允许用于具名 `Model` 或具名 `Value`
的真实存储字段；可选择字段子集，但所选 order 必须从 0 开始、连续且不重复。`Entity`、`Projection`
应使用 `identifier`，`Enum` 与 tuple/newtype Value 没有可供选择的具名字段，因此这些形状都拒绝
`key_part`。

`ignore_case()` 只描述当前 unique 字段，默认 true；scope 路径按源码顺序返回。物理索引名、列名、排序和数据库参数不进入
这些类型。

### Reference

```rust
pub enum ReferenceSelection {
    Entity,
    Property(PropertyPath),
}

impl ReferenceMetadata {
    pub fn target(&self) -> &'static DeclaredEntityTarget;
    pub fn selection(&self) -> &'static ReferenceSelection;
    pub fn existing(&self) -> bool;
    pub fn same_as(&self) -> Option<&'static PropertyPath>;
}
```

省略 `property` 等价 `ReferenceSelection::Entity`；`property = id` 是普通结构化 PropertyPath，不定义特殊字符串协议。
`same_as()` 对应 需求规范的 `path` 绑定语义。声明 metadata 不保存解析后目标；解析结果属于
`ResolvedReference`。

### 约束与 selector

```rust
pub enum ConstraintMetadata {
    Text(TextConstraint),
    Decimal(DecimalConstraint),
    Time(TimeConstraint),
    Sequence(SequenceConstraint),
    Map(MapConstraint),
}

pub enum SelectorPosition {
    Element,
    MapKey,
    MapValue,
}

impl SelectorMetadata {
    pub fn position(&self) -> SelectorPosition;
    pub fn constraints(&self) -> &'static [ConstraintMetadata];
    pub fn validators(&self) -> &'static [ValidatorMetadata];
    pub fn codec(&self) -> Option<&'static CodecMetadata>;
    pub fn redact(&self) -> Option<&'static RedactMetadata>;
}

impl SequenceConstraint {
    pub fn min_items(&self) -> Option<usize>;
    pub fn max_items(&self) -> Option<usize>;
    pub fn unique_items(&self) -> bool;
    pub fn element(&self) -> Option<&'static SelectorMetadata>;
}

impl MapConstraint {
    pub fn min_entries(&self) -> Option<usize>;
    pub fn max_entries(&self) -> Option<usize>;
    pub fn key(&self) -> Option<&'static SelectorMetadata>;
    pub fn value(&self) -> Option<&'static SelectorMetadata>;
}
```

Text、Decimal、Time 的参数和枚举使用需求规范确定的名称。Money 不另设平行类型，而是
`DecimalConstraint::semantic() == DecimalSemantic::Money`。selector 不能递归包含 selector；深层结构必须通过具名
Value/Model/Enum descriptor 导航。

### Validator、Codec、Redact 与 Serde

```rust
impl ValidatorMetadata {
    pub fn declared_id(&self) -> &'static str;
    pub fn params(&self) -> &'static [NamedValidationArgument<'static>];
    pub fn depends_on(&self) -> &'static [PropertyPath];
}

pub enum CodecReference {
    RustType(&'static ValueCodecDescriptor),
    DeclaredId(&'static str),
}

impl CodecMetadata {
    pub fn codec(&self) -> &'static CodecReference;
    pub fn source(&self) -> CodecSource;
}

pub enum CodecSource {
    Field,
    CanonicalValue,
    Selector(SelectorPosition),
}
```

Validator ID 当前保存为经过宏展开期 ASCII 语法校验的声明字符串；模型 crate 不公开临时 `ValidatorId`。validator
occurrence 顺序保持源码顺序，参数直接复用 `qubit-validator` 的 `ValidationArgument` 与
`NamedValidationArgument`。`ModelResolver` 通过 `ValidatorRegistry` 解析注册状态、精确值类型和可读依赖属性，
不使用字符串判断 Rust 类型相等。

`CodecReference::RustType` 直接生成
`C: Default + qubit_codec::ValueEncoder<T, Output = String> + qubit_codec::ValueDecoder<str, Output = T>` 编译期约束；
Rust codec 类型通过 `ValueCodecDescriptor::of::<C, T>()` 形成可执行描述符；按 ID 使用的 codec 通过
`register_value_codec!` 注册。`DeclaredId` 只保存经过语法校验的稳定字符串，由 `ModelResolver` 通过
`ValueCodecRegistry` 解析；模型 crate 不定义替代 registry。

`RedactMetadata` 复用 `qubit-redact::Sensitivity` 和既有 domain capability。模型层允许定义窄的声明枚举
`RedactModeMetadata::{Level, Skip, Nested, Map, KeyedBy, Json}` 及
`RedactPosition::{Field, Element, MapKey, MapValue}`，但不得复制执行策略或声称 `qubit-redact` 已公开不存在的统一模式
枚举。实际输出继续委托 `Redact`、`RedactLevelValue`、`RedactMapValue` 等能力。`SerdeFieldMetadata` 保存最终
serialize/deserialize name、双向 skip、flatten、with 和自动 omit/default 来源；显式 Serde 配置优先于模型默认。

当前 selector 执行面只接受 `redact(level = "...")`：element/map value 通过递归 level capability 执行，map key
通过专用 map-key capability 执行。Serde 在脱敏后的 map key 发生碰撞时必须返回错误，禁止后写值静默覆盖前写值；
其他 selector redact mode 在获得对应 runtime capability 前应于宏展开期拒绝。

### Validator 解析与绑定

字段上的 `#[validator(...)]` 生成按源码顺序保存的 `ValidatorMetadata`。宏负责校验稳定 ID、参数字面量和依赖路径语法；`ModelResolver` 使用显式提供的 `ValidatorRegistry` 绑定注册项，验证 validator 的值类型，并把成功结果保存为 `ResolvedValidator`。缺失 ID、重复注册、值类型不兼容或依赖属性不可读都属于显式解析错误。标准约束与自定义 validator 保持两个层次：前者是可移植的声明式 schema，后者是通过稳定 ID 绑定的扩展执行策略。

## 泛型模型

模型层复用反射泛型结构，不再定义平行的参数/表达式体系：

```rust
impl GenericModelMetadata {
    pub fn model_id(&self) -> ModelId;
    pub fn role(&self) -> ModelRole;
    pub fn definition(&self) -> &'static qubit_reflect::expression::GenericDefinitionDescriptor;
    pub fn fields(&self) -> &'static [FieldMetadata];
    pub fn variants(&self) -> &'static [EnumVariantMetadata];
}

impl TypeMetadata {
    pub fn concrete_generic(
        &self,
    ) -> Option<&'static qubit_reflect::ConcreteGenericDescriptor>;
}
```

最终规则：

- Entity 和 Projection 不支持泛型；Model、Enum、Value 支持 type parameter、primitive const parameter 和 where
  clause；所有模型角色拒绝 lifetime parameter。
- type parameter 需要 `Reflect + 'static`，具体角色闭包需要的更强 bound 由宏准确生成。
- 带 ID 的泛型声明只注册一个引用 `TypeDefinitionDescriptor` 的 `GenericModelMetadata`。
- `TypeMetadata::of::<Page<User>>()` 返回 concrete metadata；其 `model_id() == None`、`is_registered() == false`，
  `generic_definition()` 返回定义级模型元数据。
- concrete descriptor 的参数和 const value 直接从 `ConcreteGenericDescriptor` 导航；首版不生成 concrete ModelId。
- 未带 ID 的泛型声明不进入 `ModelRegistry`，但 concrete 类型仍可静态查询。
- 泛型定义字段的 `type_ref()` 可以是 `TypeRef::Symbolic`；concrete metadata 中可解析的字段使用
  `TypeRef::Resolved`。
- 泛型 Enum 的定义 descriptor 必须保存所有 variant 及 payload 字段；variant 字段上的约束、validator、codec、
  Redact 和 Serde overlay 不得因顶层 `fields()` 为空而丢失。

这关闭了 需求规范关于首版 const generic 的分歧：首版支持 `bool`、`char`、整数和 `usize/isize` 等
`qubit-reflect::ConstArgumentValue` 已支持的 primitive const 类型；其他 const 类型明确拒绝。

## `ModelId` 与模型注册表

### `ModelId`

避免在无 `unsafe` 的前提下把任意 `&str` 强转为 borrowed newtype，最终保留静态和 owned 两种类型：

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ModelId(&'static str);

impl ModelId {
    pub const fn new(value: &'static str) -> Self; // invalid literal const-panic
    pub const fn try_new(value: &'static str) -> Result<Self, ModelIdError>;
    pub const fn as_str(self) -> &'static str;
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ModelIdBuf(Box<str>);

impl ModelIdBuf {
    pub fn parse(value: &str) -> Result<Self, ModelIdError>;
    pub fn as_str(&self) -> &str;
}
```

宏只生成 `ModelId::new("literal")`。动态输入使用 `ModelIdBuf`。两者执行相同的
`Segment ('.' Segment)*` ASCII 验证，不强制 namespace 大小写风格。

### 注册来源

具体角色与泛型定义都通过统一反射 fragment 注册相应 model capability，`ModelRegistry` 只从冻结
`ReflectRegistry` 快照投影。泛型定义是一等反射根并保留权威 source identity；模型层不维护第二套 inventory。

### `ModelRegistry`

```rust
impl ModelRegistry {
    pub fn try_global() -> Result<&'static Self, ModelRegistryError>;
    pub fn global() -> &'static Self;

    pub fn metadata(&self, id: &str) -> Option<&'static TypeMetadata>;
    pub fn generic(&self, id: &str) -> Option<&'static GenericModelMetadata>;
    pub fn metadata_for(&self, descriptor: &'static TypeDescriptor) -> Option<&'static TypeMetadata>;
    pub fn generic_metadata_for(&self, id: TypeDefinitionId) -> Option<&'static GenericModelMetadata>;
    pub fn by_type_id(
        &self,
        type_id: std::any::TypeId,
    ) -> Option<&'static TypeMetadata>;

    pub fn generic_definitions(&self) -> &'static [&'static GenericModelMetadata];
}
```

按字符串查询对非法格式返回 `None`，适合只读查询；需要区分“ID 非法”和“不存在”时，调用者先使用
`ModelIdBuf::parse()`。`by_type_id()` 只索引已注册 concrete 类型，不公开匿名 concrete cache。

`try_global()` 先初始化 `ReflectRegistry`，从冻结快照投影具体模型与泛型定义 capability 及其权威来源；
反射 fragment 冲突包装为 `ModelRegistryErrorKind::ReflectionRegistry`。
`global()` 只是在错误时 panic 的便利入口。

## 显式 resolver 与解析后视图

声明 metadata 永远只保存本地事实。跨 crate ID、角色、Property 和策略兼容性由显式 resolver 完成：

```rust
pub struct ResolveInputs<'a> {
    pub models: &'a ModelRegistry,
}

pub struct ModelResolver<'a> { /* immutable inputs */ }

impl<'a> ModelResolver<'a> {
    pub fn new(inputs: ResolveInputs<'a>) -> Self;
    pub fn resolve_all(&self) -> Result<ResolvedModelGraph, ModelResolveErrors>;
}

impl ResolvedModelGraph {
    pub fn registry(&self) -> &'static ModelRegistry;
    pub fn reference(&self, field: &FieldMetadata) -> Option<&ResolvedReference>;
    pub fn projection_source(
        &self,
        projection: &ProjectionMetadata,
    ) -> Option<&ResolvedProjectionSource>;
    pub fn validator(&self, occurrence: &ValidatorMetadata) -> Option<&ResolvedValidator>;
    pub fn codec(&self, occurrence: &CodecMetadata) -> Option<&ResolvedCodec>;
    pub fn query(&self, entity: &EntityMetadata) -> Option<&QueryMetadata>;
}
```

`ResolvedModelGraph` 是一次完整验证后的 immutable snapshot。不存在“部分成功但 getter 静默返回 None”的状态；
`resolve_all()` 失败时返回全部确定性排序错误，不发布图。

`QueryMetadata` 的最终拥有者是 `ResolvedModelGraph`，不是 `EntityMetadata`：

```rust
impl QueryMetadata {
    pub fn filters(&self) -> &'static [QueryField];
    pub fn unique_keys(&self) -> &'static [UniqueQueryKey];
    pub fn filter(&self, path: &PropertyPath) -> Option<&QueryField>;
    pub fn filter_by_flat_name(&self, name: &str) -> Option<&QueryField>;
}

impl QueryField {
    pub fn path(&self) -> &'static PropertyPath;
    pub fn flat_name(&self) -> &'static str;
    pub fn descriptor(&self) -> Option<&'static TypeDescriptor>;
    pub fn reasons(&self) -> IndexingReasons;
}
```

只有 Entity 产生根查询视图。resolver 负责根 identifier/global unique 排除、scoped unique、普通值递归、reference
一跳和平面名冲突。字段局部 `indexing_reasons()` 不依赖 resolver。

### 错误 API

```rust
pub enum ModelRegistryErrorKind {
    ReflectionRegistry,
    DuplicateModelId,
    RegistrationConflict,
    UnsupportedPlatform,
}

pub enum ModelResolveErrorKind {
    MissingModelId,
    WrongModelRole,
    MissingProperty,
    UnreadableProperty,
    TypeMismatch,
    MissingValidator,
    ValidatorTypeMismatch,
    MissingCodec,
    CodecTypeMismatch,
    InvalidProjectionSource,
    InvalidValueClosure,
    QueryNameConflict,
    InvalidReferenceGraph,
}

impl ModelResolveError {
    pub fn kind(&self) -> ModelResolveErrorKind;
    pub fn path(&self) -> Option<&PropertyPath>;
    pub fn model_id(&self) -> Option<&str>;
    pub fn expected_role(&self) -> Option<ModelRole>;
    pub fn actual_role(&self) -> Option<ModelRole>;
    pub fn expected_type(&self) -> Option<std::any::TypeId>;
    pub fn actual_type(&self) -> Option<std::any::TypeId>;
    pub fn sources(&self) -> &[qubit_reflect::identity::FragmentIdentity];
}

impl ModelResolveErrors {
    pub fn errors(&self) -> &[ModelResolveError];
    pub fn into_errors(self) -> Vec<ModelResolveError>;
}
```

错误排序键固定为 kind、model ID、结构化 path、source identity。Display 只负责稳定英文诊断，不把本地化文案作为
机器协议。

## 隐藏生产 ABI

普通用户不得手工构造 metadata。宏生成代码只使用版本化隐藏模块：

```rust,ignore
#[doc(hidden)]
pub mod __private {
    pub use qubit_reflect::__private as reflect;

    pub mod v3 {
        // checked metadata factories
        // model capability registration
        // concrete reflection projection / generic model registration
        // property adapters and provider seal
        // compile-time assertion helpers
    }
}
```

规则：

- 当前生成代码只引用 `__private::v4`；旧构造路径不保留兼容别名。
- hidden builder 在 `OnceLock` 初始化时检查字段与反射 descriptor 的 index/name/type 对齐、角色与 shape、属性互斥、
  property 合并和 adapter TypeId；检查失败表示宏/runtime 版本不兼容，panic 文案必须含 ABI 版本和 source identity。
- capability adapter 类型必须准确为 `ModelMetadataProvider`，同一 concrete descriptor 重复注册该 key 会由 reflect
  capability conflict 拒绝。
- 统一反射 fragment 与模型层泛型 fragment 都只保存静态 identity 和 factory function；具体模型不再重复提交
  统一反射 inventory。用户代码和 metadata 构造推迟到 registry 初始化或首次静态查询。
- derive 与 runtime 使用精确 patch-compatible 依赖约束，并保留正常、renamed、missing、invalid-runtime 和 facade
  fixture。

## `rs-model-derive` 内部实现

### 共享流水线

```text
TokenStream
  -> ParsedDeclaration / ParsedModelImpl
  -> DeclarationIr / ModelImplIr
  -> ValidatedDeclaration / ValidatedModelImpl
  -> GeneratedItems
```

入口函数只选择 `MacroKind`：

```rust,ignore
enum MacroKind {
    Entity,
    Projection,
    Model,
    Enum,
    Value,
    ModelImpl,
}
```

parser 只保留 token 与 span；normalizer 把简写转为唯一语义 IR；validator 只处理当前声明和可由 Rust bound 证明的事实；
expander 分别生成 retained item、reflect delegation、metadata provider、capability、registration、默认能力和诊断断言。

### 推荐模块布局

```text
src/
├── lib.rs
├── runtime_path.rs
├── parse/
│   ├── declaration.rs
│   ├── model_impl.rs
│   ├── attributes.rs
│   └── types.rs
├── ir/
│   ├── declaration.rs
│   ├── role.rs
│   ├── field.rs
│   ├── property.rs
│   ├── constraint.rs
│   ├── strategy.rs
│   ├── representation.rs
│   └── registration.rs
├── normalize/
│   ├── declaration.rs
│   ├── fields.rs
│   ├── selectors.rs
│   └── capabilities.rs
├── validate/
│   ├── declaration_shape.rs
│   ├── role_composition.rs
│   ├── model_id.rs
│   ├── identity.rs
│   ├── relation.rs
│   ├── constraints.rs
│   ├── selectors.rs
│   ├── properties.rs
│   ├── capabilities.rs
│   └── representation.rs
└── expand/
    ├── declaration.rs
    ├── reflect.rs
    ├── metadata.rs
    ├── fields.rs
    ├── properties.rs
    ├── role.rs
    ├── capability.rs
    ├── registration.rs
    ├── defaults.rs
    ├── serde.rs
    ├── redact.rs
    └── assertions.rs
```

当前 `ModelInput -> normalize::ModelIr -> validate -> expand` 骨架可以保留，但旧的 role-neutral `ModelIr` 应升级为
`DeclarationIr { role, shape, ... }`；lookup_relation、ownership、generator、旧 primary key/key/index 字段不得继续进入
新 IR。

### 反射委托生成

每个类型宏必须：

1. 保留原始 struct/enum 和非模型属性。
2. 生成 `Reflect` 派生及 `#[reflect(crate = resolved_facade)]`。
3. 将 `#[opaque]`、字段访问策略和模型需要的 query name 转成受支持的反射 helper。
4. 从反射 descriptor 构造领域 overlay；不得重新解析 runtime type name 或重建字段访问器。
5. 注册 model metadata provider capability。
6. 仅在有 ModelId 时提交 model registration。
7. 生成默认 derive、Serde 和 Redact 实现以及 trait-bound assertions。

模型 macro 与 `Reflect` macro 都会检查字段 token，但职责不同：反射宏决定 Rust 结构和安全访问；模型宏决定领域属性。
模型宏生成的 overlay 在初始化时按 index 与反射字段再次对齐，防止两个 proc-macro 版本漂移。

## 测试策略

| 风险 | 首选测试 |
| --- | --- |
| 属性 token、重复参数、错误 span | parser 单元测试 + trybuild fail |
| 简写规范化和多错误聚合 | normalize/validate 白盒测试 |
| reflect facade 与依赖重命名 | 多 crate runtime fixture |
| descriptor 唯一性 | `std::ptr::eq` runtime test |
| overlay 与字段/variant 对齐 | checked builder 单元测试 + runtime test |
| recursive/opaque/symbolic type | metadata 与 derive integration test |
| 字段/Property 动态访问 | runtime test + Miri |
| capability ID/adapter 冲突 | reflect/model 集成测试 |
| ModelId 重复、排序、source | registry 单元测试 + linked fixture |
| generic definition/concrete cache | 并发 runtime test |
| source/reference/strategy 跨 crate 解析 | linked workspace fixture |
| query 一跳、路径冲突、唯一键 | resolver golden test |
| 默认能力、Serde、Redact | runtime 行为与序列化 snapshot |

至少执行：

```bash
cargo test --manifest-path rs-reflect/Cargo.toml
cargo test --manifest-path rs-model-metadata/Cargo.toml
cargo test --manifest-path rs-model-derive/Cargo.toml

cargo clippy --manifest-path rs-model-metadata/Cargo.toml --all-targets --all-features -- -D warnings
cargo clippy --manifest-path rs-model-derive/Cargo.toml --all-targets --all-features -- -D warnings

cargo doc --manifest-path rs-model-metadata/Cargo.toml --all-features --no-deps
cargo doc --manifest-path rs-model-derive/Cargo.toml --all-features --no-deps
```

具体实施时再按各仓库脚本补充 `style-check.sh`、`ci-check.sh` 和 Miri 命令。

## 关键不变量

1. 每个 concrete Rust 类型只有一个 `TypeDescriptor` 根；模型层只引用它。
2. `TypeMetadata::of::<T>()`、descriptor capability 和已注册 concrete metadata 返回同一静态对象。
3. 静态 metadata 查询不初始化 `ModelRegistry`，不解析字符串 ID。
4. `ModelRegistry` 只包含有稳定 ModelId 的 concrete 类型或 generic definition。
5. `type_name()` 只用于诊断；类型相等使用 `TypeId`，协议身份使用 ModelId。
6. opaque 和 symbolic 状态通过 `TypeRef` 显式暴露，不伪造成 resolved descriptor。
7. 所有借用 erased value 的生命周期都来自输入 borrow，不制造 `'static`。
8. proc-macro 不读取文件系统、数据库、网络或链接后 registry。
9. 当前声明可判定的错误在编译期报告；跨 crate 图错误只在显式 resolver 中报告。
10. reference 循环合法；descriptor、resolver 和 query 展开都有有限边界。
11. Value 纯值闭包、reference 一跳和 opaque 截断在 derive、metadata、resolver、consumer 四层含义一致。
12. Redact/Serde 默认输出 fail closed；缺少 adapter 不得泄露敏感值。
13. hidden ABI 构造时重新检查会影响内存安全或图一致性的事实。
14. 公共 metadata 全部 immutable、`Send + Sync`、可静态共享；失败的全局初始化同样被缓存。

## 验证与完成条件

只有全部满足以下条件，才可以称为最终 API 已实现：

- 六个模型宏公开、共享流水线并具有 Rustdoc。
- 模型类型的 reflect descriptor 全局唯一，model metadata capability 与静态入口指针一致。
- 本文列出的公共类型和方法均已实现，不存在仅有类型名而无查询接口的断链。
- 五种角色、Field、Property、constraint、strategy、generic、registry、resolver 和 query 均有对应测试。
- 所有跨 crate 行为由真实 linked workspace fixture 验证。
- Property borrowed adapter 通过 Miri 或等价内存安全验证。
- 需求规范明确排除的旧 API 在公开导出、parser、IR、expansion、README 和用户指南中均不存在。
- `rs-reflect`、`rs-model-metadata`、`rs-model-derive` 的 test、clippy、rustdoc、style 和 CI 全部通过。
- 需求规范、本文、公开 Rust API、README 与用户指南使用同一组最终术语和约束。
