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

/// Returns whether `candidate` is one of the declared field overlays by
/// identity.
///
/// # Parameters
///
/// * `fields` - The field overlays declared for the model.
/// * `candidate` - The field overlay checked using pointer identity.
///
/// # Returns
///
/// `true` when `candidate` refers to an element of `fields`.
#[must_use]
pub(in crate::type_metadata) fn contains_field(fields: &[FieldMetadata], candidate: &FieldMetadata) -> bool {
    fields.iter().any(|field| eq(field, candidate))
}

/// Creates a stable ABI diagnostic for invalid generated metadata.
///
/// # Parameters
///
/// * `code` - The stable diagnostic code identifying the ABI invariant.
/// * `message` - The static description of the invariant violation.
///
/// # Returns
///
/// An ABI violation carrying the supplied code and message.
#[inline]
pub(in crate::type_metadata) const fn abi_violation(code: &'static str, message: &'static str) -> AbiViolation {
    AbiViolation::new(code, message)
}
