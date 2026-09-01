// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_model_metadata::ModelRegistry;
use qubit_model_metadata::ModelResolver;
use qubit_model_metadata::ResolveInputs;
use qubit_model_metadata::TypeMetadata;

fn main() {
    let _ = core::mem::size_of::<model_a::Source>();
    let _ = core::mem::size_of::<model_b::Target>();
    let registry = ModelRegistry::try_global()
        .expect("cross-crate registrations should be valid");
    assert!(registry.get("test.linked.Source").is_some());
    assert!(registry.get("test.linked.Target").is_some());
    let graph = ModelResolver::new(ResolveInputs {
        models: registry,
        validators: qubit_model_metadata::__private::qubit_validator::ValidatorRegistry::global(),
        codecs: qubit_model_metadata::__private::qubit_codec::ValueCodecRegistry::global(),
    })
        .resolve_all()
        .expect("cross-crate reference should resolve");
    let field = TypeMetadata::of::<model_a::Source>()
        .field("target_id")
        .expect("source field");
    assert_eq!(
        graph.reference(field).expect("resolved reference").target().model_id().unwrap().as_str(),
        "test.linked.Target",
    );
}
