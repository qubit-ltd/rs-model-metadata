// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use proc_macro2::Span;

use super::named_fields_attribute::NamedFieldsAttribute;
use super::ownership_attribute::OwnershipAttribute;
use super::primary_key_attribute::PrimaryKeyAttribute;

/// Parsed model-level attribute syntax.
pub(crate) enum ModelAttribute {
    /// Declares a named value object as text-capable for field constraints.
    Textual(
        /// The span of the `textual` capability marker.
        Span,
    ),
    /// A primary-key declaration.
    PrimaryKey(
        /// Parsed `primary_key(...)` syntax for this model.
        PrimaryKeyAttribute,
    ),
    /// An index declaration.
    Index(
        /// Parsed `index(...)` name and field list.
        NamedFieldsAttribute,
    ),
    /// A logical-key declaration.
    Key(
        /// Parsed `key(...)` name and field list.
        NamedFieldsAttribute,
    ),
    /// An ownership declaration.
    Ownership(
        /// Parsed `ownership(owner = Type)` syntax.
        OwnershipAttribute,
    ),
}
