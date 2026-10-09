# qubit-model-derive

[![Rust CI](https://github.com/qubit-ltd/rs-model-metadata/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-model-metadata/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-model-metadata/coverage-badge.json)](https://qubit-ltd.github.io/rs-model-metadata/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-model-derive.svg?color=blue)](https://crates.io/crates/qubit-model-derive)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

`qubit-model-derive` attaches business meaning to Rust domain types. Its role
macros and field attributes turn one type declaration into metadata that other
libraries can inspect: identifiers, uniqueness, constraints, references, and
query indexes. `qubit-model-metadata` reads and resolves that metadata alongside
the structural reflection supplied by `qubit-reflect`.

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

## Example: account and profile

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

#[Entity(id = "guide.person.PersonInfo")]
struct PersonInfo {
    #[identifier]
    id: Id,
    #[reference(entity = "guide.iam.User", property = id)]
    user_id: Option<Id>,
    #[text(non_blank, max_chars = 128)]
    name: String,
}

fn main() {
    let user_metadata = TypeMetadata::of::<User>();
    let username_field = user_metadata.field("username").unwrap();
    assert!(username_field.is_unique());
    assert_eq!(username_field.text_constraint().unwrap().max_chars(), Some(64));

    let person_metadata = TypeMetadata::of::<PersonInfo>();
    assert!(person_metadata.field("user_id").unwrap().reference().is_some());
}
```

The attributes keep business rules next to the fields they govern. A validation
consumer can check text limits and query the database for uniqueness; a constrained data
generator can select an existing user's ID for `PersonInfo`; a REST query layer
can use `indexed` to decide which filters to accept. The macro records the
declarations; consumers decide how to execute them.

## Declaration map

| Declaration | Meaning |
| --- | --- |
| `Entity`, `Projection`, `Model`, `Value`, `Enum` | Select the type's domain role and generate reflection plus metadata. |
| `identifier`, `key_part`, `unique`, `indexed`, `reference` | Describe identity, logical keys, uniqueness, queryability, and relations. |
| `text`, `number`, `collection`, `validator`, `selector` | Describe built-in constraints and custom rule bindings. |
| `codec`, `redact`, Serde options | Describe value conversion, output masking, and wire behavior. |
| `ModelImpl` | Expose eligible getters and setters as model properties. |

Invalid local declarations fail during macro expansion. Cross-model targets and
relationships are checked when `StructureResolver` builds a graph from a
`ModelRegistry`; executable rules are bound separately. See the [declaration
guide](doc/user_guide.md) for syntax and examples, and the [runtime guide](../doc/user_guide.md)
for consuming the metadata.

## Learn More

- [English user guide](doc/user_guide.md)
- [中文用户指南](doc/user_guide.zh_CN.md)
- Local API documentation: run `cargo doc --open`
- [Design](doc/design.md) · [设计文档](doc/design.zh_CN.md)
- [中文 README](README.zh_CN.md)

Run the commands below from the **repository workspace root** (`rs-model-metadata`),
not its `derive` subdirectory. The CI, alignment, and coverage scripts live there.

## Testing

```bash
# Run tests with the default feature set
cargo test --workspace --locked

# Run tests with all declared features
cargo test --workspace --all-features --locked

# Test only the derive package
cargo test -p qubit-model-derive --all-features --locked

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
API documentation and tests current, and run `./.infra/bin/align-ci.sh` to format code and
`./.infra/bin/ci-check.sh` to satisfy CI requirements before submitting a pull request.

## Author

**Haixing Hu** - *Qubit Co. Ltd.*

Repository: [https://github.com/qubit-ltd/rs-model-metadata/tree/main/derive](https://github.com/qubit-ltd/rs-model-metadata/tree/main/derive)
