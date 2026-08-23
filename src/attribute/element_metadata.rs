// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::AttributeMetadata;

/// Static constraints applied to each element of a sequence field.
#[derive(Clone, Copy, Debug)]
pub struct ElementMetadata {
    /// Element-level attributes in declaration order.
    attributes: &'static [AttributeMetadata],
}

impl ElementMetadata {
    /// Creates element metadata from static element-level attributes.
    ///
    /// # Parameters
    ///
    /// * `attributes` - The non-empty text or decimal constraints applied to
    ///   each element.
    ///
    /// # Panics
    ///
    /// Panics when `attributes` is empty, contains unsupported metadata, or
    /// repeats a constraint kind. Element-type compatibility is validated by
    /// [`crate::FieldMetadata::new`].
    ///
    /// # Returns
    ///
    /// Element metadata containing the supplied constraints.
    #[must_use]
    pub const fn new(attributes: &'static [AttributeMetadata]) -> Self {
        assert!(
            !attributes.is_empty(),
            "element metadata requires at least one attribute"
        );
        validate_element_metadata_attributes(attributes);
        Self { attributes }
    }

    /// Returns the element-level attributes in declaration order.
    ///
    /// # Returns
    ///
    /// The statically allocated element-level attribute slice.
    #[must_use]
    #[inline(always)]
    pub const fn attributes(self) -> &'static [AttributeMetadata] {
        self.attributes
    }
}

/// Validates the supported and unique constraints stored for collection
/// elements.
///
/// # Parameters
///
/// * `attributes` - The non-empty element attribute slice to validate.
///
/// # Panics
///
/// Panics when an attribute is not a text or decimal constraint or when either
/// constraint kind appears more than once.
const fn validate_element_metadata_attributes(attributes: &'static [AttributeMetadata]) {
    let mut index = 0;
    while index < attributes.len() {
        match attributes[index] {
            AttributeMetadata::Text(_) | AttributeMetadata::Decimal(_) => {}
            _ => panic!("element metadata only supports text and decimal attributes"),
        }
        let mut previous = 0;
        while previous < index {
            match (attributes[previous], attributes[index]) {
                (AttributeMetadata::Text(_), AttributeMetadata::Text(_))
                | (AttributeMetadata::Decimal(_), AttributeMetadata::Decimal(_)) => {
                    panic!("element metadata attributes must be unique")
                }
                _ => {}
            }
            previous += 1;
        }
        index += 1;
    }
}
