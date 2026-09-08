// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Immutable model registration and lookup.

mod error;
mod model_entry;
mod model_registry;

pub use self::error::ModelRegistryError;
pub use self::error::ModelRegistryErrorKind;
pub use self::model_entry::ModelEntry;
pub use self::model_registry::ModelRegistry;
