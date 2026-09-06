// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Boundary coverage for model-owned reflection code generation.

use model_runtime::ModelRegistry;
use model_runtime::PropertyValue;
use model_runtime::ReflectRegistry;
use model_runtime::ReflectedMut;
use model_runtime::ReflectedOwned;
use model_runtime::ReflectedRef;
use model_runtime::TypeExpression;
use model_runtime::TypeMetadata;
use qubit_model_derive::Model;
use qubit_model_derive::ModelImpl;

const GENERIC_DECLARATION_START_LINE: u32 = line!();
#[Model(id = "test.derive.CodegenBoundaryGeneric")]
#[allow(dead_code, reason = "the registration must not require a concrete monomorph")]
struct CodegenBoundaryGeneric<T> {
    value: T,
}
const GENERIC_DECLARATION_END_LINE: u32 = line!();

#[Model]
struct CodegenBoundaryProperties {
    name: String,
    alias: Option<String>,
    tags: Vec<String>,
}

#[ModelImpl]
impl CodegenBoundaryProperties {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, value: String) {
        self.name = value;
    }

    pub fn alias(&self) -> Option<&String> {
        self.alias.as_ref()
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }
}

#[test]
fn test_generic_model_registration_preserves_definition_identity_and_source() {
    let registry = ModelRegistry::try_global().expect("generated registrations must be valid");
    let generic = registry
        .generic("test.derive.CodegenBoundaryGeneric")
        .expect("generic definition must register without a concrete monomorph");

    assert_eq!(generic.model_id().as_str(), "test.derive.CodegenBoundaryGeneric");
    assert_eq!(registry.generic_definitions().len(), 1);
    assert!(registry.metadata("test.derive.CodegenBoundaryGeneric").is_none());
    assert_eq!(generic.definition().generics().parameters().len(), 1);
    assert_eq!(generic.fields().len(), 1);
    assert!(matches!(
        generic.fields()[0].type_ref().as_symbolic(),
        Some(TypeExpression::Parameter(name)) if name.as_ref() == "T",
    ));

    let source = registry
        .source("test.derive.CodegenBoundaryGeneric")
        .expect("generic definition must retain its declaration source");
    assert_eq!(source.declaring_crate(), "qubit-model-derive");
    assert!(
        source
            .module_path()
            .starts_with("model_codegen_boundary_tests::__qubit_reflect_type_definition_registration_")
    );
    assert!(!source.module_path().contains("reflect_codegen"));
    assert_eq!(source.member_kind(), "type-definition");
    assert!((GENERIC_DECLARATION_START_LINE..=GENERIC_DECLARATION_END_LINE).contains(&source.line()));
    assert!(source.column() > 0);

    let reflection = ReflectRegistry::initialize().expect("reflection registrations must be valid");
    assert_eq!(reflection.definition_source(generic.definition().id()), Some(source));
}

#[test]
fn test_model_impl_preserves_borrowed_and_mutable_property_shapes() {
    let metadata = TypeMetadata::of::<CodegenBoundaryProperties>();
    let properties = metadata.try_properties().expect("properties must merge");
    let name = properties.property("name").expect("name property");
    let alias = properties.property("alias").expect("alias property");
    let tags = properties.property("tags").expect("tags property");
    let mut value = CodegenBoundaryProperties {
        name: "before".to_owned(),
        alias: Some("visible".to_owned()),
        tags: vec!["first".to_owned(), "second".to_owned()],
    };

    let PropertyValue::Borrowed(borrowed) = name.get(ReflectedRef::new(&value)).expect("borrowed str getter") else {
        panic!("name must remain a borrowed property");
    };
    assert_eq!(borrowed.as_str(), Some("before"));

    let PropertyValue::OptionalBorrowed(Some(borrowed)) =
        alias.get(ReflectedRef::new(&value)).expect("optional borrowed getter")
    else {
        panic!("alias must preserve optional borrowing");
    };
    assert_eq!(borrowed.downcast_ref::<String>().map(String::as_str), Some("visible"));

    let PropertyValue::BorrowedSlice(borrowed) = tags.get(ReflectedRef::new(&value)).expect("borrowed slice getter")
    else {
        panic!("tags must remain a borrowed slice property");
    };
    assert_eq!(borrowed.len(), 2);
    drop(borrowed);

    name.set(ReflectedMut::new(&mut value), ReflectedOwned::new("after".to_owned()))
        .expect("string setter");
    assert_eq!(value.name, "after");
}

#[test]
fn test_model_owned_expanders_use_only_the_model_codegen_facade() {
    let metadata = include_str!("../src/expand/metadata.rs");
    let model_impl = include_str!("../src/expand/model_impl.rs");
    assert!(
        !metadata.contains("codegen_v2"),
        "metadata expansion must not directly reference reflection codegen_v2",
    );
    assert!(
        !model_impl.contains("codegen_v2"),
        "ModelImpl expansion must not directly reference reflection codegen_v2",
    );

    let runtime_facade = include_str!("../../src/__private.rs");
    assert!(
        runtime_facade.contains("pub use qubit_reflect::__private::codegen_v2;"),
        "the reflection derive facade must retain its codegen_v2 entry",
    );
}
