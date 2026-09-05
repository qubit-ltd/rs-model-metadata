// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Failures when resolving effective model properties.

use qubit_reflect::capability::CapabilityConflict;
use qubit_reflect::error::RegistryError;

use crate::PropertyBuildErrors;

/// Distinguishes unavailable reflection from invalid property declarations.
#[derive(Clone, Debug, thiserror::Error)]
pub enum PropertyResolutionError {
    /// Intrinsic capabilities cannot form a valid set for the property owner.
    #[error("property capability resolution failed: {0}")]
    Capability(#[from] CapabilityConflict),
    /// The process-wide reflection snapshot could not be initialized.
    #[error("reflection registry error: {0}")]
    Reflection(#[from] RegistryError),
    /// Linked field and method declarations cannot form valid properties.
    #[error("property assembly error: {0}")]
    Assembly(#[from] &'static PropertyBuildErrors),
}
