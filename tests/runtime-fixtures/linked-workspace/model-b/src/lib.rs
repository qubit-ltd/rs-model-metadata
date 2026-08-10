use qubit_model_derive::ModelMetadata;

#[derive(ModelMetadata)]
#[model(id = "test.linked.Target")]
pub struct Target {
    pub id: i64,
}

#[derive(ModelMetadata)]
#[cfg(feature = "duplicate-fixture")]
#[model(id = "test.linked.Duplicate")]
pub struct Duplicate;
