// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Shareable errors produced while building model registries.

mod model_registry_error;
mod model_registry_error_kind;

pub use self::model_registry_error::ModelRegistryError;
pub use self::model_registry_error_kind::ModelRegistryErrorKind;
