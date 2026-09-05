// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Runtime coverage for resolved Projection producers and projectors.

use model_runtime::__private::qubit_id::Id;
use model_runtime::ModelRegistry;
use model_runtime::ModelResolver;
use model_runtime::ProjectionExecutionError;
use model_runtime::PropertyValue;
use model_runtime::ReflectedRef;
use model_runtime::ResolveInputs;
use qubit_codec::ValueCodecRegistry;
use qubit_model_derive::Entity;
use qubit_model_derive::ModelImpl;
use qubit_model_derive::Projection;
use qubit_validator::ValidatorRegistry;

#[Entity(id = "projection.Source")]
struct Source {
    #[identifier]
    id: Id,
    name: String,
}

#[Projection(id = "projection.Good", source = Source)]
struct GoodProjection {
    #[identifier]
    id: Id,
    name: String,
}

#[Projection(id = "projection.Bad", source = Source)]
struct BadProjection {
    #[identifier]
    id: Id,
}

#[ModelImpl]
impl Source {
    pub fn good(&self) -> GoodProjection {
        GoodProjection {
            id: self.id,
            name: self.name.clone(),
        }
    }

    pub fn bad(&self) -> BadProjection {
        BadProjection {
            id: Id::new(self.id.value() + 1),
        }
    }
}

#[test]
fn test_resolver_discovers_and_executes_projection_producers() {
    let registry = ModelRegistry::try_global().expect("valid registration index");
    let graph = ModelResolver::new(ResolveInputs {
        models: registry,
        validators: ValidatorRegistry::global(),
        codecs: ValueCodecRegistry::global(),
    })
    .resolve_all()
    .expect("valid projection graph");
    assert_eq!(graph.projection_producers().len(), 2);

    let source = Source {
        id: Id::new(7),
        name: "source".to_owned(),
    };
    let good = graph
        .projection_producers()
        .iter()
        .find(|producer| {
            producer
                .projection()
                .model_id()
                .is_some_and(|id| id.as_str() == "projection.Good")
        })
        .expect("good producer");
    let PropertyValue::Owned(value) = good.project(ReflectedRef::new(&source)).expect("matching identifier") else {
        panic!("owned getter must produce an owned projection");
    };
    assert_eq!(
        value
            .downcast_ref::<GoodProjection>()
            .map(|projection| projection.name.as_str()),
        Some("source"),
    );

    let bad = graph
        .projection_producers()
        .iter()
        .find(|producer| {
            producer
                .projection()
                .model_id()
                .is_some_and(|id| id.as_str() == "projection.Bad")
        })
        .expect("bad producer");
    assert!(matches!(
        bad.project(ReflectedRef::new(&source)),
        Err(ProjectionExecutionError::IdentifierMismatch),
    ));
}
