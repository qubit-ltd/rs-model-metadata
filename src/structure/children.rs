// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Enumerates one context-bound descriptor layer without applying traversal
//! policy.

use qubit_reflect::TypeDescriptor;
use qubit_reflect::descriptor::TypeKind;

use crate::structure::StructuralEdge;
use crate::structure::StructuralEdgeKind;

fn type_argument<'ctx>(descriptor: TypeDescriptor<'ctx>, index: usize) -> Option<TypeDescriptor<'ctx>> {
    descriptor
        .generic_arguments()
        .get_at(index)?
        .type_argument()
        .ok()?
        .find_concrete_type()
}

/// Calls `visit` for each direct structural child that resolves in the bound
/// context.
pub(crate) fn children<'ctx>(descriptor: TypeDescriptor<'ctx>, mut visit: impl FnMut(StructuralEdge<'ctx>)) {
    match descriptor.kind() {
        TypeKind::Optional(_) => {
            if let Some(target) = type_argument(descriptor, 0) {
                visit(StructuralEdge {
                    target,
                    kind: StructuralEdgeKind::OptionalElement,
                });
            }
        }
        TypeKind::SmartPointer(_) => {
            if let Some(target) = type_argument(descriptor, 0) {
                visit(StructuralEdge {
                    target,
                    kind: StructuralEdgeKind::PointerPointee,
                });
            }
        }
        TypeKind::Sequence(_) => {
            if let Some(target) = type_argument(descriptor, 0) {
                visit(StructuralEdge {
                    target,
                    kind: StructuralEdgeKind::SequenceElement,
                });
            }
        }
        TypeKind::Set(_) => {
            if let Some(target) = type_argument(descriptor, 0) {
                visit(StructuralEdge {
                    target,
                    kind: StructuralEdgeKind::SetElement,
                });
            }
        }
        TypeKind::Array(_) => {
            if let Some(target) = type_argument(descriptor, 0) {
                visit(StructuralEdge {
                    target,
                    kind: StructuralEdgeKind::ArrayElement,
                });
            }
        }
        TypeKind::Slice(_) => {
            if let Some(target) = type_argument(descriptor, 0) {
                visit(StructuralEdge {
                    target,
                    kind: StructuralEdgeKind::SliceElement,
                });
            }
        }
        TypeKind::Map(_) => {
            for (index, kind) in [(0, StructuralEdgeKind::MapKey), (1, StructuralEdgeKind::MapValue)] {
                if let Some(target) = type_argument(descriptor, index) {
                    visit(StructuralEdge { target, kind });
                }
            }
        }
        TypeKind::Tuple(tuple) => {
            let argument_count = descriptor.generic_arguments().len();
            let arity = tuple.fields().len().max(argument_count);
            for index in 0..arity {
                // Tuple field descriptors may retain only source expressions.
                // Concrete reflected tuple registrations expose exact element
                // identities as positional generic arguments.
                let target = tuple
                    .fields()
                    .iter()
                    .nth(index)
                    .and_then(|field| field.find_concrete_type())
                    .or_else(|| type_argument(descriptor, index));
                if let Some(target) = target {
                    visit(StructuralEdge {
                        target,
                        kind: StructuralEdgeKind::TupleElement { index },
                    });
                }
            }
        }
        TypeKind::Struct(structure) => {
            for field in structure.fields().iter() {
                if let Some(target) = field.find_concrete_type() {
                    visit(StructuralEdge {
                        target,
                        kind: StructuralEdgeKind::StructField {
                            index: field.index(),
                            name: Some(field.name()),
                            query_name: None,
                        },
                    });
                }
            }
        }
        TypeKind::Enum(enumeration) => {
            for variant in enumeration.variants().iter() {
                for field in variant.fields().iter() {
                    if let Some(target) = field.find_concrete_type() {
                        visit(StructuralEdge {
                            target,
                            kind: StructuralEdgeKind::EnumVariantField {
                                variant_index: variant.index(),
                                variant_name: variant.name(),
                                field_index: field.index(),
                                field_name: Some(field.name()),
                                query_name: None,
                            },
                        });
                    }
                }
            }
        }
        _ => {}
    }
}
