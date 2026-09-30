// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Source coordinates for one field or selector declaration.

use crate::metadata::SelectorPosition;

/// Identifies an occurrence without requiring a registered model ID.
///
/// A location can retain generator-provided file coordinates, owner identity,
/// and field or variant position. [`Self::unknown`] represents metadata with no
/// source coordinates; [`Self::with_selector`] adds a selected container
/// position without changing the field coordinates.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::DeclarationLocation;
/// use qubit_model_metadata::metadata::SelectorPosition;
///
/// let location = DeclarationLocation::unknown()
///     .with_selector(SelectorPosition::Element);
/// assert_eq!(location.file, None);
/// assert_eq!(location.selector, Some(SelectorPosition::Element));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeclarationLocation {
    /// Source file containing the declaration, if supplied by a generator.
    pub file: Option<&'static str>,
    /// One-based source line.
    pub line: Option<u32>,
    /// One-based source column.
    pub column: Option<u32>,
    /// Rust owner type name, when concrete metadata is available.
    pub owner: Option<&'static str>,
    /// Enum variant index; absent for struct fields.
    pub variant: Option<usize>,
    /// Field index within the struct or variant.
    pub field: Option<usize>,
    /// Selected container position; absent for the whole field.
    pub selector: Option<SelectorPosition>,
}

impl DeclarationLocation {
    /// Represents hand-constructed metadata without fabricated source
    /// coordinates.
    ///
    /// # Returns
    ///
    /// A location with all source coordinates and declaration indices absent.
    #[must_use]
    #[inline]
    pub const fn unknown() -> Self {
        Self {
            file: None,
            line: None,
            column: None,
            owner: None,
            variant: None,
            field: None,
            selector: None,
        }
    }

    /// Selects a container position while preserving the field source.
    ///
    /// # Parameters
    ///
    /// * `selector` - Position within the selected container.
    ///
    /// # Returns
    ///
    /// The location with `selector` set and all other coordinates preserved.
    #[must_use]
    #[inline]
    pub const fn with_selector(mut self, selector: SelectorPosition) -> Self {
        self.selector = Some(selector);
        self
    }
}
