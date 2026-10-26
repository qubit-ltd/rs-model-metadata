// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use crate::metadata::SelectorPosition;

/// Identifies where a codec declaration originated.
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
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CodecSource {
    /// A codec declared directly on a field.
    Field,
    /// A codec declared for a value object's canonical representation.
    CanonicalValue,
    /// A codec declared for a nested selector position.
    Selector(SelectorPosition),
}
