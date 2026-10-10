// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Stable, portable model identifiers shared by model-aware crates.

mod has_model_id;
mod model_id;

pub use has_model_id::HasModelId;
pub use model_id::ModelId;
pub use model_id::ModelIdBuf;
pub use model_id::ModelIdError;
