# rs-model-metadata-derive design

[简体中文](design.zh_CN.md) · [User guide](user_guide.md) · [Runtime design](../../doc/design.md)

This document describes the implemented `qubit-model-derive` 0.2.0 procedural macro crate (Rust 1.94, edition 2024). Its output is consumed by `qubit-model-metadata` 0.2.0 and `qubit-reflect` 0.2. The package is currently `publish = false`.

## Scope and compilation pipeline

The public attribute macros are `Entity`, `Projection`, `Model`, `Enum`, `Value`, and `ModelImpl`. The first five describe a type's role; `ModelImpl` adds property accessors from an impl block. One entry point dispatches to a shared pipeline: parse Rust and attributes, normalize declarations, validate local semantics and conflicts, then expand reflection and model capabilities. The implementation is divided among `parse`, `ir`, `normalize`, `validate`, `compiler`, and `expand`. Invalid local declarations produce compiler diagnostics rather than partial metadata.

The macros emit static `TypeMetadata`, field and variant declarations, reflection capabilities and registration fragments. Concrete metadata is projected through the frozen reflection snapshot; generic model definitions retain their own registration fragments. Generated code calls checked runtime constructors through `__private::v7`. The runtime and derive ABI therefore change together.

## Roles and identity

An `Entity` is a non-generic named struct with one `#[identifier]` field of the exact `qubit_id::Id` type. `Projection` also has an identifier and may have a fixed source or remain open. `Model` describes structured data; `Enum` describes domain variants; `Value` is a value object whose closure must not conceal entity, projection, model or reference semantics. Stable IDs are supplied by `id = "..."` where required. A concrete Rust type is also identified by `TypeId`; the two identities have different lifetimes and purposes.

Role macros generate the documented default Rust capabilities, including reflection, clone, comparison and redacted formatting/Serde where their bounds allow them. `Value` and `Enum` also default to Eq and Hash. Explicit capability switches change generated Rust behavior while retaining model metadata. Rust trait bounds diagnose unavailable operations. Enum payload metadata remains variant-local, and generic declarations are checked against their actual concrete instantiations.

## Fields, properties and declarations

Field attributes record identifiers, keys, indexing, uniqueness, references, constraints, validators, selectors, codecs, redaction and Serde behavior. Each concrete field has a `FieldLocation`; source locations and declaration order are retained for diagnostics. `ModelImpl` reflects methods and assembles ordinary fields with eligible getter/setter contributions. A getter-only method becomes a computed property unless `#[model_property(skip)]` excludes it. Presence conditions such as `cfg` are shared by impl reflection and property adapters, so disabled methods cannot contribute accessors.

Reference object paths use `/` and `..`; property paths use `.`. The macro checks local syntax and type-shape requirements. Cross-model targets, structural relations, validator registrations and codec execution are checked later by the registry, resolver or explicit execution binder. A declaration is not proof that its execution shape is supported. Consult the [runtime support matrix](../../doc/user_guide.md#limitations-execution-support-and-explicit-refusal) before attaching executable constraints to collection interiors or selectors.

## Failure boundaries and verification

Local attribute conflicts and unsupported role shapes fail compilation. Rust trait or adapter bounds fail at their use site. Registry initialization checks generated ABI and identities; `StructureResolver` checks relations; optional validation/codec plans bind executable behavior. Macro tests include successful runtime metadata assertions, compile-fail UI fixtures, renamed runtime dependencies and cross-crate scenarios. See the [user guide](user_guide.md) for declaration examples and [runtime design](../../doc/design.md) for snapshot and execution ownership.
