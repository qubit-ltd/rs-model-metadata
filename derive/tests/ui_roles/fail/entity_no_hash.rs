use qubit_model_derive::Entity;

#[Entity(id = "test.Entity", no_hash)]
struct InvalidEntity {
    #[identifier]
    id: model_runtime::__private::qubit_id::Id,
}

fn main() {}
