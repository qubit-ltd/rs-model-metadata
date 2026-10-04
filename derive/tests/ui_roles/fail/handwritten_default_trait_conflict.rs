use qubit_model_derive::Model;

#[Model]
struct Value;

impl std::fmt::Debug for Value {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Value")
    }
}

fn main() {}
