use qubit_model_derive::Model;

#[Model(no_eq)]
struct InvalidModel {
    value: u8,
}

fn main() {}
