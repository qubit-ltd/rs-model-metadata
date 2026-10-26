// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Public conversion behavior for property resolution failures.

use std::error::Error;

use qubit_model_metadata::metadata::PropertyResolutionError;
use qubit_reflect::capability::CapabilityError;
use qubit_reflect::error::ContextError;

#[test]
fn test_reflection_conversion_keeps_the_context_error() {
    let cause = ContextError::new("registry is unavailable");
    let error = PropertyResolutionError::from(cause.clone());

    let PropertyResolutionError::Reflection(retained) = &error else {
        panic!("context error should become the reflection variant");
    };
    assert_eq!(retained, &cause);
    assert_eq!(error.to_string(), "reflection registry error: registry is unavailable");
    assert_eq!(error.source().expect("retained source").to_string(), cause.to_string());
}

#[test]
fn test_capability_conversion_keeps_the_capability_failure() {
    let cause = CapabilityError::Context(ContextError::new("context lookup failed"));
    let error = PropertyResolutionError::from(cause.clone());

    let PropertyResolutionError::Capability(retained) = &error else {
        panic!("capability error should become the capability variant");
    };
    assert_eq!(retained.as_ref(), &cause);
    assert_eq!(
        error.to_string(),
        "property capability resolution failed: reflection context failed: context lookup failed"
    );
}
