// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Runtime coverage for resolved Projection producers and projectors.

use std::sync::LazyLock;

use model_runtime::__private::ReflectedRef;
use model_runtime::__private::qubit_id::Id;
use model_runtime::metadata::PropertyValue;
use model_runtime::metadata::TypeMetadata;
use model_runtime::registry::ModelRegistry;
use model_runtime::resolve::ProjectionExecutionError;
use model_runtime::resolve::ResolveInputs;
use model_runtime::resolve::StructureResolver;
use qubit_model_derive::Entity;
use qubit_model_derive::ModelImpl;
use qubit_model_derive::Projection;

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

// A projection may be borrowed from an independent cache; it is not an
// Entity storage field, where embedding a Projection is deliberately invalid.
static CACHED_PROJECTION: LazyLock<GoodProjection> = LazyLock::new(|| GoodProjection {
    id: Id::new(7),
    name: "cached".to_owned(),
});

#[Projection(id = "projection.Bad", source = Source)]
struct BadProjection {
    #[identifier]
    id: Id,
}

#[ModelImpl]
impl Source {
    pub fn borrowed(&self) -> &GoodProjection {
        &CACHED_PROJECTION
    }

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
    let graph = StructureResolver::new(ResolveInputs {
        roots: &[],
        models: registry,
    })
    .resolve()
    .expect("valid projection graph");
    assert_eq!(graph.projection_producers().len(), 3);

    let source = Source {
        id: Id::new(7),
        name: "source".to_owned(),
    };
    let good = graph
        .projection_producers()
        .iter()
        .find(|producer| producer.property().name() == "good")
        .expect("good producer");
    assert_eq!(good.source().type_id(), TypeMetadata::of::<Source>().type_id());
    assert_eq!(
        good.projection().type_id(),
        TypeMetadata::of::<GoodProjection>().type_id()
    );
    let getter = good.projector().expect("executable getter");
    assert_eq!(getter.rust_method_name(), "good");
    assert!(std::ptr::eq(getter, good.property().getter().expect("property getter")));
    assert!(matches!(
        good.project(ReflectedRef::new(&7_u32)),
        Err(ProjectionExecutionError::Field(_))
    ));
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
    let borrowed = graph
        .projection_producers()
        .iter()
        .find(|producer| producer.property().name() == "borrowed")
        .expect("borrowed producer");
    let PropertyValue::Borrowed(value) = borrowed
        .project(ReflectedRef::new(&source))
        .expect("matching borrowed identifier")
    else {
        panic!("borrowed projector must preserve the borrow");
    };
    assert!(std::ptr::eq(
        value.downcast_ref::<GoodProjection>().expect("projection type"),
        &*CACHED_PROJECTION
    ));
}
