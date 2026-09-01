use qubit_model_derive::Model;
use qubit_model_derive::ModelImpl;

#[Model]
struct Profile {
    value: String,
}

#[ModelImpl]
impl Profile {
    fn private(&self) -> &str {
        &self.value
    }

    pub async fn asynchronous(&self) -> String {
        self.value.clone()
    }

    pub fn generic<T>(&self) -> String {
        self.value.clone()
    }

    pub fn set_value(&self, value: String) {
        let _ = value;
    }
}

fn main() {}
