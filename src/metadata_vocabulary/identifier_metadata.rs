// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::identifier_assignment::IdentifierAssignment;

/// Describes identifier assignment for one field.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::{IdentifierAssignment, IdentifierMetadata};
///
/// let identifier = IdentifierMetadata::new(IdentifierAssignment::Database);
/// assert_eq!(identifier.assigned_by(), IdentifierAssignment::Database);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IdentifierMetadata {
    /// The layer responsible for assigning the identifier value.
    assigned_by: IdentifierAssignment,
}

impl IdentifierMetadata {
    /// Creates identifier metadata.
    ///
    /// # Parameters
    ///
    /// * `assigned_by` - Layer that supplies the identifier value.
    ///
    /// # Returns
    ///
    /// Metadata recording the identifier assignment source.
    #[must_use]
    #[inline]
    pub const fn new(assigned_by: IdentifierAssignment) -> Self {
        Self { assigned_by }
    }

    /// Returns the identifier assignment source.
    ///
    /// # Returns
    ///
    /// The application or database layer responsible for assigning the value.
    #[must_use]
    #[inline]
    pub const fn assigned_by(&self) -> IdentifierAssignment {
        self.assigned_by
    }
}
