// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Stable FNV-1a fingerprints for generated private identifiers.

/// Computes the stable FNV-1a fingerprint for `value`.
pub(crate) fn stable_fingerprint(value: &str) -> u64 {
    value.bytes().fold(0xcbf29ce484222325_u64, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    })
}

#[cfg(test)]
mod tests {
    use super::stable_fingerprint;

    /// Confirms the shared implementation preserves standard FNV-1a output.
    #[test]
    fn test_stable_fingerprint() {
        assert_eq!(stable_fingerprint(""), 0xcbf29ce484222325);
        assert_eq!(stable_fingerprint("hello"), 0xa430d84680aabd0b);
    }
}
