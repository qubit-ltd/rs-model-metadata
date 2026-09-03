# `rs-model-derive` Final Design

[中文版](rs-model-derive-final-design.zh_CN.md) | [Requirements](rs-model-derive-requirements.md)

## Decision summary

The crate is a compiler front end for model declarations, not an independent
runtime schema system. It reuses the one `qubit-reflect` descriptor for Rust
structure and attaches `TypeMetadata` as a typed capability. Stable model IDs,
relations, validators, and codecs are resolved explicitly after the complete
application is linked.

The six public attributes keep their existing names and syntax. Generated code
targets model ABI v3 in the versioned hidden facade of
`qubit-model-metadata`, preserving renamed-dependency support while consuming
only the `qubit-reflect` `codegen_v2` protocol.

## Crate boundaries and data flow

```text
attribute tokens
    │
    ▼
parse ──► declaration IR ──► normalize ──► validate ──► expand
                                                        │
                                                        ▼
                                  Reflect descriptor + TypeMetadata capability
                                                        │
                                                        ▼
                         unified reflection fragments + generic registrations
```

Parsing owns syntax and duplicate-option detection. The IR owns source-located
facts and contains no code generation. Normalization derives canonical
semantics such as implicit container constraints and capability defaults.
Validation checks role shapes, field combinations, order invariants, and
method contracts. Expansion is the only stage that emits Rust tokens.

The implementation mirrors those boundaries:

- `parse/{declaration,options,fields,constraints,validator}.rs` parses syntax;
- `ir/` contains located declarations and role kinds;
- `normalize/declaration.rs` canonicalizes and validates semantic combinations;
- `validate/declaration.rs` handles syntax-tree shape and helper rewriting;
- `expand/{declaration,metadata,fields,role,capabilities,registration,
  model_impl}.rs` emits focused code-generation products and delegates all
  generic type-expression generation to `qubit-reflect`;
- `compiler/` contains shared diagnostics and canonical standard-type path
  classification.

Modules use the ordinary Rust module tree; there is no `#[path]` wiring. A
small number of cohesive IR/codegen files intentionally hold related private
types, with documented style justifications where required.

## Role and metadata design

`Entity` and `Projection` have an exact `qubit_id::Id` identifier. An Entity is
persistent identity; a Projection is an open or fixed view. `Model` represents
structured data, `Enum` preserves three name domains, and `Value` represents a
value object with optional transparent representation.

Named `Model` and `Value` fields may declare `key_part(order = n)`. This is an
ordered logical key used by value semantics and downstream query/schema tools,
not persistence identity. Allowing a field subset supports natural keys that
exclude descriptions or payloads. Requiring named fields and contiguous unique
orders makes the key deterministic and introspectable. Entity/Projection use
`identifier`; Enum and tuple/newtype Value have no named-field selection, so
all four shapes reject `key_part`.

Fields retain structural facts and normalized semantic occurrences. Properties
merge a field, getter, and setter by canonical name. `ModelImpl` emits borrowed
or owned adapters without extending lifetimes. Setter failures retain the input
value when user code has not been invoked. Compatibility assertions make
getter/setter type mismatch a compile-time error.

## Type syntax and constraint design

Standard-library behavior inferred from syntax is deliberately narrow. Exact
unqualified or canonical `core`, `alloc`, and `std` paths are recognized;
unrelated qualified lookalikes are not. This prevents `domain::Option` or
`domain::Vec` from receiving implicit Serde/container behavior merely because
their final identifier matches.

Constraint parsers use one seen-slot per singleton option and sets for repeated
validator parameter/dependency names. Errors are emitted at the second
occurrence. Empty validator arrays are rejected early because token generation
cannot infer their element type. Decimal comparison normalizes sign, leading
zeroes, trailing fractional zeroes, and scale without floating-point parsing.

Generated trait assertions validate text, decimal, temporal, sequence, map,
identifier, validator, and codec targets while preserving accurate field spans.

## Registration and resolver design

Direct `TypeMetadata::of` access does not touch the global model registry.
Descriptor capability and property lookup use the frozen reflection snapshot.
Concrete declarations no longer submit a parallel model registration: the
model registry projects their typed capability and authoritative provenance
from `ReflectRegistry`. Generic declarations retain one model-owned
registration that points at the symbolic definition generated by
`qubit-reflect`; each concrete type keeps its ordinary reflection descriptor
and links to that definition.

The explicit resolver consumes the complete model, validator, and codec
registries. It checks ID uniqueness, role compatibility, projection sources,
reference properties, query paths, and executable strategy bindings. This
separation keeps local compilation deterministic and avoids order-dependent
global initialization.

## Diagnostics and safety

A shared accumulator combines independent recoverable errors. Parsing failures
remain source-located `syn::Error`s; semantic failures point at the closest
field, option, or declaration span. Missing runtime resolution is combined with
other diagnostics instead of short-circuiting them.

Generated `Debug`, `Display`, and `Serialize` implementations pass through the
redaction layer. Unsafe pre-existing output implementations are rejected unless
redaction is explicitly disabled under its own consistency rules. Map-key
redaction detects output collisions. Generated private symbols include stable
fingerprints to avoid collisions across target paths and annotated impls.

## Test and release strategy

The verification pyramid consists of:

1. private unit tests for parser normalization and canonical path recognition;
2. trybuild pass/fail fixtures for public syntax and diagnostic snapshots;
3. runtime tests for metadata, properties, resolution, Serde, and redaction;
4. linked and renamed dependency fixtures for registration and facade lookup;
5. a compiling crate-level doctest and bilingual scenario documentation;
6. coverage thresholds enforced by the shared remote `.rs-ci` submodule;
7. downstream `rs-platform` CI over real declarations.

Release acceptance requires the style checker, full crate CI, coverage gates,
and downstream integration to pass without broad exclusions or lowered
thresholds. The historical discussion log remains archived and is not a
specification source.
