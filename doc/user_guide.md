# Qubit Model Derive User Guide

[中文](user_guide.zh_CN.md) · [README](../README.md) · [API documentation](https://docs.rs/qubit-model-derive)

Applies to `qubit-model-derive` 0.1.0 and `qubit-model-metadata` 0.1.0.

`Model` is the supported attribute macro; legacy derive aliases are unavailable.

## Purpose and Audience

Use this crate when a Rust domain-model declaration must remain the source of
truth for metadata consumed by validation, schema-oriented tools, or application
code. `#[Model(...)]` produces the static implementations and registrations
consumed by `qubit-model-metadata`; it does not validate data.

## Conceptual Model

The attribute macro reads a supported declaration and `#[field(...)]` attributes at
compile time. It emits `HasTypeShape` and `HasTypeMetadata`; the runtime crate
exposes the resulting immutable metadata through typed queries.

```text
Rust model + #[field(...)]
            │ compile time
            ▼
Model derive ──► static runtime metadata ──► metadata_of::<T>()
```

## Scenario: Describe an Account

Install matching runtime and derive versions:

```toml
[dependencies]
qubit-model-derive = "0.1.0"
qubit-model-metadata = "0.1.0"
```

Declare metadata next to the model, then query its normalized result:

```rust
use qubit_model_derive::Model;
use qubit_model_metadata::{AttributeQuery, metadata_of};

#[Model(id = "example.Account")]
struct Account {
    #[field(identifier(generated))]
    id: i64,
    #[field(unique(ignore_case), text(min_chars = 3, max_chars = 320))]
    email: String,
}

let metadata = metadata_of::<Account>();
let email = metadata.field("email").expect("declared field");
assert!(metadata.primary_key().expect("primary key").contains("id"));
assert_eq!(email.text_constraint().and_then(|value| value.max_chars()), Some(320));
```

`identifier(generated)` becomes a model-level primary key, while field-level
`unique(ignore_case)` becomes a model-level unique constraint. Consumers query
the canonical metadata rather than macro-input spelling.

## Supported Declarations and Shapes

The macro accepts named-field structs, unit structs, single-field tuple
newtypes, and fieldless enums. Generic models, multi-field tuple structs,
unions, and enum variants carrying data are rejected.

Type structure comes from `HasTypeShape`, not parsed type-name strings. Supported
shapes include scalars, `Option<T>`, `Vec<T>`, `HashSet<T>`, `BTreeSet<T>`,
`HashMap<K, V>`, `BTreeMap<K, V>`, fixed arrays, and models deriving metadata.
`Option<Vec<String>>` and `Vec<Option<String>>` remain distinct; only outer
`Option` makes a field nullable.

Enable external scalar support where required:

```toml
[dependencies]
qubit-model-metadata = { version = "0.1.0", features = ["chrono", "big-decimal"] }
chrono = { version = "0.4", default-features = false, features = ["std"] }
bigdecimal = "0.4"
```

## Attribute Reference

Model-level attributes describe the whole model:

| Attribute | Meaning |
| --- | --- |
| `id = "example.Account"` | Required stable model ID; its final segment must match the Rust type name. |
| `primary_key(fields(id), generated(id))` | Ordered primary key; a generated field must belong to it. |
| `unique(name = "account", fields(org_id, username), ignore_case(username))` | Per-field uniqueness comparison. |
| `index(name = "created_at", fields(created_at))` | Ordered index fields. |
| `key(name = "account", fields(org_id, username))` | Logical key. |
| `ownership(owner = Organization)` | Owning model. |

Field-level attributes are:

| Attribute | Purpose |
| --- | --- |
| `identifier`, `unique`, `index` | Single-field key, uniqueness, and index shorthands. |
| `text(...)` | Character/byte ranges, `repertoire`, `non_blank`, and `format`. |
| `sequence(...)`, `map(...)` | Container size bounds and sequence `unique_items`. |
| `time(...)`, `decimal(...)`, `money(...)` | Temporal or decimal semantics and bounds. |
| `reference(...)`, `lookup_relation(...)` | Target model and target field relation metadata. |
| `sensitive(...)`, `codec`, `generator` | Handling and strategy-name metadata only. |
| `opaque` | Explicitly hides an external type's structure. |

`text` supports `min_chars`, `max_chars`, `min_bytes`, `max_bytes`,
`repertoire = unicode|ascii`, `non_blank`, and `format = email|uri|uuid`.
`sequence` accepts `min_items`, `max_items`, and `unique_items`; `map` accepts
`min_entries` and `max_entries`. `time` uses `precision =
second|millisecond|microsecond|nanosecond` and `normalization = preserve|utc`.
`decimal` and `money` accept `precision`, `scale`, and `rounding =
half_up|half_even|down|up`, and cannot be combined.

The macro rejects wrong scopes, duplicate or conflicting declarations, invalid
ranges, unavailable type capabilities, and invalid local field references.

## Relations and Opaque Fields

Relations name a stable target model ID and target field:

```rust
use qubit_model_derive::Model;

#[Model(id = "example.Organization")]
struct Organization { #[field(identifier)] id: i64 }

#[Model(id = "example.Membership")]
struct Membership {
    #[field(reference(target = "example.Organization", target_field = id, must_exist = true))]
    organization_id: i64,
}
```

The source crate does not need a Cargo dependency on `Organization`. Call
`ModelRegistry::validate_graph()` from a linked complete model set to validate
target existence, target-field compatibility, `same_as`, and required cycles.

For an external type that intentionally lacks structural metadata, use `opaque`:

```rust
use qubit_model_derive::Model;

struct ExternalToken;
#[Model(id = "example.ImportRecord")]
struct ImportRecord { #[field(opaque)] token: ExternalToken }
```

Otherwise an external field type must implement `HasTypeShape`. An opaque field
preserves visible standard container wrappers and represents its leaf as
`TypeShape::Opaque`; it cannot combine with shape-dependent
`text`, `sequence`, `map`, `time`, `decimal`, or `money` constraints.

## Errors and Diagnostics

Compile errors identify the invalid declaration. Check that the runtime crate is
present, the model form is supported, the attribute spelling and scope are
correct, field references exist, and decimal `scale` does not exceed `precision`.
For an unknown external type, implement `HasTypeShape` or deliberately choose
`opaque`. The runtime package may be renamed in Cargo; the macro resolves it by
package name.

## Limitations and Best Practices

Keep domain constraints beside the model and consume generated metadata through
the runtime crate. This macro does not provide table/column mapping, JSON
formats, validation messages, codec/generator execution, or global discovery.

## Further Reading

- [Runtime metadata user guide](../../rs-model-metadata/doc/user_guide.md)
- [README](../README.md)
- [README](../README.md)
- [中文用户手册](user_guide.zh_CN.md)
