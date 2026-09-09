# `rs-model-derive` Final Requirements

[中文版](rs-model-derive-requirements.zh_CN.md) | [Coverage ledger](rs-model-derive-requirements-coverage.zh_CN.md)

## Status and terminology

Status: frozen after consolidated user confirmation on 2026-09-10. See the
[scope and closure review](rs-model-requirements-freeze-review.zh_CN.md).

This document is the English companion to the Chinese final requirements. The
two documents describe the same target contract; requirement IDs and behavioral
semantics are maintained in the Chinese document. Complete Rust signatures are
defined in the final design and API reference, which must satisfy these
requirements. “Must”, “must not”, “should”, and “may” are normative terms.

Historical discussion is non-normative. The final design, API documentation,
implementation, and tests must conform to the requirements; existing code does
not redefine required behavior.

## System contract

`qubit-model-derive` provides five role attributes—`Entity`, `Projection`,
`Model`, `Enum`, and `Value`—and the model implementation attribute `ModelImpl`. Every role
uses one parse → IR → normalize → validate → expand pipeline and delegates
structural reflection to `qubit-reflect`. Generated model metadata is a typed
capability of that same reflection descriptor; it is not a second type graph.

Generated code must use the versioned private facade of
`qubit-model-metadata`. Applications do not need a direct `qubit-reflect`
dependency. A renamed runtime dependency must work. Missing-runtime diagnostics
must not hide independent declaration errors.

## Roles

- `Entity` is a non-generic named struct with exactly one `#[identifier]` field
  of type `qubit_id::Id`. Database-assigned identifiers are Entity-only.
- `Projection` is a non-generic named struct with exactly one `Id` identifier.
  It is either open or names one fixed source by Rust type or stable model ID.
- `Model` accepts named and unit structs, but not tuple structs.
- `Enum` accepts enums and preserves distinct Rust, canonical model, and Serde
  variant names. Explicit canonical names must be unique.
  Variant indices follow the current declaration order, not a stable cross-version
  persistence code (REQ-VAR-003).
  Payload fields may declare explicit references; Entity/Projection payloads must
  do so. Enums have no identifier or independent persistence identity. An enum
  containing references cannot enter a Value's transitive field closure (REQ-ENUM-003,
  REQ-VAL-004).
- `Value` accepts a named struct or a one-field tuple/newtype. `transparent`
  requires exactly one field and uses the inner representation.
- `Model`, `Enum`, and `Value` may use type parameters, where clauses, and
  supported primitive const generics. Model roles do not accept lifetimes.

The role attribute must precede user derives. It must recognize explicit
derives visible on that declaration and avoid generating duplicate capabilities.
Implementations supplied by separate impl blocks or other macros require the
corresponding capability switches to disable automatic generation; output
capabilities must also satisfy the redaction contract. Impl reflection can record
methods and trait implementations, but those facts do not automatically suppress
trait generation by the type-declaration macro (REQ-ROLE-008).
Default capabilities are `Clone`,
`PartialEq`, `Eq`, `Hash`, `Redact`, redaction-aware `Debug`, `Display`,
`Serialize`, and `Deserialize`. Corresponding `no_*` switches disable them;
`copy`, `default`, `partial_ord`, and `ord` are opt-in. Unit-only enums are
`Copy` unless `no_copy` is present. Conflicting capability combinations must
produce compile-time diagnostics.

## Fields and properties

A field is a real storage slot. A property is a name-based view assembled from
a field, getter, and setter. Property merging must be deterministic, preserve
borrowing, and reject incompatible getter/setter types and conflicting accessor
contributions. Multiple `ModelImpl` blocks may contribute distinct accessors to
the same type; their metadata must be merged.

`ModelImpl` extends impl reflection with model semantics. It provides the
reflection capabilities of `reflect_impl`, including its inherent/trait impl,
generic, and dynamic-invocation boundaries. Only public, safe, synchronous,
non-generic methods matching getter/setter shapes on inherent impls contribute
properties. Other methods retain reflection metadata without being rejected for
not being properties. Trait impls record implementation and method information
without automatically contributing properties. Non-invocable methods retain
the reflection contract's unavailable reasons (REQ-PROP-001).
Getters use `&self` and may return owned values, `&T`, `&str`, `&[T]`, or
`Option<&T>`. Setters use `&mut self`, accept one owned value, and return unit.
Generated adapters must have collision-resistant deterministic names.

## Identity, indexes, relationships, and keys

Indexed fields provide metadata for downstream query-filter generation. A future
consumer could interpret `nickname = "abc"` as substring matching, like SQL
`LIKE '%abc%'`, and produce `min_age`/`max_age`, `min_birthday`/`max_birthday`,
or `min_create_time`/`max_create_time` for values suitable for ordered comparison.
These are use cases, not a requirement that this crate generate filter objects,
SQL, or comparison algorithms. This crate records meaningful indexed declarations,
indexing reasons, source paths, and types. Parameter naming, operations, interval
endpoints, defaults, adapters, and execution belong to future consumer designs
(REQ-QRY-016).

An identifier must be a direct field whose exact Rust type is `qubit_id::Id`.
True aliases of that type are accepted; unrelated types with the same name,
newtype wrappers, optional/container types, and nested field paths are not
identifiers (REQ-ID-002).

`identifier`, `indexed`, `unique`, and `reference` describe storage identity,
querying, and inter-model relationships. References may store a complete Entity,
its `qubit_id::Id`, or another selected Property value, and may name a target by
Rust entity type or stable entity ID. Cross-model target,
role, and property checks belong to the resolver because only the complete
linked model set can establish them.

Reference `path` navigates entity bindings in the current object-instance
context using `/` separators and `..` for parent navigation, including
`street/district`, `../country`, and `../../province/country`. The final syntax
uses `/` uniformly; existing dot-separated reference.path declarations must be
migrated during reconstruction. Metadata distinguishes property navigation from
parent navigation as structured steps rather than requiring consumers to parse
raw strings. Crossing a reference follows its
bound full Entity rather than merely its stored ID or Projection. The endpoint
must match the declared target Entity; `property` then selects the value using
its separate dot-separated property path. Ordinary Property paths retain `.`;
validator object navigation and property selection follow REQ-VLD-005. Parent navigation
requires an object-graph context and cannot be resolved solely from a type
registry. For an order containing a list of order items, an item's `path = ".."`
refers to the order; the list is not an additional domain parent object.
Ordinary relative paths start at the current object without a separate `.`
marker. This crate preserves declarations and navigation semantics; consumers
resolve instance bindings and missing-parent fallback. For example, a generator
may prepare and, where needed, persist a new order when generating an order item
without a parent context, as in the Java implementation. This does not require
the metadata crate to create objects, access databases, or select a fallback policy.

`key_part` describes an ordered logical key for value semantics. It is allowed
only on real named fields of `Model` and `Value`; it is rejected on `Entity`,
`Projection`, `Enum`, and tuple/newtype values. A key may select a subset of
fields. Selected orders must be unique and contiguous from zero. These rules
are independent from Entity identity.

Query-specific leaf expansion and flat-name collision checks belong to consumers,
not to this crate's macros or StructureResolver. This crate validates declaration
types and relationships and preserves indexing reasons and structured paths.
Explicit indexed combined with an implicit indexing reason remains a redundant
declaration error (REQ-QRY-003, REQ-ERR-004).

For text-capable fields, `unique` defaults to `ignore_case = true`; explicit
`false` selects case-sensitive comparison. Non-text fields use their value
comparison semantics, and explicitly setting `ignore_case` on them is an error.
Text capability is determined by type capability, including Value types that
explicitly provide it, rather than by the spelling `String` (REQ-UNQ-003).
Case-insensitive comparison uses locale-independent Unicode default case
folding without implicit trimming, accent removal, or Unicode normalization.
The implementation contract fixes a shared Unicode version. Persistence
consumers that cannot implement equivalent comparison must report unsupported
semantics rather than silently substituting database-default collation.

## Declarative constraints and selectors

Text, decimal/money, temporal, sequence, and map constraints must preserve
their declared parameters and reject duplicate singleton options. Minimums
must not exceed maximums. Decimal bounds use canonical non-exponential decimal
strings: an optional leading minus is accepted, a leading plus is rejected,
leading zeroes are normalized, one decimal point is allowed, and `1.` is
equivalent to `1`.

`element`, `map_key`, and `map_value` select one non-recursive container
position. Selectors may carry the allowed constraint, validator, and redaction
metadata. Constraint target types are checked in generated code. Syntax-only
recognition of standard `Option`, text, and collection types must use canonical
paths and must not treat `domain::Option`, `domain::String`, `domain::Vec`, or
similar lookalikes as standard containers.

## Validators, codecs, opaque values, and output safety

Type-level constraints and additional field/selector constraints apply together;
usage-site declarations cannot replace or cancel the type's own constraints.
Metadata retains each declaration and its origin. Consumers execute the combined
constraints; this crate need not solve arbitrary constraint satisfiability
(REQ-SEL-009).

Repeated validator IDs are allowed at a field or selector. Each declaration is an
independent occurrence, potentially with different parameters or dependencies,
preserved and consumed in source order without ID-based merging (REQ-VLD-006,
REQ-SEL-002).

Validator dependencies must support paths and parent-object navigation, following
the structured property/parent navigation approach of reference.path. Metadata
preserves declarations; consumers provide instance navigation and dependency values.
Each dependency independently selects an object with a slash-separated path
supporting `..`, then selects a property with an ordinary dot-separated Property
path. Omitting the object path selects the current object. Field and selector
declarations start from the field's owning object; declarations inside an element
type start from that element object. Collections add no domain-parent level.
Metadata retains structured navigation steps, property selection, and declaration
location (REQ-META-084). Navigation reads the explicitly supplied object graph;
it does not automatically follow reference bindings to fetch unavailable Entities.
Instance navigation and missing-parent handling belong to consumers. These semantics
are confirmed; concrete macro parameter syntax belongs to the design stage, and
existing depends_on examples only illustrate local dependencies (REQ-VLD-005).
For example, one occurrence may depend on local gender and on birthday selected
from the parent object with path `..`. Structural
checks validate what explicit type context can establish; parent-instance-dependent
parts remain pending contextual resolution rather than being rejected solely for
lack of parent information in the registry (REQ-VLD-008).

A validator occurrence has a stable ID, ordered named parameters, and readable
dependency property paths. Duplicate parameter names and duplicate dependency
paths are invalid. Empty parameter arrays are invalid because their element
type cannot be inferred. Integer overflow must be diagnosed at its source
span. Structural resolution validates dependency paths; binding to an
executable validator occurs later when `ValidationPlan::build` consumes the
structure graph and a validator registry.

Model fields reference validators uniformly by stable ID. Registrations associate
the ID, executable implementation, and exact supported input type; execution
trait signatures belong to the validator component contract. Structural
resolution checks dependency existence and readability. The optional validation
adapter checks input types, arguments, and dependency values during binding
(REQ-VLD-007, REQ-VLD-008). The old model-level `with = RustType` requirement is
withdrawn (REQ-ERR-006).

A Rust codec declaration records only a stable Rust type identity. With the
optional `codec` feature enabled, `bind_codecs` resolves Rust identities and
stable codec IDs through `ValueCodecRegistry` and checks the registered value
type. Registration checks encoder, decoder, and construction trait requirements
at compile time; field macros only record declarations. Rust-type codec
references also require matching registrations, and the optional adapter checks
the target value type of each occurrence (REQ-CODEC-002, REQ-CODEC-005,
REQ-CODEC-006). `opaque` preserves intentionally unavailable structural type information
without pretending metadata is missing.

Codec selection uses explicit field declaration, then type canonical codec,
then no codec. Explicitly selecting the same codec as the type canonical codec
is valid and remains a field-level selection (REQ-CODEC-007).

Opaque fields cannot also be identifiers or references, and cannot hide Entity,
Projection, or Model roles to bypass role checks (REQ-OPAQUE-004).
Opaque fields may retain indexed/unique declarations together with their type
identity. Comparison, filter-generation, and persistence adapters are defined
and checked by the relevant consumers, not required uniformly when this crate
accepts declarations (REQ-OPAQUE-005).

Automatically generated `Debug`, `Display`, and `Serialize` must apply redaction.
`no_redact` disables only the current type's generated Redact implementation;
retained output implementations still honor redaction already implemented by nested
field types. Direct field/selector redaction declarations on the current type remain
incompatible with no_redact, but nested types' own rules do not prohibit it
(REQ-CAP-009).
Visible explicit output derives that can be identified as conflicting with the
redaction contract are rejected. Users who disable automatic output and provide
handwritten implementations are responsible for satisfying that contract; macros
do not prove arbitrary handwritten output safe (REQ-RED-010). Named standard
`Option` and collection fields default when absent and are omitted when empty;
`keep_serializing` suppresses only that implicit omission and is invalid on
other field shapes. Redacted map-key collisions must fail serialization rather
than silently overwrite data.

## Metadata, registration, and resolution

ModelImpl supports an explicit method-level exclusion from automatic Property
assembly while retaining method reflection. Unmarked methods continue to follow
getter/setter shape recognition. Excluding one method does not remove contributions
from a same-named storage field or other methods. Marker syntax belongs to design
(REQ-PROP-013).

Getters are not additional declaration sites for text, indexed, validator, codec,
or redact metadata; existing field and selector declaration sites remain in use.
An eligible getter that is not explicitly excluded automatically contributes a
computed Property when no same-named storage field exists. No computed macro or
marker is provided (REQ-PROP-007/008, REQ-OUT-002).

References support single values, optional values, Box/Rc/Arc wrappers, sequences,
sets, arrays, and their combinations. Compatibility checks compare the stored
reference value with the selected target property after traversing these wrappers;
metadata retains optionality and collection structure. Direct Map references remain
unsupported. Collection query, assembly, and persistence policies belong to consumers
(REQ-REF-004/008).

Explicit Serde configuration overrides automatic naming and omission policies,
but cannot bypass redaction. Custom serialization that cannot safely compose with
the selected redaction mode under the qubit-redact contract must cause a macro
error; handwritten output follows REQ-RED-010 (REQ-SER-001).

`redact(skip)` omits the entire field from Debug, Display/text, and JSON/Serde
output while redaction is enabled, regardless of value shape or whether the field
is named or positional. This includes tuple and enum payload fields, newtypes, and
transparent Value fields. It cannot be applied through element/map selectors.
Disabled redaction restores the field subject to independent Serde skip controls.
When the only payload is omitted, qubit-redact and the serializer determine the
legal empty outer representation; the omitted payload must not be emitted merely
to preserve shape, and redacted output need not round-trip to the original object
(REQ-RED-008). Restrictions on automatic empty-field omission do not prohibit
explicit redaction or Serde skips (REQ-SER-005).

Requirements specify query capabilities, returned information, missing/error
semantics, lifetimes, sharing boundaries, and stable entry points. Complete Rust
signatures are maintained in the final design and API reference rather than
duplicated here (REQ-META-002). Registry lookup distinguishes absent metadata
from initialization/capability failures. Field and property descriptors may be
absent for opaque or symbolic type references.

`TypeMetadata::of::<T>()` and descriptor capability lookup are static and must
not initialize a global registry. `TypeRef` distinguishes resolved, opaque, and
symbolic types. Generic declarations with stable IDs register one definition;
concrete monomorphizations refer back to it and do not invent stable model IDs.
Definitions without a model ID also retain their generic-model capability and
remain discoverable from concrete instances. A stable ID controls ID-based
discovery, not the availability of model metadata (REQ-META-033, REQ-GEN-005).
Const generics support primitive Rust integer, bool, and char parameters,
constant arguments, and direct parameter references such as `[T; N]`. This
version does not require the model system to interpret complex const
expressions performing operations on generic parameters (REQ-GEN-008).

Registries are initialized only after all participating crates are linked.
`StructureResolver` validates stable IDs, roles, projection sources, references,
queries, and validator dependency paths, then exposes a structure-only `ModelGraph`.
Callers can explicitly supply types without model IDs and concrete generic instances
as resolution roots. The resolver checks their reachable references, Properties,
and structural dependencies without requiring stable IDs or linked registrations
for those roots. Stable-ID references still use the explicitly supplied registry.
Errors identify types without IDs using diagnostic type information and graph paths,
without synthesizing IDs (REQ-RES-005/007).
Optional codec and validation adapters bind executable registries after structural
resolution.
Rust `type_name()` is not a stable model identifier.

## Downstream implementation requirements and design references

Query execution, filter generation, random object generation, and DAO persistence
are downstream responsibilities. Requirements describing their use of metadata
do not require the metadata or derive crate to implement those business algorithms.
Chinese chapter 10 is the consolidated record, separating confirmed downstream
requirements from candidate designs that are not mandatory acceptance criteria.

Validator components own executable registrations, context, and structured
violations. Adapters explicitly bind exact input types, parameters, and dependencies.
Each dependency has its own object path and property selection, including parent
navigation under REQ-VLD-005. Consumers supply instance context and missing-parent
handling without fetching unavailable reference targets automatically. Execution
honors cumulative constraints, Option/selector/opaque boundaries, repeated validator
occurrences, and source order. Codec components own registered bidirectional text
execution and explicit binding; normalization is not performed by validators.

Future filter consumers can use indexed/type metadata for substring matching
(nickname `abc` corresponding to SQL `LIKE '%abc%'`) and ordered min/max conditions
for age, birthday, and creation time. Names, matching rules, endpoints, defaults,
and capability adapters await downstream design.

REQ-QRY-005 through REQ-QRY-014 are retained only as downstream design references:
root filter selection, separate unique lookups, scoped-unique filters, indexed-only
complex traversal, empty-expansion diagnostics, underscore-flattened names,
collision handling, one-hop references, separate value traversal, and AND composition.
Neither root ID/global-unique exclusion nor one-hop traversal is mandatory.
Consumers adopting flattening or expansion own the corresponding checks and must
not silently lose conditions. These policies cannot reject otherwise valid metadata.

Object generators plan creation and reuse from identifier/reference/Projection
metadata. Creating a missing parent Entity is a recorded Java scenario, not a fixed
fallback policy. Uniqueness consumers own structured key extraction, case folding,
scope comparison, batch/existing-data checks, and unsatisfiable-space errors.
KeyComponentValue representation belongs to downstream design. Schema/DAO consumers
own physical constraints and database-assigned ID propagation. Output integrations
follow qubit-redact; documentation consumers expose metadata without sensitive values.

## Diagnostics and acceptance

An Entity QueryMetadata view contains query-related declarations and resolved
relationships, not filter selection, expansion, flattened names, or execution plans
(REQ-META-087). The prohibition on anonymous ModelRegistry ID registrations does
not prohibit reflection registrations, model capabilities, static queries, or explicit
resolution roots for types without IDs (REQ-REG-011).

Tests in this crate cover its metadata, structural checks, and required integrations.
Confirmed downstream requirements are accepted in their respective components;
candidate designs require traceability, not implementations or tests in this crate
(REQ-ACC-001).

Independent recoverable declaration errors should be accumulated in one macro
invocation. Duplicate options, illegal shapes, role/field conflicts, invalid
bounds, overflow, and unsupported property methods must point at useful source
spans. Runtime and resolver failures use typed error APIs.

Implementation acceptance follows REQ-ACC-001 through REQ-ACC-008: meaningful
tests, cross-crate fixtures, safe access and generic-cache checks, and aligned
documentation. The ledger lists each requirement and its acceptance owner without
treating old tests as proof of the new contract. Design-reserved syntax and public
signatures must be settled before implementation delivery. Requirement freezing
does not claim that design or implementation is complete.

## Snapshot and provider revision, 2026-09-05

Global property queries return `PropertyResolutionError`, distinguishing reflection initialization (`Reflection`) from declaration assembly (`Assembly`). `property_fragments` is also fallible. Explicit `_in` queries and `ModelRegistry::properties_for` use the supplied snapshot, including during `StructureResolver` traversal.

Generic model macros select their own provider identifier through `definition_provider_v2`; its parameterless function returns the canonical static type definition without choosing a monomorph. Model generators never infer reflect's internal function names. Concrete model capabilities must keep providers isolated by `TypeId`.

## Closure clarifications, 2026-09-10

Historical design and guides do not override this candidate. Projection source and
producer declarations are checked structurally; consumers invoking producers check
result identifiers against source instances. Metadata queries never execute getters
(REQ-PRJ-007). Structural validator checks only resolve what explicit type context
can establish; parent-dependent paths follow REQ-VLD-008 (REQ-RES-002).
When scale and precision are both declared, scale cannot exceed precision; equal
min/max values require both endpoints inclusive, otherwise the interval is empty
and rejected (REQ-DEC-003).
