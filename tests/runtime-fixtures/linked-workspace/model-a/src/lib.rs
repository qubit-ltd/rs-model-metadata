use qubit_model_derive::Model;

#[Model(id = "test.linked.Source")]
pub struct Source {
    #[field(reference(target = "test.linked.Target", target_field = id))]
    pub target_id: i64,
}

#[cfg(feature = "duplicate-fixture")]
#[Model(id = "test.linked.Duplicate")]
pub struct Duplicate;

#[Model(id = "test.linked.MissingTarget")]
pub struct MissingTarget {
    #[field(reference(target = "test.linked.Absent", target_field = id))]
    pub target_id: i64,
}
