// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// The repertoire accepted by a text constraint.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::TextRepertoire;
///
/// assert_eq!(TextRepertoire::default(), TextRepertoire::Unicode);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextRepertoire {
    /// All Unicode scalar values are accepted.
    #[default]
    Unicode,
    /// Only ASCII characters are accepted.
    Ascii,
}
