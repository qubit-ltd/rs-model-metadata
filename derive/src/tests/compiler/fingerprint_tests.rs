// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests for stable fingerprints.

use crate::compiler::fingerprint::stable_fingerprint;

/// Confirms the shared implementation preserves standard FNV-1a output.
#[test]
fn test_stable_fingerprint() {
    assert_eq!(stable_fingerprint(""), 0xcbf29ce484222325);
    assert_eq!(stable_fingerprint("hello"), 0xa430d84680aabd0b);
}
