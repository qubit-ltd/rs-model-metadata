// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::redact_metadata::RedactMetadata;
use super::selector_position::SelectorPosition;
use super::validator_metadata::ValidatorMetadata;
use crate::constraint_metadata::ConstraintMetadata;
use crate::metadata::CodecMetadata;

/// Non-recursive semantics applied to a collection position.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::{SelectorMetadata, SelectorPosition};
///
/// let selector = SelectorMetadata::new(SelectorPosition::Element, &[], &[], None, None);
/// assert_eq!(selector.position(), SelectorPosition::Element);
/// assert!(selector.codec().is_none());
/// ```
#[derive(Clone, Copy, Debug)]
pub struct SelectorMetadata {
    /// The nested collection position described by this metadata.
    position: SelectorPosition,
    /// Constraints applied at the selected position.
    constraints: &'static [ConstraintMetadata],
    /// Validators applied at the selected position.
    validators: &'static [ValidatorMetadata],
    /// The optional codec declaration for the selected position.
    codec: Option<&'static CodecMetadata>,
    /// The optional redaction declaration for the selected position.
    redact: Option<&'static RedactMetadata>,
}

impl SelectorMetadata {
    /// Creates selector metadata.
    ///
    /// # Parameters
    ///
    /// * `position` - Collection position described by this selector.
    /// * `constraints` - Constraints applied at that position.
    /// * `validators` - Validators applied at that position.
    /// * `codec` - Optional codec declaration for the selected value.
    /// * `redact` - Optional redaction policy for the selected value.
    ///
    /// # Returns
    ///
    /// Metadata for the selected collection position.
    #[must_use]
    #[inline]
    pub const fn new(
        position: SelectorPosition,
        constraints: &'static [ConstraintMetadata],
        validators: &'static [ValidatorMetadata],
        codec: Option<&'static CodecMetadata>,
        redact: Option<&'static RedactMetadata>,
    ) -> Self {
        Self {
            position,
            constraints,
            validators,
            codec,
            redact,
        }
    }

    /// Returns the nested collection position described by this metadata.
    ///
    /// # Returns
    ///
    /// The element, map-key, or map-value position represented here.
    #[must_use]
    #[inline]
    pub const fn position(&self) -> SelectorPosition {
        self.position
    }

    /// Returns constraints in source order.
    ///
    /// # Returns
    ///
    /// Constraints declared for this position without reordering.
    #[must_use]
    #[inline]
    pub const fn constraints(&self) -> &'static [ConstraintMetadata] {
        self.constraints
    }

    /// Returns validators in source order.
    ///
    /// # Returns
    ///
    /// Validators declared for this position without reordering.
    #[must_use]
    #[inline]
    pub const fn validators(&self) -> &'static [ValidatorMetadata] {
        self.validators
    }

    /// Returns the codec declaration, or `None` when absent.
    ///
    /// # Returns
    ///
    /// The selector codec metadata, or `None` when no codec was declared.
    #[must_use]
    #[inline]
    pub const fn codec(&self) -> Option<&'static CodecMetadata> {
        self.codec
    }

    /// Returns the redaction declaration, or `None` when absent.
    ///
    /// # Returns
    ///
    /// The selector redaction metadata, or `None` when absent.
    #[must_use]
    #[inline]
    pub const fn redact(&self) -> Option<&'static RedactMetadata> {
        self.redact
    }
}
