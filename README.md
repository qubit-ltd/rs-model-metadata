# qubit-model-derive

[![Rust CI](https://github.com/qubit-ltd/rs-model-derive/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-model-derive/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-model-derive/coverage-badge.json)](https://qubit-ltd.github.io/rs-model-derive/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-model-derive.svg?color=blue)](https://crates.io/crates/qubit-model-derive)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

`qubit-model-derive` turns Rust domain declarations into Qubit model metadata.
It is for application and framework authors who need one type to have both
ordinary Rust structure reflection and model-specific semantics—identity,
constraints, relations, redaction, serialization policy, and safe properties—
without maintaining a parallel schema by hand.

## Installation

This crate targets Rust 1.94 and edition 2024. Applications normally depend on
the derive crate and the `qubit-model-metadata` runtime facade:

```toml
[dependencies]
qubit-model-derive = "0.1"
qubit-model-metadata = "0.1"
```

Generated code resolves `qubit-model-metadata` with `proc-macro-crate`; a
renamed runtime dependency is supported. Application crates do not need a
direct dependency on `qubit-reflect`.

## Quick Start

Consider a login service that needs a stable user identity, must avoid exposing
email addresses in its logs, and wants framework code to discover a writable
`email` property. Declare the model once:

```rust,ignore
use qubit_model_derive::{Entity, ModelProperties};
use qubit_model_metadata::{ModelDescriptorExt, TypeMetadata};

#[Entity(id = "example.User")]
pub struct User {
    #[identifier]
    id: u64,
    #[unique(ignore_case = true)]
    #[redact(level = "medium")]
    email: String,
}

#[ModelProperties]
impl User {
    pub fn email(&self) -> &str { &self.email }
    pub fn set_email(&mut self, value: String) { self.email = value; }
}

let metadata = TypeMetadata::of::<User>();
assert!(metadata.field("id").unwrap().is_identifier());
assert!(metadata.property("email").unwrap().is_writable());
assert!(metadata.descriptor().model_metadata().is_some());
```

The role macro delegates Rust structure to `qubit-reflect`, then attaches one
typed `TypeMetadata` capability to that same descriptor. The generated
`Debug`, `Display`, and `Serialize` implementations use the redaction policy,
so the email is not emitted as ordinary plain-text output.

## What It Provides

Six attribute macros share one parse, normalize, validate, and expand
pipeline:

- `#[Entity]` declares a persistent, identity-bearing model.
- `#[Projection]` declares an open or fixed view of an entity.
- `#[Model]` declares ordinary structured data.
- `#[Enum]` declares a domain enum and preserves Rust, canonical, and Serde
  names.
- `#[Value]` declares a value object; `transparent` supports a one-field
  wrapper.
- `#[ModelProperties]` merges public inherent getters and setters with fields
  into safe property metadata.

The five role macros supply `Clone`, redaction-aware `Debug`, `Display`, and
`Serialize`, plus `Deserialize`, `PartialEq`, `Eq`, `Hash`, and `Redact` by
default. Individual `no_*` options disable those interfaces; `copy`,
`default`, `partial_ord`, and `ord` are opt-in. An all-unit enum is `Copy`
unless it specifies `no_copy`.

Role attributes must appear before any user `#[derive(...)]`. This lets the
macro detect implementations that would duplicate or bypass redacted output.

## Boundaries

Static metadata lookup through `TypeMetadata::of::<T>()` and
`ModelDescriptorExt::model_metadata()` does not initialize a global registry.
Use `ModelRegistry` and `ModelResolver` only after all participating model
crates are linked, when resolving IDs, references, projection sources, or
queries.

Lower-case `#[validator(...)]` records validated declaration metadata only; it
does not register, resolve, or execute validators. A Rust codec must satisfy
the `qubit-codec` `Default`, `ValueEncoder`, and `ValueDecoder` contracts.
For redacted map keys, serialization fails if distinct source keys redact to
the same output key instead of silently overwriting data.

## Learn More

- [English user guide](doc/user_guide.md)
- [中文用户指南](doc/user_guide.zh_CN.md)
- [API documentation](https://docs.rs/qubit-model-derive)
- [Final API and implementation design](doc/2026-08-31-182016-rs-model-derive-final-api-and-implementation-design.md)
- [中文 README](README.zh_CN.md)

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
