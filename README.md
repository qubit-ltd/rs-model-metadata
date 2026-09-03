# qubit-model-metadata

[![Rust CI](https://github.com/qubit-ltd/rs-model-metadata/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-model-metadata/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-model-metadata/coverage-badge.json)](https://qubit-ltd.github.io/rs-model-metadata/coverage/)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

`qubit-model-metadata` adds stable domain meaning to Rust types that have
already been described by `qubit-reflect`. It is for framework and application
authors who need generated model roles, field semantics, stable IDs, and an
explicit way to resolve relationships across linked model crates without
creating a second reflection system.

## Installation

The runtime crate supports Rust 1.94 and edition 2024. The Qubit model crates
are currently internal and unpublished (`publish = false`), so use checkout
paths and adjust them for your workspace layout:

```toml
[dependencies]
qubit-model-metadata = { version = "0.1", path = "../rs-model-metadata" }
qubit-model-derive = { version = "0.1", path = "../rs-model-derive" }
qubit-id = { version = "0.6", path = "../../rust-common/rs-id" }
```

`qubit-id` supplies the exact `Id` type required by `Entity` and `Projection`
identifiers. Applications that run `ModelResolver` also need direct
`qubit-validator` and `qubit-codec` dependencies; the complete setup is in the
user guide.

## Quick Start

An account service can describe an account once and inspect the resulting
metadata without starting a global registry. The derive macro supplies the
role-aware metadata, while `TypeMetadata` exposes it through the same
`TypeDescriptor` used by `qubit-reflect`.

```rust,ignore
use qubit_model_derive::Entity;
use qubit_id::Id;
use qubit_model_metadata::{ModelDescriptorExt, TypeDescriptor, TypeMetadata};

#[Entity(id = "example.User")]
struct User {
    #[identifier]
    id: Id,
    #[unique(ignore_case = true)]
    email: String,
}

let metadata = TypeMetadata::of::<User>();
assert_eq!(metadata.model_id().unwrap().as_str(), "example.User");
assert!(std::ptr::eq(metadata.descriptor(), TypeDescriptor::of::<User>()));
assert!(metadata.descriptor().model_metadata().is_some());
```

The result is static metadata for `User`; `TypeMetadata::of` does not initialize
the global model registry. Descriptor capability and property lookup use the
frozen `ReflectRegistry` snapshot so separately emitted model overlays remain
visible. See the user guide for the subsequent cross-crate resolution step.

## Why This Project Exists

Reflection answers structural questions such as a type's fields and their
Rust types. Domain models also need concepts such as identifiers, constraints,
references, properties, roles, and persistent model IDs. This crate keeps
those concerns attached to the reflection descriptor rather than duplicating
the reflection model.

## What It Provides

- `qubit-model-derive` generates metadata for `#[Entity]`, `#[Projection]`,
`#[Model]`, `#[Enum]`, `#[Value]`, and `#[ModelImpl]` declarations.
- `TypeMetadata` provides static role, field, property, generic-template, and
  optional `ModelId` metadata for generated types.
- `ModelRegistry` projects concrete models and their source provenance from the
  frozen `ReflectRegistry` snapshot; only generic model templates keep a
  model-owned registration fragment.
- `ModelResolver` performs an explicit resolution pass over model, validator,
  and codec registries.
- Resolution produces an immutable `ResolvedModelGraph`, or deterministic
  aggregated errors when relationships, roles, properties, validators, or
  codecs cannot be resolved.
- The resolved graph exposes references, projection sources and producers,
  executable validator and codec bindings, merged properties, and query
  metadata derived from indexed fields.

It does not replace `qubit-reflect`, and static metadata lookup does not
implicitly register models or resolve cross-model relationships. Generated
metadata is checked against descriptor, field, property, role, and codec
invariants before it crosses the hidden model ABI v3 boundary. Generated model
code consumes only `qubit-reflect::__private::codegen_v2`.

## Learn More

- [English user guide](doc/user_guide.md)
- [简体中文用户指南](doc/user_guide.zh_CN.md)
- [`qubit-model-derive` declaration guide](https://github.com/qubit-ltd/rs-model-derive/blob/main/doc/user_guide.md)
- Local API documentation: run `cargo doc --open`
- [中文版 README](README.zh_CN.md)

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
