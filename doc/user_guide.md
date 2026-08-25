# Qubit Model Derive User Guide

[中文](user_guide.zh_CN.md) · [README](../README.md) · [API documentation](https://docs.rs/qubit-model-derive)

Applies to `qubit-model-derive` 0.1.0 and `qubit-model-metadata` 0.1.0.

This crate exposes two attribute macros: `#[Model(...)]` for structs and
`#[Enum(...)]` for fieldless enums. There is no `#[derive(Model)]` alias.

## Purpose and Audience

Read this guide when a Rust domain declaration must remain the source of truth
for metadata consumed by validation, schema-oriented tools, or application code.
The macros emit static implementations and registrations for
`qubit-model-metadata`. They do not validate instance data.

Choose `#[Model]` for a named-field struct, a unit struct, or a single-field
tuple newtype. Choose `#[Enum]` for a fieldless enum. Using the wrong macro is a
compile error.

## Conceptual Model

At compile time the matching macro reads the type, its `id`, and any
`#[field(...)]` attributes. It then emits default traits, a `Display`
implementation, Serde rename rules, and the runtime metadata traits.

```text
struct + #[field(...)]  ──►  #[Model]  ──►  TypeKind::Struct | Newtype
fieldless enum          ──►  #[Enum]   ──►  TypeKind::Enum
                                            │
                                            ▼
                         HasTypeShape + HasTypeMetadata + ModelRegistry
                                            │
                                            ▼
                                    metadata_of::<T>()
```

The two macros share the ID grammar, the three runtime traits, and the
automatic `ModelRegistry` registration. They differ in accepted shapes, default
traits, Serde naming, `Display`, and whether field constraints exist.

A model ID uses ASCII snake_case module segments and an ASCII UpperCamelCase
final segment that matches the Rust type name, for example
`example.AccountStatus`.

## Scenario: Describe an Account and Its Status

An application stores accounts. Each account has a generated identifier, a
unique email, and a lifecycle status. The status is a closed set of names, not
a struct with fields. Success means both types compile, register, and answer
typed metadata queries.

### Installation and Minimal Configuration

```toml
[dependencies]
qubit-model-derive = "0.1.0"
qubit-model-metadata = "0.1.0"
serde = { version = "1", features = ["derive"] }
```

Both macros require `serde` in the consuming crate, even when serialization is
later disabled with `no_serialize`. Enable runtime features only for the scalar
types you actually use:

```toml
[dependencies]
qubit-model-metadata = { version = "0.1.0", features = ["chrono", "big-decimal"] }
chrono = { version = "0.4", default-features = false, features = ["std"] }
bigdecimal = "0.4"
```

Add `qubit-redact` only when a declaration participates in redaction:

```toml
[dependencies]
qubit-redact = { version = "0.5", features = ["serde", "derive"] }
```

If `qubit-model-metadata` is renamed locally, the expansion follows the Cargo
package name:

```toml
[dependencies]
model_runtime = { package = "qubit-model-metadata", version = "0.1.0" }
```

### Core Workflow

Declare the status with `#[Enum]`, the account with `#[Model]`, then inspect the
normalized metadata:

```rust
use qubit_model_derive::Enum;
use qubit_model_derive::Model;
use qubit_model_metadata::AttributeQuery;
use qubit_model_metadata::TypeKind;
use qubit_model_metadata::metadata_of;

#[Enum(id = "example.AccountStatus")]
enum AccountStatus {
    Active,
    Suspended,
}

#[Model(id = "example.Account")]
struct Account {
    #[field(identifier(generated))]
    id: i64,
    #[field(unique(ignore_case), text(min_chars = 3, max_chars = 320))]
    email: String,
    status: AccountStatus,
}

let account = metadata_of::<Account>();
let email = account.field("email").expect("declared field");
assert!(account.primary_key().expect("primary key").contains("id"));
assert_eq!(email.text_constraint().and_then(|value| value.max_chars()), Some(320));

let status = AccountStatus::Suspended;
assert_eq!(format!("{status}"), "SUSPENDED");
assert_eq!(status.name(), "SUSPENDED");
assert_eq!(AccountStatus::from_name("ACTIVE"), Some(AccountStatus::Active));
assert!(matches!(metadata_of::<AccountStatus>().kind(), TypeKind::Enum(_)));
```

`identifier(generated)` becomes a model-level primary key.
`unique(ignore_case)` becomes a model-level unique constraint. Consumers query
that canonical metadata rather than the input spelling of the attributes.

The enum side is equally concrete: `AccountStatus::Suspended` displays as
`SUSPENDED`, serializes as `"SUSPENDED"`, and round-trips through `from_name`.

## `#[Model]`: Struct Capabilities

`#[Model]` rewrites a struct declaration and emits metadata for it.

### Accepted shapes

| Shape | Metadata kind | Notes |
| --- | --- | --- |
| Named-field struct | `TypeKind::Struct` | Field constraints and model-level keys apply here. |
| Unit struct | `TypeKind::Struct` | No fields. |
| Single-field tuple newtype | `TypeKind::Newtype` | The inner field is named `"0"` in metadata. A non-opaque newtype inherits the inner type's `TypeCapabilities`. |

Generic models, multi-field tuple structs, unions, and enums are rejected. An
enum must use `#[Enum]`.

### Default traits and naming

Unless disabled, a struct receives `Clone`, `Debug`, `Eq`, `PartialEq`, `Hash`,
`Serialize`, and `Deserialize`. The macro also implements `Display` with
Debug-shaped output, so a named struct prints as `Account { email: "..." }`.
Serde field names use `snake_case`.

Structs do not receive `Copy`, `PartialOrd`, or `Ord`. `no_copy` on a struct is
a compile error.

### Serde omission for optional and collection fields

When serialization is enabled, `#[Model]` omits a directly declared `Option<T>`
whose value is `None`. It also omits an empty directly declared `Vec`,
`LinkedList`, `VecDeque`, `HashMap`, `BTreeMap`, `HashSet`, `BTreeSet`,
`BinaryHeap`, or fixed array. When deserialization is enabled, those collection
fields receive `#[serde(default)]`, so an omitted field becomes its empty
default.

```rust
use std::collections::HashMap;

use qubit_model_derive::Model;

#[Model(id = "example.SearchFilter", no_hash)]
struct SearchFilter {
    query: Option<String>,
    labels: Vec<String>,
    facets: HashMap<String, String>,
    #[field(keep_serializing)]
    explicit_labels: Vec<String>,
}

let filter = SearchFilter {
    query: None,
    labels: Vec::new(),
    facets: HashMap::new(),
    explicit_labels: Vec::new(),
};
assert_eq!(
    serde_json::to_string(&filter).expect("serialize filter"),
    r#"{"explicit_labels":[]}"#
);
```

`#[field(keep_serializing)]` opts that field out of both automatically injected
rules, preserving `null` or an empty value during serialization. It does not
remove an explicit field-level `#[serde(...)]` attribute. The macro recognizes
only direct type syntax, not aliases. A fixed array is empty only when its
length is zero; an ordinary nonzero-length array is therefore retained.

### Model-level attributes

These attributes describe the whole model. Except for `id`, they are valid only
on named structs.

| Attribute | Meaning |
| --- | --- |
| `id = "example.Account"` | Required stable model ID. |
| `textual` | Marks a named struct as a text-capable value object, so field constraints such as `text(format = mobile)` can target it. |
| `primary_key(fields(id), generated(id))` | Ordered primary key. A generated field must belong to it. |
| `unique(name = "account", fields(org_id, username), ignore_case(username))` | Unique constraint with per-field comparison. |
| `index(name = "created_at", fields(created_at))` | Ordered index. |
| `key(name = "account", fields(org_id, username))` | Logical key. |
| `ownership(owner = Organization)` | Owning model type. |

`identifier` on a field is the single-field shorthand for a primary key.
`unique` and `index` on a field are the corresponding single-field shorthands.

### Field-level attributes

Write field constraints as `#[field(...)]`. Nullability comes from `Option<T>`,
not from a `nullable` flag.

| Attribute | Purpose |
| --- | --- |
| `identifier`, `identifier(generated)` | Single-field primary key. |
| `unique`, `unique(ignore_case)` | Single-field unique constraint. |
| `index` | Single-field index. |
| `text(...)` | Character and byte ranges, repertoire, `non_blank`, and format. |
| `sequence(...)` | Item-count bounds and `unique_items`. |
| `map(...)` | Entry-count bounds. |
| `element(text(...))`, `element(decimal(...))` | Constraints on each sequence element. |
| `time(...)` | Temporal precision and normalization. |
| `decimal(...)`, `money(...)` | Decimal semantics and bounds; the two cannot be combined. |
| `reference(...)` | Direct reference to another model ID and field path. |
| `lookup_relation(...)` | Lookup against a target type in scope. |
| `codec`, `generator` | Strategy-name metadata only; the macro does not run a strategy. |
| `opaque` | Hide an external type's structure. |
| `keep_serializing` | Keep `None` or an empty supported collection in Serde output; suppress the macro's automatic `serde(default)` for that field. |

`text` accepts `min_chars`, `max_chars`, `min_bytes`, `max_bytes`,
`repertoire = unicode\|ascii`, `non_blank`, and
`format = email\|mobile\|uri\|uuid`. `sequence` accepts `min_items`,
`max_items`, and `unique_items`. `map` accepts `min_entries` and `max_entries`.
`time` uses `precision = second\|millisecond\|microsecond\|nanosecond` and
`normalization = preserve\|utc`. `decimal` and `money` accept `precision`,
`scale`, and `rounding = half_up\|half_even\|down\|up`. Money requires `scale`.
`codec` and `generator` accept `codec = "name"` or `codec(name = "name")`.

Type structure comes from `HasTypeShape`, not from parsed type-name strings.
Supported wrappers include scalars, `Option<T>`, `Vec<T>`, `LinkedList<T>`,
`VecDeque<T>`, `HashSet<T>`, `BTreeSet<T>`, `HashMap<K, V>`, `BTreeMap<K, V>`,
`BinaryHeap<T>`, fixed arrays, and other derived models. `Option<Vec<String>>`
and `Vec<Option<String>>` remain distinct; only an outer `Option` makes a
field nullable.

## `#[Enum]`: Fieldless Enum Capabilities

`#[Enum]` rewrites a fieldless enum and emits variant metadata for it.

### Accepted shapes

Every variant must be a unit variant. Data-carrying variants, structs, unions,
and generic enums are rejected.

### Default traits and naming

Unless disabled, an enum receives `Clone`, `Copy`, `Debug`, `Eq`, `PartialEq`,
`PartialOrd`, `Ord`, `Hash`, `Serialize`, and `Deserialize`. The macro adds
`#[must_use]` unless one is already present. Serde variant names use
`SCREAMING_SNAKE_CASE`.

`Display` writes the canonical serialized name, not the Rust identifier:
`AccountStatus::Suspended` displays as `SUSPENDED`.

### Canonical names

The macro generates:

```rust
pub const fn name(&self) -> &'static str;
pub fn from_name(name: &str) -> Option<Self>;
```

Both methods, `Display`, Serde, and `EnumVariantMetadata` share the same name.
By default that name is the variant ident in `SCREAMING_SNAKE_CASE`. A variant
may override it:

```rust
use qubit_model_derive::Enum;

#[Enum(id = "example.SerializedStatus")]
enum SerializedStatus {
    #[serde(rename = "reviewing")]
    Reviewing,
    #[serde(rename(serialize = "invalid-state"))]
    Invalid,
}

assert_eq!(SerializedStatus::Reviewing.name(), "reviewing");
assert_eq!(
    SerializedStatus::from_name("invalid-state"),
    Some(SerializedStatus::Invalid)
);
```

Empty or duplicate serialized names are compile errors. Enum variants do not
accept `#[field(...)]` or `#[model(...)]` attributes.

## Advanced Usage

### Disabling default capabilities

Both macros accept the same family of `no_*` flags. Unknown `no_*` names are
rejected. Dependency rules are applied after parsing:

- `no_partial_eq` also disables `Eq`, `PartialOrd`, and `Ord`
- `no_eq` or `no_partial_ord` also disables `Ord`

`no_copy` is valid only on `#[Enum]`. `no_debug` does not remove `Display`; a
struct can still print with Debug-shaped `Display` when `Debug` is off.

```rust
use qubit_model_derive::Model;

#[Model(id = "example.Relaxed", no_display, no_eq, no_hash, no_serialize)]
struct Relaxed {
    value: f64,
}
```

### Redaction

`#[Model(..., redact)]` or `#[Enum(..., redact)]` enables the redaction derive
explicitly. On a struct, any field-level `#[redact(...)]` enables it
automatically. Field semantics are delegated to `qubit-redact`; this crate does
not keep a second redaction implementation.

```rust
use qubit_model_derive::Model;
use qubit_redact::Redactor;

#[Model(id = "example.Credential")]
struct Credential {
    username: String,
    #[field(opaque)]
    #[redact(level = "secret")]
    password: String,
}

let value = Credential {
    username: "alice".to_owned(),
    password: "raw-secret".to_owned(),
};
let output = Redactor::standard().redact(&value);
assert!(!output.text().as_str().contains("raw-secret"));
assert!(!serde_json::to_string(&value).unwrap().contains("raw-secret"));
```

When redaction is on, `Debug`, `Display`, and `Serialize` follow `qubit-redact`
field modes and the global disabled-policy. Direct Serde has no summary
channel; use `Redactor::redact` when completion or audit reasons matter.
Deserialize stays available unless `no_deserialize` is set.

### Relations and graph validation

A direct reference names a stable target model ID. The source crate does not
need a Cargo dependency on that target:

```rust
use qubit_model_derive::Model;

#[Model(id = "example.Organization")]
struct Organization {
    #[field(identifier)]
    id: i64,
}

#[Model(id = "example.Membership")]
struct Membership {
    #[field(reference(
        target = "example.Organization",
        target_field = id,
        must_exist = true
    ))]
    organization_id: i64,
}
```

`lookup_relation(target = Organization, target_field = id)` instead names a
Rust type that must be in scope. `reference` may also set `same_as` to a local
field path.

The derive validates one model only. Call `ModelRegistry::validate_graph()`
from a linked complete set to check target existence, target-field
compatibility, `same_as`, lookup relations, ownership, required-reference
cycles, and ownership cycles.

### Opaque fields and textual value objects

For an external type that intentionally lacks `HasTypeShape`, mark the field
`opaque`:

```rust
use qubit_model_derive::Model;

struct ExternalToken;

#[Model(id = "example.ImportRecord")]
struct ImportRecord {
    #[field(opaque)]
    token: ExternalToken,
}
```

An opaque field keeps recognized container wrappers and represents the leaf as
`TypeShape::Opaque`. It cannot combine with `text`, `sequence`, `map`, `time`,
`decimal`, or `money`.

A named struct that should itself behave as text uses `textual`. A newtype of
`String` inherits `TypeCapabilities::TEXT` without that marker:

```rust
use qubit_model_derive::Model;

#[Model(id = "example.Phone", textual)]
struct Phone {
    country_area: Option<String>,
    city_area: Option<String>,
    number: String,
}

#[Model(id = "example.PhoneLoginParams")]
struct PhoneLoginParams {
    #[field(text(format = mobile))]
    mobile: Option<Phone>,
}
```

## Errors and Diagnostics

Failures are compile errors on the offending syntax. Typical causes:

| Diagnostic | What to check |
| --- | --- |
| Missing `qubit-model-metadata` or `serde` | The consuming crate's `[dependencies]`. |
| Missing `qubit-redact` | Required only when `redact` or `#[redact(...)]` is present. |
| `#[Model]` on an enum | Switch to `#[Enum]`. |
| `#[Enum]` on a struct or data-carrying variant | Switch to `#[Model]`, or drop the variant payload. |
| Missing or duplicate `id` | Every declaration needs one `id = "module.Type"`. |
| ID type segment mismatch | The final ID segment must equal the Rust type name. |
| Unsupported shape | No generics, unions, multi-field tuples, or data enums. |
| Wrong attribute scope | Model-level keys only on named structs; field attributes only on fields. |
| Capability mismatch | `text` needs a text-capable type; `ignore_case` has the same requirement. |
| Duplicate serialized enum name | Two variants collapsed to the same Serde name. |
| `no_copy` on a struct | Allowed only on `#[Enum]`. |

`nullable` and `computed` are rejected with an explicit message: use
`Option<T>`, and declare a real field instead of a computed one.

Decimal `scale` must not exceed `precision`. For an unknown external type,
implement `HasTypeShape` or choose `opaque`.

## Troubleshooting

1. Confirm the consuming crate lists `qubit-model-derive`, `qubit-model-metadata`,
   and `serde`.
2. Confirm the macro matches the shape: struct → `#[Model]`, fieldless enum →
   `#[Enum]`.
3. If a field constraint fails, inspect the field type's `HasTypeShape`
   capabilities rather than its type-name spelling. Type aliases are resolved
   by Rust.
4. If a relation looks valid locally but fails later, run
   `ModelRegistry::validate_graph()` on the linked set. The derive does not
   prove that a target ID exists in another crate.
5. If enum `name()` / `from_name()` disagree with a handwritten mapping, check
   `#[serde(rename)]` and remember the default is `SCREAMING_SNAKE_CASE`, not
   the Rust ident.

## Limitations and Best Practices

Keep domain constraints beside the declaration and consume metadata through
`qubit-model-metadata`. Do not treat this crate as a validator, an ORM, or a
schema exporter.

- Codec and generator attributes store strategy names only.
- Sensitive-value handling is expressed with `qubit-redact`, not with a
  `sensitive` field attribute.
- `ownership(owner = Type)` names a Rust type in scope; `reference` names a
  stable model ID string.
- Prefer field shorthands for single-field keys, and model-level attributes for
  composite keys.

## Further Reading

- [Project README](../README.md)
- [Runtime metadata user guide](../../rs-model-metadata/doc/user_guide.md)
- [Redaction runtime guide](https://github.com/qubit-ltd/rs-redact/blob/main/doc/user_guide.md)
- [API documentation](https://docs.rs/qubit-model-derive)
- [中文用户手册](user_guide.zh_CN.md)
