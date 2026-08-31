use qubit_model_derive::Model;

#[Model]
struct Borrowed<'a> {
    value: &'a str,
}

fn main() {}
