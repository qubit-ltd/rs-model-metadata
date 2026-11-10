# rs-model-metadata design

[简体中文](design.zh_CN.md) · [User guide](user_guide.md) · [Derive design](../derive/doc/design.md)

This document describes `qubit-model-metadata` 0.2.0 (Rust 1.94, edition 2024). `qubit-reflect` describes Rust structure; `qubit-model-derive` adds business semantics to domain declarations; this runtime makes those semantics available to other libraries and resolves their relationships. Persistence and application policy belong to consumers.

## Domain semantics and consumers

Field annotations encode facts a structural reflector cannot infer: `text` and `decimal` limits, `unique` scopes, `reference` targets, `indexed` queryability, and custom `validator` bindings. Ordinary integer and floating-point range checks can be declared through a custom validator and executed when that validator is registered and bound at runtime; supported values depend on the validator's input contract. Keeping these facts with the domain type gives consumers a shared source of rules instead of separate validation, test-data, and query schemas.

| Consumer | Metadata it reads | Decision it still owns |
| --- | --- | --- |
| Validation | Field constraints, uniqueness scopes, validators, dependencies, and access paths. | Instance checks and any lookup needed to enforce database uniqueness. |
| Constrained test-data generation | Length/range limits, uniqueness, and references. | Generating values, reserving unique values, and choosing existing referenced records. |
| REST query layer | `indexed` and other query declarations in `ModelGraph`. | Accepting filters, building SQL, and executing queries. |

The optional `validation` feature binds supported declarations into a `ValidationPlan`. Random data generation and REST query execution can consume this metadata contract; `ModelGraph` itself performs neither operation.

```text
Rust declarations --derive--> TypeMetadata + reflect capabilities
                              |
ReflectionContext snapshot --> ModelRegistry ----> StructureResolver ----> ModelGraph
                                     |                    |
                             PropertyAccessPath     optional validation/codec binders
```

The public vocabulary lives in `metadata`, `registry`, `resolve`, and `access_path`. `generic`, `validation`, and `codec` are separate optional features; the default feature set is empty. The independent `qubit-model-id` crate carries the stable model ID protocol.

The reflection boundary follows `rs-reflect` codegen v4: runtime and generated
code use `DynamicRef<'a, Local>`, `DynamicMut<'a, Local>`, and
`DynamicOwned<Local>` for checked dynamic access, while `#[ReflectImpl]`
registers generated capabilities. The crate's `__private::v8` facade is a
separate internal runtime/derive ABI version, not an application API. Enum payload execution
uses the reflected `VariantDescriptor::is_active_dynamic` check before reading
a payload field; a mismatched dynamic root is an execution error.

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
2. `ModelRegistry::from_context(&reflection)` projects a caller-supplied `ReflectionContext` snapshot. It invokes that snapshot's metadata providers, retains their provenance, and resolves `ModelImpl` providers from the same context. Reachable anonymous child models can be discovered even though they have no stable `ModelId`. It never consults the global context to fill gaps.
3. `ModelRegistry::from_static_metadata(...)` builds a registry whose indexed entries come from explicitly supplied metadata and provenance. The supplied `ReflectionContext` remains bound for capability and property resolution, so providers registered in that context are visible. A context without a matching property provider yields the type's local properties. A generated nested metadata provider on an explicit root can discover a child without adding it to the registry index; for example, with `generic`, a concrete generic child model is discovered this way. Merely including a non-generic child type in the reflection context does not provide its model metadata; register its metadata provider in the context when it must be discoverable. The bound context must still supply the reflection facts needed to resolve each child descriptor.

The registry indexes stable model IDs and exact `TypeId`s. With `generic`, generic definitions can also be looked up by their process-local definition identity; anonymous definitions are not added to the stable-ID list. Registry construction rejects duplicate IDs, conflicting registrations for one concrete type, capability targets outside the reflection snapshot, incompatible providers, and metadata inconsistent with its reflection descriptor.

The following complete program builds an explicit reflection context, projects
it into a model registry, and looks up the registered model:

<!-- example: core -->
```toml
[dependencies]
qubit-model-metadata = { version = "0.2", path = "../rs-model-metadata", default-features = false }
qubit-model-derive = { version = "0.2", path = "../rs-model-metadata/derive" }
qubit-id = "0.7"
qubit-reflect = { version = "0.2", path = "../rs-reflect" }
```

<!-- example: core/design-context -->
```rust
use qubit_id::Id;
use qubit_model_derive::Entity;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_reflect::ReflectionContext;

#[Entity(id = "docs.design.ContextRegistered")]
struct ContextRegistered {
    #[identifier]
    id: Id,
}

fn main() {
    let reflection = ReflectionContext::builder()
        .include_linked()
        .build()
        .expect("linked reflection context");
    let models = ModelRegistry::from_context(&reflection).expect("context model registry");
    assert!(models.get("docs.design.ContextRegistered").is_some());
}
```

Metadata, property, and path caches are scoped to their registry. The bounded path cache retains at most 256 successfully compiled paths, distinguishing root metadata, path segments, and read/write mode; failures are not cached and can be retried. An `Arc<PropertyAccessPath>` already returned to a caller remains usable after cache eviction. Build a new path against the registry for each distinct reflection snapshot or overlay.

## Resolution scope and steps

`StructureResolver` checks declarations against one explicit `ModelRegistry` and a set of roots. `StructureResolver::new` audits all registered concrete models along with explicit roots. `for_roots` and `for_static_roots` resolve only the subgraph reachable from the supplied roots; the latter owns its root array so the resulting graph can be retained independently of a caller-local slice. Anonymous roots participate without receiving invented stable IDs. A root's generated nested metadata providers can discover reachable children when they yield metadata; with `generic`, this includes a concrete nested generic child model. A non-generic child field alone does not make its model metadata available: the bound context needs a provider for that child. The registry index, explicit roots, root-generated metadata providers, context providers, and reflection facts serve distinct roles. Nested traversal still requires the context facts needed to resolve the child descriptor. Resolution does not fill missing facts from the process-wide registry, and lookup failures remain errors.

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
3. `plan.validate(&value, &options)` checks the concrete instance root type, then reads and executes rules in declaration order. Rule violations are returned in `Ok(ValidationReport)`. Accessor, dependency, and traversal-budget failures return `Err(ModelValidationError)` with the report accumulated before execution stopped. Dynamic callers use `validate_dynamic(PropertyBorrowed::new_any(&value), &options)`.

This complete program builds and reuses a plan for a concrete value:

<!-- example: validation -->
```toml
[dependencies]
qubit-model-metadata = { version = "0.2", path = "../rs-model-metadata", features = ["validation"] }
qubit-model-derive = { version = "0.2", path = "../rs-model-metadata/derive" }
qubit-reflect = { version = "0.2", path = "../rs-reflect" }
qubit-validator = { version = "0.1.0", path = "../../rust-common/rs-validator" }

[patch.crates-io]
qubit-validator = { path = "../../rust-common/rs-validator" }
```

<!-- example: validation/design-plan -->
```rust
use std::sync::Arc;
use qubit_model_derive::Model;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_model_metadata::validation::ValidationBuildInputs;
use qubit_model_metadata::validation::ValidationOptions;
use qubit_model_metadata::validation::ValidationPlan;
use qubit_validator::ValidatorRegistry;

#[Model]
struct Account {
    #[text(non_blank)]
    name: String,
}

fn main() {
    let root = TypeMetadata::of::<Account>();
    let models = ModelRegistry::try_global().expect("linked model metadata");
    let graph = Arc::new(StructureResolver::for_static_roots(&models, [root])
        .resolve().expect("valid model graph"));
    let validators = ValidatorRegistry::empty();
    let plan = ValidationPlan::build(root, ValidationBuildInputs {
        graph: Arc::clone(&graph),
        validators: &validators,
    }).expect("built-in text rule bound");

    let valid = Account { name: "Ada".to_owned() };
    assert!(plan.validate(&valid, &ValidationOptions::default())
        .expect("validation executed").is_valid());
    let invalid = Account { name: String::new() };
    let report = plan.validate(&invalid, &ValidationOptions::default())
        .expect("validation executed");
    assert!(!report.is_valid());
    assert_eq!(report.violations()[0].path().render(), "name");
}
```

The walker preserves each actual use path as a separate occurrence, even when several fields lead to the same child type. Its current shape boundary is:

| Shape | Execution contract |
| --- | --- |
| Direct named or tuple enum payload value rule | Executes only for the active variant; inactive variants are skipped, and report paths include variant and field. |
| Enum payload selector or nested model rule | Rejected; payload support is limited to direct value rules. |
| `Vec<T>` / `[T; N]` `Element`; `HashMap<K, V>` / `BTreeMap<K, V>` `MapKey` and `MapValue` | Executes each member through a generated borrowing adapter; borrowed-slice getters also expose elements. `Option`-wrapped collections skip selectors when absent. |
| Selector rule with dependencies; unknown/custom collection; owned collection getter; implicit member unwrapping; model rules inside collection members | Rejected during capability checking and plan construction. |

Recursive shapes with no execution work can terminate; recursive paths with executable work are rejected rather than silently producing incomplete plans. `Reference` traversal stops at the stored field, and `opaque` traversal stops inside the opaque structure; explicit rules on the outer value remain available.

`ValidationOptions::default()` selects all fields and `CollectAll`, with maximum depth 64, 100,000 visited nodes, 100 violations, and 1,000,000 comparisons. `FailFast` and the violation limit produce a successful report that may be truncated. Exhausting depth, node, or comparison budgets is an execution error with a partial report. These shared budgets cover root/property/dependency/member reads and rule calls; selector element rule calls and each pair checked by outer sequence uniqueness consume comparison budget. Unknown-length selector probes reserve node budget for a possible read and rule call, plus comparison budget, before `next()`; an exhaustion probe may consume these budgets without yielding a member. The budgets do not limit work or allocations inside a custom validator. Map entry path indices follow the collection iterator order (which is not stable for `HashMap`). Field selection matches complete field paths, not path prefixes.

Plan construction has a separate fixed `BuildBudget`: maximum model, wrapper, and type-expression depth 64; 100,000 charged discovery and binding work units; 10,000 declaration occurrences; 10,000 bound rules; and 100 ordinary diagnostics. One work unit is charged before inspecting each model, field, structural edge, expression node, declaration, selector, validator, reverse-reachability edge, or binding attempt; inspecting an item in separate phases charges each visit. Attempting the 101st ordinary diagnostic stops construction and adds one terminal `BuildLimitExceeded` error, so the returned list has at most 101 entries. Capabilities and plan building share the same discovery limits, and binding output is checked before it is appended. These limits do not constrain the caller-created graph or reflection context, nor allocations or work inside external validator binders. They do not change the per-instance execution budgets above.

`ValidationBuildError::build_limit()` exposes a `ValidationBuildLimit` for limit errors, including its `ValidationBuildLimitKind` and fixed capacity; other build errors return `None`. The five categories are Depth (64), Work (100,000), Occurrences (10,000), Bindings (10,000), and Diagnostics (100 ordinary diagnostics followed by one terminal error). The category enum is `#[non_exhaustive]`, so downstream matches need a wildcard arm. Root-level errors may happen before declaration traversal and therefore may have no path, declaration source, or binder source. The `BuildLimitExceeded` kind remains a unit variant for compatibility; consumers should use the accessor rather than parse display text.

MM-003 was reproduced with the same root and a fixed Work limit: before the fix, the root-only graph succeeded at 10 units while the graph sharing an unrelated model required 31. After root-scoped structural discovery, both succeeded at 10 units; the shared graph first required 14 units. The remaining graph-wide work is the expression fallback's name lookup, which deliberately scans for globally unique names and is charged so lookup work remains metered. The fix reduces unrelated structural scanning but does not eliminate all graph-wide work or change name-resolution semantics.

`ValidationBuildErrorKind` gains the public `BuildLimitExceeded` variant, which can require downstream exhaustive matches to add a fallback or explicit case. The current rs-platform runtime wraps the aggregate error and has no exhaustive kind match to migrate.

With `codec`, stable codec IDs or Rust codec types in declarations must be bound through an explicitly supplied codec registry. The binding priority is field-specific codec, the Value's canonical codec, then no codec. `bind_codecs` resolves declarations to registered implementations and returns bindings; it does not call `encode` or `decode`, persist data, or apply application policy. Metadata contains no codec instances and does not implement a persistence workflow. See the [user guide](user_guide.md) for supported shapes and runnable usage.

## Error phases and integration

| Phase | Typical failure | Application response |
| --- | --- | --- |
| Derive expansion and Rust type checking | Attribute syntax, role shape, trait bounds, or duplicate accessors. | Correct the declaration or capability selection; runtime retries cannot fix a compile-time error. |
| Checked metadata and registry construction | `AbiViolation`, duplicate IDs, invalid capability targets, or provider conflicts. | Check runtime/derive version alignment, linked registrations, and reflection snapshot provenance. |
| `StructureResolver` | Invalid Reference target, Value closure, uniqueness scope, or other cross-model relation. | Inspect the owner, field location, and declaration source, then correct the model relation. |
| Validation or codec binding | Missing registration, type mismatch, or unsupported execution shape. | Supply an adapter or adjust the consumer; do not remove a real model constraint merely to pass binding. |
| Instance execution | A `ValidationReport` with violations or an execution error with a partial report. | Apply business policy to violations; preserve execution errors and their partial results rather than treating them as valid data. |

Generated constructors use the synchronized `__private::v8` ABI and reflection provider contract. Keep runtime and derive versions aligned. Applications should depend on the public `metadata`, `registry`, `resolve`, `validation`, and `codec` APIs rather than the generated-code ABI. The platform's production integration is described in the [rs-platform design](../../rs-platform/doc/design.md). Historical requirements and migration ledgers are background records, not substitutes for current contracts.

A missing lookup is distinct from a failed lookup: absence is an ordinary result, while registry, property, and resolution failures carry errors that should retain their diagnostic context.

## Source and review map

- Static type, field, and property metadata: `src/type_metadata.rs`, `src/field_metadata.rs`, `src/property.rs`, and `src/local_property_set.rs`.
- Registry construction, provenance, and caches: `src/registry/`; structural resolution and graph queries: `src/resolve/`.
- Declaration vocabulary, paths, and locations: `src/metadata/` and `src/relation/`; execution binding: `src/validation/` and `src/codec/`.
- Public API examples, runnable workflows, and limitations: the [user guide](user_guide.md).
