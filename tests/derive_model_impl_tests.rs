// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the shared implementation behind both public derive entry points.

use model_runtime::metadata_of;
use qubit_model_derive::{Model, ModelMetadata};

#[allow(dead_code)]
#[derive(Model)]
struct CurrentModel {
    value: String,
}

#[allow(dead_code)]
#[derive(ModelMetadata)]
struct LegacyModel {
    value: String,
}

#[test]
fn test_public_derives_share_model_metadata_expansion() {
    let current = metadata_of::<CurrentModel>();
    let legacy = metadata_of::<LegacyModel>();

    assert_eq!(current.fields().count(), 1);
    assert_eq!(legacy.fields().count(), 1);
    assert_eq!(
        current.field("value").expect("current field").name(),
        "value"
    );
    assert_eq!(legacy.field("value").expect("legacy field").name(), "value");
}
