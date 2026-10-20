// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Direct structural edges retain declaration order and context-bound targets.

use std::any::TypeId;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

use qubit_reflect::Reflect;
use qubit_reflect::ReflectionContext;
use qubit_reflect::TypeDescriptor;

use crate::structure::StructuralEdge;
use crate::structure::StructuralEdgeKind;
use crate::structure::children;

#[allow(dead_code)]
#[Reflect]
struct NamedRecord {
    number: u8,
    enabled: bool,
}

#[allow(dead_code)]
#[Reflect]
struct PositionalRecord(u8, bool);

#[allow(dead_code)]
#[Reflect]
enum Event {
    Ready,
    Pair(u8, bool),
    Named { value: String },
}

fn structural_context() -> ReflectionContext {
    ReflectionContext::builder()
        .include_type::<(u8, Option<bool>)>()
        .include_type::<()>()
        .include_type::<u8>()
        .include_type::<bool>()
        .include_type::<Option<bool>>()
        .include_type::<NamedRecord>()
        .include_type::<PositionalRecord>()
        .include_type::<Event>()
        .include_type::<String>()
        .include_type::<BTreeMap<String, u8>>()
        .include_type::<BTreeSet<u8>>()
        .include_type::<[u8; 3]>()
        .include_type::<[u8]>()
        .include_type::<Option<u8>>()
        .include_type::<Box<u8>>()
        .include_type::<Vec<u8>>()
        .include_type::<&'static u8>()
        .include_type::<*const u8>()
        .build()
        .expect("all structural test types should register")
}

fn collect_edges<'ctx>(descriptor: TypeDescriptor<'ctx>) -> Vec<StructuralEdge<'ctx>> {
    let mut edges = Vec::new();
    children(descriptor, |edge| edges.push(edge));
    edges
}

fn assert_edges<'ctx>(descriptor: TypeDescriptor<'ctx>, expected: &[(StructuralEdgeKind, TypeId)]) {
    let actual = collect_edges(descriptor);
    assert_eq!(actual.len(), expected.len(), "unexpected direct edge count");
    for (edge, (kind, target)) in actual.iter().zip(expected) {
        assert_eq!(edge.kind, *kind);
        assert_eq!(edge.target.type_id(), *target);
        assert!(std::ptr::eq(edge.target.context(), descriptor.context()));
    }
}

#[test]
fn test_structure_children_tuple_order_and_single_layer() {
    let context = structural_context();
    assert_edges(
        context.type_of::<(u8, Option<bool>)>(),
        &[
            (StructuralEdgeKind::TupleElement { index: 0 }, TypeId::of::<u8>()),
            (
                StructuralEdgeKind::TupleElement { index: 1 },
                TypeId::of::<Option<bool>>(),
            ),
        ],
    );
    assert!(collect_edges(context.type_of::<()>()).is_empty());
}

#[test]
fn test_structure_children_struct_names_and_positions() {
    let context = structural_context();
    assert_edges(
        context.type_of::<NamedRecord>(),
        &[
            (
                StructuralEdgeKind::StructField {
                    index: 0,
                    name: Some("number"),
                    query_name: None,
                },
                TypeId::of::<u8>(),
            ),
            (
                StructuralEdgeKind::StructField {
                    index: 1,
                    name: Some("enabled"),
                    query_name: None,
                },
                TypeId::of::<bool>(),
            ),
        ],
    );
    assert_edges(
        context.type_of::<PositionalRecord>(),
        &[
            (StructuralEdgeKind::TupleElement { index: 0 }, TypeId::of::<u8>()),
            (StructuralEdgeKind::TupleElement { index: 1 }, TypeId::of::<bool>()),
        ],
    );
}

#[test]
fn test_structure_children_enum_variant_and_field_order() {
    let context = structural_context();
    assert_edges(
        context.type_of::<Event>(),
        &[
            (
                StructuralEdgeKind::EnumVariantField {
                    variant_index: 1,
                    variant_name: "Pair",
                    field_index: 0,
                    field_name: Some("0"),
                    query_name: None,
                },
                TypeId::of::<u8>(),
            ),
            (
                StructuralEdgeKind::EnumVariantField {
                    variant_index: 1,
                    variant_name: "Pair",
                    field_index: 1,
                    field_name: Some("1"),
                    query_name: None,
                },
                TypeId::of::<bool>(),
            ),
            (
                StructuralEdgeKind::EnumVariantField {
                    variant_index: 2,
                    variant_name: "Named",
                    field_index: 0,
                    field_name: Some("value"),
                    query_name: None,
                },
                TypeId::of::<String>(),
            ),
        ],
    );
}

#[test]
fn test_structure_children_map_key_before_value() {
    let context = structural_context();
    assert_edges(
        context.type_of::<BTreeMap<String, u8>>(),
        &[
            (StructuralEdgeKind::MapKey, TypeId::of::<String>()),
            (StructuralEdgeKind::MapValue, TypeId::of::<u8>()),
        ],
    );
}

#[test]
fn test_structure_children_container_and_wrapper_edges() {
    let context = structural_context();
    for (descriptor, kind) in [
        (context.type_of::<Vec<u8>>(), StructuralEdgeKind::SequenceElement),
        (context.type_of::<BTreeSet<u8>>(), StructuralEdgeKind::SetElement),
        (context.type_of::<[u8; 3]>(), StructuralEdgeKind::ArrayElement),
        (context.type_of::<[u8]>(), StructuralEdgeKind::SliceElement),
        (context.type_of::<Option<u8>>(), StructuralEdgeKind::OptionalElement),
        (context.type_of::<Box<u8>>(), StructuralEdgeKind::PointerPointee),
    ] {
        assert_edges(descriptor, &[(kind, TypeId::of::<u8>())]);
    }
}

#[test]
fn test_structure_children_resolves_concrete_generic_targets_in_context() {
    let context = ReflectionContext::builder()
        .include_type::<Option<String>>()
        .build()
        .expect("optional descriptor should register");
    let descriptor = context.type_of::<Option<String>>();
    let edges = collect_edges(descriptor);
    assert_edges(
        descriptor,
        &[(StructuralEdgeKind::OptionalElement, TypeId::of::<String>())],
    );
    assert!(std::ptr::eq(edges[0].target.context(), &context));
}

#[test]
fn test_structure_children_leaves_preserve_pointer_boundaries() {
    let context = structural_context();
    for descriptor in [
        context.type_of::<u8>(),
        context.type_of::<String>(),
        context.type_of::<&'static u8>(),
        context.type_of::<*const u8>(),
    ] {
        assert!(collect_edges(descriptor).is_empty());
    }
}
