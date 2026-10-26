// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use crate::metadata::CodecReference;
use crate::metadata::CodecSource;

/// One codec occurrence.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::{CodecMetadata, CodecReference, CodecSource};
///
/// static REFERENCE: CodecReference = CodecReference::DeclaredId("example.codec");
/// let codec = CodecMetadata::new(&REFERENCE, CodecSource::Field);
/// assert_eq!(codec.source(), CodecSource::Field);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct CodecMetadata {
    /// The Rust-type or textual codec declaration.
    codec: &'static CodecReference,
    /// The metadata location that supplied the declaration.
    source: CodecSource,
}

impl CodecMetadata {
    /// Creates codec metadata.
    ///
    /// # Parameters
    ///
    /// * `codec` - Static Rust-type or registry-ID declaration.
    /// * `source` - Metadata location that supplied the declaration.
    ///
    /// # Returns
    ///
    /// Codec metadata preserving the declaration and its source.
    #[must_use]
    #[inline]
    pub const fn new(codec: &'static CodecReference, source: CodecSource) -> Self {
        Self { codec, source }
    }

    /// Returns the codec declaration.
    ///
    /// # Returns
    ///
    /// The static codec reference attached to this occurrence.
    #[must_use]
    #[inline]
    pub const fn codec(&self) -> &'static CodecReference {
        self.codec
    }

    /// Returns the declaration source.
    ///
    /// # Returns
    ///
    /// The field, canonical value, or selector source for this declaration.
    #[must_use]
    #[inline]
    pub const fn source(&self) -> CodecSource {
        self.source
    }
}
