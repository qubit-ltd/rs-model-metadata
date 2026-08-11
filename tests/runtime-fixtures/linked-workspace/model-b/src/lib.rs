use qubit_model_derive::Model;

#[Model(id = "test.linked.Target")]
pub struct Target {
    pub id: i64,
}

#[qubit_model_derive::Model(id = "test.linked.Duplicate")]
#[cfg(feature = "duplicate-fixture")]
pub struct Duplicate;
