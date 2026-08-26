# Qubit Model Derive

[![Rust CI](https://github.com/qubit-ltd/rs-model-derive/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-model-derive/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-model-derive/coverage-badge.json)](https://qubit-ltd.github.io/rs-model-derive/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-model-derive.svg?color=blue)](https://crates.io/crates/qubit-model-derive)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

`qubit-model-derive` turns a Rust domain declaration into static metadata and
the everyday traits that surround it. Validation, persistence, and schema tools
need field structure, constraints, and a stable identity; keeping those facts in
a second registry drifts from the code. This crate provides two attribute
macros, `#[Model]` and `#[Enum]`. The declaration stays the source of truth, and
`qubit-model-metadata` exposes the generated implementations at runtime.

## Installation

Use matching versions of the derive crate, the runtime crate, and Serde. Both
macros require `serde` in the consuming crate:

```toml
[dependencies]
qubit-model-derive = "0.1"
qubit-model-metadata = "0.1"
serde = { version = "1", features = ["derive"] }
```

An expansion without `qubit-model-metadata` or `serde` emits a compile error
that names the missing dependency. Add `qubit-redact` only when a model or enum
participates in redaction.

These are attribute macros, written `#[Model(...)]` and `#[Enum(...)]`. There is
no `#[derive(Model)]` alias.

## Quick Start

An account record and its lifecycle status are two different shapes: a struct
and an enum. Declare each with the matching macro, attach
field constraints as standalone attributes such as `#[identifier]` and
`#[text(...)]`, then query the generated metadata:

```rust
use qubit_model_derive::Enum;
use qubit_model_derive::Model;
use qubit_model_metadata::AttributeQuery;
use qubit_model_metadata::TypeKind;
use qubit_model_metadata::metadata_of;

#[Enum(id = "example.AccountStatus")]
enum AccountStatus {
    Active,
    Suspended,
}

#[Model(id = "example.Account")]
struct Account {
    #[identifier]
    id: i64,
    #[unique(ignore_case)]
    #[text(min_chars = 3, max_chars = 320)]
    email: String,
    status: AccountStatus,
}

fn main() {
    let account = metadata_of::<Account>();
    assert!(account.primary_key().expect("primary key").contains("id"));
    assert!(!account.field("email").expect("email field").is_nullable());

    let status = AccountStatus::Suspended;
    assert_eq!(status.name(), "SUSPENDED");
    assert_eq!(AccountStatus::from_name("ACTIVE"), Some(AccountStatus::Active));
    assert!(matches!(metadata_of::<AccountStatus>().kind(), TypeKind::Enum(_)));
}
```

`Account` contributes a struct registration with a primary key and a unique
email constraint. `AccountStatus` contributes an enum registration plus
canonical names used by `Display`, Serde, `name`, and `from_name`. Neither query
parses Rust type names at runtime.

## Why This Project Exists

A domain type is not only a Rust layout. Downstream tools also need keys,
uniqueness, text and numeric bounds, and a stable ID that survives crate
renames. Inferring those facts from type-name strings breaks for aliases and
renamed dependencies. This crate keeps the facts beside the declaration and lets
Rust resolve the actual types at compile time.

## What It Provides

Both macros require a stable `id = "module.Type"`. The final segment must match
the Rust type name. Each expansion implements `HasTypeShape`,
`HasTypeMetadata`, and `HasModelRegistration`, and registers one entry in the
immutable global `ModelRegistry`.

The runtime crate is resolved by Cargo package name. A local rename still works:

```toml
[dependencies]
model_runtime = { package = "qubit-model-metadata", version = "0.1.0" }
```

### `#[Model]`

`#[Model]` accepts named-field structs, unit structs, and single-field tuple
newtypes. Applying it to an enum is a compile error; use `#[Enum]` instead.

Model-level keys such as `primary_key`, `index`, `key`, and `ownership` belong
in the `#[Model(...)]` argument list. Field constraints are standalone field
attributes such as `#[identifier]`, `#[unique(...)]`, `#[text(...)]`, and
`#[reference(...)]`. The removed `#[field(...)]` wrapper is rejected with a
compile error.

For a struct it generates:

- Default traits: `Clone`, `Debug`, `Eq`, `PartialEq`, `Hash`, `Serialize`, and
  `Deserialize`
- A `Display` implementation with Debug-shaped output
- `#[serde(rename_all = "snake_case")]`
- Serde omission defaults: `Option<T>` values set to `None` and empty direct
  standard collections are omitted; collection fields also receive
  `#[serde(default)]` for a missing input field
- Static `TypeKind::Struct` or `TypeKind::Newtype` metadata
- Field, key, uniqueness, index, text, collection, temporal, decimal,
  reference, codec, and generator metadata from standalone field attributes and
  model-level arguments

`no_copy` is rejected on structs. `#[Model(..., redact)]` or any field
`#[redact(...)]` delegates formatting and serialization to `qubit-redact`.

Unknown external field types must opt in with `#[opaque]`. An opaque field keeps
visible `Option`, sequence, set, array, and map wrappers and exposes its leaf as
`TypeShape::Opaque`. Without `opaque`, the field type must implement
`HasTypeShape`. `opaque` cannot combine with shape-dependent constraints such as
`text`, `sequence`, `map`, `time`, `decimal`, or `money`.

The collection omission rule recognizes directly declared `Vec`, `LinkedList`,
`VecDeque`, `HashMap`, `BTreeMap`, `HashSet`, `BTreeSet`, `BinaryHeap`, and
fixed arrays. Type aliases are not recognized. Use `#[keep_serializing]` on an
`Option` or supported collection field to opt out of the macro's automatic
omission and defaulting, preserving `null` or an empty value during
serialization.

Field-level `#[unique(...)]` shorthands normalize into model-level unique
constraints. Composite uniqueness uses `respectTo = [other_fields]` on the
annotated field.

### `#[Enum]`

`#[Enum]` accepts unit, tuple, struct, and mixed enums. Applying it to a struct
is a compile error. Generic enums remain unsupported.

For an enum it generates:

- Default traits: `Clone`, `Debug`, `Eq`, `PartialEq`, `PartialOrd`, `Ord`,
  `Hash`, `Serialize`, and `Deserialize`; a fully unit enum also receives
  `Copy`
- `#[must_use]` unless the declaration already has one
- `#[serde(rename_all = "SCREAMING_SNAKE_CASE")]`
- A `Display` implementation that writes the canonical serialized name and
  Debug-shaped tuple or struct payloads
- `name(&self) -> &'static str`; fully unit enums also receive
  `from_name(&str) -> Option<Self>`
- Static `TypeKind::Enum` metadata whose variants expose `Unit`, `Tuple`, or
  `Struct` shape and reuse `FieldMetadata` for payload fields

`#[serde(rename = "...")]` or `#[serde(rename(serialize = "..."))]` on a variant
overrides that canonical name for metadata, `Display`, and `name`, as well as
`from_name` when it exists. Duplicate serialized names are rejected.

Payload fields support local constraints such as `text`, `sequence`, `map`,
`time`, decimal constraints, element constraints, strategies, `opaque`, and
redaction. Record-level helpers (`identifier`, `unique`, `indexed`, `reference`,
and `lookup_relation`) and model-level keys are rejected because enum variants
do not share one record-wide field set. Tuple payload metadata uses names
`"0"`, `"1"`, and so on. `no_copy` remains valid on all enums.

### What it does not provide

The macros do not validate instance data, map tables or columns, define
PostgreSQL-specific types, export JSON schemas, or run codec/generator
strategies. Cross-model checks such as target existence, projection
compatibility, and ownership cycles belong to
`ModelRegistry::validate_graph()` on a linked model set.

## Known Limits

- Generic models, multi-field tuple structs, and unions are rejected.
- Model-level constraints such as `primary_key`, `index`, `key`, and
  `ownership` apply only to named structs.
- `reference(entity = "module.Type", ...)` names a stable target ID and does not
  require a Cargo dependency on that target.

## Learn More

- [User guide](doc/user_guide.md)
- [Runtime metadata user guide](../rs-model-metadata/doc/user_guide.md)
- [Redaction runtime guide](https://github.com/qubit-ltd/rs-redact/blob/main/doc/user_guide.md)
- [API documentation](https://docs.rs/qubit-model-derive)
- [中文文档](README.zh_CN.md)

## Testing

```bash
# Run tests with the default feature set
cargo test

# Run tests with all declared features
cargo test --all-features

# Project CI checks
./ci-check.sh

# Check code coverage
./coverage.sh
```

## License

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for the
full license text.

## Contributing

Contributions are welcome. Please follow the Rust API guidelines, keep public
API documentation and tests current, and run `./align-ci.sh` to format code and
`./ci-check.sh` to satisfy CI requirements before submitting a pull request.

## Author

**Haixing Hu** - *Qubit Co. Ltd.*

Repository: [https://github.com/qubit-ltd/rs-model-derive](https://github.com/qubit-ltd/rs-model-derive)
