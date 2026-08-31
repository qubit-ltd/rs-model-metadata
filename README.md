# qubit-model-derive

[简体中文](README.zh_CN.md) | [中文用户指南](doc/user_guide.zh_CN.md)

`qubit-model-derive` provides the final attribute-macro API for Qubit Rust
models. Six macros share one parser, validation, normalization, and expansion
pipeline:

- `#[Entity]` for persistent identity-bearing models;
- `#[Projection]` for open or fixed entity views;
- `#[Model]` for ordinary structured data;
- `#[Enum]` for domain enums;
- `#[Value]` for value objects, including transparent wrappers;
- `#[ModelProperties]` for safe getter/setter-backed properties.

```rust,ignore
use qubit_model_derive::{Entity, ModelProperties};
use qubit_model_metadata::TypeMetadata;

#[Entity(id = "example.User")]
pub struct User {
    #[identifier]
    id: u64,
    #[unique(ignore_case = true)]
    #[redact(level = "medium")]
    email: String,
}

#[ModelProperties]
impl User {
    pub fn email(&self) -> &str { &self.email }
    pub fn set_email(&mut self, value: String) { self.email = value; }
}

let metadata = TypeMetadata::of::<User>();
assert!(metadata.property("email").unwrap().is_writable());
```

Generated code refers only to the resolved `qubit-model-metadata` facade. It
delegates Rust structure to `qubit-reflect`, emits model semantic overlays, and
registers only declarations with stable IDs. Renaming the runtime dependency is
supported.

The five role macros default to `Clone`, redacted `Debug`/`Display`/`Serialize`,
`Deserialize`, `PartialEq`, `Eq`, `Hash`, and `Redact`. Use the documented
`no_*` options to disable individual interfaces. `copy`, `default`,
`partial_ord`, and `ord` are opt-in; an all-unit enum is `Copy` unless
`no_copy` is specified.

Lower-case `#[validator(...)]` declarations currently produce metadata only.
Validator registration and execution are intentionally deferred. Codec types
must implement the `qubit-codec` `ValueEncoder` and `ValueDecoder` contracts.

Place the role attribute before any user-supplied `#[derive(...)]` so the model
macro can reuse or reject existing output implementations safely.

See the [Chinese user guide](doc/user_guide.zh_CN.md) and the dated design in
[`doc/2026-08-31-182016-rs-model-derive-final-api-and-implementation-design.md`](doc/2026-08-31-182016-rs-model-derive-final-api-and-implementation-design.md).

Licensed under Apache-2.0.
