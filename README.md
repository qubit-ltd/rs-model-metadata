# qubit-model-metadata

[![Rust CI](https://github.com/qubit-ltd/rs-model-metadata/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-model-metadata/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-model-metadata/coverage-badge.json)](https://qubit-ltd.github.io/rs-model-metadata/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-model-metadata.svg?color=blue)](https://crates.io/crates/qubit-model-metadata)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

`qubit-model-metadata` adds stable domain meaning to Rust types that have
already been described by `qubit-reflect`. It is for framework and application
authors who need generated model roles, field semantics, stable IDs, and an
explicit way to resolve relationships across linked model crates without
creating a second reflection system.

## Installation

The runtime crate supports Rust 1.94 and edition 2024. The metadata, model ID,
and derive packages are `publish = false`; use checkout paths from an application
crate beside `rs-model-metadata` in the platform workspace:

The examples use this checkout layout; run the application from `rs-platform/app`.
Keep direct dependencies on the same checkout paths used by the runtime:

```text
checkout/
  rs-platform/
    app/                 # Cargo.toml and src/main.rs
    rs-model-metadata/   # runtime, model-id/, and derive/
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
qubit-model-metadata = { version = "0.2", path = "../rs-model-metadata", default-features = false }
qubit-model-derive = { version = "0.2", path = "../rs-model-metadata/derive" }
qubit-id = "0.7"
qubit-reflect = { version = "0.2", path = "../rs-reflect" }
```

A library that only passes or stores stable model identities can depend directly
on the small `qubit-model-id` crate, without the full metadata runtime:

<!-- example: core/model-id -->
```toml
[dependencies]
qubit-model-id = { version = "0.1", path = "../rs-model-metadata/model-id" }
```

Use `qubit_model_id::{ModelId, ModelIdBuf, ModelIdError}` for that protocol.
Model declarations and registry access still require `qubit-model-metadata`.
Its `metadata` module re-exports these same ID types.
For a named, non-generic role declaration with an explicit `id`, the derive
macros also implement `qubit_model_id::HasModelId`; generic definitions and
anonymous models deliberately have no concrete model ID.

<!-- example: validation -->
```toml
[dependencies]
qubit-model-metadata = { version = "0.2", path = "../rs-model-metadata", features = ["validation"] }
qubit-model-derive = { version = "0.2", path = "../rs-model-metadata/derive" }
qubit-reflect = { version = "0.2", path = "../rs-reflect" }
qubit-validator = { version = "0.1.0", path = "../../rust-common/rs-validator" }
```

`qubit-id` supplies the exact `Id` type required by `Entity` and `Projection`
identifiers. Enable `codec` or `validation` for the corresponding execution
adapters, and `generic` for generic-definition metadata. The default feature
set is empty.

## Quick Start

An account service can describe an account once and inspect the resulting
metadata without maintaining a second model-registration pipeline. The derive
macro supplies the role-aware metadata, while `TypeMetadata` exposes it through
the same `TypeDescriptor` used by `qubit-reflect`.

<!-- example: core/quick-start -->
```rust
use qubit_model_derive::Entity;
use qubit_id::Id;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;

#[Entity(id = "example.User")]
struct User {
    #[identifier]
    id: Id,
    #[unique(ignore_case = true)]
    email: String,
}

fn main() {
    let metadata = TypeMetadata::of::<User>();
    assert_eq!(metadata.model_id().unwrap().as_str(), "example.User");
    let registry = ModelRegistry::try_global().expect("valid linked model graph");
    assert!(registry.metadata_for(metadata.descriptor()).unwrap().is_some());
}
```

The result is static metadata for `User`; `TypeMetadata::of` does not initialize
the global model registry. `ModelRegistry::from_static_metadata` builds an
isolated registry from explicit metadata and resolves properties only from
`TypeMetadata::local_properties()`. A registry built with
`ModelRegistry::from_reflect_registry` uses its frozen `ReflectRegistry` snapshot
for `metadata_for` and `properties_for`, so independently registered `ModelImpl`
providers remain visible. See the user guide for
the subsequent cross-crate resolution step. Explicit property queries return
owned views; their dynamic merge is released with the view, while a
`ModelRegistry` reuses its own cached merge only for that registry's lifetime.

## Why This Project Exists

Reflection answers structural questions such as a type's fields and their
Rust types. Domain models also need concepts such as identifiers, constraints,
references, properties, roles, and persistent model IDs. This crate keeps
those concerns attached to the reflection descriptor rather than duplicating
the reflection model.

## What It Provides

- `qubit-model-derive` generates metadata for `#[Entity]`, `#[Projection]`,
`#[Model]`, `#[Enum]`, `#[Value]`, and `#[ModelImpl]` declarations.
- `metadata` owns the execution-independent declaration vocabulary, including
  `TypeMetadata`, `ModelId`, codec references, validation arguments, and
  redaction sensitivity.
- `registry::ModelRegistry` projects concrete models from a frozen
  `ReflectRegistry` snapshot or builds an isolated registry from explicit static
  metadata. With `generic`, `generic_metadata_for` finds a definition by its
  process-local `TypeDefinitionId`, and `generic_definitions()` lists named and
  anonymous definitions. `entries()` contains only registrations with stable
  `ModelId`s, so anonymous definitions do not appear there.
- `resolve::StructureResolver` produces an immutable `resolve::ModelGraph`
  containing only structural relationships.
- With the `codec` feature, `codec::bind_codecs` binds declared codec
  occurrences after graph construction and reports deterministic errors.
- With the `validation` feature, `ValidationPlan::build` compiles declared
  property paths and binds them to the supplied `qubit-validator::ValidatorRegistry`.
  `ValidationPlan::validate` executes those immutable bindings and returns a
  structured `ValidationReport`. Build diagnostics expose
  `ConstraintRuleRef::Registry` for registry-backed rules and
  `ConstraintRuleRef::ModelIntrinsic` for rules executed by the model plan.
  Outer sequence uniqueness reserves the ID
  `qubit_validation_rules::ids::COLLECTION_UNIQUE`; registering that ID as a
  custom validator makes plan construction return `InvalidDeclaration`.
  The former `constraint_rule_ids()` method is replaced by `constraint_rules()`;
  each reference retains its stable ID through `.id().as_str()`.
  Borrowed optional model paths skip absent values and retain the complete
  property path; supported terminal `Option<String>` text rules skip `None` and
  validate `Some`. Prepared model-level rules can be appended in one ordered
  `with_model_rules` call.
- Structural, codec, and validation failures are returned independently by the
  layer that owns them; no resolver pass creates executable bindings.

It does not replace `qubit-reflect`, and static metadata lookup does not
implicitly register models or resolve cross-model relationships. Generated
metadata is checked against descriptor, field, property, and role invariants
before it crosses the hidden metadata-only ABI v7 boundary. Generated model
code uses only the curated module facade and its exact private ABI.

The global `ModelRegistry::try_global()` entry point represents the complete set
of linked registrations; a conflict makes initialization fail as a whole and
preserves the reflection error and capability conflict in its source chain. For
an isolated model view, build a `ReflectRegistry` snapshot from only the desired
descriptors, then pass that same snapshot to `ModelRegistry::from_reflect_registry`:

The following fragment uses the declared `qubit-reflect` dependency, an
application-defined `MyModel`, and an enclosing function returning `Result`.

```rust,ignore
let mut builder = qubit_reflect::registry::RegistrySnapshotBuilder::new();
builder.add_type(
    qubit_reflect::TypeDescriptor::of::<MyModel>(),
    qubit_reflect::identity::FragmentIdentity::new("example", "models", 1, 1, "type", 1),
);
let snapshot = builder.build()?;
let models = ModelRegistry::from_reflect_registry(&snapshot)?;
```

An explicit snapshot starts empty and does not automatically include every
registration linked into the process. Do not use the old hidden testing
registry helper.

Validation plans execute outer sequence counts and uniqueness, outer map entry
counts, and supported `BigDecimal` and chrono time constraints when the exact
getter and adapter shapes are available. An optional absent value is skipped;
unsupported selector traversal, enum payloads, missing collection adapters, or
incompatible value types fail plan construction. FailFast and report caps stop the
whole plan, and infrastructure errors preserve a partial report. See the user guide's
[support matrix](doc/user_guide.md#limitations-execution-support-and-explicit-refusal) and
[API migration](doc/user_guide.md#advanced-usage-field-identity-and-api-migration).

## Recoverable queries

`ModelRegistry::metadata_for` returns `Result<Option<&TypeMetadata>, ModelMetadataError>`.
When registry construction fails, `ModelRegistryError::origins()` preserves whether
the relevant capability was intrinsic or registered, including the exact fragment
identity for registered capabilities. `sources()` remains available for direct
fragment inspection.
`Ok(None)` means no matching model metadata; capability conflicts and descriptor ABI mismatches
return structured errors. `TypeMetadata::try_properties_in`, `try_property_in`, and
`property_fragments_in` propagate `PropertyResolutionError`. Explicit snapshot queries never
initialize the global registry. `ResolveError::cause()` retains the underlying failure with
model, property path, and provenance. Independent failures are aggregated; underlying failures
do not become missing-property diagnostics.

### Dynamic property paths

Compile untrusted or request-provided property segments with
`PropertyAccessPath::compile(&registry, root, segments)`, then call `read` or
`write` on instances. Reads traverse only borrowed properties and projectable
optional intermediates; an absent `Option` returns `MissingIntermediate`.
Writes require ordinary field-backed intermediate steps. Call
`check_writable()` before converting input, and use `leaf_property()` to inspect
the final property's declared type. See the user guide's [path and source-location
discussion](doc/user_guide.md#advanced-usage-paths-references-and-source-locations).

Every `qubit.model.metadata.v1` capability target must also be a member of the reflected snapshot;
`ModelRegistry::from_reflect_registry` reports `UnregisteredModelTarget` before invoking any model
provider when that membership is missing. `ModelEntry::source()` and `ModelRegistry::source()`
identify the metadata capability fragment for snapshot projections. `ModelEntry::declaration_source()`
returns the reflected type or generic definition fragment, and is `None` for static metadata entries.

Borrowed slices support direct indexed access. Explicit `into_invocation_output` materializes
per-element borrow wrappers in O(n) time without copying the underlying elements. The slice adapter
itself also requires boxing. Conversion and original access costs are measured separately.

## Declaration defaults and explicit roots

Role macros supply Clone, Debug, Display, PartialEq, Eq, Hash, Redact, Serialize,
and Deserialize by default. Use `no_*` options for intentional opt-outs; `no_eq`
also removes default Hash. `copy`, `default`, `partial_ord`, and `ord` are opt-in.
Named Option and standard collection fields default when missing and omit empty values.

Use `new` when an application must audit every model in a linked registry. Use
`for_roots` when a request or business workflow needs a plan for only its root
and reachable models. Both calls borrow the same `ModelRegistry` snapshot:
root-scoped resolution skips structural initialization of unrelated models,
while still reading reachable capabilities and `ModelImpl` property providers
from that snapshot. The local validation example requires the `validation`
feature and the direct `qubit-validator` dependency shown in the user guide.

<!-- example: validation/root-scoped -->
```rust
use std::sync::Arc;
use qubit_model_derive::Model;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_model_metadata::validation::ValidationBuildInputs;
use qubit_model_metadata::validation::ValidationOptions;
use qubit_model_metadata::validation::ValidationPlan;
use qubit_reflect::ReflectedRef;
use qubit_validator::ValidatorRegistry;

#[Model]
struct Request {
    #[text(non_blank)]
    input: String,
}

fn main() {
    let models = ModelRegistry::try_global().expect("linked registry");
    let _complete = StructureResolver::new(ResolveInputs { models: &models, roots: &[] })
        .resolve().expect("audit every linked model");

    let root = TypeMetadata::of::<Request>();
    let roots = [root];
    let local = Arc::new(StructureResolver::for_roots(ResolveInputs { models: &models, roots: &roots })
        .resolve().expect("resolve the request workflow"));
    assert!(local.model(root.type_id()).is_some());
    let validators = ValidatorRegistry::empty();
    let plan = ValidationPlan::build(root, ValidationBuildInputs {
        graph: Arc::clone(&local),
        validators: &validators,
    }).expect("build the request validation plan");
    let report = plan.validate(
        ReflectedRef::new(&Request { input: String::new() }),
        &ValidationOptions::default(),
    ).expect("validate the request");
    assert!(!report.is_valid());
}
```

With a reflection-backed registry, `for_roots` discovers reachable anonymous
models through the same snapshot. A metadata-only registry built with
`ModelRegistry::from_static_metadata` has no reflection discovery: include the
metadata for every reachable child explicitly. Use `TypeMetadata::try_of::<T>()`
when generated metadata ABI violations must be handled as an `AbiViolation`;
`TypeMetadata::of::<T>()` panics if that ABI validation fails. Invalid explicit
roots and registry entries are reported during resolution/registry construction.
Generic concrete models retain their definition even without a stable ID.
`QueryMetadata::declarations()` exposes direct indexed declarations, including implicit
identifier, unique, and reference reasons. Filter generation and matching policy belong
to consumers. `reference.path` uses `/` and `..`; Property paths retain `.`.

## Current execution capabilities and migration

| Execution declaration | Current contract |
| --- | --- |
| Outer Map entry count | Generated readable `HashMap`/`BTreeMap` length adapter; field path reports |
| Decimal / Money | Exact `BigDecimal`, including Option; scale, `DECIMAL(p,s)` precision and range; no rounding |
| Time precision | `DateTime<Utc>`, `NaiveDateTime`, `NaiveTime`: second/millisecond/microsecond/nanosecond; `NaiveDate` is rejected |
| Option | Borrowed `Option<T>` intermediates skip `None` and continue through `Some`; terminal `Option<String>` text rules skip `None` and validate `Some`; build still checks the concrete type |
| Selector constraints/dependencies; MapKey/MapValue; tuple paths; constrained enum payloads; container model interiors; owned intermediate getters requiring further traversal | `UnsupportedExecution`; outer support does not imply inner traversal |
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
- [简体中文用户指南](doc/user_guide.zh_CN.md)
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
API documentation and tests current, and run `./.infra/bin/align-ci.sh` to format code and
`./.infra/bin/ci-check.sh` to satisfy CI requirements before submitting a pull request.

## Author

**Haixing Hu** - *Qubit Co. Ltd.*

Repository: [https://github.com/qubit-ltd/rs-model-metadata](https://github.com/qubit-ltd/rs-model-metadata)
