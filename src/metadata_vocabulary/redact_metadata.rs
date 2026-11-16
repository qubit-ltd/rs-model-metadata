// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::redact_mode_metadata::RedactModeMetadata;
use super::redact_position::RedactPosition;
use super::sensitivity::Sensitivity;

/// One redact declaration using the upstream sensitivity vocabulary.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::{
///     RedactMetadata, RedactModeMetadata, RedactPosition, Sensitivity,
/// };
///
/// let redact = RedactMetadata::new(
///     Some(Sensitivity::Personal),
///     RedactModeMetadata::Level,
///     RedactPosition::Field,
/// );
/// assert_eq!(redact.sensitivity(), Some(Sensitivity::Personal));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RedactMetadata {
    /// The optional sensitivity level applied by the redact capability.
    sensitivity: Option<Sensitivity>,
    /// The redact operation to perform.
    mode: RedactModeMetadata,
    /// The structural value position to redact.
    position: RedactPosition,
}

impl RedactMetadata {
    /// Creates redact declaration metadata.
    ///
    /// # Parameters
    ///
    /// * `sensitivity` - Optional disclosure level required for redaction.
    /// * `mode` - Redaction operation selected by the declaration.
    /// * `position` - Field, sequence element, map key, or map value affected.
    ///
    /// # Returns
    ///
    /// Metadata describing the redaction policy at the selected position.
    #[must_use]
    #[inline]
    pub const fn new(sensitivity: Option<Sensitivity>, mode: RedactModeMetadata, position: RedactPosition) -> Self {
        Self {
            sensitivity,
            mode,
            position,
        }
    }

    /// Returns the configured sensitivity, if applicable.
    ///
    /// # Returns
    ///
    /// The disclosure level, or `None` when the mode does not use one.
    #[inline]
    pub const fn sensitivity(&self) -> Option<Sensitivity> {
        self.sensitivity
    }

    /// Returns the declaration mode.
    ///
    /// # Returns
    ///
    /// The redaction operation supplied by the declaration.
    #[must_use]
    #[inline]
    pub const fn mode(&self) -> RedactModeMetadata {
        self.mode
    }

    /// Returns the affected value position.
    ///
    /// # Returns
    ///
    /// The structural field or collection position affected by redaction.
    #[must_use]
    #[inline]
    pub const fn position(&self) -> RedactPosition {
        self.position
    }
}
