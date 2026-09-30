// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Failures when resolving effective model properties.

use std::sync::Arc;

use qubit_reflect::capability::CapabilityAccessError;
use qubit_reflect::error::RegistryError;
use thiserror::Error;

use crate::metadata::PropertyBuildErrors;

/// Distinguishes unavailable reflection from invalid property declarations.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::PropertyResolutionError;
///
/// fn category(error: &PropertyResolutionError) -> &'static str {
///     match error {
///         PropertyResolutionError::Capability(_) => "capability",
///         PropertyResolutionError::Reflection(_) => "reflection",
///         PropertyResolutionError::Assembly(_) => "assembly",
///     }
/// }
/// ```
#[must_use]
#[derive(Clone, Debug, Error)]
pub enum PropertyResolutionError {
    /// Intrinsic capabilities cannot form a valid set for the property owner.
    #[error("property capability resolution failed: {0}")]
    Capability(#[from] CapabilityAccessError),
    /// The process-wide reflection snapshot could not be initialized.
    #[error("reflection registry error: {0}")]
    Reflection(#[from] RegistryError),
    /// Linked field and method declarations cannot form valid properties.
    #[error("property assembly error: {0}")]
    Assembly(#[from] Arc<PropertyBuildErrors>),
}
