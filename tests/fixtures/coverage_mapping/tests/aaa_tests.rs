// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! The first executable contains an unused mapping for the inline read.

use coverage_mapping_fixture::Record;

#[test]
fn test_ordinary_read() {
    assert_eq!(Record { value: 7 }.ordinary(), 7);
}
