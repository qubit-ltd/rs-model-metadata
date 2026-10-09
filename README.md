# qubit-model-metadata

[![Rust CI](https://github.com/qubit-ltd/rs-model-metadata/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-model-metadata/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-model-metadata/coverage-badge.json)](https://qubit-ltd.github.io/rs-model-metadata/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-model-metadata.svg?color=blue)](https://crates.io/crates/qubit-model-metadata)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

`qubit-model-metadata` makes business constraints on Rust domain objects readable by
programs. `qubit-reflect` describes the Rust structure; `qubit-model-derive` adds
meaning such as identifiers, uniqueness, text limits, references, and indexes;
this crate exposes and resolves those declarations. Validation, constrained test
data generation, and REST query layers can then use the same model definition.

## Install

<!-- example: core -->
```toml
[dependencies]
qubit-model-metadata = { version = "0.2", path = "../rs-model-metadata", default-features = false }
qubit-model-derive = { version = "0.2", path = "../rs-model-metadata/derive" }
qubit-id = "0.7"
qubit-reflect = { version = "0.2", path = "../rs-reflect" }
```

These checkout paths assume the application crate is `rs-platform/app`, beside
`rs-model-metadata` and `rs-reflect`. Run the complete example from that app
crate. The metadata, model-id, and derive manifests set `publish = false`;
the version fields identify package versions but do not select a source without
the corresponding checkout paths. See the runtime guide for the full checkout
layout and validation installation.

The default feature set is empty. Enable `validation` for the validation plan,
`codec` for codec binding, or `generic` for generic-definition metadata. A library
that only exchanges stable model IDs can depend on the standalone `qubit-model-id`
package:

<!-- example: core/model-id -->
```toml
[dependencies]
qubit-model-id = { version = "0.1", path = "../rs-model-metadata/model-id" }
```

## Example: describe a user account

<!-- example: core/quick-start -->
```rust
use qubit_id::Id;
use qubit_model_derive::Entity;
use qubit_model_metadata::metadata::TypeMetadata;

#[Entity(id = "guide.iam.User")]
struct User {
    #[identifier]
    id: Id,
    #[unique(ignore_case = true)]
    #[text(non_blank, min_chars = 3, max_chars = 64, allowed_chars = ascii)]
    username: String,
    #[indexed]
    #[text(max_chars = 128)]
    display_name: String,
}

fn main() {
    let user_metadata = TypeMetadata::of::<User>();
    let username_field = user_metadata.field("username").unwrap();
    assert!(username_field.is_unique());
    assert_eq!(username_field.text_constraint().unwrap().max_chars(), Some(64));
    assert!(user_metadata.field("display_name").unwrap().is_indexed());
}
```

The annotations are declarations, not checks on a `User` instance. A validation
consumer can check the length rule and query the database for uniqueness; a test-data generator can
use the same limits and uniqueness requirement; a query layer can admit
`display_name` as a filter because it is indexed. The application supplies the
database lookup, generation strategy, and query execution.

## What the runtime provides

| API | Use |
| --- | --- |
| `TypeMetadata` and `FieldMetadata` | Read roles, fields, constraints, and annotations without constructing a value. |
| `ModelRegistry` | Find models by stable `ModelId` or Rust type in one reflection snapshot. |
| `StructureResolver` and `ModelGraph` | Check and resolve references, projection sources, uniqueness scopes, and query declarations across models. |
| `PropertyAccessPath` | Compile a property path, then read or write a model instance through supported accessors. |
| Optional `validation` and `codec` features | Bind declarations to explicit rule or codec registries. |

`ModelId` is a stable external model identity. Rust `TypeId` and `FieldLocation`
identify structures within a process. `#[unique]` describes a uniqueness rule;
it does not itself query a database. `#[indexed]` marks a queryable field; the
runtime does not create a physical index or SQL statement.

The [runtime guide](doc/user_guide.md) covers registration, graph resolution,
field and property access, validation, and errors. The [derive guide](derive/doc/user_guide.md)
is the declaration reference.

## Learn More

- [English user guide](doc/user_guide.md)
- [简体中文用户指南](doc/user_guide.zh_CN.md)
- [Design](doc/design.md) · [设计文档](doc/design.zh_CN.md)
- [`qubit-model-derive` declaration guide](derive/doc/user_guide.md)
- [Coverage measurement and reproducible tooling checks](doc/coverage_measurement.md)
- The paired derive crate lives in this repository's [`derive/`](derive/) workspace member.
- Local API documentation: run `cargo doc --open`
- [中文版 README](README.zh_CN.md)

## Testing

```bash
# Run tests with the default feature set
cargo test --workspace --locked

# Run tests with all declared features
cargo test --workspace --all-features --locked

# Project CI checks
./.infra/bin/ci-check.sh

# Check code coverage
./.infra/bin/coverage.sh
```

## License

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for the
full license text.

## Contributing

Contributions are welcome. Please follow the Rust API guidelines, keep public
API documentation and tests current. Run `./.infra/bin/style-check.sh` to check the
project's Rust style, `./.infra/bin/align-ci.sh` to apply its pinned nightly and rustfmt
configuration, and `./.infra/bin/ci-check.sh` for the complete CI gate before submitting
a pull request. These rs-infra commands define the project gate; plain
`cargo fmt --all -- --check` uses the active toolchain and default rustfmt configuration,
so it is not an equivalent check. Cargo's workspace and path-dependency layout also
determines which source files each invocation reaches; use the paths reported by the
rs-infra gate to identify the files it checked or formatted.

## Author

**Haixing Hu** - *Qubit Co. Ltd.*

Repository: [https://github.com/qubit-ltd/rs-model-metadata](https://github.com/qubit-ltd/rs-model-metadata)
