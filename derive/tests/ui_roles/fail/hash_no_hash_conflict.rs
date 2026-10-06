use qubit_model_derive::Model;

#[Model(eq, hash, no_hash)]
struct Invalid { value: u8 }

fn main() {}
