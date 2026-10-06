use qubit_model_derive::Value as ValueRole;

#[ValueRole(no_eq)]
struct Value {
    value: String,
}

fn requires_hash<T: std::hash::Hash>() {}

fn main() {
    requires_hash::<Value>();
}
