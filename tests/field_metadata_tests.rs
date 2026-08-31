// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

// qubit-style: allow explicit-imports
//! Field overlays preserve resolved, opaque, and symbolic reflection facts.

use std::sync::LazyLock;

use qubit_model_metadata::__private::descriptor::field as reflect_field;
use qubit_model_metadata::FieldDescriptor;
use qubit_model_metadata::FieldMetadata;
use qubit_model_metadata::Reflect;
use qubit_model_metadata::TypeDescriptor;
use qubit_model_metadata::descriptor::TypeRef;
use qubit_model_metadata::expression::TypeExpression;
use qubit_model_metadata::identity::Visibility;

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct Fields {
    value: String,
    #[reflect(opaque)]
    secret: Vec<u8>,
}

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct SymbolicOwner;

static SYMBOLIC_FIELD: LazyLock<FieldDescriptor> = LazyLock::new(|| {
    let relationship = Box::leak(Box::new(TypeRef::Symbolic(TypeExpression::Parameter("T".into()))));
    reflect_field(
        <SymbolicOwner as Reflect>::type_descriptor,
        0,
        Some("value"),
        Some("value"),
        relationship,
        Visibility::Private,
    )
});

#[test]
fn test_field_overlay_delegates_structure_and_preserves_type_ref_kind() {
    let descriptor = TypeDescriptor::of::<Fields>();
    let resolved = FieldMetadata::from_reflect(&descriptor.fields()[0]);
    let opaque = FieldMetadata::from_reflect(&descriptor.fields()[1]);
    let symbolic = FieldMetadata::from_reflect(&SYMBOLIC_FIELD);

    assert_eq!(resolved.name(), Some("value"));
    assert_eq!(resolved.index(), 0);
    assert!(resolved.descriptor().is_some());
    assert!(matches!(resolved.type_ref(), TypeRef::Resolved(_)));
    assert!(opaque.descriptor().is_none());
    assert!(matches!(opaque.type_ref(), TypeRef::Opaque(_)));
    assert!(symbolic.descriptor().is_none());
    assert!(matches!(symbolic.type_ref(), TypeRef::Symbolic(_)));
}
