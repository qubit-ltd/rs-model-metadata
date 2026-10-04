use qubit_model_derive::Model;
use model_runtime::metadata::HasModelId;

#[Model]
struct Anonymous {
    value: u8,
}

#[Model(id = "fixture.Generic")]
struct Generic<T> {
    value: T,
}

fn requires_model_id<T: HasModelId>() {}

fn main() {
    requires_model_id::<Anonymous>();
    requires_model_id::<Generic<u8>>();
}
