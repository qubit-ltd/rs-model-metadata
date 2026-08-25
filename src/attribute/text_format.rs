// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Parsed semantic text format.
#[derive(Clone, Copy)]
pub(crate) enum TextFormat {
    /// Email address syntax.
    Email,
    /// Mainland China mobile telephone number syntax.
    Mobile,
    /// URI syntax.
    Uri,
    /// UUID string syntax.
    Uuid,
}
