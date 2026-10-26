// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Selects which layer assigns an entity identifier.
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
pub enum IdentifierAssignment {
    /// Application code assigns the value.
    Application,
    /// The backing database assigns the value.
    Database,
}
