// qubit-style: allow test-file-name
use qubit_model_metadata::capability::clone_key;
use qubit_model_metadata::registry::ReflectRegistry;

fn main() {
    let _ = clone_key();
    let _ = ReflectRegistry::initialize();
}
