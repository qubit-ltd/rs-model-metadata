// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Adapter type used to provide generic model definition metadata.

#[cfg(feature = "generic")]
use crate::generic::GenericModelMetadata;

/// The typed capability adapter supplied by generic model declarations.
/// Generated registrations provide this callback to return the declaration's
/// static metadata.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::__private::v8::GenericModelMetadataProvider;
///
/// fn metadata_from(
///     provider: GenericModelMetadataProvider,
/// ) -> &'static qubit_model_metadata::generic::GenericModelMetadata {
///     provider()
/// }
/// ```
#[doc(hidden)]
#[cfg(feature = "generic")]
pub type GenericModelMetadataProvider = fn() -> &'static GenericModelMetadata;
