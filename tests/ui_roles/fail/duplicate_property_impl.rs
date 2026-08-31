use qubit_model_derive::Model;
use qubit_model_derive::ModelProperties;

#[Model]
struct Item {
    value: String,
}

#[ModelProperties]
impl Item {
    pub fn value(&self) -> &str { &self.value }
}

#[ModelProperties]
impl Item {
    pub fn value_length(&self) -> usize { self.value.len() }
}

fn main() {}
