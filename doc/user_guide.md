# Consuming model metadata

[简体中文](user_guide.zh_CN.md) · [README](../README.md) · [Declaration guide](../derive/doc/user_guide.md)

This guide covers the 0.1 workspace contract for framework authors. It uses a
user-directory scenario: inspect a model, resolve its indexed declarations, then
bind optional execution services explicitly. Rust 1.94 and edition 2024 are
required. Use the local checkout dependencies in the README; the default feature
set is empty. `generic`, `validation`, and `codec` enable their respective APIs.

## Model identity and structural resolution

Field describes storage; Property combines storage and eligible accessors. Both
reuse rs-reflect descriptors. TypeMetadata is static and independent of any model
instance. A stable ModelId is optional: an anonymous model still has exact TypeId
identity and can be supplied as a root.

```rust
use qubit_id::Id;
use qubit_model_derive::Entity;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::{ResolveInputs, StructureResolver};

#[Entity]
struct User {
    #[identifier]
    id: Id,
    #[indexed]
    nickname: String,
}

let root = TypeMetadata::of::<User>();
assert!(root.model_id().is_none());
let models = ModelRegistry::from_metadata(&[]).unwrap();
let roots = [root];
let graph = StructureResolver::new(ResolveInputs { models: &models, roots: &roots })
    .resolve().unwrap();
let query = graph.query(root.as_entity().unwrap()).unwrap();
assert_eq!(query.declarations().len(), 2);
assert_eq!(query.declarations()[1].path().segments(), &["nickname"]);
```

The query view contains `id` and `nickname`: identifiers also contribute an
indexed reason. It does not expand filters or impose flattened-name uniqueness.
Each QueryDeclaration retains its source Field, path, type through the Field,
and indexing reasons. A later filter crate may choose substring matching for
nickname and min/max parameters for an ordered age or timestamp. Those choices
are not metadata output contracts.

For linked model crates, use `ModelRegistry::try_global()` or construct an
explicit rs-reflect snapshot and project it with `ModelRegistry::from_reflect_registry`.
Link the participating crates before freezing the registry. Resolution traverses
the registered models and explicit roots, deduplicates concrete TypeIds, and
checks references, Projection sources, Property conflicts, and Value closure.
An explicit registry never acquires unprovided registrations from a global one.
Generic definitions remain associated with concrete metadata whether or not the
definition has a stable ID; only named definitions enter the stable-ID registry.
Repeated concurrent concrete metadata queries share the same allocation.

## Paths, references, and source locations

ObjectPath contains NavigationStep::Property and NavigationStep::Parent. Its
Display form uses `/`; PropertyPath selects ordinary properties with `.`.
An empty dependency ObjectPath means the current object. An omitted reference
path means no reuse request. Containers do not count as domain parent objects.

A reference binding path follows the full referenced Entity behind a stored ID
or Projection. The target property selection is separate. Parent-dependent
reference and validator declarations retain ContextRequirement::ParentObject in
the resolved view. `ModelGraph::dependencies()` keeps distinct dependency
occurrences. DeclarationLocation retains source coordinates, owner name,
variant/field indices, and selector position, including unnamed Enum payloads.

## Explicit execution adapters

With `validation`, construct `ValidationPlan::build(root, ValidationBuildInputs
{ graph: &graph, validators: &validators })`. The validator registry is supplied
by the caller. Binding checks the declared stable IDs, parameters, readable
Property paths, and available input/dependency types. Repeated IDs are separate
occurrences. Standard constraints use the existing validation-rules adapters.

For parent dependencies, `build_with_context` accepts nearest-parent-first type
metadata. `validate_with_context` accepts the corresponding borrowed parent
instances. Binding may defer unknown parent types; include the parent models in
the graph so execution can compile and type-check the deferred suffix. Missing
parents return a structured dependency error, not an absent Option value.
A supplied root of the wrong Rust type is rejected even for an empty plan.

The plan is read-only. ValidationOptions controls selection, fail-fast behavior,
and traversal budgets. Nested direct and optional model validators are collected
automatically across supported boundaries; opaque fields stop traversal.
Execution capability is narrower than metadata expressiveness: the current
collection adapter supports element validators through borrowed-slice getters;
unsupported selector positions or constraint adapters return explicit build
errors. Map metadata remains available to other consumers. Consult
ValidationCapabilities before selecting this adapter; metadata declarations do
not imply that every execution backend supports them.

With `codec`, call `codec::bind_codecs` using CodecBindInputs and an explicit
codec registry after graph resolution. Selection order is explicit field codec,
Value canonical codec, then no codec. Rust-type codec declarations also require
registration. Explicitly naming the canonical codec is valid. Occurrence identity
includes exact TypeId, optional stable ModelId, Property path, and source.

## Errors and troubleshooting

| Symptom | Check |
| --- | --- |
| Missing model target | The target ID and linked registrations in the supplied registry |
| Capability or Property conflict | The fallible query's cause and original declaration sources |
| Anonymous model absent from graph | Include its TypeMetadata in ResolveInputs.roots |
| Missing parent dependency | Supply nearest-parent-first instance context and graph metadata |
| Unsupported validation selector | Check ValidationCapabilities or use another consumer |
| Codec missing/type mismatch | Registration, explicit reference, and exact codec value type |

`ModelRegistry::metadata_for` returns Result<Option<_>>: Ok(None) means absent;
errors retain capability or ABI failures. `try_properties_in`, `try_property_in`,
and `property_fragments_in` preserve assembly errors. ResolveErrors aggregates
independent failures. A bad reachable descriptor may be reported both during
closure discovery and at a specific relationship path. ResolveError exposes the
owner Rust identity, optional stable ID, declaration position when known, and
underlying cause. Do not turn failures into empty metadata.

The graph borrows its immutable registry. Static metadata never borrows graph
allocations or instances. Borrowed Property values cannot escape their source
lifetimes and do not require Send. The new generated-code protocol is the private
checked v6 facade over rs-reflect codegen v3; applications should use public APIs.

Database access, random object creation, physical indexes, filter generation,
Unicode comparison execution, and missing-parent business fallback remain
consumer responsibilities. See the [frozen requirements](../derive/doc/rs-model-derive-requirements.zh_CN.md),
[design](../derive/doc/rs-model-derive-final-design.md), and local API docs generated
by `cargo doc --workspace --all-features --no-deps`.
