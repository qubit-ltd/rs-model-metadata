// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Stable FNV-1a fingerprints for generated private identifiers.

/// Computes the 64-bit FNV-1a hash of `value`'s UTF-8 bytes.
///
/// The standard offset basis and prime make the result stable across runs.
#[must_use]
#[inline]
pub(crate) fn stable_fingerprint(value: &str) -> u64 {
    value.bytes().fold(0xcbf29ce484222325_u64, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    })
}
