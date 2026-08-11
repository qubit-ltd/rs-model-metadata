// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the public `Model` attribute expansion.

use model_runtime::metadata_of;
use qubit_model_derive::Model;

#[allow(dead_code)]
#[Model(id = "test.derive.CurrentModel")]
struct CurrentModel {
    value: String,
}

#[allow(dead_code)]
#[Model(id = "test.derive.SecondModel")]
struct SecondModel {
    value: String,
}

#[test]
fn test_model_attributes_share_metadata_expansion() {
    let current = metadata_of::<CurrentModel>();
    let second = metadata_of::<SecondModel>();

    assert_eq!(current.fields().count(), 1);
    assert_eq!(second.fields().count(), 1);
    assert_eq!(
        current.field("value").expect("current field").name(),
        "value"
    );
    assert_eq!(second.field("value").expect("second field").name(), "value");
}
