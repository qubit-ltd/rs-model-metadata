# Qubit Model Metadata

[![Rust CI](https://github.com/qubit-ltd/rs-model-metadata/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-model-metadata/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-model-metadata/coverage-badge.json)](https://qubit-ltd.github.io/rs-model-metadata/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-model-metadata.svg?color=blue)](https://crates.io/crates/qubit-model-metadata)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

`qubit-model-metadata` gives validation, schema tooling, and application code a
typed, immutable view of a Rust domain model: fields, type shapes, constraints,
keys, and relations. Those facts usually come from `qubit-model-derive`, so
callers do not keep a second registry in sync with source, and they do not parse
`type_name` strings to recover structure.

## Installation

This crate is the runtime query API. To generate metadata from a model
declaration, add the companion derive crate and Serde. Both macros require
`serde` in the consuming crate. The crate requires Rust 1.94 or later.

```toml
[dependencies]
qubit-model-metadata = "0.1"
qubit-model-derive = "0.1"
serde = { version = "1", features = ["derive"] }
```

Optional Cargo features add shape support for external scalar types:

| Feature | Supported types |
|---|---|
| `chrono` | `chrono::NaiveDate`, `NaiveTime`, `NaiveDateTime`, and `DateTime<Utc>` |
| `big-decimal` | `bigdecimal::BigDecimal` |
| `id` | `qubit_id::Id` |

## Quick Start

A signup service stores accounts. Before it writes a row, a schema helper needs
the primary key, the email length limit, and whether email uniqueness ignores
case. Declare the model once; the query reads static slices and function
pointers and does not allocate a metadata graph.

```rust
use qubit_model_derive::Model;
use qubit_model_metadata::TypeShape;
use qubit_model_metadata::UniqueComparison;
use qubit_model_metadata::metadata_of;

#[Model(id = "example.Account")]
struct Account {
    #[field(identifier)]
    id: i64,
    #[field(text(min_chars = 3, max_chars = 320), unique(ignore_case))]
    email: String,
}

fn main() {
    let metadata = metadata_of::<Account>();
    let email = metadata.field("email").expect("email metadata");

    assert!(metadata.primary_key().expect("primary key").contains("id"));
    assert!(matches!(email.field_type().shape(), TypeShape::Scalar(_)));
    assert_eq!(email.text_constraint().and_then(|text| text.max_chars()), Some(320));
    assert_eq!(
        metadata
            .unique_constraints()
            .next()
            .and_then(|unique| unique.comparison_of("email")),
        Some(UniqueComparison::IgnoreCase)
    );
}
```

`Model` is an attribute macro, not `#[derive(Model)]`. It emits the runtime
traits and a process-local registration; `#[field(...)]` declares field
metadata. `unique(ignore_case)` becomes a model-level unique constraint.

## Why This Project Exists

Consumers of a domain model need more than Rust's memory layout. They need
field structure and constraints with stable semantics. A hand-maintained
registry drifts from the source declaration. Parsing `type_name` fails for
aliases and renamed dependencies.

This crate describes structure with recursive traits and identifies types with
the current process's `TypeId`. Type names stay diagnostic display data.
`TypeId` is for in-process lookup, not persistence or a cross-process stable
identifier. The portable identifier is `ModelId`.

## What It Provides

- Recursive `TypeShape` metadata for supported scalars, `Option<T>`, `Vec<T>`,
  `HashSet`/`BTreeSet`, `HashMap`/`BTreeMap`, fixed arrays, named models, and
  explicitly opaque fields.
- Capability flags used when attributes are validated. `Option` inherits the
  inner type's capabilities. Arrays expose both `SEQUENCE` and `ARRAY`, so
  element uniqueness is expressible while the const length remains
  authoritative.
- Static value objects for models, fields, enums, newtypes, constraints, keys,
  indexes, and relations, with typed getters.
- Allocation-free field, attribute, key, index, and nested field-path queries.
- Const-compatible public constructors that reject reversed ranges, a decimal
  scale greater than precision, and empty key-like field sets.
- An immutable `ModelRegistry` over registrations linked into the process, plus
  `validate_graph()` for cross-model references when the full set is present.
- Optional `chrono` and `bigdecimal` scalar integrations behind explicit Cargo
  features.

It does not map databases, emit validation messages, define serialization
formats, or run codecs, generators, or redaction.

## Known Limits

- The global registry contains only model crates linked into this process.
  Unlinked crates are absent by design. Tools that need a closed set can call
  `ModelRegistry::from_registrations`.
- Registry construction checks ID validity, registration/metadata ID agreement,
  and duplicate IDs or identities. It does not walk relations. Call
  `ModelRegistry::validate_graph()` after linking a complete model set.
- User-defined field types must implement `HasTypeShape`. The companion macro
  supports `#[field(opaque)]`, which keeps visible `Option`, sequence, set,
  array, and map wrappers and leaves only the leaf uninterpreted.
- `FieldMetadata::is_nullable()` inspects only an outer `Option`.
  `Option<Vec<String>>` is nullable; `Vec<Option<String>>` is not.

## Learn More

- [User guide](doc/user_guide.md)
- [中文用户手册](doc/user_guide.zh_CN.md)
- [API documentation](https://docs.rs/qubit-model-metadata)
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

Repository: [https://github.com/qubit-ltd/rs-model-metadata](https://github.com/qubit-ltd/rs-model-metadata)
