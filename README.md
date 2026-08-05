# Qubit Model Derive

[![Rust CI](https://github.com/qubit-ltd/rs-model-derive/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-model-derive/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-model-derive/coverage-badge.json)](https://qubit-ltd.github.io/rs-model-derive/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-model-derive.svg?color=blue)](https://crates.io/crates/qubit-model-derive)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

`qubit-model-derive` supplies `#[derive(Model)]` for Rust domain models. It turns a model declaration into the static, strongly typed metadata exposed by `qubit-model-metadata`, so validation, schema-oriented tooling, and application code can query model structure without a runtime registry or string-based type inspection.

## Installation

Use matching versions of the derive and runtime crates:

```toml
[dependencies]
qubit-model-derive = "0.1"
qubit-model-metadata = "0.1"
```

The runtime crate is required: an expansion without a `qubit-model-metadata` dependency emits a compile error explaining the missing dependency.

`ModelMetadata` remains available as a compatibility alias for existing users;
new code should use `Model`.

## Quick Start

For an account model, derive metadata once and query it wherever the application needs to inspect the model:

```rust
use qubit_model_derive::Model;
use qubit_model_metadata::metadata_of;

#[derive(Model)]
struct Account {
    #[model(identifier)]
    id: i64,
    #[model(unique(ignore_case), text(min_chars = 3, max_chars = 320))]
    email: String,
}

fn main() {
    let metadata = metadata_of::<Account>();
    assert!(metadata.primary_key().expect("primary key").contains("id"));
    assert!(!metadata.field("email").expect("email field").is_nullable());
}
```

The derive generates immutable metadata and the required runtime traits. The query observes the declared primary key and field metadata; it does not parse Rust type names at runtime.

## Why This Project Exists

Rust models often need more than their Rust type: validation, persistence, and schema tooling need field structure and domain constraints too. Keeping those facts in separate registries drifts from the model declaration, while inferring them from type-name strings breaks for aliases and renamed dependencies. This crate keeps the declaration as the source of truth and lets Rust resolve the actual types at compile time.

## What It Provides

- Derives static `HasTypeShape` and `HasTypeMetadata` implementations for named-field and unit structs, single-field tuple newtypes, and fieldless enums.
- Generates field, type, key, uniqueness, index, text, collection, temporal, decimal, reference, sensitivity, codec, and generator metadata from supported `#[model(...)]` attributes.
- Resolves the runtime package by Cargo package name. If `qubit-model-metadata` is renamed locally, the expansion uses that local dependency name, including when a same-named module would otherwise shadow it:

  ```toml
  [dependencies]
  model_runtime = { package = "qubit-model-metadata", version = "0.1.0" }
  ```

- Requires unknown external field types to opt in explicitly with `#[model(opaque)]`. An opaque field preserves its Rust type name and exposes `TypeShape::Opaque`, without requiring the external type to implement `HasTypeShape`.

For an opaque field, use the marker only when structural inspection is intentionally unavailable:

```rust
struct ExternalToken;

#[derive(Model)]
struct ImportRecord {
    #[model(opaque)]
    token: ExternalToken,
}
```

Without `opaque`, an external type must implement `HasTypeShape`; `opaque` cannot be combined with shape-dependent field constraints such as `text`, `sequence`, `map`, `time`, `decimal`, or `money`.

## Known Limits

- Multi-field tuple structs, data-carrying enum variants, unions, and generic models are rejected.
- The macro performs local declaration and capability validation. Phase D relationship work is out of scope: it does not provide an explicit model set or graph-wide validation of target fields, type compatibility, or relationship cycles.
- `same_as` currently accepts one local Rust field only. Nested relation paths remain part of the deferred model-set validation work.
- It does not define table/column mappings, PostgreSQL-specific types, JSON export formats, or codec/generator strategy implementations.

## Learn More

- [User guide](doc/user_guide.md)
- [Model metadata and derive design](doc/model-metadata-and-derive-design.md)
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
