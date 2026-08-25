// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// A semantic format accepted by a text constraint.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::TextFormat;
///
/// assert_ne!(TextFormat::Email, TextFormat::Uuid);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextFormat {
    /// An email address.
    Email,
    /// A mainland China mobile telephone number.
    Mobile,
    /// A URI.
    Uri,
    /// A UUID string.
    Uuid,
}
