use qubit_model_derive::Model;

#[derive(Default)]
struct BadCodec;

#[Model]
struct Encoded {
    #[codec(BadCodec)]
    value: String,
}

fn main() {}
