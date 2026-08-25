# Qubit Model Metadata User Guide

[中文](user_guide.zh_CN.md) · [README](../README.md) · [API documentation](https://docs.rs/qubit-model-metadata)

Applies to `qubit-model-metadata` 0.1.0.

## Purpose and Audience

This guide is for application and tool authors who consume static domain-model
metadata at runtime: schema helpers, validators, and code that must inspect
fields, constraints, keys, and relations without a mutable registry.

`qubit-model-derive` usually produces the metadata. This crate is the typed
query API. The full `#[Model]` / `#[Enum]` attribute language lives in the
[derive user guide](https://github.com/qubit-ltd/rs-model-derive/blob/main/doc/user_guide.md).
The macros do not validate instance data.

## Conceptual Model

`HasTypeShape` describes a recursive structural shape. Named models also
implement `HasTypeMetadata` and expose `TypeMetadata`. `metadata_of::<T>()`
returns a `&'static` value built from static slices and function pointers.

```text
HasTypeShape ──► TypeRef ──► TypeShape
       │
HasTypeMetadata ──► TypeMetadata ──► FieldMetadata + model attributes
       │
HasModelRegistration ──► MODEL_REGISTRATIONS ──► ModelRegistry
```

A named model has a portable `ModelId` (for example `example.Account`) and a
process-local `TypeIdentity`. `TypeIdentity` compares types with Rust's
`TypeId`. Type names are diagnostic display data. Do not persist a `TypeId` or
treat it as a cross-process identifier.

`TypeKind` classifies a named type as `Struct`, `Enum`, or `Newtype`. Field
queries apply to structs; enums and newtypes expose an empty `struct_fields()`
slice.

## Scenario

A signup service stores accounts. Each account has a generated identifier, a
unique email, optional tags, and a nested contact record. Success means the
service can:

1. compile the declarations;
2. read the primary key, email constraints, and ignore-case uniqueness;
3. see that `tags: Option<Vec<String>>` is nullable at the outer layer;
4. resolve `contact.email` through a static field path.

## Installation and Minimal Configuration

The crate requires Rust 1.94 or later. Add the runtime crate. When metadata
should be generated from declarations, also add the companion derive crate and
Serde. Both macros require `serde` in the consuming crate.

```toml
[dependencies]
qubit-model-metadata = "0.1.0"
qubit-model-derive = "0.1.0"
serde = { version = "1", features = ["derive"] }
```

Enable runtime features only for the scalar types you actually use:

```toml
[dependencies]
qubit-model-metadata = { version = "0.1.0", features = ["chrono", "big-decimal"] }
chrono = { version = "0.4", default-features = false, features = ["std"] }
bigdecimal = "0.4"
```

`chrono` covers `NaiveDate`, `NaiveTime`, `NaiveDateTime`, and `DateTime<Utc>`.
`big-decimal` covers `BigDecimal`.

## Core Workflow

Declare the nested contact and the account. `Model` is an attribute macro, not
`#[derive(Model)]`. `#[field(identifier)]` becomes a model-level primary key.
`unique(ignore_case)` becomes a model-level unique constraint.

```rust
use qubit_model_derive::Model;
use qubit_model_metadata::FieldPath;
use qubit_model_metadata::TypeShape;
use qubit_model_metadata::UniqueComparison;
use qubit_model_metadata::metadata_of;

#[Model(id = "example.Contact")]
struct Contact {
    #[field(text(max_chars = 320))]
    email: String,
}

#[Model(id = "example.Account")]
struct Account {
    #[field(identifier)]
    id: i64,
    #[field(text(min_chars = 3, max_chars = 320), unique(ignore_case))]
    email: String,
    tags: Option<Vec<String>>,
    contact: Contact,
}

fn inspect_account() {
    let metadata = metadata_of::<Account>();
    let email = metadata.field("email").expect("declared field");

    assert!(metadata.primary_key().expect("primary key").contains("id"));
    assert_eq!(email.text_constraint().and_then(|value| value.max_chars()), Some(320));
    assert_eq!(
        metadata
            .unique_constraints()
            .next()
            .and_then(|unique| unique.comparison_of("email")),
        Some(UniqueComparison::IgnoreCase)
    );
    assert!(matches!(
        metadata.field("tags").expect("tags field").field_type().shape(),
        TypeShape::Optional(_)
    ));
    assert!(metadata.field("tags").expect("tags field").is_nullable());

    let nested = metadata
        .resolve_field_path(FieldPath::new(&["contact", "email"]))
        .expect("nested field");
    assert_eq!(nested.name(), "email");
}
```

`field` returns `Option` because the queried name might not be declared. Treat
absence as a configuration error, not as an impossible state.

A `ModelId` uses ASCII snake_case module segments and an ASCII UpperCamelCase
final segment, for example `example.Account`. Empty segments, Rust keywords as
module segments, and a final segment that is not UpperCamelCase are rejected.

## Advanced Usage

### Type shapes and nullability

`TypeRef` is a small copyable handle. `shape()` returns a recursive `TypeShape`:
scalar, named model, optional value, sequence, set, map, fixed array, or
`Opaque`. For macro-produced opaque fields, visible standard wrappers such as
`Option`, sequences, sets, arrays, and maps remain in the shape; only the leaf
is `Opaque`. The outer shape controls nullability and relation projection.

```rust
use qubit_model_metadata::TypeRef;
use qubit_model_metadata::TypeShape;

let shape = TypeRef::of::<Option<Vec<String>>>().shape();
assert!(matches!(shape, TypeShape::Optional(_)));
```

`FieldMetadata::is_nullable()` checks only the outermost `Option`. Therefore
`Option<Vec<String>>` is nullable and `Vec<Option<String>>` is not. Arrays keep
their const length in `TypeShape::Array` and expose both sequence and array
capabilities; sequence `min_items` / `max_items` are rejected on arrays because
the length is already fixed by the type.

`TypeRef::strip_optional()` removes one outer `Option`. `named_metadata()` then
resolves a named struct after that optional layer, when a resolver is present.

### Attribute queries

`TypeMetadata` has typed getters for `primary_key`, `unique_constraints`,
`indexes`, `keys`, and `ownership`. Import `AttributeQuery` for generic
`attribute` and `attributes_of` over `AttributeKind`. Fields expose
`text_constraint`, `sequence_constraint`, `map_constraint`,
`temporal_constraint`, `decimal_constraint`, `element_metadata`, `reference`,
`lookup_relation`, `codec`, and `generator` where those attributes exist.

```rust
use qubit_model_metadata::AttributeKind;
use qubit_model_metadata::AttributeQuery;
use qubit_model_metadata::metadata_of;

let metadata = metadata_of::<Account>();
assert_eq!(metadata.attributes_of(AttributeKind::Unique).count(), 1);
assert!(matches!(
    metadata.attribute(AttributeKind::PrimaryKey),
    Some(_)
));
```

`AttributeMetadata` is non-exhaustive. Prefer the typed getters, or handle
future variants instead of relying on an exhaustive match.

### Field paths

`FieldPath` stores static segments. `resolve_field_path` walks resolvable named
struct metadata to a terminal field. One outer `Option` on an intermediate named
field is stripped; a non-struct intermediate value, a missing segment, or a
named type without a metadata resolver is a typed error.

Path resolution diagnoses one local traversal. It is not global graph
validation. After linking a complete model set, call
`ModelRegistry::validate_graph()`.

### Model registry

Linked model crates contribute `ModelRegistration` values to the distributed
`MODEL_REGISTRATIONS` slice. `ModelRegistry::try_global()` builds an immutable
index lazily. It never panics; `global()` panics if the linked set is invalid.

Construction checks:

- each registration ID and metadata ID against the `ModelId` protocol;
- that the two IDs match;
- that no two registrations share a stable ID or a `TypeIdentity`.

It does not allocate a metadata graph for ordinary queries and does not walk
relations. `get(id)` looks up metadata by stable ID; `resolve` implements
`MetadataResolver` for runtime identity lookup.

Tools that need a closed collection can call
`ModelRegistry::from_registrations` with an explicit slice instead of the
process-wide set.

### Manual construction

Public constructors are const-compatible, so advanced users can assemble static
metadata without the derive crate. They enforce local invariants: field order
and capability compatibility, non-empty key / unique / index field sets,
monotonic text and sequence ranges, and decimal scale no greater than
precision. Invalid input panics. Derive remains the safer default because the
declaration stays next to the type.

`TypeRef::opaque::<T>()` marks a type uninterpreted while keeping its Rust
name. `TypeRef::opaque_with_shape` is for producers that can see standard
container syntax but still leave the leaf opaque.

## Errors and Diagnostics

| API | Failure | What it means |
|---|---|---|
| `metadata_of::<T>()` | Does not compile | `T` does not implement `HasTypeMetadata`. |
| `TypeMetadata::field` | `None` | No declared field has that normalized name. |
| `TypeMetadata::resolve_field_path` | `FieldPathResolveError` | Empty path, missing segment, intermediate non-struct, or named metadata that cannot be resolved. |
| `ModelId::try_new` / `validate` | `ModelIdError` | Empty ID, empty segment, invalid module or type segment, or a Rust keyword as a module segment. |
| `ModelRegistry::from_registrations` / `try_global` | `ModelRegistryError` | Invalid IDs, registration/metadata ID mismatch, duplicate ID, or duplicate identity. |
| `ModelRegistry::global` | Panic | Same failures as `try_global`; use `try_global` when the caller must not abort. |
| `ModelRegistry::validate_graph` | `ModelGraphErrors` | One or more `ModelGraphError` values: missing targets, missing target fields, incompatible projections, invalid `same_as`, missing owners, required-reference cycles, or ownership cycles. |
| `TypeMetadata::new`, `FieldMetadata::new`, constraint / key constructors | Panic | Local invariant violated at construction time. |

`FieldPathResolveError` variants:

- `EmptyPath`
- `FieldNotFound { segment }`
- `IntermediateNotStruct { segment }`
- `NamedMetadataUnavailable { segment }`

`ModelGraphErrors::errors()` returns every independently discovered graph
problem in deterministic order. Registry construction deliberately skips this
walk so a partial linked set remains usable.

## Troubleshooting

| Symptom | Check |
|---|---|
| `metadata_of::<T>()` does not compile | Ensure `T` implements `HasTypeMetadata`, normally through `#[Model]` or `#[Enum]`. |
| Model rejects an external field type | Enable `chrono` or `big-decimal`, implement `HasTypeShape`, or use `#[field(opaque)]` when the leaf should stay uninterpreted. |
| A field is unexpectedly nullable | Inspect the outermost `TypeShape`. Only outer `Option<T>` is nullable. |
| Path resolution fails | Confirm every segment, that intermediate named types are structs with resolvers, and that the path is not empty. |
| A tool cannot find a model | Link and register the model crate, or build a `ModelRegistry` from an explicit registration set. |
| `global()` panics at startup | Call `try_global()` and inspect `ModelRegistryError` for invalid or duplicate IDs. |
| Relations look consistent locally but fail together | Call `validate_graph()` after every participating model crate is linked. |

## Limitations and Best Practices

- This crate stores and queries metadata. It does not map databases, execute
  codecs or generators, redact values, or produce validation error messages.
- `Opaque` means the leaf is intentionally uninterpreted. It is not a
  substitute for structure a consumer still needs.
- Prefer derive-generated metadata so IDs, fields, and attributes stay next to
  the type. Manual `const` construction is for tools and tests that cannot
  depend on the macros.
- Query with typed getters. `AttributeMetadata` can grow new variants.
- Use `ModelId` when an identifier must survive a process restart. Use
  `TypeIdentity` only inside the current binary.
- Do not treat `validate_graph()` as part of ordinary `metadata_of` queries.
  Run it when the linked model set is complete.

## Further Reading

- [Derive user guide](https://github.com/qubit-ltd/rs-model-derive/blob/main/doc/user_guide.md)
- [README](../README.md)
- [API documentation](https://docs.rs/qubit-model-metadata)
- [中文用户手册](user_guide.zh_CN.md)
