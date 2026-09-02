use qubit_model_derive::Model;

#[Model]
struct OptionalTextUnique {
    #[unique]
    value: Option<String>,
}

fn main() {}
