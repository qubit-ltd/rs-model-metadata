use qubit_model_derive::Model;

#[Model(no_redact)]
struct Secret {
    #[redact(level = "high")]
    value: String,
}

fn main() {}
