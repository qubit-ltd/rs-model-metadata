// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! One declaration omitted from codec binding due to unavailable type metadata.

use super::CodecBindSkipReason;
use crate::metadata::DeclarationLocation;
use crate::metadata::TypeMetadata;

/// Diagnostic for a field or selector whose target type could not be resolved.
///
/// The property follows the same canonical path used by codec occurrences.
/// A skip does not imply that the declaration should have a codec.
#[derive(Clone, Debug)]
pub struct CodecBindSkip {
    /// Metadata for the model that owns the declaration.
    model: &'static TypeMetadata,
    /// Canonical path of the field or enum payload.
    property: Box<str>,
    /// Available source coordinates and declaration position.
    location: DeclarationLocation,
    /// Type-resolution failure that caused the declaration to be skipped.
    reason: CodecBindSkipReason,
}

impl CodecBindSkip {
    /// Constructs a skip diagnostic for one declaration.
    ///
    /// # Parameters
    ///
    /// * `model` - Metadata for the owning model.
    /// * `property` - Canonical field or payload path.
    /// * `location` - Available declaration coordinates and selector position.
    /// * `reason` - Why the declaration type could not be resolved.
    ///
    /// # Returns
    ///
    /// The skip diagnostic with its property path stored as a string slice.
    pub(super) fn new(
        model: &'static TypeMetadata,
        property: Box<str>,
        location: DeclarationLocation,
        reason: CodecBindSkipReason,
    ) -> Self {
        Self {
            model,
            property,
            location,
            reason,
        }
    }

    /// Returns metadata for the model that owns the skipped declaration.
    #[must_use]
    pub const fn model(&self) -> &'static TypeMetadata {
        self.model
    }

    /// Returns the canonical property path for the skipped declaration.
    #[must_use]
    pub fn property(&self) -> &str {
        &self.property
    }

    /// Returns source coordinates and field or selector position, if available.
    #[must_use]
    pub const fn location(&self) -> &DeclarationLocation {
        &self.location
    }

    /// Returns why the declaration could not be inspected.
    #[must_use]
    pub const fn reason(&self) -> CodecBindSkipReason {
        self.reason
    }
}
