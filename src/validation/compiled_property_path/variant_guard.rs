// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Active-variant guard for enum payload property paths.

/// Active-variant check performed after the containing enum has been read.
#[derive(Clone, Copy, Debug)]
pub(crate) struct VariantGuard {
    /// Number of owner-path properties to read before checking the enum.
    pub(crate) after_steps: usize,
    /// Reflected variant whose payload field is being validated.
    pub(crate) declaring_type: std::any::TypeId,
    /// Index of the variant in its declaring enum's reflected variant list.
    pub(crate) variant_index: usize,
}
