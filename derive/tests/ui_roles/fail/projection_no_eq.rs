use qubit_model_derive::Projection;

#[Projection(id = "test.Projection", no_eq)]
struct InvalidProjection {
    #[identifier]
    id: model_runtime::__private::qubit_id::Id,
}

fn main() {}
