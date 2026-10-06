use qubit_model_derive::Model;

#[Model(eq, no_partial_eq)]
struct Invalid { value: u8 }

fn main() {}
