// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Enumerates one descriptor layer without applying consumer traversal policy.

use qubit_reflect::TypeDescriptor;

use crate::structure::StructuralEdge;
use crate::structure::StructuralEdgeKind;

/// Calls `visit` once for each direct structural child of `descriptor`.
///
/// Fields, variants, and tuple elements retain declaration order; map keys
/// precede values. Targets retain their original `TypeRef`, including opaque
/// and symbolic references. No registry lookup, recursion, getter invocation,
/// or path allocation occurs. References and raw pointers remain leaves.
///
/// The callback receives static declaration facts and may impose its own
/// resolution, recursion, and cycle policy. Roots without visible structure
/// invoke no callback. Panics from the callback propagate to the caller.
///
/// # Parameters
///
/// - `descriptor`: Frozen root whose direct child relations are enumerated.
/// - `visit`: Callback invoked for each relation in structural order.
pub(crate) fn children(descriptor: &'static TypeDescriptor, mut visit: impl FnMut(StructuralEdge)) {
    if let Some(optional) = descriptor.as_optional() {
        visit(StructuralEdge {
            target: optional.element_type(),
            kind: StructuralEdgeKind::OptionalElement,
        });
    } else if let Some(pointer) = descriptor.as_smart_pointer() {
        visit(StructuralEdge {
            target: pointer.pointee_type(),
            kind: StructuralEdgeKind::PointerPointee,
        });
    } else if let Some(sequence) = descriptor.as_sequence() {
        visit(StructuralEdge {
            target: sequence.element_type(),
            kind: StructuralEdgeKind::SequenceElement,
        });
    } else if let Some(set) = descriptor.as_set() {
        visit(StructuralEdge {
            target: set.element_type(),
            kind: StructuralEdgeKind::SetElement,
        });
    } else if let Some(array) = descriptor.as_array() {
        visit(StructuralEdge {
            target: array.element_type(),
            kind: StructuralEdgeKind::ArrayElement,
        });
    } else if let Some(slice) = descriptor.as_slice() {
        visit(StructuralEdge {
            target: slice.element_type(),
            kind: StructuralEdgeKind::SliceElement,
        });
    } else if let Some(map) = descriptor.as_map() {
        visit(StructuralEdge {
            target: map.key_type(),
            kind: StructuralEdgeKind::MapKey,
        });
        visit(StructuralEdge {
            target: map.value_type(),
            kind: StructuralEdgeKind::MapValue,
        });
    } else if let Some(tuple) = descriptor.as_tuple() {
        for (index, target) in tuple.elements().iter().enumerate() {
            visit(StructuralEdge {
                target,
                kind: StructuralEdgeKind::TupleElement { index },
            });
        }
    } else if descriptor.as_struct().is_some() {
        for field in descriptor.fields() {
            visit(StructuralEdge {
                target: field.field_type(),
                kind: StructuralEdgeKind::StructField {
                    index: field.index(),
                    name: field.rust_name(),
                    query_name: field.query_name(),
                },
            });
        }
    } else if descriptor.as_enum().is_some() {
        for variant in descriptor.variants() {
            for field in variant.fields() {
                visit(StructuralEdge {
                    target: field.field_type(),
                    kind: StructuralEdgeKind::EnumVariantField {
                        variant_index: variant.index(),
                        variant_name: variant.rust_name(),
                        field_index: field.index(),
                        field_name: field.rust_name(),
                        query_name: field.query_name(),
                    },
                });
            }
        }
    }
}
