use qubit_model_derive::Entity;

#[Entity(id = "example.GenericEntity")]
struct GenericEntity<T> {
    id: T,
}

fn main() {}
