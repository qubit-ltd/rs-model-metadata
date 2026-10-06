// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Direct structural edges retain declaration order and unresolved references.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use qubit_reflect::__private::codegen_v3::descriptor;
use qubit_reflect::Reflect;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::descriptor::OpaqueTypeDescriptor;
use qubit_reflect::descriptor::SmartPointerKind;
use qubit_reflect::descriptor::TypeRef;
use qubit_reflect::expression::TypeExpression;

use crate::structure::StructuralEdge;
use crate::structure::StructuralEdgeKind;
use crate::structure::children;

#[allow(dead_code)]
#[derive(Reflect)]
struct NamedRecord {
    #[reflect(rename = "lookup_number")]
    number: u8,
    enabled: bool,
}

#[allow(dead_code)]
#[derive(Reflect)]
struct PositionalRecord(u8, bool);

#[allow(dead_code)]
#[derive(Reflect)]
enum Event {
    Ready,
    Pair(u8, bool),
    #[reflect(rename = "lookup_named")]
    Named {
        #[reflect(rename = "lookup_value")]
        value: String,
    },
}

static OPAQUE: OpaqueTypeDescriptor = descriptor::opaque_member::<String>();
static OPAQUE_REF: TypeRef = TypeRef::Opaque(&OPAQUE);
static SYMBOLIC_REF: TypeRef = TypeRef::Symbolic(TypeExpression::SelfType);
static OPTIONAL_SYMBOLIC: TypeDescriptor =
    descriptor::optional::<Option<String>>("Option<String>", &SYMBOLIC_REF);
static POINTER_OPAQUE: TypeDescriptor =
    descriptor::smart_pointer::<Box<String>>("Box<String>", SmartPointerKind::Box, &OPAQUE_REF);

/// Collects one descriptor layer for assertions without resolving its targets.
fn collect_edges(descriptor: &'static TypeDescriptor) -> Vec<StructuralEdge> {
    let mut edges = Vec::new();
    children(descriptor, |edge| edges.push(edge));
    edges
}

/// Checks edge kinds and target identities in the exact traversal order.
fn assert_edges(
    descriptor: &'static TypeDescriptor,
    expected: &[(StructuralEdgeKind, &'static TypeDescriptor)],
) {
    let actual = collect_edges(descriptor);
    assert_eq!(actual.len(), expected.len(), "unexpected direct edge count");
    for (edge, (kind, target)) in actual.iter().zip(expected) {
        assert_eq!(edge.kind, *kind);
        assert!(std::ptr::eq(
            edge.target.as_resolved().expect("resolved fixture target"),
            *target
        ));
    }
}

#[test]
fn test_structure_children_tuple_order_and_single_layer() {
    assert_edges(
        TypeDescriptor::of::<(u8, Option<bool>)>(),
        &[
            (
                StructuralEdgeKind::TupleElement { index: 0 },
                TypeDescriptor::of::<u8>(),
            ),
            (
                StructuralEdgeKind::TupleElement { index: 1 },
                TypeDescriptor::of::<Option<bool>>(),
            ),
        ],
    );
    assert!(collect_edges(TypeDescriptor::of::<()>()).is_empty());
}

#[test]
fn test_structure_children_struct_names_and_positions() {
    assert_edges(
        TypeDescriptor::of::<NamedRecord>(),
        &[
            (
                StructuralEdgeKind::StructField {
                    index: 0,
                    name: Some("number"),
                    query_name: Some("lookup_number"),
                },
                TypeDescriptor::of::<u8>(),
            ),
            (
                StructuralEdgeKind::StructField {
                    index: 1,
                    name: Some("enabled"),
                    query_name: Some("enabled"),
                },
                TypeDescriptor::of::<bool>(),
            ),
        ],
    );
    assert_edges(
        TypeDescriptor::of::<PositionalRecord>(),
        &[
            (
                StructuralEdgeKind::StructField {
                    index: 0,
                    name: None,
                    query_name: None,
                },
                TypeDescriptor::of::<u8>(),
            ),
            (
                StructuralEdgeKind::StructField {
                    index: 1,
                    name: None,
                    query_name: None,
                },
                TypeDescriptor::of::<bool>(),
            ),
        ],
    );
}

#[test]
fn test_structure_children_enum_variant_and_field_order() {
    assert_edges(
        TypeDescriptor::of::<Event>(),
        &[
            (
                StructuralEdgeKind::EnumVariantField {
                    variant_index: 1,
                    variant_name: "Pair",
                    field_index: 0,
                    field_name: None,
                    query_name: None,
                },
                TypeDescriptor::of::<u8>(),
            ),
            (
                StructuralEdgeKind::EnumVariantField {
                    variant_index: 1,
                    variant_name: "Pair",
                    field_index: 1,
                    field_name: None,
                    query_name: None,
                },
                TypeDescriptor::of::<bool>(),
            ),
            (
                StructuralEdgeKind::EnumVariantField {
                    variant_index: 2,
                    variant_name: "Named",
                    field_index: 0,
                    field_name: Some("value"),
                    query_name: Some("lookup_value"),
                },
                TypeDescriptor::of::<String>(),
            ),
        ],
    );
}

#[test]
fn test_structure_children_map_key_before_value() {
    assert_edges(
        TypeDescriptor::of::<BTreeMap<String, u8>>(),
        &[
            (StructuralEdgeKind::MapKey, TypeDescriptor::of::<String>()),
            (StructuralEdgeKind::MapValue, TypeDescriptor::of::<u8>()),
        ],
    );
}

#[test]
fn test_structure_children_container_and_wrapper_edges() {
    for (descriptor, kind) in [
        (
            TypeDescriptor::of::<Vec<u8>>(),
            StructuralEdgeKind::SequenceElement,
        ),
        (
            TypeDescriptor::of::<BTreeSet<u8>>(),
            StructuralEdgeKind::SetElement,
        ),
        (
            TypeDescriptor::of::<[u8; 3]>(),
            StructuralEdgeKind::ArrayElement,
        ),
        (
            TypeDescriptor::of::<[u8]>(),
            StructuralEdgeKind::SliceElement,
        ),
        (
            TypeDescriptor::of::<Option<u8>>(),
            StructuralEdgeKind::OptionalElement,
        ),
        (
            TypeDescriptor::of::<Box<u8>>(),
            StructuralEdgeKind::PointerPointee,
        ),
    ] {
        assert_edges(descriptor, &[(kind, TypeDescriptor::of::<u8>())]);
    }
}

#[test]
fn test_structure_children_preserves_unresolved_targets() {
    for (descriptor, target, kind) in [
        (
            &OPTIONAL_SYMBOLIC,
            &SYMBOLIC_REF,
            StructuralEdgeKind::OptionalElement,
        ),
        (
            &POINTER_OPAQUE,
            &OPAQUE_REF,
            StructuralEdgeKind::PointerPointee,
        ),
    ] {
        let edges = collect_edges(descriptor);
        assert_eq!(edges.len(), 1);
        let edge = edges.first().expect("unresolved relation remains visible");
        assert_eq!(edge.kind, kind);
        assert!(std::ptr::eq(edge.target, target));
        assert!(edge.target.as_resolved().is_none());
    }
}

#[test]
fn test_structure_children_leaves_preserve_pointer_boundaries() {
    for descriptor in [
        TypeDescriptor::of::<u8>(),
        TypeDescriptor::of::<String>(),
        TypeDescriptor::of::<&'static u8>(),
        TypeDescriptor::of::<*const u8>(),
    ] {
        assert!(collect_edges(descriptor).is_empty());
    }
}
