use qubit_model_derive::Model;
use qubit_model_derive::ModelImpl;

#[Model]
struct Item {
    value: String,
}

#[ModelImpl]
impl Item {
    pub fn value(&self) -> &str {
        &self.value
    }
}

#[ModelImpl]
impl Item {
    pub fn value_length(&self) -> usize {
        self.value.len()
    }
}

fn main() {}
