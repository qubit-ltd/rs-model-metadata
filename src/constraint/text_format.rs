// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Semantic formats supported by text constraints.

/// A semantic format accepted by a text constraint.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::TextFormat;
///
/// assert_ne!(TextFormat::EmailAscii, TextFormat::Uuid);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextFormat {
    /// An ASCII email address shape; this does not check whether the mailbox
    /// exists.
    EmailAscii,
    /// A mainland China mobile telephone number.
    Mobile,
    /// A URI.
    Uri,
    /// A UUID string.
    Uuid,
}
