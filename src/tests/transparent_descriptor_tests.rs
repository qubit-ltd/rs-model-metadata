// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Transparent traversal stops when a nested relation has no root descriptor.

use qubit_reflect::__private::codegen_v3::descriptor;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::descriptor::OpaqueTypeDescriptor;
use qubit_reflect::descriptor::SmartPointerKind;
use qubit_reflect::descriptor::TypeRef;
use qubit_reflect::expression::TypeExpression;

use crate::transparent_descriptor::transparent_descriptor;

static OPAQUE: OpaqueTypeDescriptor = descriptor::opaque_member::<String>();
static OPAQUE_REF: TypeRef = TypeRef::Opaque(&OPAQUE);
static SYMBOLIC_REF: TypeRef = TypeRef::Symbolic(TypeExpression::SelfType);
static BOX_OPAQUE: TypeDescriptor =
    descriptor::smart_pointer::<Box<String>>("Box<String>", SmartPointerKind::Box, &OPAQUE_REF);
static BOX_OPAQUE_REF: TypeRef = TypeRef::Resolved(&BOX_OPAQUE);
static BOX_SYMBOLIC: TypeDescriptor =
    descriptor::smart_pointer::<Box<String>>("Box<String>", SmartPointerKind::Box, &SYMBOLIC_REF);
static BOX_SYMBOLIC_REF: TypeRef = TypeRef::Resolved(&BOX_SYMBOLIC);
static OPTION_BOX_OPAQUE: TypeDescriptor =
    descriptor::optional::<Option<Box<String>>>("Option<Box<String>>", &BOX_OPAQUE_REF);
static OPTION_BOX_SYMBOLIC: TypeDescriptor =
    descriptor::optional::<Option<Box<String>>>("Option<Box<String>>", &BOX_SYMBOLIC_REF);

#[test]
fn test_transparent_descriptor_stops_at_opaque_or_symbolic_child() {
    assert!(transparent_descriptor(&OPTION_BOX_OPAQUE).is_none());
    assert!(transparent_descriptor(&OPTION_BOX_SYMBOLIC).is_none());
}
