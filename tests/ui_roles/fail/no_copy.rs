use qubit_model_derive::Enum;

#[Enum(no_copy)]
enum Status {
    Ready,
    Failed,
}

fn requires_copy<T: Copy>() {}

fn main() {
    requires_copy::<Status>();
}
