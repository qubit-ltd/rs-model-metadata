#![allow(dead_code)]

#[qubit_model_derive::Model(
    id = "test.derive.InvalidReferenceIndex",
    no_clone,
    no_debug,
    no_display,
    no_partial_eq,
    no_hash,
    no_serialize,
    no_deserialize
)]
struct InvalidReferenceIndex {
    #[field(reference(target = "test.derive.Target", target_field = id), index)]
    target_id: i64,
}

#[qubit_model_derive::Model(
    id = "test.derive.Target",
    no_clone,
    no_debug,
    no_display,
    no_partial_eq,
    no_hash,
    no_serialize,
    no_deserialize
)]
struct Target {
    id: i64,
}

fn main() {}
