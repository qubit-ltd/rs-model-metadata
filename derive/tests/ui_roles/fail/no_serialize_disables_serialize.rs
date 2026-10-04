use qubit_model_derive::Model;

#[Model(no_serialize)]
struct Value {
    value: String,
}

fn requires_serialize<T: serde::Serialize>() {}
fn requires_deserialize<T: for<'de> serde::Deserialize<'de>>() {}

fn main() {
    requires_deserialize::<Value>();
    requires_serialize::<Value>();
}
