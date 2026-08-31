use qubit_model_derive::Enum;
use qubit_model_derive::Model;
use qubit_model_derive::Value;

#[Model]
struct TupleModel(u64, u64);

#[Value]
struct EmptyValue;

#[Enum]
struct NotAnEnum {
    value: u64,
}

fn main() {}
