// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Explicit access to a model's generated registration.

use crate::model_registration::ModelRegistration;
use crate::type_metadata::HasTypeMetadata;

/// Exposes the static registration generated for one model type.
pub trait HasModelRegistration: HasTypeMetadata {
    /// Returns the immutable registration generated for this type.
    fn model_registration() -> &'static ModelRegistration;
}

/// Returns the generated registration for `T`.
///
/// # Type Parameters
///
/// * `T` - The model type whose registration is requested.
///
/// # Returns
///
/// The static registration emitted for `T`.
#[must_use]
#[inline(always)]
pub fn registration_of<T: HasModelRegistration>() -> &'static ModelRegistration {
    T::model_registration()
}
