// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Classifies how a property stores or computes its value.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::PropertyStorageKind;
///
/// let kind = PropertyStorageKind::Computed;
/// assert_eq!(kind, PropertyStorageKind::Computed);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PropertyStorageKind {
    /// A reflected field stores the value.
    FieldBacked,
    /// A getter computes the value.
    Computed,
    /// No reflected field or explicit getter provides the value.
    Virtual,
}
