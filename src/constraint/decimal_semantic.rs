// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// The domain meaning of a decimal value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecimalSemantic {
    /// A general-purpose decimal number.
    Number,
    /// A monetary amount.
    Money,
}
