// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! The second executable actually calls both public read methods.

use coverage_mapping_fixture::Record;

#[test]
fn test_both_reads() {
    let record = Record { value: 7 };
    assert_eq!(record.value(), 7);
    assert_eq!(record.ordinary(), 7);
}
