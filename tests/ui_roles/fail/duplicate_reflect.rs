use model_runtime::Reflect;
use qubit_model_derive::Model;

#[Model]
#[derive(Reflect)]
#[reflect(crate = model_runtime)]
struct DuplicateReflect;

fn main() {}
