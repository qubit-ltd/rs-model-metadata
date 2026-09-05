// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Internal re-exports for immutable model registry components.

mod model_entry;
mod model_registry;
mod model_registry_error;

pub use self::model_entry::ModelEntry;
pub use self::model_registry::ModelRegistry;
pub use self::model_registry_error::ModelRegistryError;
pub use self::model_registry_error::ModelRegistryErrorKind;
