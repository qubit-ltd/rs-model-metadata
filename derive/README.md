# qubit-model-derive

[![Rust CI](https://github.com/qubit-ltd/rs-model-metadata/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-model-metadata/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-model-metadata/coverage-badge.json)](https://qubit-ltd.github.io/rs-model-metadata/coverage/)
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

This crate targets Rust 1.94 and edition 2024. Its manifests use `publish = false`;
the examples below use the derive crate and runtime from local checkouts:

The examples use this checkout layout; run the application from `rs-platform/app`.
Keep direct dependencies on the same checkout paths used by the runtime:

```text
checkout/
  rs-platform/
    app/                 # Cargo.toml and src/main.rs
    rs-model-metadata/   # runtime and derive/
    rs-reflect/
  rust-common/
    rs-id/
    rs-validator/
    rs-validation-rules/
    rs-redact/
    rs-datatype/
```

The `core` installation below serves examples marked `core/...`. Programs marked
`validation/...` use the separate validation installation in the runtime guide. Copy each complete program
into `src/main.rs` and run `cargo run`. Fragments marked `rust,ignore` require
the surrounding API objects or application types described beside them.
`publish = false` describes these manifests; it does not establish whether any
crate version has been published. An offline resolution failure is only evidence
about the current local dependency cache.

<!-- example: core -->
```toml
[dependencies]
qubit-model-derive = { version = "0.2", path = "../rs-model-metadata/derive" }
qubit-model-metadata = { version = "0.2", path = "../rs-model-metadata" }
qubit-id = "0.7"
```

Generated code resolves `qubit-model-metadata` with `proc-macro-crate`; a
renamed runtime dependency is supported. Generated declarations alone do not need a direct dependency on `qubit-reflect`;
programs using its public APIs must declare the same checkout as the runtime.

## Quick Start

Consider a login service that needs a stable user identity, must avoid exposing
email addresses in its logs, and wants framework code to discover a writable
`email` property. Declare the model once:

<!-- example: core/quick-start -->
```rust
use qubit_id::Id;
use qubit_model_derive::{Entity, ModelImpl};
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;

#[Entity(id = "example.User")]
pub struct User {
    #[identifier]
    id: Id,
    #[unique(ignore_case = true)]
    #[redact(level = "high")]
    email: String,
}

#[ModelImpl]
impl User {
    pub fn email(&self) -> &str { &self.email }
    pub fn set_email(&mut self, value: String) { self.email = value; }
}

fn main() {
    let metadata = TypeMetadata::of::<User>();
    assert!(metadata.field("id").unwrap().is_identifier());
    assert!(metadata.try_property("email").unwrap().unwrap().is_writable());
    let registry = ModelRegistry::try_global().expect("valid linked model graph");
    assert!(registry.metadata_for(metadata.descriptor()).unwrap().is_some());
}
```

The role macro delegates Rust structure to `qubit-reflect`, then attaches one
typed `TypeMetadata` capability to that same descriptor. The role defaults supply
Rust behavior, with Debug, Display and Serialize delegated to rs-redact.

Generated model code uses the hidden metadata-only ABI v7 facade. Concrete models and
generic definitions are both discovered through the unified frozen reflection
snapshot; the model layer owns no separate inventory.

`#[key_part(order = n)]` describes the ordered fields that form the logical
key of a named `Model` or named `Value`. A key may use only some fields, but
the selected orders must be unique and contiguous from zero. It is not an
entity identifier and is therefore rejected on `Entity`, `Projection`,
`Enum`, and tuple/newtype values.

## Why This Project Exists

Model-aware frameworks need both Rust structure and domain rules. Maintaining
those facts in a separate schema makes field names, types, accessors, and
constraints drift apart. This crate compiles the model declaration itself and
reuses the single `qubit-reflect` descriptor, so metadata consumers observe
the same structure that Rust code uses.

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
- `#[ModelImpl]` merges public inherent getters and setters with fields
  into safe property metadata.

All roles default to Clone, Debug, Display, PartialEq, Redact, Serialize, and
Deserialize. Structural equality and hashing defaults depend on the role:

| Role | PartialEq | Eq | Hash |
| --- | --- | --- | --- |
| Entity | Yes | Opt-in | Opt-in |
| Projection | Yes | Opt-in | Opt-in |
| Model | Yes | Opt-in | Opt-in |
| Value | Yes | Yes | Yes |
| Enum | Yes | Yes | Yes |

Use `#[Entity(id = "example.Person", eq, hash)]` when structural Eq and Hash
are required. This is a source-breaking change: migrate only models whose
consumers require these traits. Every stored field participates; use a stable
identifier for business identity comparisons and mutable entity collection keys.
`eq` enables Eq; `hash` requires enabled Eq, including Eq supplied by `ord` or a
value role. `ord` enables Eq and ordering but does not enable Hash. `partial_ord`
only requires PartialEq. `eq, no_eq` and `hash, no_hash` are rejected.
`no_*` suppresses automatic capabilities; `no_eq` also removes default Hash.
An all-unit Enum also defaults to Copy. `copy`, `default`, `partial_ord`, and
`ord` are opt-in. Future traits remain opt-in until an explicit role option is added.
Place the role attribute before explicit derives so duplicate derives are visible.
Handwritten implementations require the corresponding opt-out.

`#[ModelImpl]` forwards reflection options and preserves ordinary methods and
trait impl reflection. Only public, safe, synchronous, nongeneric inherent accessors
contribute Properties. A getter without a stored field is automatically computed;
`#[model_property(skip)]` excludes only that method's Property contribution.
Generic runtime adapters use explicit reflection `specialize(...)` bindings.

## Boundaries

Direct metadata lookup through `TypeMetadata::of::<T>()` does not initialize
the global model registry. Descriptor capability and property lookup use the
frozen reflection snapshot. Use `registry::ModelRegistry` and
`resolve::StructureResolver` only after all participating crates are linked,
when resolving IDs, references, projection sources, and queries. With the metadata
runtime's `validation` feature, the downstream `ValidationPlan::build` receives
an explicit `qubit-validator::ValidatorRegistry` and owns validator binding and
execution.
Codec execution is a separate optional adapter: enable the runtime's `codec`
feature and call `codec::bind_codecs` with a `qubit-codec` registry.

Lower-case `#[validator(...)]` emits a syntax-checked occurrence. The
downstream validation plan binds its stable ID to a prepared `qubit-validator`
rule and resolves readable dependencies. Rust codec declarations retain only
their stable ID or Rust type identity; the codec adapter binds executable
descriptors and checks their exact value type.
For redacted map keys, serialization fails if distinct source keys redact to
the same output key instead of silently overwriting data.

The crate describes model semantics; it does not define physical database
indexes, execute validators, or turn Rust `type_name()` output into a stable
model identity. Those responsibilities remain with explicit downstream
consumers and the resolved model graph.

## Current execution capabilities and migration

| Execution declaration | Current contract |
| --- | --- |
| Outer Map entry count | Generated readable `HashMap`/`BTreeMap` length adapter; field path reports |
| Decimal / Money | Exact `BigDecimal`, including Option; scale, `DECIMAL(p,s)` precision and range; no rounding |
| Time precision | `DateTime<Utc>`, `NaiveDateTime`, `NaiveTime`: second/millisecond/microsecond/nanosecond; `NaiveDate` is rejected |
| Option | `None` skips inner constraints; `Some` executes; build still checks the concrete type |
| Selector constraints/dependencies; MapKey/MapValue; container model interiors | `UnsupportedExecution`; outer support does not imply inner traversal |
| Enum/raw wrappers and recursive paths | Reachable work is discovered; unsupported use paths fail both capabilities and plan construction; no-work wrappers can pass |

A reflection-backed registry discovers anonymous children reachable from the root,
even through raw reflection wrappers. Supply only that root in `ResolveInputs.roots`;
all discovery stays in the supplied snapshot. Metadata-only registries require
explicit child metadata and do not import reflection capabilities.
One-field tuple `Value` declarations obey the same value-closure checks as named
Values; `transparent` controls representation, not execution support.
Entity role checks also traverse tuple fields: `(InnerEntity,)` without an
explicit reference is rejected as `InvalidEntityNesting`. Newtype Values cannot
hide a Model/Entity/Projection, reference, unresolved descriptor, or raw struct
in their value closure (`InvalidValueClosure`); primitive Value/Enum closures remain legal.
Unnamed payload fields do not become named Properties. `ModelImpl` providers,
signatures and accessor adapters share the
method/impl `cfg` and nested `cfg_attr` presence conditions. Mutually exclusive
accessors are supported; enabled conflicting pairs are still diagnosed.

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
