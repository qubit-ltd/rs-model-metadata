// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Original causes retained by contextual resolution errors.

use thiserror::Error as ThisError;

use crate::metadata::ModelMetadataError;
use crate::metadata::PropertyResolutionError;

/// The original failure retained by a contextual model resolution error.
/// A resolver exposes this cause through `ResolveError::cause()` after
/// `StructureResolver::resolve()` returns contextual failures.
/// The value is supplied by the resolver as part of a `ResolveErrors` result;
/// callers inspect it through the public diagnostic accessors.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::resolve::ModelResolutionCause;
/// use qubit_model_metadata::resolve::ResolveErrors;
///
/// fn has_property_cause(errors: &ResolveErrors) -> bool {
///     errors.errors().iter().any(|error| {
///         matches!(error.cause(), Some(ModelResolutionCause::Properties(_)))
///     })
/// }
/// ```
#[must_use]
#[derive(Clone, Debug, ThisError)]
pub enum ModelResolutionCause {
    /// Reflected metadata could not be obtained or validated.
    #[error(transparent)]
    Metadata(#[from] ModelMetadataError),
    /// A property set could not be obtained or assembled.
    #[error(transparent)]
    Properties(#[from] PropertyResolutionError),
}
