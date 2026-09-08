use qubit_model_derive::Model;

#[Model(id = "test.Generic")]
struct Generic<T> {
    value: T,
}

fn main() {}
