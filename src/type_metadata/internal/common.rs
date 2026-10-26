// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use core::ptr::eq;

use crate::metadata::AbiViolation;
use crate::metadata::FieldMetadata;

/// Returns whether `candidate` is one of the declared field overlays.
#[must_use]
pub(super) fn contains_field(fields: &[FieldMetadata], candidate: &FieldMetadata) -> bool {
    fields.iter().any(|field| eq(field, candidate))
}

/// Creates a stable ABI diagnostic for invalid generated metadata.
#[inline]
pub(super) const fn abi_violation(code: &'static str, message: &'static str) -> AbiViolation {
    AbiViolation::new(code, message)
}
