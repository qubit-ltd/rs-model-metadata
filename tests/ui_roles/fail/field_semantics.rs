use qubit_model_derive::Model;

#[Model(no_redact)]
struct InvalidFields {
    #[unique]
    #[unique(ignore_case = false)]
    name: String,
    #[key_part(order = 1)]
    first: u64,
    #[key_part(order = 1)]
    second: u64,
}

fn main() {}
