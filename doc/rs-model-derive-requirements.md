# `rs-model-derive` Final Requirements

[中文版](rs-model-derive-requirements.zh_CN.md) | [Coverage ledger](rs-model-derive-requirements-coverage.zh_CN.md)

## Status and terminology

This document is the English companion to the Chinese final requirements. The
two documents describe the same stable contract; requirement IDs and detailed
metadata signatures are maintained in the Chinese document and its coverage
ledger. “Must”, “must not”, “should”, and “may” are normative terms.

Historical discussion is non-normative. Public behavior is established by the
requirements, final design, generated API documentation, and executable tests.

## System contract

`qubit-model-derive` provides five role attributes—`Entity`, `Projection`,
`Model`, `Enum`, and `Value`—and the property attribute `ModelImpl`. Every role
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
- `Value` accepts a named struct or a one-field tuple/newtype. `transparent`
  requires exactly one field and uses the inner representation.
- `Model`, `Enum`, and `Value` may use type parameters, where clauses, and
  supported primitive const generics. Model roles do not accept lifetimes.

The role attribute must precede user derives so unsafe duplicate output
implementations can be detected. Default capabilities are `Clone`,
`PartialEq`, `Eq`, `Hash`, `Redact`, redaction-aware `Debug`, `Display`,
`Serialize`, and `Deserialize`. Corresponding `no_*` switches disable them;
`copy`, `default`, `partial_ord`, and `ord` are opt-in. Unit-only enums are
`Copy` unless `no_copy` is present. Conflicting capability combinations must
produce compile-time diagnostics.

## Fields and properties

A field is a real storage slot. A property is a name-based view assembled from
a field, getter, and setter. Property merging must be deterministic, preserve
borrowing, and reject incompatible getter/setter types and duplicate annotated
implementations.

`ModelImpl` accepts a public, safe, synchronous, non-generic inherent impl.
Getters use `&self` and may return owned values, `&T`, `&str`, `&[T]`, or
`Option<&T>`. Setters use `&mut self`, accept one owned value, and return unit.
Generated adapters must have collision-resistant deterministic names.

## Identity, indexes, relationships, and keys

`identifier`, `indexed`, `unique`, and `reference` describe storage identity,
querying, and inter-model relationships. References use `qubit_id::Id` and may
name a target by Rust entity type or stable entity ID. Cross-model target,
role, and property checks belong to the resolver because only the complete
linked model set can establish them.

`key_part` describes an ordered logical key for value semantics. It is allowed
only on real named fields of `Model` and `Value`; it is rejected on `Entity`,
`Projection`, `Enum`, and tuple/newtype values. A key may select a subset of
fields. Selected orders must be unique and contiguous from zero. These rules
are independent from Entity identity.

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

A validator occurrence has a stable ID, ordered named parameters, and readable
dependency property paths. Duplicate parameter names and duplicate dependency
paths are invalid. Empty parameter arrays are invalid because their element
type cannot be inferred. Integer overflow must be diagnosed at its source
span. Binding to an executable validator occurs during graph resolution.

A Rust codec must satisfy the exact encoder/decoder contract for the declared
value type. Stable codec IDs are resolved through `ValueCodecRegistry`; this
requires the application’s direct `qubit-codec` dependency to enable its
`registry` feature. `opaque` preserves intentionally unavailable structural
type information without pretending metadata is missing.

Redaction-aware output is fail-closed. Existing `Debug` or `Serialize`
implementations that could bypass redaction are rejected. Named standard
`Option` and collection fields default when absent and are omitted when empty;
`keep_serializing` suppresses only that implicit omission and is invalid on
other field shapes. Redacted map-key collisions must fail serialization rather
than silently overwrite data.

## Metadata, registration, and resolution

`TypeMetadata::of::<T>()` and descriptor capability lookup are static and must
not initialize a global registry. `TypeRef` distinguishes resolved, opaque, and
symbolic types. Generic declarations with stable IDs register one definition;
concrete monomorphizations refer back to it and do not invent stable model IDs.

Registries are initialized only after all participating crates are linked.
`ModelResolver` validates stable IDs, roles, projection sources, references,
queries, validator bindings, and codec bindings, then exposes a resolved graph.
Rust `type_name()` is not a stable model identifier.

## Diagnostics and acceptance

Independent recoverable declaration errors should be accumulated in one macro
invocation. Duplicate options, illegal shapes, role/field conflicts, invalid
bounds, overflow, and unsupported property methods must point at useful source
spans. Runtime and resolver failures use typed error APIs.

Acceptance requires formatting, default style rules without blanket
exemptions, unit and integration tests, trybuild pass/fail snapshots, runtime
fixtures, at least one compiling doctest, coverage thresholds, and downstream
`rs-platform` integration. The coverage ledger maps every stable requirement
group to its primary executable evidence.

## Snapshot and provider revision, 2026-09-05

Global property queries return `PropertyResolutionError`, distinguishing reflection initialization (`Reflection`) from declaration assembly (`Assembly`). `property_fragments` is also fallible. Explicit `_in` queries and `ModelRegistry::properties_for` use the supplied snapshot, including during `ModelResolver` traversal.

Generic model macros select their own provider identifier through `definition_provider_v2`; its parameterless function returns the canonical static type definition without choosing a monomorph. Model generators never infer reflect's internal function names. Concrete model capabilities must keep providers isolated by `TypeId`.
