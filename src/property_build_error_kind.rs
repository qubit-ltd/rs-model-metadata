// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Stable categories for local property assembly failures.

/// Classifies one invalid field/getter/setter property combination.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PropertyBuildErrorKind {
    /// A property name is empty or declared more than once after merging.
    InvalidName,
    /// A property has no field, getter, or setter source.
    MissingSource,
    /// A backing field does not belong to the model type.
    ForeignField,
    /// A property's selected type differs from its backing field.
    FieldTypeMismatch,
    /// A getter target or output differs from the selected property type.
    GetterTypeMismatch,
    /// A setter target or input differs from the selected property type.
    SetterTypeMismatch,
}
