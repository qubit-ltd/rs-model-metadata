// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_model_metadata::ModelGraphError;
use qubit_model_metadata::ModelRegistry;

fn main() {
    let _ = core::mem::size_of::<model_a::MissingTarget>();
    let registry = ModelRegistry::try_global()
        .expect("a missing reference target must not invalidate registration");
    assert!(registry.get("test.linked.Absent").is_none());
    assert!(registry.get("test.linked.MissingTarget").is_some());
    let errors = registry
        .validate_graph()
        .expect_err("the missing reference target must be reported by graph validation");
    assert!(errors.errors().iter().any(|error| {
        matches!(
            error,
            ModelGraphError::MissingTarget { target, .. }
                if target.as_str() == "test.linked.Absent"
        )
    }));
}
