# Qubit Model Metadata

[![Rust CI](https://github.com/qubit-ltd/rs-model-metadata/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-model-metadata/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-model-metadata/coverage-badge.json)](https://qubit-ltd.github.io/rs-model-metadata/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-model-metadata.svg?color=blue)](https://crates.io/crates/qubit-model-metadata)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

`qubit-model-metadata` provides immutable, strongly typed metadata for Rust domain models. It lets validation, schema-oriented tooling, and application code inspect model fields, type shapes, constraints, keys, and relations without a mutable runtime registry or string-based type inference.

## Installation

Add the runtime crate and, when metadata should be generated from model declarations, the companion derive crate:

```toml
[dependencies]
qubit-model-metadata = "0.1"
qubit-model-derive = "0.1.0"
```

Optional Cargo features add shape support for external scalar types:

| Feature | Supported types |
|---|---|
| `chrono` | `chrono::NaiveDate`, `NaiveTime`, `NaiveDateTime`, and `DateTime<Utc>` |
| `big-decimal` | `bigdecimal::BigDecimal` |

## Quick Start

For an account model, derive its static metadata once and query it through the runtime API:

```rust
use qubit_model_derive::ModelMetadata;
use qubit_model_metadata::{TypeShape, metadata_of};

#[derive(ModelMetadata)]
struct Account {
    #[model(identifier)]
    id: i64,
    #[model(text(min_chars = 3, max_chars = 320), unique(ignore_case))]
    email: String,
}

let metadata = metadata_of::<Account>();
let email = metadata.field("email").expect("email metadata");

assert!(metadata.primary_key().expect("primary key").contains("id"));
assert!(matches!(email.field_type().shape(), TypeShape::Scalar(_)));
assert_eq!(email.text_constraint().and_then(|text| text.max_chars()), Some(320));
```

The query reads static slices and function pointers. It does not allocate a metadata graph at runtime.

## Why This Project Exists

Domain-model consumers need more than Rust's memory representation: they need field structure and constraints with stable semantics. Separate registries drift from source declarations, while parsing `type_name` strings fails for aliases and renamed dependencies. This crate uses recursive traits for structure and runtime-local `TypeId` identity; type names remain diagnostic display data. `TypeId` is for in-process metadata lookup, not persistence or cross-process stable identifiers.

## What It Provides

- Recursive `TypeShape` metadata for supported scalars, `Option<T>`, `Vec<T>`, sets, maps, fixed arrays, named models, and explicitly opaque fields.
- Capability flags used by derive-time validation. Options and newtypes inherit inner capabilities; arrays expose both `SEQUENCE` and `ARRAY`, so uniqueness is expressible while their fixed length remains authoritative.
- Static model, field, enum, newtype, constraint, key, index, and relation value objects with typed getters.
- Allocation-free field, attribute, key, index, and nested field-path queries.
- Const-compatible public constructors that reject reversed ranges, decimal scales above precision, and empty key-like field sets.
- Optional `chrono` and `bigdecimal` scalar integrations behind explicit Cargo features.

## Known Limits

- The runtime crate does not automatically discover models or provide a global registry.
- It does not define database mappings, validation error messages, serialization formats, codecs, generators, or redaction implementations.
- Cross-model graph validation and relationship-cycle checks are outside this crate's local metadata API.
- User-defined types must implement `HasTypeShape`; the companion derive supports an explicit `#[model(opaque)]` escape hatch when structure is intentionally unavailable.

## Learn More

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
