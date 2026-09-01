// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Runtime coverage for model-graph role and opaque boundaries.

use model_runtime::__private::qubit_id::Id;
use model_runtime::ModelRegistry;
use model_runtime::ModelResolveErrorKind;
use model_runtime::ModelResolver;
use model_runtime::ResolveInputs;
use qubit_codec::ValueCodecRegistry;
use qubit_model_derive::Entity;
use qubit_validator::ValidatorRegistry;

#[Entity(id = "graph.Nested")]
struct Nested {
    #[identifier]
    id: Id,
}

#[Entity(id = "graph.DirectOwner")]
struct DirectOwner {
    #[identifier]
    id: Id,
    nested: Nested,
}

#[Entity(id = "graph.OpaqueOwner")]
struct OpaqueOwner {
    #[identifier]
    id: Id,
    #[opaque]
    nested: Nested,
}

#[test]
fn resolver_rejects_entity_embedding_and_opaque_model_hiding() {
    let registry = ModelRegistry::try_global().expect("valid registration index");
    let errors = ModelResolver::new(ResolveInputs {
        models: registry,
        validators: ValidatorRegistry::global(),
        codecs: ValueCodecRegistry::global(),
    })
    .resolve_all()
    .expect_err("invalid graph boundaries must prevent publication");

    assert!(
        errors
            .errors()
            .iter()
            .any(|error| error.kind() == ModelResolveErrorKind::InvalidEntityNesting)
    );
    assert!(
        errors
            .errors()
            .iter()
            .any(|error| error.kind() == ModelResolveErrorKind::OpaqueModel)
    );
}
