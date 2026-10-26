// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Identifier-assignment strategies for model fields.

/// Strategy for assigning an identifier from application or database state.
#[derive(Clone, Copy)]
pub(crate) enum IdentifierAssignmentIr {
    /// The application assigns the identifier before persistence.
    Application,
    /// The database assigns the identifier during persistence.
    Database,
}
