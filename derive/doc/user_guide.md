# Declaring domain models

[简体中文](user_guide.zh_CN.md) · [README](../README.md) · [Runtime guide](../../doc/user_guide.md)

This guide covers the 0.1 workspace contract for application authors. Declare a
model once, inspect its metadata, and let explicit consumers implement validation,
querying, or persistence. Rust 1.94 and edition 2024 are required. These crates are
unpublished; use the checkout dependencies shown in the README.

## A user with safe output and discoverable properties

```rust
use qubit_id::Id;
use qubit_model_derive::{Entity, ModelImpl};
use qubit_model_metadata::metadata::TypeMetadata;

#[Entity(id = "guide.User")]
struct User {
    #[identifier]
    id: Id,
    #[indexed]
    nickname: String,
    #[redact(level = "secret")]
    token: String,
    tags: Vec<String>,
}

#[ModelImpl]
impl User {
    pub fn display_name(&self) -> String { self.nickname.clone() }
    #[model_property(skip)]
    pub fn diagnostic(&self) -> usize { self.tags.len() }
}

fn main() {
    let metadata = TypeMetadata::of::<User>();
    assert!(metadata.field("nickname").unwrap().is_indexed());
    assert!(metadata.try_property("display_name").unwrap().unwrap().is_computed());
    assert!(metadata.try_property("diagnostic").unwrap().is_none());
}
```

The stored fields remain Properties. The getter creates a computed Property
without a separate marker. `diagnostic` remains reflected as a method.
Debug, Display, and Serialize use rs-redact. `redact(skip)` omits the whole
field for named, positional, Enum, and transparent shapes; disabling the
application redaction policy restores it. A map-key redaction collision returns
a serialization error. Do not combine a raw serialization adapter with a
redaction mode that observes the field.

## Roles and Rust capabilities

| Role | Shape and purpose |
| --- | --- |
| Entity | Nongeneric named struct, exactly one direct `qubit_id::Id` identifier |
| Projection | Nongeneric named view with an identifier and optional fixed Entity source |
| Model | Named or unit struct, optionally generic |
| Enum | Unit, tuple, named, or mixed variants, optionally generic |
| Value | Named value object or one-field tuple wrapper; optional `transparent` |

Entity requires a stable model `id`; it is optional for the other roles. The ID
controls registration, not the availability of metadata for anonymous models. Enable the runtime `generic` feature for generic definitions and
concrete specializations; lifetime parameters are unsupported.

Roles default to Clone, Debug, Display, PartialEq, Eq, Hash, Redact, Serialize,
and Deserialize. An all-unit Enum also defaults to Copy. `no_*` options suppress
generated implementations; `no_eq` also removes default Hash, and
`no_partial_eq` removes equality, hashing, and ordering. `copy`, `default`,
`partial_ord`, and `ord` are opt-in. Enum `default` requires exactly one standard
`#[default]` unit variant. Default values need not satisfy domain constraints.

Place the role attribute above explicit derives. Visible explicit derives are
not generated twice. Use the corresponding opt-out for handwritten impls.
`no_redact` forbids local field or selector redaction; nested types retain their
own safe output implementations. HashMap-bearing models commonly need `no_hash`.
Generic bounds follow stored field capabilities, including PhantomData and const arrays.

Named Option and standard collection fields default when absent and omit empty
values. `#[keep_serializing]` suppresses only automatic omission. Explicit Serde
controls have precedence. Positional fields do not receive automatic omission.

Default serialization delegates to rs-redact, which rejects Serde `flatten`.
For plain Serde flattening, select `#[Model(no_redact)]`; this opt-out does not
allow local field or selector redaction declarations. The metadata still records
flattening, directional names, skip controls and the origins of defaults.

## Relationships and query declarations

```rust,ignore
#[reference(entity = Country, property = id, path = "street/district/country")]
country: Id,
#[reference(entity = Order, property = id, path = "..")]
order: Id,
```

These illustrative fields require the named Entity types in the application.
`reference.path` traverses object bindings with `/` and `..`; through a reference
it follows the full Entity even when storage contains an ID or Projection.
Omitting it requests no binding reuse. A container adds no domain parent.
The separate `property` selection uses dotted Property paths. Option, smart
pointers, sequences, sets, and arrays may wrap references; direct Map references
are rejected. Enum payload references are allowed, but a Value cannot hide them
inside its transitive value closure.

Identifier, unique, and reference contribute implicit indexed reasons; adding
`#[indexed]` again is a redundancy error. Unique scope uses `respect_to(...)`.
Case-insensitive uniqueness defaults on text-capable fields, including aliases;
explicit `ignore_case` on other fields is invalid. `key_part(order = n)` records
ordered logical key components on named Model or Value fields.

A consumer might turn indexed `nickname` into substring matching and indexed
`birthday`, `create_time`, or `age` into min/max filter parameters. These are
usage examples, not generated filter APIs. The runtime exposes direct indexed
declarations and leaves matching operations and physical indexes to consumers.

## Validators, selectors, and codecs

```rust,ignore
#[validator(id = "person.birthday",
    depends_on(gender, birthday(path = "..", property = birthday)),
    params(strict = true))]
#[validator(id = "person.birthday", params(strict = false))]
value: String,
```

Each occurrence retains its own parameters, dependencies, and declaration
location, even when IDs repeat. Bare dependency slots select a current-object
Property; structured slots separate object `path` from `property`. Declarations
do not execute validators. The optional runtime adapter binds an explicit
validator registry and accepts parent context separately.

Text, decimal/money, time, sequence, and map constraints record their declared
bounds. `element(...)`, `map_key(...)`, and `map_value(...)` retain selector
positions. Existing Set uniqueness and fixed-array size constraints must not be
restated redundantly. `opaque` stops internal traversal while preserving outer
shape. Remove old `target`, `on_none`, and `validate_nested` macro options;
execution policy belongs to consumers.
The runtime `FieldAttributeMetadata::ValidateNested` marker and
`FieldMetadata::validate_nested()` method are also removed. The plan builder
discovers nested declarations according to its support matrix without an extra
marker; reference and opaque semantics still define traversal boundaries.

Codecs retain an explicit stable ID or Rust codec type. The codec must be
registered with the supplied codec registry. Explicit field selection takes
precedence over a Value's canonical codec; selecting that same canonical codec
explicitly is legal.

## Declaration support versus execution support

A macro accepting a declaration does not promise that a particular consumer can
execute it. The current runtime validation adapter binds supported text constraints
and custom validators on named fields, including direct and optional nested models.
Provide an actual borrowed intermediate getter such as `Option<&Child>` when the
adapter must traverse an optional child. A `Vec<T>` storage type alone does not
provide element access: explicit element validators need a borrowed-slice getter.

Constrained enum payloads, tuple/newtype interiors, model declarations inside
container elements, work-bearing cycles, Decimal/Time/Map constraints, erased
uniqueness, selector constraints/dependencies, and unsupported unwrap/owned
intermediate shapes produce `UnsupportedExecution` at plan construction. A unit
enum or cycle with no reachable execution declarations remains a valid ordinary
value. Reference fields validate only their explicit stored-field rules; opaque
fields stop internal traversal without deleting outer declarations.

Use `ValidationCapabilities::check(root, &graph)` to inspect access support,
then `ValidationPlan::build` with the actual custom validator registry. Both
retain source locations; a missing custom registration retains its original ID.
See the runtime guide's [support matrix](../../doc/user_guide.md#execution-support-and-explicit-refusal)
and [complete validation program](../../doc/user_guide.md#validate-a-profile-and-keep-partial-results)
for report limits, partial errors, and the real `rs-platform` integration boundary.
Do not remove a model constraint or add `opaque` just to make a backend accept it.

Generated model code uses checked `__private::v7`; update the runtime and macro
crate together. There is no v5/v6 compatibility facade. Concrete field identities
use owner TypeId, variant index, and field index; model IDs remain the stable
external naming mechanism. `rs-reflect` retains its independent protocol version.

## Reflection and diagnostics

ModelImpl forwards supported reflect_impl options. Trait methods and methods
that cannot act as Properties remain reflected. Generic runtime adapters require
explicit `specialize(T = String, ...)` bindings, following rs-reflect's boundary.
Only public, safe, synchronous, nongeneric inherent accessor methods contribute
Properties. Borrowed getters retain their lifetimes and do not require Send.

Compile errors cover local shape, redundancy, and capability conflicts.
Cross-crate targets and complete structural relationships are checked by
StructureResolver. Parent requirements remain explicit rather than becoming
invalid declarations. Fallible `try_*` queries preserve capability and Property
assembly failures; see the runtime guide for execution and error handling.

The [requirements](rs-model-derive-requirements.zh_CN.md) and
[final design](rs-model-derive-final-design.md) define the contract. Build local
API documentation with `cargo doc --workspace --all-features --no-deps`.
