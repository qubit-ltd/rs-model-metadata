# Qubit Model Metadata User Guide

[中文](user_guide.zh_CN.md) · [README](../README.md) · [API documentation](https://docs.rs/qubit-model-metadata)

Applies to `qubit-model-metadata` 0.1.0.

## Purpose and Audience

`qubit-model-metadata` is the runtime representation and query API for static
Rust domain-model metadata. The companion derive crate usually produces the
metadata; this guide explains how applications and tools consume it rather than
the full derive-attribute language.

## Conceptual Model

`HasTypeShape` describes a recursive structural shape. Named models also
implement `HasTypeMetadata`, exposing `TypeMetadata`. `metadata_of` retrieves a
`&'static` value built from static slices and function pointers.

```text
HasTypeShape ──► TypeRef ──► TypeShape
       │
HasTypeMetadata ──► TypeMetadata ──► FieldMetadata + model attributes
```

`TypeIdentity` compares types with `TypeId` in the current process. Type names
are diagnostic display information; do not persist a `TypeId` or treat it as a
cross-process stable identifier.

## Scenario: Inspect an Account

Install the runtime and derive crates:

```toml
[dependencies]
qubit-model-metadata = "0.1.0"
qubit-model-derive = "0.1.0"
```

Query generated metadata with its typed API:

```rust
use qubit_model_derive::Model;
use qubit_model_metadata::{AttributeQuery, TypeShape, metadata_of};

#[derive(Model)]
struct Account {
    #[model(identifier)]
    id: i64,
    #[model(text(max_chars = 320), unique(ignore_case))]
    email: String,
    tags: Option<Vec<String>>,
}

let metadata = metadata_of::<Account>();
let email = metadata.field("email").expect("declared field");
assert!(metadata.primary_key().expect("primary key").contains("id"));
assert_eq!(email.text_constraint().and_then(|value| value.max_chars()), Some(320));
assert!(matches!(
    metadata.field("tags").expect("tags field").field_type().shape(),
    TypeShape::Optional(_)
));
```

`field` returns `Option` because a queried name might not be declared. Make
absence explicit in application code instead of assuming it cannot occur.

## Type Shapes and Nullability

`TypeRef` is a small copyable handle. `shape()` returns a recursive `TypeShape`:
scalar, named model, optional value, sequence, set, map, fixed array, or
`Opaque`. The outer shape controls capability and nullability.

```rust
use qubit_model_metadata::{TypeRef, TypeShape};

let shape = TypeRef::of::<Option<Vec<String>>>().shape();
assert!(matches!(shape, TypeShape::Optional(_)));
```

`FieldMetadata::is_nullable()` checks only outer `Option`. Therefore
`Option<Vec<String>>` is nullable but `Vec<Option<String>>` is not. Arrays
retain their const length in `TypeShape::Array` and expose sequence and array
capabilities. Enable `chrono` for `NaiveDate`, `NaiveTime`, `NaiveDateTime`, and
`DateTime<Utc>`; enable `big-decimal` for `BigDecimal`.

## Querying Attributes and Paths

Import `AttributeQuery` for typed convenience methods. Model queries include
`primary_key`, `unique_constraints`, `indexes`, `attributes_of`, and `attribute`.
Fields provide `text_constraint`, `reference`, `lookup_relation`, `sensitive`,
`codec`, and `generator` where applicable.

```rust
use qubit_model_metadata::{AttributeKind, AttributeQuery, metadata_of};

let metadata = metadata_of::<Account>();
assert!(metadata.indexes().all(|index| !index.fields().is_empty()));
assert_eq!(metadata.attributes_of(AttributeKind::Index).count(), 0);
```

`AttributeMetadata` is non-exhaustive. Prefer the typed getters, or handle
future enum variants safely rather than relying on an exhaustive match.

`FieldPath` stores static segments. `resolve_field_path` follows resolvable named
struct metadata to a terminal field:

```rust
use qubit_model_metadata::{FieldPath, metadata_of};

let path = FieldPath::new(&["contact", "email"]);
let result = metadata_of::<Account>().resolve_field_path(path);
```

The result can report a missing segment, a non-struct intermediate value, or a
named type whose metadata cannot be resolved. Treat these as integration or
configuration diagnostics, not as global graph validation.

## Construction and Boundaries

Public constructors are const-compatible, so advanced users can construct static
metadata manually. They validate local invariants including field order,
non-empty key sets and paths, monotonic ranges, and decimal scale no greater than
precision. Derive is normally the safer option because it keeps declarations by
their model.

The crate builds an immutable global `ModelRegistry` lazily from distributed
registrations linked into the process. Only linked model crates participate;
constructing a registry from an explicit registration set remains available for
tools that need a controlled model collection. Registry construction checks
registration consistency and duplicate IDs, but it does not allocate a metadata
graph for queries or validate relationship cycles. The crate does not map
databases, execute codecs/generators/redaction, or produce validation messages.
`Opaque` means that a type is intentionally uninterpreted; it is not a
replacement for required structure.

## Troubleshooting

| Symptom | Check |
| --- | --- |
| `metadata_of::<T>()` does not compile | Ensure `T` implements `HasTypeMetadata`, normally through `Model`. |
| Derive rejects an external field type | Enable a needed feature, implement `HasTypeShape`, or intentionally use `#[model(opaque)]`. |
| A field is unexpectedly nullable | Inspect its outer `TypeShape`; only outer `Option<T>` is nullable. |
| Path resolution fails | Verify every segment, intermediate named structs, and their metadata resolvers. |
| A tool cannot find a model | Ensure the model crate is linked and registered, or construct a `ModelRegistry` from the tool's explicit registration collection. |

## Further Reading

- [Derive user guide](../../rs-model-derive/doc/user_guide.md)
- [Model metadata and derive design](../../doc/model-metadata-and-derive-design.md)
- [README](../README.md)
- [中文用户手册](user_guide.zh_CN.md)
