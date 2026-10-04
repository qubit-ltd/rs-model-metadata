//! Stable, portable model identifiers shared by model-aware crates.

mod has_model_id;
mod model_id;

pub use has_model_id::HasModelId;
pub use model_id::ModelId;
pub use model_id::ModelIdBuf;
pub use model_id::ModelIdError;
