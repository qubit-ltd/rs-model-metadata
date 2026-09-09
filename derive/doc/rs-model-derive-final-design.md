# Model Metadata and Derive Refactoring Design

Version: 2026-09-10. This target design implements the user-confirmed [frozen requirements](rs-model-derive-requirements.md); it does not claim current implementation support. The [Chinese design](rs-model-derive-final-design.zh_CN.md) maintains the full interface and algorithm specification. See the [evidence-based gap assessment](rs-model-implementation-gaps.zh_CN.md) and [implementation plan](../../doc/plans/2026-09-10-model-metadata-plan.md).

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

ValidationPlan identifies roots by TypeId plus optional ModelId, removing required-ID expects. Compile dependency prefixes and retain context-dependent suffixes. Missing execution context produces structured dependency errors rather than panic or silent skipping; consumers supply instance navigation. Existing target/on_none/validate_nested extensions are not silently promoted into the frozen model contract and migrate to established selector/Option semantics or downstream execution configuration.

Remove product filter expansion and flat-name errors from StructureResolver. Move unique-scope existence/readability checks into structural relations so they are not lost. QueryMetadata::declarations() returns QueryDeclaration entries exposing field(), reasons(), and path(). Include indexed identifiers, global unique fields, and references. Consumers navigate nested metadata/edges themselves; the graph does not materialize infinite expanded paths. Remove filters/filter_by_flat_name from the target API.

## Public surfaces and ABI

Reuse existing TypeMetadata static queries, FieldMetadata, Property views, role navigation, ModelId/ModelRegistry, and explicit graph navigation, subject to the frozen semantics and the changed signatures above. The Chinese design's module table assigns every public interface family and its absence/error/ownership behavior. Single missing declarations return None; collections return empty borrowed views; initialization and assembly errors remain distinguishable from absence.

Publish changed hidden constructors through synchronized `__private::v6` in metadata and derive, retaining checked construction, capability validation, and the reflect definition_provider_v2 contract. Do not change reflect's own ABI version or guess internal provider names. During migration v5 may support old fixtures; final generated code uses v6 consistently.

Macros diagnose local syntax and output conflicts, Rust bounds diagnose exact type/capability requirements, registries diagnose IDs/ABI, resolvers diagnose structural relations, and adapters/consumers diagnose execution inputs and outcomes. Runtime getters are never called to validate static declarations.

## Migration and verification

Implementation order is shared APIs, default output and ModelImpl, declaration semantics, generic/anonymous graph, adapters, real consumers, and final integration. T2/T3 can proceed independently after T1; shared graph work follows declaration and Property contracts.

Migrate duplicated derives to generated capabilities or explicit opt-outs, dotted reference paths to slashes, old dependency forms to structured slots, and filter API consumers out of metadata. Recheck the 131 current rs-platform declarations with actual compilation rather than treating textual counts as proof.

Acceptance covers real output contents, safe borrowed getters, trait/generic impl reflection, anonymous/generic roots and concurrency, wrapped/Enum references, Value closure, mixed local/parent validator dependencies, repeated IDs, codec bindings, and valid models previously rejected by flat-name policies. Test genuine unique-scope failures remain rejected. Cross-crate and renamed-runtime fixtures, rustdoc, feature builds, Clippy, and downstream checks close the per-requirement ledger. No execution of this plan is implied by documenting it.
