// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Provider function type for generated concrete model metadata.

use crate::metadata::TypeMetadata;

/// Typed adapter returning the static metadata supplied by a generated model.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::__private::ModelMetadataProvider;
/// use qubit_model_metadata::metadata::TypeMetadata;
///
/// fn metadata_from(provider: ModelMetadataProvider) -> &'static TypeMetadata {
///     provider()
/// }
/// ```
///
/// Generated model registrations supply the provider callback.
#[doc(hidden)]
pub type ModelMetadataProvider = fn() -> &'static TypeMetadata;
