// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! One declaration omitted from codec binding due to unavailable type metadata.

use crate::metadata::DeclarationLocation;
use crate::metadata::TypeMetadata;

use super::CodecBindSkipReason;

/// Diagnostic for a field or selector whose target type could not be resolved.
///
/// The property follows the same canonical path used by codec occurrences.
/// A skip does not imply that the declaration should have a codec.
#[derive(Clone, Debug)]
pub struct CodecBindSkip {
    model: &'static TypeMetadata,
    property: Box<str>,
    location: DeclarationLocation,
    reason: CodecBindSkipReason,
}

impl CodecBindSkip {
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
