// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Integration tests for the model-owned reflection codegen helpers.

use std::sync::OnceLock;

use qubit_model_metadata::__private::codegen_v2::inventory;
use qubit_model_metadata::__private::codegen_v2::registration::CapabilityRegistration;
use qubit_model_metadata::__private::codegen_v2::registration::CapabilityTarget;
use qubit_model_metadata::__private::codegen_v2::registration::FragmentKind;
use qubit_model_metadata::__private::codegen_v2::registration::FragmentPayload;
use qubit_model_metadata::__private::codegen_v2::registration::RegistrationFragment;
use qubit_model_metadata::__private::codegen_v2::registration::RuntimeIdentity;
use qubit_model_metadata::__private::codegen_v2::registration::StaticFragmentIdentity;
use qubit_model_metadata::__private::v4;
use qubit_model_metadata::__private::v4::register_generic_model_capability;
use qubit_model_metadata::FragmentIdentity;
use qubit_model_metadata::GenericModelMetadata;
use qubit_model_metadata::ModelId;
use qubit_model_metadata::ModelRegistry;
use qubit_model_metadata::ModelRole;
use qubit_model_metadata::Reflect;
use qubit_model_metadata::ReflectRegistry;
use qubit_model_metadata::TypeDefinitionDescriptor;
use qubit_model_metadata::TypeDescriptor;
use qubit_model_metadata::TypeRef;
use qubit_reflect::__private::testing::build_registry;

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata, definition_provider_v2 = first_definition)]
#[allow(dead_code, reason = "the reflection derive registers this definition-only fixture")]
struct FirstGeneric<T> {
    value: T,
}

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata, definition_provider_v2 = second_definition)]
#[allow(dead_code, reason = "the reflection derive registers this definition-only fixture")]
enum SecondGeneric<T> {
    Value(T),
}

/// Returns the model metadata attached to the first generic definition.
fn first_metadata() -> &'static GenericModelMetadata {
    static METADATA: OnceLock<GenericModelMetadata> = OnceLock::new();
    METADATA.get_or_init(|| {
        v4::generic_model_metadata(
            ModelId::new("example.FirstGeneric"),
            ModelRole::Model,
            first_definition(),
            &[],
            &[],
        )
    })
}

/// Returns the model metadata attached to the second generic definition.
fn second_metadata() -> &'static GenericModelMetadata {
    static METADATA: OnceLock<GenericModelMetadata> = OnceLock::new();
    METADATA.get_or_init(|| {
        v4::generic_model_metadata(
            ModelId::new("example.SecondGeneric"),
            ModelRole::Enum,
            second_definition(),
            &[],
            &[],
        )
    })
}

mod registrations {
    use super::register_generic_model_capability;

    register_generic_model_capability! {
        definition = super::first_definition,
        metadata = super::first_metadata,
        source = ("fixture-first", "fixture::first", 41, 7, 0x1111),
    }

    register_generic_model_capability! {
        definition = super::second_definition,
        metadata = super::second_metadata,
        source = ("fixture-second", "fixture::second", 43, 9, 0x2222),
    }
}

/// Returns the target identity for the controlled first-definition conflict.
fn first_conflict_identity() -> RuntimeIdentity {
    RuntimeIdentity::Capabilities(CapabilityTarget::TypeDefinition(first_definition().id()))
}

/// Returns the payload for the controlled first-definition conflict.
fn first_conflict_payload() -> FragmentPayload {
    FragmentPayload::Capability(CapabilityRegistration::for_definition(
        first_definition(),
        vec![v4::generic_model_capability(first_metadata)],
    ))
}

/// A second claim used to expose the first macro invocation's source facts.
static FIRST_CONFLICT: RegistrationFragment = RegistrationFragment::new(
    FragmentKind::Capability,
    StaticFragmentIdentity::new("conflict", "conflict::first", 1, 1, "conflict", 1),
    first_conflict_identity,
    first_conflict_payload,
);

/// Returns the target identity for the controlled second-definition conflict.
fn second_conflict_identity() -> RuntimeIdentity {
    RuntimeIdentity::Capabilities(CapabilityTarget::TypeDefinition(second_definition().id()))
}

/// Returns the payload for the controlled second-definition conflict.
fn second_conflict_payload() -> FragmentPayload {
    FragmentPayload::Capability(CapabilityRegistration::for_definition(
        second_definition(),
        vec![v4::generic_model_capability(second_metadata)],
    ))
}

/// A second claim used to expose the second macro invocation's source facts.
static SECOND_CONFLICT: RegistrationFragment = RegistrationFragment::new(
    FragmentKind::Capability,
    StaticFragmentIdentity::new("conflict", "conflict::second", 1, 1, "conflict", 2),
    second_conflict_identity,
    second_conflict_payload,
);

/// Asserts that the model helper preserves reflection's canonical descriptor
/// root for `T`.
fn assert_reflected_root<T: Reflect + ?Sized>() {
    let reference: &'static TypeRef = v4::reflected_type_ref::<T>();
    let resolved = reference.as_resolved().expect("the helper must return a resolved root");
    assert!(std::ptr::eq(resolved, TypeDescriptor::of::<T>()));
}

/// Finds the inventory fragment that installs `expected_metadata` on
/// `definition`.
fn find_generic_model_capability_fragment(
    definition: &'static TypeDefinitionDescriptor,
    expected_metadata: &'static GenericModelMetadata,
) -> &'static RegistrationFragment {
    inventory::iter::<RegistrationFragment>
        .into_iter()
        .find(|fragment| {
            let Ok(registry) = build_registry(&[*fragment]) else {
                return false;
            };
            registry
                .definition_capability(definition.id(), v4::generic_model_metadata_key())
                .is_some_and(|provider| std::ptr::eq(provider(), expected_metadata))
        })
        .expect("the generic model capability fragment must be discoverable")
}

/// Asserts source facts exposed by a controlled capability conflict.
fn assert_fragment_source(
    fragment: &'static RegistrationFragment,
    conflict: &'static RegistrationFragment,
    expected: &FragmentIdentity,
) {
    let error = build_registry(&[fragment, conflict]).expect_err("the duplicate capability must conflict");
    let (first, second) = error
        .conflicting_fragments()
        .expect("the conflict must retain both fragment sources");
    assert!(first == expected || second == expected);
}

#[test]
fn test_reflected_type_ref_preserves_sized_and_unsized_descriptor_roots() {
    assert_reflected_root::<u32>();
    assert_reflected_root::<str>();
    assert_reflected_root::<[u32]>();
}

#[test]
fn test_generic_model_registration_preserves_definition_providers_and_sources() {
    let reflection = ReflectRegistry::initialize().expect("generic capabilities must register");
    let models = ModelRegistry::from_reflect_registry(reflection).expect("generic models must project");

    let cases = [
        (first_definition(), first_metadata(), "example.FirstGeneric"),
        (second_definition(), second_metadata(), "example.SecondGeneric"),
    ];
    for (definition, expected_metadata, model_id) in cases {
        let provider = reflection
            .definition_capability(definition.id(), v4::generic_model_metadata_key())
            .expect("the definition must carry a generic model provider");
        assert!(std::ptr::eq(provider(), expected_metadata));
        assert!(std::ptr::eq(
            models
                .generic(model_id)
                .expect("the projected generic model must exist"),
            expected_metadata,
        ));
        assert!(std::ptr::eq(
            models
                .source(model_id)
                .expect("the projected model must retain its definition source"),
            reflection
                .definition_source(definition.id())
                .expect("the generic definition must retain its source"),
        ));
    }

    assert_ne!(
        models.source("example.FirstGeneric").expect("first definition source"),
        models
            .source("example.SecondGeneric")
            .expect("second definition source"),
    );

    let first_fragment = find_generic_model_capability_fragment(first_definition(), first_metadata());
    assert_fragment_source(
        first_fragment,
        &FIRST_CONFLICT,
        &FragmentIdentity::new(
            "fixture-first",
            "fixture::first",
            41,
            7,
            "generic-model-capability",
            0x1111,
        ),
    );
    let second_fragment = find_generic_model_capability_fragment(second_definition(), second_metadata());
    assert_fragment_source(
        second_fragment,
        &SECOND_CONFLICT,
        &FragmentIdentity::new(
            "fixture-second",
            "fixture::second",
            43,
            9,
            "generic-model-capability",
            0x2222,
        ),
    );
}
