// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Public API regression coverage for invalid Projection producers.

use std::any::TypeId;

use model_runtime::__private::qubit_id::Id;
use model_runtime::metadata::TypeMetadata;
use model_runtime::registry::ModelRegistry;
use model_runtime::resolve::ResolveErrorKind;
use model_runtime::resolve::StructureResolver;
use qubit_model_derive::Entity;
use qubit_model_derive::ModelImpl;
use qubit_model_derive::Projection;

#[Entity(id = "projection.error.Source")]
struct Source {
    #[identifier]
    id: Id,
}

#[Entity(id = "projection.error.OtherSource")]
struct OtherSource {
    #[identifier]
    id: Id,
}

#[Projection(id = "projection.error.Wrong", source = OtherSource)]
struct WrongProjection {
    #[identifier]
    id: Id,
}

#[ModelImpl]
impl Source {
    pub fn wrong_projection(&self) -> WrongProjection {
        WrongProjection { id: self.id }
    }
}

#[test]
fn test_resolver_reports_projection_producer_source_mismatch() {
    let models = ModelRegistry::try_global().expect("valid registration index");
    let errors = StructureResolver::for_static_roots(models, [TypeMetadata::of::<Source>()])
        .resolve()
        .expect_err("a Projection produced by the wrong Entity must be rejected");

    assert_eq!(errors.errors().len(), 1);
    let error = &errors.errors()[0];
    assert_eq!(error.kind(), ResolveErrorKind::InvalidProjectionProducer);
    assert_eq!(error.owner_type_id(), Some(TypeId::of::<Source>()));
    assert_eq!(error.model_id(), Some("projection.error.Wrong"));
}
