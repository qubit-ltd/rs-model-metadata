# rs-model-metadata-derive design

[简体中文](design.zh_CN.md) · [User guide](user_guide.md) · [Runtime design](../../doc/design.md)

This document describes the implemented `qubit-model-derive` 0.2.0 procedural macro crate (Rust 1.94, edition 2024). Its output is consumed by `qubit-model-metadata` 0.2.0 and `qubit-reflect` 0.2. The package is currently `publish = false`.

## Scope and compilation pipeline

The public attribute macros are `Entity`, `Projection`, `Model`, `Enum`, `Value`, and `ModelImpl`. The first five describe a type's role; `ModelImpl` adds property accessors from an impl block. One entry point dispatches to a shared pipeline: parse Rust and attributes, normalize declarations, validate local semantics and conflicts, then expand reflection and model capabilities. The implementation is divided among `parse`, `ir`, `normalize`, `validate`, `compiler`, and `expand`. Invalid local declarations produce compiler diagnostics rather than partial metadata.

The macros emit static `TypeMetadata`, field and variant declarations, reflection capabilities and registration fragments. Concrete metadata is projected through the frozen reflection snapshot; generic model definitions retain their own registration fragments. Generated code calls checked runtime constructors through `__private::v7`. The runtime and derive ABI therefore change together.

## Input contract and role selection

The macros operate on compile-time Rust syntax. Choose a role from the domain meaning of the type, then choose which Rust capabilities to expose:

| Macro | Accepted declaration | Important constraints |
| --- | --- | --- |
| `Entity` | Non-generic named struct | Requires a stable `id` and exactly one `#[identifier]` field whose actual type is `qubit_id::Id`; a type alias for `Id` is accepted. |
| `Projection` | Non-generic named struct | Requires exactly one `#[identifier]` field of type `qubit_id::Id` (or an alias); may be `open` or name one fixed source using `source` or `source_id`. |
| `Model` | Named or unit struct, optionally generic | Tuple structs are not accepted. |
| `Enum` | Enum, optionally generic | Unit, named, and tuple payloads are represented; canonical variant names must be unique. |
| `Value` | Non-empty named struct or single-field tuple struct | `transparent` requires exactly one field. Its value closure cannot hide entity, projection, model, or reference semantics. |
| `ModelImpl` | Impl block | Only eligible inherent methods contribute properties; trait methods may still be reflected. |

Entity `id` is the stable external name of a model type; the `#[identifier]` field carries the identity value of an entity instance. A concrete Rust type also has a process-local `TypeId`; it serves a different purpose from the stable model ID. Other roles may be anonymous, so generating `TypeMetadata` does not guarantee that a global registry has an entry addressable by a string ID. Role models do not accept lifetime parameters. Const parameters are restricted to primitive integer types, `bool`, and `char`. `Entity` and `Projection` cannot have generic parameters or a `where` clause. Generic runtime capabilities for supported roles are guarded by the `generic` feature.

`Projection` may remain open or select a fixed source. `open` cannot be combined with `source` or `source_id`, and the two source options are mutually exclusive. The identifier requirement is enforced through the runtime's private sealed `IdentifierType` contract: the only implementation is `qubit_id::Id`, so aliases work while newtypes and containers do not.

## Compilation pipeline and failure location

The six public entry points in `derive/src/lib.rs` dispatch through `entry::expand`. For the five type-role macros, `expand::pipeline::run` performs these steps:

1. Parse macro arguments and the `DeriveInput` with `syn`, check the role's Rust shape, and resolve the actual dependency name for the runtime crate. A missing runtime dependency produces a diagnostic; renamed dependencies are supported.
2. Reject explicit derives that would duplicate generated `Reflect` implementations. Parse fields, variants, Serde settings, role options, and declaration locations into the internal IR. Field indices and Enum variant ownership are retained.
3. Normalize shorthand into a consistent IR. This includes eligible defaults and empty-value omission for `Option` and collection fields, as well as selector positions and index reasons.
4. Validate role options, identifier counts, variant names, field-attribute conflicts, logical-key order, type shapes, and constraint combinations. Independent local errors are combined into compiler diagnostics.
5. Prepare default output capabilities and rewrite field helpers for reflection, Serde, and redaction. Add the reflection derive, `definition_provider_v2` for generic declarations, and the runtime `__private::v7::model_capability` support as applicable.
6. Emit the original item, static metadata providers, registration fragments, and trait/output implementations. Generic output is guarded by the runtime feature facade and fails clearly when the required feature is disabled.

`ModelImpl` follows a separate path: it parses the `ItemImpl`, preserves the original impl and its reflection support, scans methods, then creates accessor adapters and providers for the target type. `#[model_property(skip)]` controls property selection and is removed from the method; a skipped method remains an ordinary reflected method.

These stages have distinct failure boundaries. Invalid local declarations, missing dependencies, and conflicting macro options are reported during expansion. Rust trait or adapter bounds fail during Rust type checking. Generated ABI and identity consistency are checked when metadata is registered; cross-model relations are checked by the resolver; executable validation or codec behavior is bound by explicit runtime plans. A successful macro expansion is not whole-program validation.

## Generated capabilities and output boundaries

The five role macros default to `Clone`, `PartialEq`, `Redact`, `Debug`, `Display`, `Serialize`, and `Deserialize`, subject to the field and generic trait bounds. `Value` and `Enum` also default to `Eq` and `Hash`. An all-unit `Enum` additionally defaults to `Copy`. `Entity`, `Projection`, and `Model` require explicit `eq` and `hash` options for those capabilities; `ord` requires equality but does not enable hashing. Options such as `default`, `partial_ord`, and `ord` are explicit, while corresponding `no_*` switches can disable generated implementations. Conflicting switches fail in the macro, and unsatisfied trait bounds are diagnosed by Rust.

The macro recognizes explicit user derives to avoid generating a duplicate trait implementation. Default `Debug`, `Display`, and `Serialize` use `qubit-redact` for controlled output; `Deserialize` follows the input-side Serde contract. `no_redact` conflicts with field or selector redaction rules, and custom output derives cannot bypass redaction rules that remain enabled. `#[redact(skip)]` skips an entire field. If redacting a map key causes a key collision, serialization returns an error rather than silently overwriting data.

Eligible named `Option` and standard collection fields receive generated missing-value defaults and empty-value omission. Explicit Serde configuration takes precedence; `keep_serializing` disables only implicit omission. Tuple fields do not receive the named-field defaults. Enum Rust variant names, canonical names, and Serde wire names are tracked separately. Without a type-level `rename_all`, the wire name defaults to the canonical name, and an explicit variant-level rename takes precedence.

## Field metadata and location guarantees

Field declarations preserve each attribute occurrence in source order; repeated IDs or rules are not merged into one entry. Every concrete field receives a `FieldLocation`, and attributes, constraints, references, validators, and selectors retain declaration-location information for diagnostics. `#[identifier]`, `#[unique]`, and `#[reference]` imply indexing, so an additional redundant `#[indexed]` on the same field is a compile-time error. On named `Model` and `Value` fields, `#[key_part(order = n)]` records logical key order; it does not generate a database primary key.

Reference declarations record a target ID/type, target property, and object-navigation information. Object paths use `/` and `..`; property paths use dots. The macro checks local syntax and conflicts, but cannot prove at compile time that a model in another crate will be linked into the final binary. Target existence and type/property compatibility must be checked by a structure resolver using a common snapshot.

Constraints, validators, selectors, and codecs are declarations until runtime binding. Repeated `#[validator(id = "...")]` attributes remain separate occurrences with their own arguments, dependencies, and locations. A codec may be named by stable ID or Rust type; selection between a field codec and a Value canonical codec is resolved during runtime binding. The macro does not invoke rules, codecs, or getters, and does not validate a particular object instance.

## ModelImpl property selection

`ModelImpl` first reflects the impl block, then selects methods that can contribute properties. Contributors must be public, synchronous, safe, non-generic inherent methods. Trait methods and methods outside the accessor contract may be reflected, but do not enter the Property set. Eligible getters and setters are paired with stored fields by property name; a getter without a stored field contributes a computed property. Conflicting accessors are diagnosed rather than selected by source order.

The accessor's `cfg` and nested `cfg_attr` conditions are applied to the provider, signature, and adapter. A conditionally disabled method therefore contributes neither a property nor an adapter that refers to a missing type. Generic impl specializations follow the explicit `specialize(...)` support in `rs-reflect`; arbitrary generic instances are not inferred. The borrow shape of a getter return value also affects whether downstream consumers can traverse or execute constraints. Property metadata alone does not guarantee that every consumer can execute the property.

## Cross-component verification and migration

When developing or upgrading the macros, verify Rust capabilities and metadata for valid declarations, diagnostics for invalid declarations, renamed runtime dependencies and cross-crate use, standalone `ModelImpl` providers, generic and anonymous models, real downstream declarations, and compatibility with runtime ABI `__private::v7`. Derive UI and runtime fixtures, root-crate metadata tests, and downstream consumers cover different parts of this contract. See the [user guide](user_guide.md) for declaration examples and [runtime design](../../doc/design.md) for snapshot and execution ownership.
