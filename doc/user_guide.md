# qubit-model-derive User Guide

[README](../README.md) | [中文用户指南](user_guide.zh_CN.md) | [Final design](rs-model-derive-final-design.md)

## Purpose and Audience

This guide is for domain-model authors and framework developers using
`qubit-model-derive` 0.1.0. It explains how to declare the six public macros,
inspect generated metadata, resolve cross-model relations, and interpret their
diagnostics. The crate requires Rust 1.94 and edition 2024.

## Conceptual Model

Each role macro generates the Rust descriptor through `qubit-reflect`, then
adds `TypeMetadata` as a typed capability on that same descriptor. A field is
a real storage slot. A property is a name-based view formed from a field,
getter, and setter.

```text
role declaration -> Reflect descriptor -> model metadata capability
                                       -> frozen registry projection
generic role declaration -------------> generic ModelRegistration
ModelImpl impl -----------------> property capability
```

Direct `TypeMetadata::of` lookup has no global model-registry dependency.
Descriptor capability and property lookup freeze the reflection snapshot so
separate fragments remain visible. The generated facade is model ABI v3 and
uses only reflection `codegen_v2`. A model registry and resolver are needed for
stable IDs, references, projection sources, and queries spanning the complete
linked model set.

## Installation and Minimal Configuration

The derive crate is not currently published to crates.io (`publish = false`).
In a checkout containing both repositories, use paths such as the following
and adjust them for your workspace layout:

```toml
[dependencies]
qubit-model-derive = { version = "0.1", path = "../rs-model-derive" }
qubit-model-metadata = { version = "0.1", path = "../rs-model-metadata" }
qubit-id = { version = "0.6", path = "../../rust-common/rs-id" }
qubit-codec = { version = "0.14", features = ["registry"] }
serde_json = "1"
```

The graph-resolution example imports `ValueCodecRegistry`, which is available
only when the direct `qubit-codec` dependency enables `registry`. Generated
code resolves a renamed `qubit-model-metadata` dependency automatically; an
application crate does not need a direct `qubit-reflect` dependency.

## Scenario: A User Entity and Login Request

Assume a login service needs a redacted email value, a persistent user record,
and a request that points at that user. Success means that static metadata can
discover the identifier and writable email property, serialized output does
not expose the raw email, and the linked resolver can bind the request's
reference to `example.User`.

Declare a transparent value, an entity, a reference-bearing model, and both a
field-backed property and a computed property:

```rust
use qubit_id::Id;
use qubit_model_derive::{Entity, Model, ModelImpl, Projection, Value};

#[Value(transparent)]
pub struct Email(
    #[redact(level = "medium")]
    String,
);

#[Entity(id = "example.User")]
pub struct User {
    #[identifier(assigned_by = application)]
    id: Id,
    #[unique(ignore_case = false)]
    #[redact(nested)]
    email: Email,
    #[serde(default)]
    aliases: Vec<String>,
}

#[Projection(id = "example.UserView", source = User)]
pub struct UserView {
    #[identifier]
    id: Id,
    #[redact(nested)]
    email: Email,
}

#[Model(id = "example.Login")]
pub struct Login {
    #[reference(entity_id = "example.User", property = id)]
    user_id: Id,
}

#[ModelImpl]
impl User {
    pub fn email(&self) -> &Email { &self.email }
    pub fn set_email(&mut self, value: Email) { self.email = value; }
    pub fn alias_slice(&self) -> &[String] { &self.aliases }
    pub fn view(&self) -> UserView {
        UserView { id: self.id, email: self.email.clone() }
    }
}
```

The generated serializer follows the nested redaction rule, so the observable
JSON cannot contain the original address:

```rust,ignore
let user = User {
    id: Id::new(7),
    email: Email("alice@example.com".to_owned()),
    aliases: Vec::new(),
};
let json = serde_json::to_string(&user)?;
assert!(!json.contains("alice@example.com"));
```

## Core Workflow

### Inspect static metadata

The type is usable immediately; this path does not initialize
`ModelRegistry`:

```rust,ignore
use qubit_model_metadata::{ModelDescriptorExt, TypeMetadata};

let user = TypeMetadata::of::<User>();
assert!(user.field("id").unwrap().is_identifier());
assert!(user.try_property("email").unwrap().unwrap().is_writable());
assert!(user.try_property("alias_slice").unwrap().unwrap().is_computed());
assert!(user.descriptor().model_metadata().is_some());
```

`TypeMetadata` overlays the unique `TypeDescriptor`; it does not create a
second structure graph. For a field whose type is opaque or symbolic in a
generic definition, `descriptor()` returns `None`; use `type_ref()` to inspect
the actual `Resolved`, `Opaque`, or `Symbolic` form.

### Resolve the linked model graph

After all business crates are linked, resolve declarations that need other
models. This is where a missing target, an incorrect role, or an unknown
referenced property is reported:

```rust,ignore
use qubit_codec::ValueCodecRegistry;
use qubit_model_metadata::{
    ModelRegistry, ModelResolver, PropertyPath, ResolveInputs, TypeMetadata,
};
use qubit_validator::ValidatorRegistry;

fn inspect_graph() -> Result<(), Box<dyn std::error::Error>> {
    let models = ModelRegistry::try_global()?;
    let validators = ValidatorRegistry::try_global()?;
    let codecs = ValueCodecRegistry::try_global()?;
    let graph = ModelResolver::new(ResolveInputs { models, validators, codecs })
        .resolve_all()?;

    let field = TypeMetadata::of::<Login>().field("user_id").unwrap();
    assert_eq!(
        graph.reference(field).unwrap().target().model_id().unwrap().as_str(),
        "example.User",
    );

    let user = TypeMetadata::of::<User>().as_entity().unwrap();
    let query = graph.query(user).unwrap();
    assert!(query.unique_keys().iter().any(|key| {
        key.path().is_some_and(|path| path == PropertyPath::new(&["email"]))
    }));
    assert_eq!(graph.projection_producers().len(), 1);
    Ok(())
}
```

The resolved graph is published only when every registration is valid. It
also owns Entity query views and Entity-to-Projection producer edges; local
`FieldMetadata` deliberately does not guess those cross-model results.

## Advanced Usage

### Roles and Generic Rules

- `Entity` and `Projection` accept named structs, require one `#[identifier]`,
  and do not support generics.
- A `Projection` with no source is open. `source = EntityType` and
  `source_id = "..."` declare a fixed source and are mutually exclusive.
- `Model` accepts named or unit structs, but not tuple structs.
- `Enum` accepts enums and records each variant's Rust, canonical, and Serde
  names.
- `Value` accepts a named-field struct or a one-field tuple struct.
  `transparent` requires exactly one field and makes `Serialize`,
  `Deserialize`, and `Display` use the inner representation.
- `Model`, `Enum`, and `Value` support type parameters, where clauses, and
  reflect-supported primitive const generics, but not lifetime parameters.
- An ID-bearing generic declaration registers its definition. Concrete
  monomorphizations have no `ModelId`; use `generic_definition()` to navigate
  to the template and its symbolic fields.

### Field Declarations

Useful field declarations include:

- `#[identifier(assigned_by = application|database)]` and `#[indexed]`;
- `#[unique(respect_to(tenant_id), ignore_case = true)]`;
- `#[reference(entity = User, property = id)]` or
  `#[reference(entity_id = "example.User", property = id)]`;
- `#[key_part(order = 0)]`;
- `#[text(...)]`, `#[decimal(...)]`, `#[money(...)]`, `#[time(...)]`,
  `#[sequence(...)]`, and `#[map(...)]`;
- non-recursive `#[element(...)]`, `#[map_key(...)]`, and
  `#[map_value(...)]` selectors;
- `#[codec(MyCodec)]` or `#[codec(id = "example.codec")]`;
- `#[redact(level = "medium")]`, `skip`, `nested`, `map`, `keyed_by`, and
  `json` modes;
- `#[validator(id = "example.rule", params(...), depends_on(...))]`.

The identifier defaults to `assigned_by = application`; only an Entity may
use `database`. `#[indexed]` takes no options and describes logical filter
capability. `#[unique]` defaults to case-insensitive comparison, which is
valid only for text-capable fields; `respect_to(...)` lists the scope paths in
source order. A reference selects exactly one `entity = RustType` or
`entity_id = "..."`; `property` chooses the stored target property,
`existing` defaults to `true`, and `path` binds to another reference in the
same object graph.

#### Constraint Option Reference

- `text` supports `min_chars`, `max_chars`, `min_bytes`, `max_bytes`,
  `non_blank`, `allowed_chars`, and `format`. Character sets are `unicode`,
  `printable_unicode`, `ascii`, `printable_ascii`, and `code`; formats are
  `email`, `cn_mobile`, `uri`, and `uuid`.
- `decimal` and `money` support `precision`, `scale`, string-valued `min` and
  `max`, `min_inclusive`, `max_inclusive`, and `rounding`. `money` requires
  `scale`; decimal defaults to `half_even`, while money defaults to
  `unnecessary`.
- `time` requires `precision = second|millisecond|microsecond|nanosecond`.
- `sequence` supports `min_items`, `max_items`, and `unique_items`; `map`
  supports `min_entries` and `max_entries`. Each declaration must contain at
  least one option, and every minimum must be no greater than its maximum.

Constraint targets are checked at compile time. Standard leaf constraints do
not implicitly descend into a container: use `element`, `map_key`, or
`map_value`. A selector may contain leaf constraints, validators, one codec,
and `redact(level = "...")`, but it cannot contain another selector.

`key_part` is a value-semantic key, not persistence identity. It is accepted
only on real named fields of `Model` and `Value`. A declaration may select a
subset of its fields, but selected orders must be unique and contiguous from
zero. For example, a locale-aware code can key on `namespace` and `code`
while leaving a descriptive label outside the key:

```rust
use qubit_model_derive::Value;

#[Value]
struct LocalizedCode {
    #[key_part(order = 0)]
    namespace: String,
    #[key_part(order = 1)]
    code: String,
    label: String,
}
```

Use `#[identifier]` for Entity/Projection identity. `key_part` is rejected on
those roles, on `Enum`, and on tuple/newtype values because those shapes do
not represent an ordered selection of named storage fields.

Selectors accept only `redact(level = "...")` for redaction. `element` and
`map_value` apply to container values. `map_key` redacts output keys; if two
source keys map to the same redacted key, Serde serialization returns an error
rather than overwriting a value.

`#[opaque]` keeps the outer field shape but stops descriptor traversal at the
leaf. Use it for externally supplied values that cannot implement reflection.
It cannot be combined with `#[reference]` and cannot hide a registered Entity,
Projection, or Model from resolver checks.

`validator` produces syntax-validated occurrence metadata. `ModelResolver`
binds its ID to an executable `qubit-validator` descriptor, checks the value
type, and resolves dependency properties. A Rust codec is checked at compile
time for `Default + ValueEncoder<Value, Output = String> +
ValueDecoder<str, Output = Value>`.

Validator parameters accept booleans, integers, strings, and non-empty
homogeneous arrays of those literals. Multiple validator occurrences retain
source order. Codecs can be selected by Rust type or stable ID; a canonical
whole-value codec is available only as `#[Value(codec = CodecType)]`.

### Default Interfaces, Serde, and Redaction

The five role macros generate `Clone`, `PartialEq`, `Eq`, `Hash`, `Redact`,
`Debug`, `Display`, `Serialize`, and `Deserialize` by default. Output
interfaces use `qubit-redact` with fail-closed behavior; `Deserialize` handles
input only.

Use `no_clone`, `no_debug`, `no_display`, `no_partial_eq`, `no_eq`, `no_hash`,
`no_redact`, `no_serialize`, or `no_deserialize` to disable an interface.
`no_redact` is valid only when no field or selector carries a redaction rule;
the remaining output interfaces then use their plain implementations. `copy`,
`default`, `partial_ord`, and `ord` are opt-in. An all-unit enum is `Copy` by
default unless it has `no_copy`.

Put a role attribute before user `#[derive(...)]`. With redaction enabled, the
macro rejects existing `Debug` or `Serialize` implementations that could
bypass protected output. With `no_redact`, it can reuse a compatible existing
implementation instead of deriving it again.

Named `Option` and standard collection fields receive default Serde behavior:
they deserialize from an omitted value and omit an empty value while
serializing. `#[keep_serializing]` preserves empty output; an explicit Serde
attribute takes precedence.

### ModelImpl

`#[ModelImpl]` accepts an inherent impl and turns its public, safe,
synchronous, non-generic methods into property fragments. A getter has `&self`
and returns `T`, `&T`, `&str`, `&[T]`, or `Option<&T>`. A setter has
`&mut self, T` and returns `()`. Getter metadata preserves whether the result
is owned, borrowed, optionally borrowed, or a borrowed slice. A setter returns
its replacement value when it fails before invoking user code. Duplicate
properties or duplicate annotated impls fail at compile time.

Fields, getters, and setters merge by name:

- a field with no accessor is `FieldBacked`;
- a getter without a field is `Computed`;
- a setter without a field is `Virtual`;
- matching accessors enrich the same property instead of creating duplicates.

### Query Views and Automatic Projections

After successful resolution, `ResolvedModelGraph::query` exposes query facts
for Entities. Identifier and global-unique paths are lookup keys; scoped
unique and explicitly indexed paths can become filters. Nested value paths may
be flattened with `_`, references expand by at most one Entity hop, and a
flattened-name collision makes resolution fail instead of choosing one path.
These facts describe logical query capability, not physical database indexes.

A readable Entity property whose return type is a fixed-source Projection
forms a `ResolvedProjectionProducer`. In the scenario, `User::view` produces
`UserView`. Calling `project` invokes the erased getter and then verifies that
the Projection retained the Entity's exact `qubit_id::Id`; changing the ID
returns `ProjectionExecutionError::IdentifierMismatch`. A declaration without
an executable getter can still be used by a DAO or deserializer, but automatic
projection returns `MissingProjector`.

## Errors and Diagnostics

- The macro aggregates independent declaration-shape, option, singleton-field,
  and key-order errors in one diagnostic pass.
- If the runtime facade cannot be resolved, add `qubit-model-metadata`; a
  renamed dependency is supported.
- Cross-model ID, role, source, reference, and property checks belong to
  `ModelResolver`, because only the complete linked set can establish them.
- `ModelRegistry::try_global()`, `ValidatorRegistry::try_global()`, and
  `ValueCodecRegistry::try_global()` return initialization errors. Their
  `global()` shortcuts panic when the cached registry is invalid, so use the
  fallible forms at application startup.
- `resolve_all()` returns a deterministically ordered `ModelResolveErrors`
  collection and does not publish a partial graph. Inspect each error's kind,
  model ID, property path, expected/actual role or type, and source identity.
- `TypeRef::Opaque` and `TypeRef::Symbolic` have no concrete descriptor. Test
  `type_ref()` rather than treating `descriptor() == None` as missing metadata.
- When a codec bound fails, implement the exact `ValueEncoder` and
  `ValueDecoder` contracts. There is no separate codec-contract ID type to add.

## Troubleshooting

| Symptom | Check first |
| --- | --- |
| A role macro reports an existing derive conflict | Place the role attribute before `#[derive(...)]`; remove an unsafe duplicate or use the permitted `no_*` option. |
| A reference is unresolved | Build the application with every model crate linked, then run `ModelResolver` over `ModelRegistry::try_global()?`. |
| Global registry initialization panics | Replace `global()` with `try_global()` during diagnosis and inspect the cached duplicate-ID, registration, validator, or codec error. |
| A property is absent or rejected | Confirm that the method is a public, safe, synchronous, non-generic inherent getter or setter with a supported receiver and type. |
| Graph resolution reports a query-name conflict | Compare complete `PropertyPath` values that flatten to the same `_`-joined name; rename the model property rather than adding a physical-index option. |
| A collection field disappears from JSON | Check the default omission rule, `#[keep_serializing]`, and explicit Serde attributes. |
| Map serialization fails after redaction | Treat the collision as a data-model issue; choose a redaction policy that preserves distinguishable keys or avoid redacting map keys. |

## Limitations and Best Practices

Do not initialize `ModelRegistry` for ordinary static metadata queries and do
not use Rust `type_name()` as a stable `ModelId`. Resolve cross-model checks
once after the full model set is linked. The resolver binds validator
occurrences to executable registrations, but this crate does not orchestrate
when application validation runs.

`#[indexed]`, `#[unique]`, and `#[key_part]` expose logical facts for
downstream schema, query, and diagnostic consumers; they do not configure
tables, columns, index order, or database-specific features. Keep deeply
nested reusable rules in named `Value`, `Model`, or `Enum` types because
selectors do not recursively contain other selectors.

Generated `Eq` and `Hash` implementations use structural field semantics. Do
not place a mutable Entity directly in a `HashSet` or use it as a `HashMap`
key if any participating field can change; prefer its stable identifier.

## Further Reading

- [README](../README.md)
- [中文用户指南](user_guide.zh_CN.md)
- Generate local API documentation with `cargo doc --open`.
- [Final design](rs-model-derive-final-design.md)
