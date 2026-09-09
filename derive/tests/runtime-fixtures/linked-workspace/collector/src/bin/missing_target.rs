// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow test-file-name
// The filename is part of a Cargo or trybuild fixture protocol.

use core::mem::size_of;

use model_a::MissingTarget;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::ResolveErrorKind;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;

fn main() {
    let _ = size_of::<MissingTarget>();
    let registry = ModelRegistry::try_global().expect("a missing reference target must not invalidate registration");
    assert!(registry.metadata("test.linked.Absent").is_none());
    assert!(registry.metadata("test.linked.MissingTarget").is_some());
    let errors = StructureResolver::new(ResolveInputs {
        roots: &[],
        models: registry,
    })
    .resolve()
    .expect_err("the missing reference target must be reported by graph validation");
    assert!(errors.errors().iter().any(|error| {
        error.kind() == ResolveErrorKind::MissingModelId && error.model_id() == Some("test.linked.Absent")
    }));
}
