use qubit_model_derive::Model;

#[derive(Model)]
#[model(id = "test.linked.Source")]
pub struct Source {
    #[model(reference(target = "test.linked.Target", target_field = id))]
    pub target_id: i64,
}

#[derive(Model)]
#[cfg(feature = "duplicate-fixture")]
#[model(id = "test.linked.Duplicate")]
pub struct Duplicate;

#[derive(Model)]
#[model(id = "test.linked.MissingTarget")]
pub struct MissingTarget {
    #[model(reference(target = "test.linked.Absent", target_field = id))]
    pub target_id: i64,
}
