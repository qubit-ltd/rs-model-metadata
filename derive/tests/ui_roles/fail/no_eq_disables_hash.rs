use qubit_model_derive::Model;

#[Model(no_eq)]
struct Value {
    value: String,
}

fn requires_hash<T: std::hash::Hash>() {}

fn main() {
    requires_hash::<Value>();
}
