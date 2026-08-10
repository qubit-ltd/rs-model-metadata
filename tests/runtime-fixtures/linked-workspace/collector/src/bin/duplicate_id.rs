use qubit_model_metadata::ModelRegistry;

fn main() {
    let _ = core::mem::size_of::<model_a::Duplicate>();
    let _ = core::mem::size_of::<model_b::Duplicate>();
    let error = ModelRegistry::try_global()
        .expect_err("duplicate IDs must make the global registry invalid");
    assert!(error.to_string().contains("test.linked.Duplicate"));
}
