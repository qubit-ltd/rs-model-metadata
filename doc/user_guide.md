# qubit-model-metadata User Guide

[简体中文](user_guide.zh_CN.md) | [README](../README.md) | Local API docs: `cargo doc --open`

Applies to `qubit-model-metadata` 0.1.x, Rust 1.94, and edition 2024.

## Purpose and Audience

This guide is for framework and application developers who declare domain
models with `qubit-model-derive` and need to inspect their metadata or resolve
relationships after all model crates have been linked. It explains the boundary
between structural reflection and domain semantics, then follows an account
model from declaration to an immutable resolved graph. The model ABI described
here is v4 and consumes the reflection `codegen_v2` protocol.

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
global model registry, and the completed registry resolves `Login.account_id` to the
`Account` entity's `id` property.

## Installation and Minimal Configuration

The Qubit model crates are currently internal and unpublished. Add them from
adjacent checkouts, adjusting paths for your workspace:

```toml
[dependencies]
qubit-model-metadata = { version = "0.1", path = "../rs-model-metadata" }
qubit-model-derive = { version = "0.1", path = "../rs-model-metadata/derive" }
qubit-id = { version = "0.6", path = "../../rust-common/rs-id" }
qubit-validator = { version = "0.1", path = "../../rust-common/rs-validator" }
qubit-codec = { version = "0.14", features = ["registry"] }
```

`ValueCodecRegistry` is available only with `qubit-codec`'s `registry`
feature. Keep that feature enabled in the application crate that imports and
supplies the codec registry to `ModelResolver`. The resolver accepts all three
registries explicitly, so an application using it needs direct dependencies on
`qubit-validator` and `qubit-codec` even when a particular model declares no
validator or codec. `qubit-id` supplies the exact identifier type required by
`Entity` and `Projection`.

Use a model-role derive macro for every type passed to `TypeMetadata::of`.
The macro generates the required metadata provider and bounds; a type derived
only with structural reflection is not enough.

## Core Workflow

Declare an entity and a model that refers to it. The `#[reference]` declaration
uses the stable entity ID and the public property name to be resolved later.

```rust,ignore
use qubit_id::Id;
use qubit_model_derive::Entity;
use qubit_model_derive::Model;

#[Entity(id = "example.Account")]
pub struct Account {
    #[identifier]
    pub id: Id,
    #[unique(ignore_case = true)]
    pub email: String,
}

#[Model(id = "example.Login")]
pub struct Login {
    #[reference(entity_id = "example.Account", property = id)]
    pub account_id: Id,
}
```

Inspect the account locally. `TypeMetadata::of` is a static query and does not
initialize the global model registry. Property lookup freezes the reflection
snapshot to merge separately emitted `ModelImpl` capability fragments:

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
use qubit_model_metadata::ModelRegistry;
use qubit_model_metadata::ModelResolver;
use qubit_model_metadata::ResolveInputs;
use qubit_model_metadata::TypeMetadata;
use qubit_validator::ValidatorRegistry;

fn resolve_models() -> Result<(), Box<dyn std::error::Error>> {
    let models = ModelRegistry::try_global()?;
    let validators = ValidatorRegistry::try_global()?;
    let codecs = ValueCodecRegistry::try_global()?;
    let graph = ModelResolver::new(ResolveInputs {
        models,
        validators,
        codecs,
    })
    .resolve_all()?;
    let field = TypeMetadata::of::<Login>().field("account_id").unwrap();
    let reference = graph.reference(field).unwrap();
    assert_eq!(reference.target().model_id().unwrap().as_str(), "example.Account");
    assert_eq!(reference.property().unwrap().name(), "id");
    Ok(())
}
```

The resulting `ResolvedModelGraph` is immutable. A successful resolver pass
publishes a complete graph rather than an incrementally resolved partial view.

## Advanced Usage

### Choose the Correct Metadata Stage

Use `TypeMetadata::of::<T>()` for local, static inspection. It validates the
generated metadata against the unique reflection descriptor and panics on a
hidden-ABI violation; use `TypeMetadata::try_of::<T>()` when that failure must
remain recoverable.

Use `ModelRegistry` when stable-ID or exact Rust `TypeId` lookup is needed.
`ModelRegistry::try_global()` first freezes `ReflectRegistry`, then projects
both concrete and generic-definition model capabilities with their
authoritative reflection provenance. It does not discover unlinked crates.
`ModelRegistry::from_metadata` builds an isolated explicit snapshot for tests;
`from_reflect_registry` projects directly from a supplied frozen reflection
snapshot.

Use `ModelResolver` only after the complete intended model set is available.
It validates cross-model edges and executable strategy bindings, then returns
one immutable `ResolvedModelGraph`. A failed pass returns no partial graph.

### Roles, Stable IDs, and Generic Models

`Entity` and `Projection` are non-generic named structs with exactly one
`qubit_id::Id` identifier. An `Entity` requires a stable model ID; a
`Projection` may be open or fixed to an Entity by Rust type or stable ID.
`Model`, `Enum`, and `Value` cover structured records, domain enums, and value
objects. A stable ID makes a declaration eligible for registration; metadata
without one remains available through `TypeMetadata::of` but not by registry
ID lookup.

`ModelId` validates static strings without allocation. Use
`ModelId::try_new` for recoverable validation and `ModelIdBuf::parse` for
dynamic input. Each dot-separated segment must match
`[A-Za-z][A-Za-z0-9_]*`; comparison is case-sensitive.

### Fields, Constraints, and Properties

Fields are physical storage slots supplied by reflection. A `Property` combines
a field and optional getter and setter under one public name. Getter adapters
preserve borrowed lifetimes; when a setter fails before execution, its
`PropertySetFailure` retains the replacement value for the caller.

Start with `TypeMetadata::fields`, `field`, or `field_at`. `FieldMetadata`
delegates structural facts to its reflection `FieldDescriptor`, while typed
getters expose identifier, uniqueness, reference, key-part, validator, codec,
redaction, and Serde semantics. Use `text_constraint`, `decimal_constraint`,
`time_constraint`, `sequence_constraint`, and `map_constraint` when only one
constraint family is relevant, or iterate `constraints()` to preserve every
declaration. Check `type_ref()` before assuming `descriptor()` is present.

For generic declarations with a model ID, the reflection registry stores one
first-class `TypeDefinitionDescriptor` carrying a generic-model capability.
`GenericModelMetadata` points to that definition. Concrete generic instances
have no `ModelId`; their field types may be resolved while definition fields
remain symbolic.

### Resolved Views and Query Metadata

The resolved graph is the lookup point for cross-model results. Use
`reference` for a field reference, `projection_source` for a fixed Projection,
and `projection_producers` for readable Entity properties that can produce a
Projection. A producer with an executable getter can be invoked with `project`;
the runtime verifies that the Entity and produced Projection keep the same
`Id`.

`properties(model)` returns the local field/getter/setter merge accepted by
the resolver. For an Entity, `query(entity)` exposes filters derived from
indexed fields and unique keys derived from identifiers and uniqueness
declarations. Flattened query names must be unambiguous; a collision fails the
resolution pass.

Validator occurrences and codec declarations remain declarative until
resolution. `validator` returns the matched executable registration and its
readable property dependencies. `codec` returns the executable codec
descriptor and, for an ID-based declaration, its registry entry.

## Errors and Diagnostics

`ModelId::try_new` and `ModelIdBuf::parse` return `ModelIdError` for empty IDs,
empty dot-separated segments, or segments outside the ASCII identifier
grammar. `ModelRegistry::try_global()` reports duplicate model IDs, conflicting
registrations, or reflection-registry initialization failure as
`ModelRegistryError`.

`resolve_all()` returns `ModelResolveErrors`, which aggregates errors in a
deterministic order instead of publishing a graph with unresolved
relationships. Inspect `errors()` and each `ModelResolveError::kind()` rather
than parsing display text. Error kinds cover invalid local properties, entity
nesting, opaque models, references, roles and types, projection contracts,
validator and codec bindings, selector types, value closure, and flattened
query-name conflicts. Optional accessors expose the involved model ID, property
path, expected and actual role or type, and source fragments when available.

Generated metadata also checks model ABI v4 invariants before publication. A panic
whose message starts with `QMM-ABI-` indicates that generated or manually
supplied hidden-ABI metadata was rejected because it violated one of those
invariants.

## Troubleshooting

- If `TypeMetadata::of::<T>()` does not compile, confirm that `T` uses a
  model-role macro and satisfies the generated trait bounds.
- If an Entity or Projection identifier is rejected, use `qubit_id::Id`
  exactly; integer primitives and application-specific ID wrappers do not meet
  the role contract.
- If `descriptor()` returns `None`, inspect `type_ref()` before treating it as
  an error; opaque and symbolic references deliberately have no concrete
  descriptor.
- If an expected model is absent from `ModelRegistry::try_global()`, make sure
  its crate is linked into the final binary and that the declaration has a
  stable model ID. Registry collection cannot see an unlinked crate.
- If `resolve_all()` returns errors, inspect each `ModelResolveError` and fix
  the stable ID, target role, property name, or matching validator/codec
  registration before retrying the full resolution pass.
- A resolved validator exposes its typed registration and readable dependency
  properties. A resolved codec exposes its executable descriptor and, for an
  ID declaration, the matching registry entry.

## Limitations and Best Practices

Use `ModelId` only for stable linked declarations and parse dynamic input with
`ModelIdBuf::parse`; Rust diagnostic type names are not persistent IDs. Keep
direct `TypeMetadata::of` inspection separate from global model-registry
initialization, and resolve only after the complete model set has been linked.
Descriptor capability and property queries intentionally share the frozen
reflection registry. This crate does not
provide an alternative reflection system, nor does it implicitly bind
cross-model references during static inspection.

Keep only one version of `qubit-model-metadata` in the final dependency graph.
Different versions own separate registration inventories and would split the
model set. Treat model IDs as an application protocol: rename them deliberately
and update every textual reference together. Prefer recoverable `try_*` entry
points in libraries and diagnostics; reserve panic-based `of` and `global`
entry points for startup paths where invalid generated metadata or registry
configuration is unrecoverable.

## Further Reading

- [README](../README.md)
- [简体中文用户指南](user_guide.zh_CN.md)
- [`qubit-model-derive` declaration guide](../derive/doc/user_guide.md)
- Local API documentation: run `cargo doc --open`

## Snapshot-scoped property queries

`TypeMetadata::try_properties()` and `try_property(name)` resolve against the global reflection snapshot and return `PropertyResolutionError::Reflection` when initialization fails, `Capability` when intrinsic declarations conflict, or `Assembly` when linked declarations disagree. `property_fragments()` also returns a `Result`; registration failure is never treated as a missing overlay.

For an explicit snapshot, use `try_properties_in(&reflection)`, `try_property_in(&reflection, name)`, and `property_fragments_in(&reflection)`. `ModelRegistry::properties_for(metadata)` uses that model registry's snapshot. Registries created with `from_metadata` use only local field properties. `ModelResolver` follows the same explicit context.
Build an isolated `reflection` value with `qubit-reflect::registry::RegistrySnapshotBuilder`; this is the public replacement for the old hidden testing registry helper. `ModelRegistry::try_global()` exposes reflection failures through its source chain, including the underlying capability conflict.

Enumerate concrete and generic models with `ModelRegistry::entries()`. Each immutable `ModelEntry` exposes its ID, concrete or generic metadata, and static fragment source; `get(id)` returns one entry. No second registration system is introduced.

`ModelRegistry::metadata_for(descriptor)` returns `Result<Option<&TypeMetadata>, ModelMetadataError>`.
Handle `Err` before treating `Ok(None)` as a non-model type. Capability errors preserve their
queried type and complete conflict; ABI errors preserve the descriptor mismatch. This query
never downgrades a failed provider lookup to explicit-metadata fallback. Provider panics and
invalid generated construction still follow their existing panic contracts.

The explicit property methods also return `Result`. `ModelResolveError::cause()` exposes
`ModelResolutionCause::Metadata` or `Properties`, with root model, property path, and source
available on the diagnostic. A failed nested path is not `MissingProperty`; independent target
or field failures remain in the returned aggregate. `PropertyValue::into_invocation_output`
materializes borrowed slices in O(n) time without copying elements; direct indexed slice access
avoids output materialization, although the erased slice adapter itself is boxed.
