# Consuming model metadata

[简体中文](user_guide.zh_CN.md) · [README](../README.md) · [Declaration guide](../derive/doc/user_guide.md)

## Purpose and audience

This guide covers the 0.1.0 contract for framework authors. It uses a
user-directory scenario: inspect a model, resolve its indexed declarations, then
bind optional execution services explicitly. Rust 1.94 and edition 2024 are
required. The default feature set is empty; `generic`, `validation`, and `codec`
enable their respective APIs.

## Conceptual model

Field describes storage; Property combines storage and eligible accessors. Both
reuse rs-reflect descriptors. TypeMetadata is static and independent of any model
instance. Entity requires a stable ModelId. Other roles may be anonymous: they still have
exact TypeId identity and can be supplied as roots.

## Scenario and minimal configuration

Start with a released dependency set. Add `validation` only when the application
will build validation plans:

```toml
[dependencies]
qubit-model-metadata = { version = "0.1.0", default-features = false }
qubit-model-derive = "0.1.0"
qubit-id = "0.6.0"
```

A metadata-only registry made with `ModelRegistry::from_metadata` does not
discover additional metadata providers through reflection. Supply every
anonymous model whose declarations should participate, including nested models,
in the explicit roots. Alternatively, use a registry built from the intended
reflection snapshot. Validation consumes the resulting graph without filling
missing declarations from the process-wide registry.

```rust
use qubit_id::Id;
use qubit_model_derive::Entity;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::{ResolveInputs, StructureResolver};

#[Entity(id = "guide.directory.User")]
struct User {
    #[identifier]
    id: Id,
    #[indexed]
    nickname: String,
}

fn main() {
    let root = TypeMetadata::of::<User>();
    assert_eq!(root.model_id().expect("Entity ID").as_str(), "guide.directory.User");
    let models = ModelRegistry::from_metadata(&[]).unwrap();
    let roots = [root];
    let graph = StructureResolver::new(ResolveInputs { models: &models, roots: &roots })
        .resolve().unwrap();
    let query = graph.query(root.type_id()).unwrap();
    assert_eq!(query.declarations().len(), 2);
    assert_eq!(query.declarations()[1].path().segments(), &["nickname"]);
}
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

Validation entry points use the supplied root's TypeId to select the graph's
canonical metadata. If the caller holds another checked overlay for the same
Rust type, that overlay cannot add or remove graph declarations.
`ValidationPlan::root()` returns the graph metadata actually used by the plan;
capability checks follow the same snapshot boundary.

## Advanced usage: field identity and API migration

A concrete field's `location()` is `Some(FieldLocation)`, containing the owner
`TypeId`, optional enum variant index, and field index. Copying `FieldMetadata`
does not change that identity. Generic definition-only fields have no concrete
location. `DeclarationLocation` is source provenance, not the graph lookup key.
`TypeId` and `FieldLocation` are process-local identities; persist a `ModelId`
when an external stable identifier is needed.

| Earlier API | Current API |
| --- | --- |
| `graph.reference(field)` | `graph.reference(location)` after handling `field.location()` |
| `graph.query(entity_payload)` | `graph.query(entity_type_id)` |
| `graph.projection_source(projection_payload)` | `graph.projection_source(projection_type_id)` |
| Address-based node lookup | `graph.model(type_id)` |
| `capabilities.supports(position)` | `ValidationCapabilities::check(root, &graph)` |
| Only the bound error's `rule()` | `declared_rule_id()` retains custom IDs even when registration is missing |

The checked generated-code protocol is `__private::v7`. Upgrade runtime, derive,
and hand-written generated-code fixtures together; older private protocols have
no compatibility shim. Application code should use the public APIs above.
`rs-reflect` keeps its own independent code-generation protocol version.

The obsolete `FieldMetadata::validate_nested()` method and
`FieldAttributeMetadata::ValidateNested` marker are removed. Neither controls
execution. Supported nested declarations are discovered by the plan builder;
use the documented reference and opaque boundaries instead of a recursion flag.

## Advanced usage: paths, references, and source locations

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

## Advanced usage: explicit execution adapters

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

## Core workflow: validate a profile and keep partial results

Enable `validation` in the runtime dependency. For the following **complete,
independent programs**, add the execution types as released direct dependencies:

```toml
[dependencies]
qubit-model-metadata = { version = "0.1.0", features = ["validation"] }
qubit-model-derive = "0.1.0"
qubit-reflect = "0.1.0"
qubit-validator = "0.1.0"
```

The profile's label and optional contact name must be nonblank. Metadata discovery
and graph resolution happen before validator binding. The explicit `Option<&Contact>`
getter supplies the borrowed intermediate object needed for nested execution;
a getter returning `&Option<Contact>` is a different access shape.

```rust
use std::num::NonZeroUsize;
use qubit_model_derive::Model;
use qubit_model_derive::ModelImpl;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_model_metadata::validation::ValidationBuildInputs;
use qubit_model_metadata::validation::ValidationCapabilities;
use qubit_model_metadata::validation::ValidationMode;
use qubit_model_metadata::validation::ValidationOptions;
use qubit_model_metadata::validation::ValidationPlan;
use qubit_reflect::ReflectedRef;
use qubit_validator::ExecutionErrorKind;
use qubit_validator::ValidatorRegistry;

#[Model]
struct Contact {
    #[text(non_blank)]
    name: String,
}

#[Model]
struct Profile {
    #[text(non_blank)]
    label: String,
    contact: Option<Contact>,
}

#[ModelImpl]
impl Profile {
    pub fn contact(&self) -> Option<&Contact> { self.contact.as_ref() }
}

fn main() {
    let root = TypeMetadata::of::<Profile>();
    let roots = [root];
    let models = ModelRegistry::try_global().expect("linked metadata and accessors");
    let graph = StructureResolver::new(ResolveInputs { models: &models, roots: &roots })
        .resolve().expect("valid structure");
    ValidationCapabilities::check(root, &graph).expect("supported access shapes");
    let validators = ValidatorRegistry::empty();
    let plan = ValidationPlan::build(root, ValidationBuildInputs {
        graph: &graph,
        validators: &validators,
    }).expect("built-in text constraints bind without custom registrations");

    let invalid = Profile {
        label: String::new(),
        contact: Some(Contact { name: String::new() }),
    };
    let report = plan.validate(ReflectedRef::new(&invalid), &ValidationOptions::default())
        .expect("validation executed");
    assert!(!report.is_valid());
    assert_eq!(report.violations().len(), 2);
    assert_eq!(report.violations()[0].path().render(), "label");
    assert_eq!(report.violations()[1].path().render(), "contact.name");

    let absent = Profile { label: "personal".to_owned(), contact: None };
    assert!(plan.validate(ReflectedRef::new(&absent), &ValidationOptions::default())
        .expect("absent optional child is skipped").is_valid());

    let fail_fast = ValidationOptions::builder().mode(ValidationMode::FailFast).build();
    let report = plan.validate(ReflectedRef::new(&invalid), &fail_fast)
        .expect("policy stop is a successful, truncated execution");
    assert_eq!(report.violations().len(), 1);
    assert!(report.is_truncated());

    let limited = ValidationOptions::builder()
        .max_nodes(NonZeroUsize::new(3).expect("positive budget")).build();
    let error = plan.validate(ReflectedRef::new(&invalid), &limited)
        .expect_err("root, label read, and rule invocation consume all three nodes");
    assert_eq!(error.error().kind(), ExecutionErrorKind::TraversalLimit);
    assert_eq!(error.partial_report().violations().len(), 1);
    assert_eq!(error.partial_report().violations()[0].path().render(), "label");
}
```

The normal report preserves both paths. An absent contact is skipped. FailFast
retains only the first violation and stops before reading the contact. The node
budget example instead returns an infrastructure error while preserving the
label violation already collected. Never treat `Err` as a valid empty report.

The registry is empty only of **custom validators**: built-in standard constraints
bind through the runtime adapter. A custom `#[validator(id = "...")]` requires a
matching registration in the supplied validator registry. Capability checking
only verifies declarations and access shapes; it never runs getters, binds custom
registrations, or proves an instance valid.

## Limitations: execution support and explicit refusal

A structurally valid graph can contain declarations unsupported by this validation
backend. For example, the following enum retains its payload constraint and source
location, but plan construction must reject executing it:

```rust
use qubit_model_derive::Enum;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_model_metadata::validation::ValidationBuildErrorKind;
use qubit_model_metadata::validation::ValidationBuildInputs;
use qubit_model_metadata::validation::ValidationPlan;
use qubit_validator::ValidatorRegistry;

#[Enum]
enum ContactMethod {
    Named { #[text(non_blank)] name: String },
}

fn main() {
    let root = TypeMetadata::of::<ContactMethod>();
    let roots = [root];
    let models = ModelRegistry::from_metadata(&[]).expect("isolated registry");
    let graph = StructureResolver::new(ResolveInputs { models: &models, roots: &roots })
        .resolve().expect("enum declarations are structurally supported");
    let validators = ValidatorRegistry::empty();
    let errors = match ValidationPlan::build(root, ValidationBuildInputs {
        graph: &graph,
        validators: &validators,
    }) {
        Err(errors) => errors,
        Ok(_) => panic!("constrained enum payloads cannot produce an empty successful plan"),
    };
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].kind(), ValidationBuildErrorKind::UnsupportedExecution);
    let location = errors[0].field_location().expect("concrete payload field");
    assert_eq!(location.owner(), root.type_id());
    assert_eq!(location.variant(), Some(0));
    assert_eq!(location.index(), 0);
    assert!(errors[0].constraint().is_some());
    assert_eq!(errors[0].constraint_rule_ids()[0].as_str(), "qubit.rules.text.non_blank");
    assert!(errors[0].source_error().is_none());
}
```

| Declaration or access shape | Current validation backend |
| --- | --- |
| Existing text constraints and custom validators on named fields | Bind and execute |
| Direct and optional nested named models | Same binder; absent optional intermediates skip; actual borrowed access required |
| Explicit element validators via a borrowed-slice getter, including nested paths | Bind and execute; validate exact element input type |
| Sequence item count | Requires an actual borrowed-slice length adapter |
| Standard constraints or dependencies inside a selector; MapKey/MapValue | `UnsupportedExecution` |
| Decimal, Time, Map constraints; erased uniqueness | `UnsupportedExecution` |
| Constrained enum payloads, tuples/newtypes, and models inside container elements | `UnsupportedExecution` |
| Recursive instance paths with reachable execution declarations | `UnsupportedExecution`; no infinite expansion |
| Unit enums, and payloads or cycles without reachable execution declarations | Accepted as ordinary values |
| Reference fields | Only explicit rules on the stored field; no referenced Entity instance is fetched |
| Opaque fields | Only outer declarations; no internal traversal |
| Owned intermediate getter results | `UnsupportedExecution` when a later borrow would be required |
| Option or smart-pointer unwrapping | Only with an actual adapter; slice elements requiring implicit unwrapping are rejected |

Shared child types at `primary.name` and `secondary.name` produce independent
occurrences. Repeated rule IDs are not deduplicated. Unsupported declarations do
not disappear; build errors aggregate independent problems in declaration order.
`ValidationCapabilities::check(root, &graph)` uses the same declaration and access
checks as plan construction. `RootNotInGraph` means the root must first be included
in `ResolveInputs.roots` or the supplied registry.

Unsupported structural paths include variant names and tuple positions, such as
`choice.First.name` and `pair.1.0.name`. Container model paths use `[]`, `[key]`,
or `[value]`, for example `items[].name`; these denote static declaration positions,
not an index from an instance. Repeated model types at distinct tuple positions
retain separate diagnostics. Reflection-only wrappers also expose nested model
declarations already present in the graph.

The downstream `rs-platform` testkit exercises real `CredentialInfo` under an
optional wrapper and at two separate paths. Its full `PersonInfo` graph resolves,
but `delete_time` carries a Time constraint that this backend rejects during plan
construction—even if a particular instance contains `None`. A complete PersonInfo
validation service still needs a time adapter; removing the declaration would
change the model's contract.

## Advanced usage: stopping policies and work budgets

Use `ValidationOptions::default()` for the default policy. For customization,
use `ValidationOptions::builder()`, set `mode`, `selection` and the desired
`max_*` budgets, then call `build()`. The independent `ValidationOptionsBuilder`
owns the configuration and transfers it on completion; the former `with_*`
configuration methods have been removed.

`ValidationOptions` defaults to CollectAll, all fields, depth 64, 100,000 nodes,
100 retained violations, and 1,000,000 selector comparisons. All budget setters
require `NonZeroUsize`. Field selection matches the complete bound field path, ignoring collection indices;
select `contact.name` for that nested field. Selecting `contact` alone does not select
its descendants. `FieldPath::from_segments` owns each supplied name without
splitting dots inside a segment. To select model-level rules with `Fields`,
include an empty segment sequence; ordinary field paths do not select those
rules. Selected model rules run before selected field rules. Selection is an
execution policy and cannot bypass declaration or binding errors.

- FailFast retains exactly the first violation, including one returned as a failed
  prerequisite. A rule returning several violations cannot bypass this policy.
- The violation cap applies to the entire report, including prerequisites. Reaching
  it stops all later getters, rules, dependencies, and selector elements.
- A policy stop returns `Ok(report)` with `is_truncated() == true`. A legal skip
  alone does not trigger FailFast. An external validator's own result allocation
  is not limited by the report cap.
- Nodes count the root once, each actual property/dependency/element read once,
  and each rule invocation once. Repeated reads count repeatedly.
- Depth counts property and element path segments plus parent hops for a dependency.
  Each access is checked before it happens.
- Comparisons count selector element rule invocations, not comparisons inside a
  custom validator. Exhaustion of depth, nodes, or comparisons returns
  `TraversalLimit` with the partial report. Accounting cannot wrap on overflow.

Inspect `ModelValidationError::error()` and `partial_report()` together. Its root,
owner, occurrence, field location, and declaration identify the failed operation;
dependency object navigation and property selection have separate getters. The
`Error::source()` chain retains the original execution/adapter cause.

## Errors and troubleshooting

Do not include registrations that replace built-in rule IDs. Such a collision
returns a root-level `InvalidDeclaration` whose `rule()` identifies the ID.
The same failed build still reports independent unsupported shapes and missing
rules; resolve all reported causes before rebuilding the plan.

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
checked v7 facade over rs-reflect codegen v3; applications should use public APIs.

Database access, random object creation, physical indexes, filter generation,
Unicode comparison execution, and missing-parent business fallback remain
consumer responsibilities. See the [frozen requirements](../derive/doc/rs-model-derive-requirements.zh_CN.md),
[design](../derive/doc/rs-model-derive-final-design.md), and local API docs generated
by `cargo doc --workspace --all-features --no-deps`.

## Further reading

- [README](../README.md)
- [简体中文用户指南](user_guide.zh_CN.md)
- [Declaration guide](../derive/doc/user_guide.md)
