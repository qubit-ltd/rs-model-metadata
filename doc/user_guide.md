# qubit-model-derive User Guide

[README](../README.md) | [中文用户指南](user_guide.zh_CN.md) | [Final design](2026-08-31-182016-rs-model-derive-final-api-and-implementation-design.md)

## Purpose and Audience

This guide is for domain-model authors and framework developers using
`qubit-model-derive` 0.1.x. It explains how to declare the six public macros,
inspect generated metadata, resolve cross-model relations, and interpret their
diagnostics. The crate requires Rust 1.94 and edition 2024.

## Conceptual Model

Each role macro generates the Rust descriptor through `qubit-reflect`, then
adds `TypeMetadata` as a typed capability on that same descriptor. A field is
a real storage slot. A property is a name-based view formed from a field,
getter, and setter.

```text
role declaration -> Reflect descriptor -> model metadata capability
                                       -> optional ModelRegistration
ModelProperties impl -----------------> property capability
```

Static lookup has no global-registry dependency. A registry and resolver are
only needed for stable IDs, references, projection sources, and queries that
span the complete linked model set.

## Scenario: A User Entity and Login Request

Assume a login service needs a redacted email value, a persistent user record,
and a request that points at that user. The derive crate is not currently
published to crates.io (`publish = false`). In a checkout containing both
repositories, use paths such as the following and adjust them for your
workspace layout:

```toml
[dependencies]
qubit-model-derive = { path = "../rs-model-derive" }
qubit-model-metadata = { path = "../rs-model-metadata" }
```

Declare a transparent value, an entity, a reference-bearing model, and one
field-backed property plus a computed property:

```rust,ignore
use qubit_model_derive::{Entity, Model, ModelProperties, Value};

#[Value(transparent)]
pub struct Email(
    #[redact(level = "medium")]
    String,
);

#[Entity(id = "example.User")]
pub struct User {
    #[identifier(assigned_by = application)]
    id: u64,
    #[unique(ignore_case = true)]
    #[redact(nested)]
    email: Email,
    #[serde(default)]
    aliases: Vec<String>,
}

#[Model(id = "example.Login")]
pub struct Login {
    #[reference(entity_id = "example.User", property = id)]
    user_id: u64,
}

#[ModelProperties]
impl User {
    pub fn email(&self) -> &Email { &self.email }
    pub fn set_email(&mut self, value: Email) { self.email = value; }
    pub fn alias_slice(&self) -> &[String] { &self.aliases }
}
```

## Core Workflow

### Inspect static metadata

The type is usable immediately; this path does not initialize
`ModelRegistry`:

```rust,ignore
use qubit_model_metadata::{ModelDescriptorExt, TypeMetadata};

let user = TypeMetadata::of::<User>();
assert!(user.field("id").unwrap().is_identifier());
assert!(user.property("email").unwrap().is_writable());
assert!(user.property("alias_slice").unwrap().is_computed());
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
use qubit_model_metadata::{ModelRegistry, ModelResolver, ResolveInputs, TypeMetadata};
use qubit_validator::ValidatorRegistry;

let models = ModelRegistry::try_global()?;
let validators = ValidatorRegistry::try_global()?;
let codecs = ValueCodecRegistry::try_global()?;
let graph = ModelResolver::new(ResolveInputs { models, validators, codecs }).resolve_all()?;
let field = TypeMetadata::of::<Login>().field("user_id").unwrap();
assert_eq!(
    graph.reference(field).unwrap().target().model_id().unwrap().as_str(),
    "example.User",
);
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Roles and Generic Rules

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

## Field Declarations

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

Selectors accept only `redact(level = "...")` for redaction. `element` and
`map_value` apply to container values. `map_key` redacts output keys; if two
source keys map to the same redacted key, Serde serialization returns an error
rather than overwriting a value.

`validator` produces syntax-validated occurrence metadata. `ModelResolver`
binds its ID to an executable `qubit-validator` descriptor, checks the value
type, and resolves dependency properties. A Rust codec is checked at compile time for
`Default + ValueEncoder<Value, Output = String> + ValueDecoder<str, Output = Value>`.

## Default Interfaces, Serde, and Redaction

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

## ModelProperties

`#[ModelProperties]` accepts a public, safe, synchronous, non-generic inherent
impl. A getter has `&self` and returns `T`, `&T`, `&str`, `&[T]`, or
`Option<&T>`. A setter has `&mut self, T` and returns `()`. Getter and setter
metadata preserve borrowing; a setter returns its replacement value when it
fails before invoking user code. Duplicate properties or duplicate annotated
impls fail at compile time.

Fields, getters, and setters merge by name:

- a field with no accessor is `FieldBacked`;
- a getter without a field is `Computed`;
- a setter without a field is `Virtual`;
- matching accessors enrich the same property instead of creating duplicates.

## Errors and Diagnostics

- The macro aggregates independent declaration-shape, option, singleton-field,
  and key-order errors in one diagnostic pass.
- If the runtime facade cannot be resolved, add `qubit-model-metadata`; a
  renamed dependency is supported.
- Cross-model ID, role, source, reference, and property checks belong to
  `ModelResolver`, because only the complete linked set can establish them.
- `TypeRef::Opaque` and `TypeRef::Symbolic` have no concrete descriptor. Test
  `type_ref()` rather than treating `descriptor() == None` as missing metadata.
- When a codec bound fails, implement the exact `ValueEncoder` and
  `ValueDecoder` contracts. There is no separate codec-contract ID type to add.

## Troubleshooting

| Symptom | Check first |
| --- | --- |
| A role macro reports an existing derive conflict | Place the role attribute before `#[derive(...)]`; remove an unsafe duplicate or use the permitted `no_*` option. |
| A reference is unresolved | Build the application with every model crate linked, then run `ModelResolver` over `ModelRegistry::try_global()?`. |
| A property is absent or rejected | Confirm that the method is a public, safe, synchronous, non-generic inherent getter or setter with a supported receiver and type. |
| A collection field disappears from JSON | Check the default omission rule, `#[keep_serializing]`, and explicit Serde attributes. |
| Map serialization fails after redaction | Treat the collision as a data-model issue; choose a redaction policy that preserves distinguishable keys or avoid redacting map keys. |

## Limitations and Best Practices

Do not initialize `ModelRegistry` for ordinary static metadata queries and do
not use Rust `type_name()` as a stable `ModelId`. Resolve cross-model checks
once after the full model set is linked. Validator execution is intentionally
outside the current API; declarations are useful for schema, documentation,
and future resolver input only.

## Further Reading

- [README](../README.md)
- [中文用户指南](user_guide.zh_CN.md)
- Generate local API documentation with `cargo doc --open`.
- [Final API and implementation design](2026-08-31-182016-rs-model-derive-final-api-and-implementation-design.md)
