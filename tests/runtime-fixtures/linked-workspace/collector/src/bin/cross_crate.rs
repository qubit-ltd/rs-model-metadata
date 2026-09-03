// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow test-file-name
// The filename is part of a Cargo or trybuild fixture protocol.

use qubit_model_metadata::ModelRegistry;
use qubit_model_metadata::ModelResolver;
use qubit_model_metadata::ResolveInputs;
use qubit_model_metadata::TypeMetadata;
use qubit_model_metadata::__private::qubit_codec::ValueCodecRegistry;
use qubit_model_metadata::__private::qubit_validator::ValidatorRegistry;
use model_a::Source;
use model_b::Target;

fn main() {
    let _ = core::mem::size_of::<Source>();
    let _ = core::mem::size_of::<Target>();
    let registry = ModelRegistry::try_global()
        .expect("cross-crate registrations should be valid");
    assert!(registry.get("test.linked.Source").is_some());
    assert!(registry.get("test.linked.Target").is_some());
    let graph = ModelResolver::new(ResolveInputs {
        models: registry,
        validators: ValidatorRegistry::global(),
        codecs: ValueCodecRegistry::global(),
    })
        .resolve_all()
        .expect("cross-crate reference should resolve");
    let field = TypeMetadata::of::<Source>()
        .field("target_id")
        .expect("source field");
    assert_eq!(
        graph.reference(field).expect("resolved reference").target().model_id().unwrap().as_str(),
        "test.linked.Target",
    );
}
