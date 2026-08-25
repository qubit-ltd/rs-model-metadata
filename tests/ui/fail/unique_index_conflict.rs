#![allow(dead_code)]

#[qubit_model_derive::Model(
    id = "test.derive.InvalidUniqueIndex",
    no_clone,
    no_debug,
    no_display,
    no_partial_eq,
    no_hash,
    no_serialize,
    no_deserialize
)]
struct InvalidUniqueIndex {
    #[field(unique, index)]
    code: String,
}

fn main() {}
