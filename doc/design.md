# rs-model-metadata design

[简体中文](design.zh_CN.md) · [User guide](user_guide.md) · [Derive design](../derive/doc/design.md)

This document describes `qubit-model-metadata` 0.2.0 (Rust 1.94, edition 2024). `qubit-reflect` describes Rust structure; `qubit-model-derive` adds business semantics to domain declarations; this runtime makes those semantics available to other libraries and resolves their relationships. Persistence and application policy belong to consumers.

## Domain semantics and consumers

Field annotations encode facts a structural reflector cannot infer: `text` and `number` limits, `unique` scopes, `reference` targets, `indexed` queryability, and custom `validator` bindings. Keeping these facts with the domain type gives consumers a shared source of rules instead of separate validation, test-data, and query schemas.

| Consumer | Metadata it reads | Decision it still owns |
| --- | --- | --- |
| Validation | Field constraints, uniqueness scopes, validators, dependencies, and access paths. | Instance checks and any lookup needed to enforce database uniqueness. |
| Constrained test-data generation | Length/range limits, uniqueness, and references. | Generating values, reserving unique values, and choosing existing referenced records. |
| REST query layer | `indexed` and other query declarations in `ModelGraph`. | Accepting filters, building SQL, and executing queries. |

The optional `validation` feature binds supported declarations into a `ValidationPlan`. Random data generation and REST query execution can consume this metadata contract; `ModelGraph` itself performs neither operation.

```text
Rust declarations --derive--> TypeMetadata + reflect capabilities
                              |
ReflectRegistry snapshot ----> ModelRegistry ----> StructureResolver ----> ModelGraph
                                     |                    |
                             PropertyAccessPath     optional validation/codec binders
```

The public vocabulary lives in `metadata`, `registry`, `resolve`, and `access_path`. `generic`, `validation`, and `codec` are separate optional features; the default feature set is empty. The independent `qubit-model-id` crate carries the stable model ID protocol.

## Core objects and ownership

These objects describe declarations and resolved structure; they do not represent persisted records or own application values.

| Object | Role | Identity and lifetime |
| --- | --- | --- |
| `TypeMetadata` | Describes one Rust type's role, fields, declarations, and local properties without an instance. | Static metadata identifies its concrete Rust type by process-local `TypeId`; `model_id()` may be absent for anonymous models. |
| `FieldMetadata` | Describes a stored field and its declarations, including named, positional, and enum-payload fields. | A concrete field can have a `FieldLocation`; a generic definition field has no concrete location. |
| `PropertyMetadata` | Describes a readable and/or writable property assembled from a field and eligible `ModelImpl` accessors, or a computed property. | It copies accessor metadata; reading may borrow from the source instance. Property assembly can fail on conflicts. |
| `ModelRegistry` | Indexes model metadata in one registration snapshot by stable ID and exact Rust type. | Borrows reflection provenance when built from a reflection snapshot; indexes and caches belong to this registry. |
| `ModelGraph` | Holds successfully resolved cross-model relations, properties, references, and query declarations. | Borrows its `ModelRegistry`; resolved identities include process-local `TypeId` and `FieldLocation`. |
| `ValidationPlan` | Holds executable rule bindings for declarations reachable from one resolved root. | Exists with the `validation` feature and retains an `Arc<ModelGraph>` plus bound rules. |

`ModelId` is the stable external identity intended for storage and interchange. `TypeId`, `FieldLocation`, and generic `TypeDefinitionId` are process-local identities and must not be persisted or sent as durable identifiers. `FieldLocation` identifies a field using its owner `TypeId`, optional enum variant, and field index; `DeclarationLocation` separately records declaration provenance for diagnostics.

`TypeMetadata::of::<T>()` is convenient for known generated metadata. Use `try_of::<T>()` when ABI validation failures must be handled as errors. `fields()` and `field(name)` inspect declarations, not instances. `try_properties()` returns an owned `ResolvedProperties` view that should be kept while borrowing its property slice; `try_property(name)` returns a copied single property. A property assembly error is not equivalent to a missing property.

## Registry construction and snapshot isolation

Choose the registry constructor according to the declarations and snapshot available to the caller:

1. `ModelRegistry::try_global()` initializes the process-wide reflection registry and projects its model registrations once. Both success and initialization errors are cached; later failures return a clone of the original error. A panic from an initializing provider propagates and is not cached.
2. `ModelRegistry::from_reflect_registry(&reflection)` projects a caller-supplied frozen `ReflectRegistry` snapshot. It invokes that snapshot's metadata providers, retains their provenance, and resolves `ModelImpl` providers from the same snapshot. Reachable anonymous child models can be discovered even though they have no stable `ModelId`. It never consults the global registry to fill gaps.
3. `ModelRegistry::from_static_metadata(...)` builds an isolated registry from explicitly supplied metadata and provenance. It uses each type's local properties only; it cannot see independently registered `ModelImpl` providers. Callers must include child metadata explicitly when resolving anonymous nested models.

The registry indexes stable model IDs and exact `TypeId`s. With `generic`, generic definitions can also be looked up by their process-local definition identity; anonymous definitions are not added to the stable-ID list. Registry construction rejects duplicate IDs, conflicting registrations for one concrete type, capability targets outside the reflection snapshot, incompatible providers, and metadata inconsistent with its reflection descriptor.

Metadata, property, and path caches are scoped to their registry. The bounded path cache retains at most 256 successfully compiled paths, distinguishing root metadata, path segments, and read/write mode; failures are not cached and can be retried. An `Arc<PropertyAccessPath>` already returned to a caller remains usable after cache eviction. Build a new path against the registry for each distinct reflection snapshot or overlay.

## Resolution scope and steps

`StructureResolver` checks declarations against one explicit `ModelRegistry` and a set of roots. `StructureResolver::new` audits all registered concrete models along with explicit roots. `for_roots` and `for_static_roots` resolve only the subgraph reachable from the supplied roots; the latter owns its root array so the resulting graph can be retained independently of a caller-local slice. Anonymous roots participate without receiving invented stable IDs. A reflection-backed snapshot can discover reachable anonymous children; a static-metadata registry requires those children to be listed explicitly.

Resolution gathers nodes by exact Rust type identity, assembles properties from the selected snapshot, and checks model roles and relationships: nested Entity rules, Value closure, Reference targets and properties, Projection sources, uniqueness scopes, and query declarations. It aggregates structural failures in `ResolveErrors`, retaining field and declaration context where available. A valid `ModelGraph` establishes structural consistency; it does not establish that validators or codecs are registered, nor that any instance value is valid.

The graph offers typed lookups for models, resolved property views, Reference targets, Projection sources, Entity query declarations, and individual declaration dependency occurrences. Query metadata reports declaration reasons such as identifier, indexed, global unique, and reference; it does not generate SQL, consumer filters, physical indexes, or every possible reachable path. `ObjectPath` uses `/` and `..` for object navigation; `PropertyPath` uses `.` for property selection. When a declaration requires parent context, the graph records that requirement and a later operation must receive the actual parent instance.

## Property access and write boundaries

`PropertyAccessPath` compiles a sequence of property names against a specific root type and registry snapshot. Compilation resolves every segment structurally and does not call getters. Read and write capabilities are checked separately: `compile` requires a readable leaf, while `compile_for_write` permits a setter-only leaf; every intermediate property must remain readable so traversal can reach the leaf. A readable path is not evidence that the same path is writable.

During a write, structural failures are detected before the leaf setter runs. A missing intermediate `Option<Model>` can be observed as a missing value on read, whereas writing through intermediate nodes requires a mutable traversable storage path.

Reads can borrow property values from the input instance. The returned `PropertyValue<'a>` cannot outlive that instance. Supported getter results include borrowed values, optional borrowed values, and supported slice shapes; an owned intermediate result does not imply that traversal can continue borrowing through it.

Setter failures distinguish rejection before execution from failure after the setter has accepted the replacement value. A pre-execution failure can retain the untouched replacement; after-execution failure cannot promise that the value is recoverable or that the object is unchanged. In particular, the runtime does not roll back mutations made by a custom setter. Applications remain responsible for authorization, persistence, and query execution.

## Optional execution stages

The core runtime stores constraints, validators, selectors, codec references, object paths, and property paths as declarations. Optional execution features bind those declarations to caller-provided registries and adapters.

With `validation`, a validation operation has three distinct stages:

1. `ValidationCapabilities::check(root, &graph)` checks whether declarations and access shapes found from the graph root are discoverable. It does not bind custom rules or invoke getters.
2. `ValidationPlan::build(root, ValidationBuildInputs { graph, validators })` binds executable occurrences to built-in rules and the supplied `ValidatorRegistry`. It aggregates independent unsupported-shape, missing-rule, and type-mismatch errors. The graph's canonical metadata for the root type is authoritative; an overlay with the same `TypeId` cannot replace its declarations.
3. `plan.validate(ReflectedRef::new(&value), &options)` checks the instance root type, then reads and executes rules in declaration order. Rule violations are returned in `Ok(ValidationReport)`. Accessor, dependency, and traversal-budget failures return `Err(ModelValidationError)` with the report accumulated before execution stopped.

The walker preserves each actual use path as a separate occurrence, even when several fields lead to the same child type. Recursive shapes with no execution work can terminate; recursive paths with executable work, constrained enum payloads, and unsupported collection-internal paths are rejected rather than silently producing incomplete plans. `Reference` traversal stops at the stored field, and `opaque` traversal stops inside the opaque structure; explicit rules on the outer value remain available.

`ValidationOptions::default()` selects all fields and `CollectAll`, with maximum depth 64, 100,000 visited nodes, 100 violations, and 1,000,000 comparisons. `FailFast` and the violation limit produce a successful report that may be truncated. Exhausting depth, node, or comparison budgets is an execution error with a partial report. These budgets cover the runtime's reads, element visits, and rule calls; they do not limit work or allocations inside a custom validator. Field selection matches complete field paths, not path prefixes.

With `codec`, stable codec IDs or Rust codec types in declarations must be bound through an explicitly supplied codec registry. The binding priority is field-specific codec, the Value's canonical codec, then no codec. Metadata contains no codec instances and does not implement persistence or an encoding workflow. See the [user guide](user_guide.md) for supported shapes and runnable usage.

## Error phases and integration

| Phase | Typical failure | Application response |
| --- | --- | --- |
| Derive expansion and Rust type checking | Attribute syntax, role shape, trait bounds, or duplicate accessors. | Correct the declaration or capability selection; runtime retries cannot fix a compile-time error. |
| Checked metadata and registry construction | `AbiViolation`, duplicate IDs, invalid capability targets, or provider conflicts. | Check runtime/derive version alignment, linked registrations, and reflection snapshot provenance. |
| `StructureResolver` | Invalid Reference target, Value closure, uniqueness scope, or other cross-model relation. | Inspect the owner, field location, and declaration source, then correct the model relation. |
| Validation or codec binding | Missing registration, type mismatch, or unsupported execution shape. | Supply an adapter or adjust the consumer; do not remove a real model constraint merely to pass binding. |
| Instance execution | A `ValidationReport` with violations or an execution error with a partial report. | Apply business policy to violations; preserve execution errors and their partial results rather than treating them as valid data. |

Generated constructors use the synchronized `__private::v7` ABI and reflection provider contract. Keep runtime and derive versions aligned. Applications should depend on the public `metadata`, `registry`, `resolve`, `validation`, and `codec` APIs rather than the generated-code ABI. The platform's production integration is described in the [rs-platform design](../../rs-platform/doc/design.md). Historical requirements and migration ledgers are background records, not substitutes for current contracts.

A missing lookup is distinct from a failed lookup: absence is an ordinary result, while registry, property, and resolution failures carry errors that should retain their diagnostic context.

## Source and review map

- Static type, field, and property metadata: `src/type_metadata.rs`, `src/field_metadata.rs`, `src/property.rs`, and `src/local_property_set.rs`.
- Registry construction, provenance, and caches: `src/registry/`; structural resolution and graph queries: `src/resolve/`.
- Declaration vocabulary, paths, and locations: `src/metadata/` and `src/relation/`; execution binding: `src/validation/` and `src/codec/`.
- Public API examples, runnable workflows, and limitations: the [user guide](user_guide.md).
