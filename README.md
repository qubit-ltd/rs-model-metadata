# qubit-model-metadata

[简体中文](README.zh_CN.md) | [User guide](doc/user_guide.md)

`qubit-model-metadata` is the domain-semantic layer for Qubit Rust models. It
reuses `qubit-reflect` as the single source of Rust structure and adds model
roles, field constraints, properties, stable model IDs, registration, and
explicit cross-model resolution.

The companion `qubit-model-derive` crate generates the metadata for
`#[Entity]`, `#[Projection]`, `#[Model]`, `#[Enum]`, `#[Value]`, and
`#[ModelProperties]` declarations.

```rust,ignore
use qubit_model_derive::Entity;
use qubit_model_metadata::{ModelDescriptorExt, TypeDescriptor, TypeMetadata};

#[Entity(id = "example.User")]
struct User {
    #[identifier]
    id: u64,
    #[unique(ignore_case = true)]
    email: String,
}

let metadata = TypeMetadata::of::<User>();
assert_eq!(metadata.model_id().unwrap().as_str(), "example.User");
assert!(std::ptr::eq(metadata.descriptor(), TypeDescriptor::of::<User>()));
assert!(metadata.descriptor().model_metadata().is_some());
```

Core boundaries:

- `TypeDescriptor`, `FieldDescriptor`, `TypeRef`, and dynamic values come from
  `qubit-reflect`; this crate does not create a parallel reflection system.
- Static metadata lookup does not initialize the global model registry.
- Cross-crate IDs, references, projection sources, queries, validators, and
  codecs are bound only by an explicit `ModelResolver` pass over model,
  validator, and codec registries.
- Validators use the typed `qubit-validator` contract; codecs use
  `qubit-codec::ValueCodecDescriptor`. Both registries expose safely erased
  executable descriptors after exact value-type checks.
- Generated metadata crosses the hidden v2 ABI only after descriptor, field,
  property, role, and codec invariants have been checked.

See the [user guide](doc/user_guide.md) for the complete workflow and the
[API documentation](https://docs.rs/qubit-model-metadata).

Licensed under Apache-2.0.
