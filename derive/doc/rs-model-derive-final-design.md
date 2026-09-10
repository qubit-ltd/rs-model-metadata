# Model Metadata and Derive Refactoring Design

Version: 2026-09-10, execution-correctness revision. This design preserves the user-confirmed [frozen requirements](rs-model-derive-requirements.md) and specifies the checked v7 identity and validation contracts below. The [Chinese edition](rs-model-derive-final-design.zh_CN.md) describes the same contract. Implementation and verification status are tracked separately in the [coverage ledger](rs-model-derive-requirements-coverage.zh_CN.md); documenting a contract does not certify that final CI has passed. The earlier [gap assessment](rs-model-implementation-gaps.zh_CN.md) and [implementation plan](../../doc/plans/2026-09-10-model-metadata-plan.md) remain dated historical evidence.

This replaces the historical design. It preserves the existing reflection foundation while restoring required generated capabilities and removing product-specific query policies.

## Architecture

One qubit-reflect descriptor owns Rust structure, generic definitions, invocation, and typed capabilities. Model metadata is its immutable domain overlay. Derive parses declarations, preserves origins, normalizes semantics, validates locally knowable facts, and generates reflection delegation, metadata, and default capabilities.

Known-type static queries do not initialize model registries or invoke getters. ModelRegistry is a stable-ID index over an explicit reflection snapshot. StructureResolver receives that registry plus explicit static roots and returns a read-only graph or deterministic structured errors. Optional adapters bind explicitly supplied strategy registries. Query execution, object generation, persistence, and instance-parent fallback belong to consumers.

## Generated capabilities and output

All roles default to Clone, Debug, Display, PartialEq, Eq, Hash, Redact, Serialize, and Deserialize. Only unit-only enums default to Copy. Default, PartialOrd, and Ord require opt-in. Retain the confirmed Eq/Hash and ordering switch dependencies. Visible derives suppress duplicate generation; implementations in separate impls require explicit no_* switches. Generate bounds based on actually accessed fields and serialization direction.

Redact generation and structural output use qubit-redact projections, not an independent masking engine. Debug and Display use borrowed redacted views; Serialize delegates the structural projection instead of serializing an already encoded JSON string. Deserialize uses the original Serde input contract. Transparent Values keep nominal Debug identity and inner Display/Serde representation.

Named Option/standard collection fields receive implicit omission and missing-value defaults only where explicit Serde configuration does not override the corresponding policy. keep_serializing suppresses only implicit omission. Redaction cannot be bypassed by custom output; reject combinations that qubit-redact cannot safely compose, while preserving supported callbacks on ordinary/skipped fields.

Skip applies to whole fields of every supported shape. Disabled redaction restores them subject to independent Serde skips. Empty outer representations remain serializer-defined; redacted output need not round-trip. Map-key collisions fail. no_redact does not disable nested types' own output behavior. The runtime facade exposes the controlled Serde/redact dependencies needed by expansion without adding validator/codec execution dependencies to default features.

## ModelImpl and Property

Delegate the full impl to reflect_impl after consuming model-only markers. Trait and generic impl reflection follows reflect's existing capability boundaries; non-invocable methods retain their reasons. Only eligible public synchronous safe non-generic inherent methods contribute Properties.

The exclusion syntax is `#[model_property(skip)]` on a method inside ModelImpl. It removes only that method's Property contribution, preserves reflection, and does not remove a same-named storage field. Other parameters, duplicate markers, and non-method placement fail. There is no computed macro and getters do not become new declaration sites for domain field attributes.

Multiple ModelImpl blocks register independent checked capabilities. Aggregate only providers visible in the supplied reflection snapshot, validate their owner and accessors, and cache by owner TypeId and provider set. Generic method contributions use reflect_impl explicit specializations.

Assemble field/getter/setter fragments deterministically. A field supplies the logical value type; otherwise derive the compatible value type from the getter and verify the setter. Preserve T/&T, String/str, Vec/slice, and Option<T>/Option<&T> compatibility. Explicit accessors win. Conflicts return PropertyBuildErrors rather than panicking. Invocation preserves reflect ownership, lifetime, and Local/ThreadSafe boundaries; static metadata does not require instances to be Send+Sync.

## Object navigation and validator declarations

PropertyPath remains a dotted property selection. Add NavigationStep::{Property(&'static str), Parent} and ObjectPath::steps() -> &'static [NavigationStep]. ReferenceMetadata::path() returns Option<&'static ObjectPath>, replacing the old same_as PropertyPath representation.

Object paths are nonempty relative slash-separated steps supporting repeated `..`; reject absolute paths, empty steps, and standalone `.`. Existing dotted reference paths migrate once. Missing reference.path means no requested binding reuse. Crossing reference declarations describes the full Entity binding; actual instance reuse belongs to consumers.

ResolveError::object_path() retains typed object navigation separately from the field or selected Property path. A local child/.. round trip needs no external parent. Structural resolution checks known local dependency existence and readability; unavailable external parents remain context requirements. DependencyBindingMetadata::object_path() returns the immutable Copy value.

Validator dependency syntax is:

```rust
#[validator(
    id = "qubit.identity.card",
    depends_on(
        gender,
        birthday(path = "..", property = birthday),
    ),
    params(strict = true),
)]
identity_card: String,
```

Bare dependencies normalize to the same-named slot/property in the current object. Structured `slot(path = "../owner", property = profile.birthday)` uses a required property and optional object path. Dotted token/string property forms normalize to PropertyPath. Old named assignment syntax migrates to the structured slot form. Each binding exposes name(), object_path(), property(), and declaration(). DeclarationLocation includes field index, optional variant index, selector, and source location.

Reject duplicate slots or duplicate full navigation/selection within an occurrence; preserve repeated validator IDs across occurrences in source order. Field/selector declarations start from the owning object; declarations inside element types start from the element. Collections add no domain-parent level. Validator paths read explicitly supplied instances and do not fetch unavailable Entities merely because a field is a reference.

## Generic definitions, roots, and graph

GenericModelMetadata::model_id() becomes Option<ModelId>. Every generic declaration generates a definition overlay/provider, including declarations without IDs. Only definitions with IDs enter the model stable-ID index. Concrete instances retain no synthetic ID or linked registration but always link to their generic definition.

Cache concrete metadata by real TypeId, publish only complete immutable values, and do not hold a model cache lock across recursive type traversal. Use reflect's lazy TypeRef edges for cycles. Initialization failures are preserved, not replaced with missing metadata. Basic integer/bool/char const generics and direct parameter references remain supported; no complex const evaluator is added.

ResolveInputs<'a> contains `models: &'a ModelRegistry<'a>` and `roots: &'a [&'static TypeMetadata]`. Resolution starts from registered concrete entries plus explicit roots and traverses reachable model fields, variants, and declared relationships. Deduplicate by TypeId. FieldLocation uses owner TypeId, optional variant index, and field index, supporting tuple payload diagnostics.

Dependencies with unknown instance parents retain a resolved prefix and an explicit ContextRequirement::ParentObject. ResolvedDependency exposes its declaration and context requirement. This is neither a resolved value nor an invalid declaration. Graph-owned aggregates borrow the registry snapshot; static metadata never borrows graph-owned temporary storage. Errors preserve kinds, causes, available IDs, type information, structural locations, and deterministic ordering.

## References and constraints

Reference compatibility first attempts exact compatible value matching, then traverses supported wrappers on the stored field while preserving shape. Do not compare Vec<Id> directly with Id or unconditionally strip containers belonging to the selected target property itself. Direct Map references remain unsupported.

Enum payloads may declare references and retain variant-local FieldMetadata. Entity/Projection payloads require explicit reference semantics. Value closure traverses Enum payloads and wrappers and rejects references or Entity/Projection/Model roles. Opaque must not hide known model roles.

Retain the sealed exact Id bound, including true aliases. Preserve explicit ignore_case separately from its effective capability-dependent value: plain non-text unique is valid, explicit non-text ignore_case is invalid. Keep the confirmed indexed, Set/array, and Eq/Hash restrictions. Type and usage-site constraints remain separate and cumulative. No getter-level constraints are introduced.

## Strategy adapters and query view

Reuse validator/codec execution contracts rather than defining parallel traits. Static metadata contains declarations without public execution-crate types. Registration validates codec bounds; optional binders check exact occurrence targets and explicit/canonical priority.

ValidationPlan identifies roots by TypeId plus optional ModelId, removing required-ID expects. Compile dependency prefixes and retain context-dependent suffixes. Missing execution context produces structured dependency errors rather than panic or silent skipping; consumers supply instance navigation. The old target/on_none/validate_nested macro extensions are outside the current contract and migrate to established selector/Option semantics or downstream execution configuration. The runtime also removes FieldAttributeMetadata::ValidateNested and FieldMetadata::validate_nested(): the shared declaration walker discovers nested execution work according to the support matrix and reference/opaque boundaries, without a per-field recursion flag.

Remove product filter expansion and flat-name errors from StructureResolver. Move unique-scope existence/readability checks into structural relations so they are not lost. QueryMetadata::declarations() returns QueryDeclaration entries exposing field(), reasons(), and path(). Include indexed identifiers, global unique fields, and references. Consumers navigate nested metadata/edges themselves; the graph does not materialize infinite expanded paths. Remove filters/filter_by_flat_name from the target API.

## Public surfaces and ABI

Reuse existing TypeMetadata static queries, FieldMetadata, Property views, role navigation, ModelId/ModelRegistry, and explicit graph navigation, subject to the frozen semantics and the changed signatures above. The interface table below records the public families and their absence/error/ownership behavior. Single missing declarations return None; collections return empty borrowed views; initialization and assembly errors remain distinguishable from absence.

Generated constructors use synchronized `__private::v7` in metadata and derive, retaining checked construction, capability validation, and the reflect definition_provider_v2 contract. Do not change reflect's own ABI version or guess internal provider names. No v5/v6 shim remains; normal, renamed-runtime, and hand-written ABI fixtures migrate together.

Macros diagnose local syntax and output conflicts, Rust bounds diagnose exact type/capability requirements, registries diagnose IDs/ABI, resolvers diagnose structural relations, and adapters/consumers diagnose execution inputs and outcomes. Runtime getters are never called to validate static declarations.

## Migration and verification

Implementation order is shared APIs, default output and ModelImpl, declaration semantics, generic/anonymous graph, adapters, real consumers, and final integration. T2/T3 can proceed independently after T1; shared graph work follows declaration and Property contracts.

Migrate duplicated derives to generated capabilities or explicit opt-outs, dotted reference paths to slashes, old dependency forms to structured slots, and filter API consumers out of metadata. Recheck the 131 current rs-platform declarations with actual compilation rather than treating textual counts as proof.

Acceptance covers real output contents, safe borrowed getters, trait/generic impl reflection, anonymous/generic roots and concurrency, wrapped/Enum references, Value closure, mixed local/parent validator dependencies, repeated IDs, codec bindings, and valid models previously rejected by flat-name policies. Test genuine unique-scope failures remain rejected. Cross-crate and renamed-runtime fixtures, rustdoc, feature builds, Clippy, and downstream checks close the per-requirement ledger. No execution of this plan is implied by documenting it.

## Concrete identity, ownership, and public interfaces

| Interface family | Contract and missing/error behavior | Implementation ownership |
| --- | --- | --- |
| TypeMetadata | Static descriptor overlay; optional ModelId; fallible metadata queries retain ABI/capability failures | `src/type_metadata.rs`, `src/type_metadata/`, `src/role.rs` |
| FieldMetadata / FieldLocation | Stored field and provenance; concrete owner/variant/index identity; definition-only fields have no location | `src/field_metadata.rs`, `src/relation/field_location.rs` |
| Property | Assembled storage/accessors; missing property differs from assembly failure; borrowed values retain instance lifetime | `src/property.rs`, `src/local_property_set.rs`, property error types |
| Role payloads and generic definitions | Entity identifier, Projection source, Enum variants, Value codec; exact concrete TypeId links definitions | `src/type_metadata/`, `src/generic/` |
| Declaration vocabulary | Ordered constraints, validators, codec references and separate object/property paths; absent singleton returns None | `src/metadata/`, `src/relation/`, `src/metadata_vocabulary.rs` |
| ModelRegistry | Immutable reflection snapshot; fallible initialization; stable-ID and exact-TypeId indexes; ABI cause is typed | `src/registry/` |
| ModelGraph | Borrowed registry; owned resolved views and deterministic model sequence; absent lookup returns None | `src/resolve/` |
| Optional execution adapters | Explicit registries, typed input checking, binding diagnostics and execution reports | `src/codec/`, `src/validation/` |

`FieldLocation` is Copy, Eq, and Hash with private `owner: TypeId`,
`variant: Option<usize>`, and `index: usize`. A concrete field, including an
anonymous or generic instance field, always has a location. A generic definition
field has None. Generation supplies `TypeId::of::<Self>()` directly, avoiding
recursive metadata initialization. Checked v7 verifies owner, index, variant,
reflection ordering, and descriptor identity. Malformed generic-only variants in
concrete enum metadata return ABI errors; duplicate-attribute accounting must
not overflow at 256 occurrences.

The graph indexes references by FieldLocation and projections/queries by TypeId.
Copying a field must not change the lookup result. Persist ModelId externally:
TypeId and FieldLocation are only valid in the current process. The following
is an **interface specification**, not a standalone Rust program:

```text
FieldMetadata::location() -> Option<FieldLocation>
FieldLocation::{owner(), variant(), index()}
ModelGraph::model(TypeId) -> Option<&'static TypeMetadata>
ModelGraph::reference(FieldLocation) -> Option<&ResolvedReference>
ModelGraph::projection_source(TypeId) -> Option<&ResolvedProjectionSource>
ModelGraph::query(TypeId) -> Option<&QueryMetadata>
ValidationCapabilities::check(root, &graph) -> Result<(), ValidationBuildErrors>
```

Validation root arguments select graph metadata by exact TypeId. The graph's
canonical overlay owns both declaration discovery and the plan's stored root;
an alternate caller overlay cannot erase graph constraints or import declarations
from another snapshot. This applies equally to capability checks and plan builds.

## Complete declaration discovery and binding

`validation_plan.rs` owns the public plan and explicitly declares its `build`
and `execute` implementation modules. Private declaration, binding, path-reading,
budget and report types live in `validation/internal`. Structural resolution
neither binds validators nor invokes getters. Building or executing a plan only
uses metadata/property views in the supplied graph; it does not fill missing
nodes from a global registry.

One declaration walker serves capability checks and plan binding. It builds a
finite TypeId dependency graph and propagates reachable execution work backwards
using a monotone worklist. Then a source-order depth-first walk emits one
occurrence per use path, with active-path cycle detection. No-work cycles terminate
without refusal; work-bearing recursive instance paths are explicitly rejected.
Type analysis may be cached by TypeId, but `left.child` and `right.child` are
separate uses and cannot be deduplicated by type.

Each occurrence retains root and owner TypeIds, original field/FieldLocation,
DeclarationLocation, complete use path, optional selector, declaration ordinal,
and the original constraint or custom validator. Named field order is preserved;
at one location standard constraints precede custom rules. Repeated rule IDs
remain separate. Unsupported enum and unnamed tuple payloads must retain variant
and field coordinates, not disappear into an empty successful plan.

Structural wrapper traversal uses active-ancestor cycle detection, not global
target-type deduplication: `(Child, (Child, Child))` contributes three uses in source
order. Unsupported diagnostic paths retain variant names and numeric tuple/unnamed
field positions (`choice.First.name`, `pair.1.0.name`). Container suffixes `[]`,
`[key]` and `[value]` denote static element positions, not instance indices. Raw
reflection-only struct and enum wrappers are traversed to reach graph-owned models.
These structural diagnostic paths are kept separate from executable property paths.

The binder shares actual getter-shape checking across root, nested, and selector
paths. Named text constraints and custom validators execute on directly borrowed
or optional nested models. Explicit element validators need borrowed-slice access;
sequence item count needs its actual length adapter. Owned intermediate results,
implicit Option/pointer element unwrapping without an adapter, selector constraints
or dependencies, MapKey/MapValue, Decimal/Time/Map constraints, erased uniqueness,
constrained enum/tuple/newtype interiors, and model work inside container elements
are rejected. Reference traversal stops at the stored field; opaque traversal
stops internally while retaining explicit outer rules. Unit enums and no-work
payloads remain ordinary values. The [runtime support matrix](../../doc/user_guide.md#limitations-execution-support-and-explicit-refusal)
is the user-facing version of these boundaries.

`ValidationCapabilities::check` runs declaration and access checks without calling
getters or binding a custom registry. Success is not proof of registered rules or
valid instances. Build binds all supported occurrences and aggregates independent
unsupported/binding errors in source order. Missing roots return RootNotInGraph.
Unknown parent types preserve deferred dependency navigation; supplied parent
metadata and instances use nearest-parent-first order, with graph-only lookup.

## Execution algorithm, budgets, and typed diagnostics

One ExecutionBudget and one ReportAccumulator serve the whole validation call.
After exact root type checking, model rules run in insertion order, then field and
selector bindings in declaration order. Execution occurrence IDs distinguish model
rules and each expanded field rule. Every loop checks the global stopped state
before a getter, dependency read, rule invocation, or element visit.

The accumulator rejects invalid outcome contracts: Invalid requires a nonempty
violation list, MissingOptional requires empty prerequisites, and FailedPrerequisite
requires nonempty prerequisites. All violations—including failed prerequisites—go
through one append path. FailFast retains exactly one; the report cap bounds the
whole report. A policy stop sets truncated and returns Ok(report). A legal skip
alone does not stop execution. External validators remain responsible for their
own allocations and internal work. An infrastructure failure immediately returns
its typed cause and all results collected so far; stopped execution cannot be
replaced by a later getter failure.

| Budget | Default | Charged before the operation |
| --- | ---: | --- |
| Depth | 64 | Property and element path segments, including parent hops for dependencies |
| Nodes | 100,000 | Root once; each actual property/dependency/element read and rule invocation once |
| Violations | 100 | Every retained violation, including prerequisites; reaching the cap stops normally |
| Comparisons | 1,000,000 | Each selector element rule invocation; not comparisons inside custom code |

All configured limits are nonzero. Checked accounting prevents overflow and charges
repeated reads repeatedly. Traversal exhaustion returns TraversalLimit plus the
partial report, distinct from successful policy truncation. Field selection matches
complete field-name paths, ignoring collection indices; it is not prefix expansion.

ValidationBuildError exposes root_type_id, owner_type_id, optional model ID,
FieldLocation, DeclarationLocation, occurrence, path, selector, declared_rule_id,
original constraint, kind, and source. A missing registration still retains the
raw custom ID. A standard constraint retains its original declaration and its
mapped rules through `constraint_rule_ids()`, including all rules of a compound
constraint even before binding. `rule()` identifies only a concrete binder failure;
no known constraint mapping yields an empty ID list. Binding and diagnostics use
one canonical mapping. UnsupportedExecution and RootNotInGraph do not fabricate
a validator source. Display is concise and contextual; Debug never traverses the
whole graph or includes model instance values.

Caller registrations cannot replace built-in rules. A conflicting registration
produces a root-level InvalidDeclaration carrying the rule ID; the canonical
built-in remains available to check other declarations. Construction still
reports independent unsupported shapes and missing rules, then rejects the
plan. Root-level registry diagnostics precede declaration-ordered failures.

ModelValidationError exposes the original ExecutionError, partial_report, root,
owner, occurrence, field/declaration provenance, and separate dependency object
and property paths. Its Error::source preserves the execution/adapter cause.
ModelRegistryError retains typed AbiViolation through abi_cause and Error::source,
as well as pre-existing capability origins and reflection cause chains.

## Migration and verification for this revision

Replace pointer/field-reference graph keys with FieldLocation or TypeId; replace
selector-only `supports` with whole-root `check`; use declared_rule_id for missing
custom registrations; regenerate all metadata with v7. Configure validation with
`ValidationOptions::builder()` and the independent `ValidationOptionsBuilder`:
methods use setting names without `with_`, and `build(self)` transfers the
completed options. `ValidationOptions::default()` retains the documented defaults;
the previous configuration setters are removed. Runtime and derive remain
unpublished 0.1.0 packages. No global validator-plan cache is introduced.

The real downstream boundary is CredentialInfo nested under an optional testkit
wrapper and at two use sites. The full PersonInfo graph still resolves; its
`delete_time` Time declaration is rejected at plan construction even for None
instances. Do not remove that constraint or add opaque to obtain a green test.

Keep property_output benchmarks and add 1/8/32-field independent-impl cold assembly,
warm property queries, graph building, plan binding, and execution measurements.
Prepare metadata and inputs outside timed loops; report cold observations separately.
Only repeatable measured benefit justifies an additional cache. Performance results
are evidence, not correctness time thresholds.

Remove all 31 runtime file-coverage exemptions without lowering thresholds. The two
derive exemptions require fresh instrumentation evidence and alternative tests.
Keep default, validation-only, codec-only, generic-only, all-feature, UI, renamed
runtime, cross-crate, Clippy, Rustdoc, and downstream checks. Execute the actual
bilingual README/guide Rust fences through `documentation_examples_tests`, rather
than compiling an approximate copy. Formal interface and declaration fragments are
specifications, not independent runnable programs. Full source/item/style review
and final CI remain separate acceptance gates recorded in the ledger.
