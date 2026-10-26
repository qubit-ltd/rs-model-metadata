// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Crate-private assembly error sharing behavior.

use std::sync::Arc;

use crate::metadata::PropertyBuildErrors;
use crate::metadata::PropertyResolutionError;

#[test]
fn test_assembly_variant_keeps_shared_build_errors_when_cloned() {
    let cause = Arc::new(PropertyBuildErrors::new(Vec::new()));
    let error = PropertyResolutionError::Assembly(Arc::clone(&cause));
    let clone = error.clone();

    let PropertyResolutionError::Assembly(retained) = clone else {
        panic!("assembly error should keep its variant when cloned");
    };
    assert!(Arc::ptr_eq(&retained, &cause));
    assert_eq!(error.to_string(), "property assembly error: ");
}
