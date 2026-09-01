# qubit-model-metadata User Guide

[简体中文](user_guide.zh_CN.md) | [README](../README.md) | [API documentation](https://docs.rs/qubit-model-metadata)

Applies to `qubit-model-metadata` 0.1.x, Rust 1.94, and edition 2024.

## Purpose and Audience

This guide is for framework and application developers who declare domain
models with `qubit-model-derive` and need to inspect their metadata or resolve
relationships after all model crates have been linked. It explains the boundary
between structural reflection and domain semantics, then follows an account
model from declaration to an immutable resolved graph.

## Conceptual Model

`qubit-reflect` owns the Rust structure. `qubit-model-metadata` attaches domain
meaning to that structure, and `qubit-model-derive` generates both views from
one declaration.

```text
Rust declaration -> TypeDescriptor -> TypeMetadata -> ModelRegistry -> ResolvedModelGraph
                         |                  |
                    FieldDescriptor    Field/Property semantics
```

A `TypeRef` can be `Resolved`, `Opaque`, or `Symbolic`. Consequently,
`FieldMetadata::descriptor()` and `PropertyMetadata::descriptor()` return an
`Option`: a missing concrete descriptor can be an intentional representation
of an opaque or symbolic type, not missing metadata.

## Scenario

An account service needs to validate its own model declaration immediately and
resolve a login model's reference after the complete application has linked all
of its model crates. Success means the account metadata is available without a
global registry, and the completed registry resolves `Login.account_id` to the
`Account` entity's `id` property.

## Installation and Minimal Configuration

Add the runtime and macro crates to the application:

```toml
[dependencies]
qubit-model-metadata = "0.1"
qubit-model-derive = "0.1"
```

Use a model-role derive macro for every type passed to `TypeMetadata::of`.
The macro generates the required metadata provider and bounds; a type derived
only with structural reflection is not enough.

## Core Workflow

Declare an entity and a model that refers to it. The `#[reference]` declaration
uses the stable entity ID and the public property name to be resolved later.

```rust,ignore
use qubit_model_derive::{Entity, Model};

#[Entity(id = "example.Account")]
pub struct Account {
    #[identifier]
    pub id: u64,
    #[unique(ignore_case = true)]
    pub email: String,
}

#[Model(id = "example.Login")]
pub struct Login {
    #[reference(entity_id = "example.Account", property = id)]
    pub account_id: u64,
}
```

Inspect the account locally. This is a static query; it does not initialize the
global model registry:

```rust,ignore
use qubit_model_metadata::TypeMetadata;

let account = TypeMetadata::of::<Account>();
assert!(account.field("id").unwrap().is_identifier());
assert!(account.try_property("email").unwrap().unwrap().is_readable());
```

Once every model crate is linked, obtain the three explicit registries and run
one resolution pass:

```rust,ignore
use qubit_codec::ValueCodecRegistry;
use qubit_model_metadata::{ModelRegistry, ModelResolver, ResolveInputs, TypeMetadata};
use qubit_validator::ValidatorRegistry;

let models = ModelRegistry::try_global()?;
let validators = ValidatorRegistry::try_global()?;
let codecs = ValueCodecRegistry::try_global()?;
let graph = ModelResolver::new(ResolveInputs { models, validators, codecs }).resolve_all()?;
let field = TypeMetadata::of::<Login>().field("account_id").unwrap();
assert_eq!(
    graph.reference(field).unwrap().target().model_id().unwrap().as_str(),
    "example.Account",
);
# Ok::<(), Box<dyn std::error::Error>>(())
```

The resulting `ResolvedModelGraph` is immutable. A successful resolver pass
publishes a complete graph rather than an incrementally resolved partial view.

## Advanced Usage

Fields are physical storage slots supplied by reflection. A `Property` combines
a field and optional getter and setter under one public name. Getter adapters
preserve borrowed lifetimes; when a setter fails before execution, its
`PropertySetFailure` retains the replacement value for the caller.

For generic declarations with a model ID, registration stores one
`GenericModelMetadata` template. Concrete generic instances have no `ModelId`
and are not registered; `generic_definition()` points back to the template.
Template field types may be symbolic while a concrete instance's field types
may be resolved.

## Errors and Diagnostics

`ModelRegistry::try_global()` reports duplicate model IDs or failure to build
the reflection registry as `ModelRegistryError`. `resolve_all()` returns
`ModelResolveErrors`, which aggregates deterministic errors instead of
publishing a graph with unresolved relationships. Its entries cover missing
model IDs, incorrect roles, absent or unreadable properties, invalid projection
sources, and validator or codec registrations that are missing or have an
incompatible value type.

Generated metadata also checks local ABI invariants before publication. A panic
whose message starts with `QMM-ABI-` indicates that generated or manually
supplied hidden-ABI metadata was rejected because it violated one of those
invariants.

## Troubleshooting

- If `TypeMetadata::of::<T>()` does not compile, confirm that `T` uses a
  model-role macro and satisfies the generated trait bounds.
- If `descriptor()` returns `None`, inspect `type_ref()` before treating it as
  an error; opaque and symbolic references deliberately have no concrete
  descriptor.
- If `resolve_all()` returns errors, inspect each `ModelResolveError` and fix
  the stable ID, target role, property name, or matching validator/codec
  registration before retrying the full resolution pass.
- A resolved validator exposes its typed registration and readable dependency
  properties. A resolved codec exposes its executable descriptor and, for an
  ID declaration, the matching registry entry.

## Limitations and Best Practices

Use `ModelId` only for stable linked declarations and parse dynamic input with
`ModelIdBuf::parse`; Rust diagnostic type names are not persistent IDs. Keep
ordinary static inspection separate from global registry initialization, and
resolve only after the complete model set has been linked. This crate does not
provide an alternative reflection system, nor does it implicitly bind
cross-model references during static inspection.

## Further Reading

- [README](../README.md)
- [简体中文用户指南](user_guide.zh_CN.md)
- [API documentation](https://docs.rs/qubit-model-metadata)
