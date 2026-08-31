use qubit_model_derive::Model;
use qubit_model_derive::Value;

#[Model]
struct RedundantMarker {
    #[keep_serializing]
    value: u64,
}

#[Value]
struct PositionalMarker(#[keep_serializing] Option<u64>);

fn main() {}
