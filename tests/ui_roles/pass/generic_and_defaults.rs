use qubit_model_derive::Enum;
use qubit_model_derive::Entity;
use qubit_model_derive::Model;
use qubit_model_derive::Projection;
use qubit_model_derive::Value;

#[Entity(id = "trybuild.Source")]
struct Source {
    #[identifier]
    id: u64,
}

#[Projection(source = Source)]
struct View {
    #[identifier]
    id: u64,
}

#[Model(no_serialize, no_deserialize)]
struct Buffer<const N: usize> {
    bytes: [u8; N],
}

#[Value(no_redact, transparent)]
struct Revision(u64);

#[Enum(no_copy)]
enum Status {
    Ready,
    Failed,
}

#[Model(no_redact)]
#[derive(serde::Serialize)]
struct ExistingSafeSerialize {
    value: String,
}

fn main() {
    let _ = Buffer::<4> { bytes: [0; 4] };
    let _ = Revision(1).to_string();
    let _ = Status::Ready;
    let _ = View { id: 1 };
    let _ = ExistingSafeSerialize { value: String::new() };
}
