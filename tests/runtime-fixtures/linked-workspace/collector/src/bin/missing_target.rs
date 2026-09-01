// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_model_metadata::ModelRegistry;
use qubit_model_metadata::ModelResolveErrorKind;
use qubit_model_metadata::ModelResolver;
use qubit_model_metadata::ResolveInputs;

fn main() {
    let _ = core::mem::size_of::<model_a::MissingTarget>();
    let registry = ModelRegistry::try_global()
        .expect("a missing reference target must not invalidate registration");
    assert!(registry.get("test.linked.Absent").is_none());
    assert!(registry.get("test.linked.MissingTarget").is_some());
    let errors = ModelResolver::new(ResolveInputs {
        models: registry,
        validators: qubit_model_metadata::__private::qubit_validator::ValidatorRegistry::global(),
        codecs: qubit_model_metadata::__private::qubit_codec::ValueCodecRegistry::global(),
    })
        .resolve_all()
        .expect_err("the missing reference target must be reported by graph validation");
    assert!(errors.errors().iter().any(|error| {
        error.kind() == ModelResolveErrorKind::MissingModelId
            && error.model_id() == Some("test.linked.Absent")
    }));
}
