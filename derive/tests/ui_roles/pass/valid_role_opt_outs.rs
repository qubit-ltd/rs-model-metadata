use qubit_model_derive::{Enum, Model, Value};

#[Value(no_eq)]
struct ValueWithoutEquality {
    value: u8,
}

#[Enum(no_hash)]
enum EnumWithoutHash {
    One,
}

#[Model(no_partial_eq)]
struct ModelWithoutPartialEquality {
    value: u8,
}

fn main() {}
