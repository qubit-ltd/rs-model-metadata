use qubit_model_derive::Enum;
use qubit_model_derive::Model;
use qubit_model_derive::Value;

#[Model]
struct InvalidModelIdentifier {
    #[identifier]
    id: u64,
}

#[Value]
struct InvalidValueReference {
    #[reference(entity_id = "example.Entity")]
    entity: u64,
}

#[Enum]
enum InvalidEnumReference {
    Data {
        #[reference(entity_id = "example.Entity")]
        entity: u64,
    },
}

fn main() {}
