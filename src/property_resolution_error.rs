// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Failures when resolving effective model properties.

use std::sync::Arc;

use qubit_reflect::capability::CapabilityError;
use qubit_reflect::error::ContextError;
use thiserror::Error;

use crate::metadata::PropertyBuildErrors;

/// Distinguishes unavailable reflection from invalid property declarations.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::PropertyResolutionError;
///
/// // `TypeMetadata::try_properties` supplies this error when resolution fails.
/// fn handle_property_resolution_error(error: PropertyResolutionError) {
///     match error {
///         PropertyResolutionError::Capability(error) => eprintln!("{error}"),
///         PropertyResolutionError::Reflection(error) => eprintln!("{error}"),
///         PropertyResolutionError::Assembly(errors) => eprintln!("{errors}"),
///     }
/// }
/// ```
#[must_use]
#[derive(Clone, Debug, Error)]
pub enum PropertyResolutionError {
    /// Intrinsic capabilities cannot form a valid set for the property owner.
    #[error("property capability resolution failed: {0}")]
    Capability(Box<CapabilityError>),
    /// The process-wide reflection snapshot could not be initialized.
    #[error("reflection registry error: {0}")]
    Reflection(#[from] ContextError),
    /// Linked field and method declarations cannot form valid properties.
    #[error("property assembly error: {0}")]
    Assembly(#[from] Arc<PropertyBuildErrors>),
}

impl From<CapabilityError> for PropertyResolutionError {
    /// Wraps a capability resolution failure in the corresponding error
    /// variant.
    fn from(error: CapabilityError) -> Self {
        Self::Capability(Box::new(error))
    }
}
