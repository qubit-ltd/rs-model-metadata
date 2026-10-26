// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Generated metadata provider for reflected model types.

/// Generated provider kept out of the public model bound.
#[doc(hidden)]
pub trait TypeMetadataProvider {
    /// Returns the generated metadata for the implementing model type.
    ///
    /// # Returns
    ///
    /// A process-lifetime reference to the generated metadata.
    #[must_use]
    fn __type_metadata() -> &'static crate::metadata::TypeMetadata;
}
