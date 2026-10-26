// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// The value position affected by a redact declaration.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::{RedactMetadata, RedactModeMetadata, RedactPosition};
///
/// let redact = RedactMetadata::new(None, RedactModeMetadata::Skip, RedactPosition::MapKey);
/// assert_eq!(redact.position(), RedactPosition::MapKey);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RedactPosition {
    /// The field value itself.
    Field,
    /// A sequence element.
    Element,
    /// A map key.
    MapKey,
    /// A map value.
    MapValue,
}
