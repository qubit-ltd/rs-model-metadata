use qubit_model_derive::Model;

#[Model]
#[derive(Debug)]
struct DebugBypass {
    secret: String,
}

#[Model]
#[derive(serde::Serialize)]
struct SerializeBypass {
    secret: String,
}

fn main() {}
