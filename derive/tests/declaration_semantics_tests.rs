// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Domain declarations retain navigation and ordered validator occurrences.

use model_runtime::metadata::NavigationStep;
use model_runtime::metadata::TypeMetadata;
use model_runtime::metadata::ValidationArgument;
use qubit_id::Id;
use qubit_model_derive::Enum;
use qubit_model_derive::Model;

#[Model]
struct Member {
    birthday: String,
    #[validator(id = "example.check", depends_on(birthday, owner(path = "..", property = birthday)))]
    #[validator(id = "example.check", params(strict = true))]
    name: String,
}

/// Dependencies have independent object navigation and property selection.
#[test]
fn test_ordered_validator_occurrences_and_parent_dependency() {
    let metadata = TypeMetadata::of::<Member>();
    let field = metadata
        .fields()
        .iter()
        .find(|field| field.name() == Some("name"))
        .expect("name field");
    let validators = field.validators();
    assert_eq!(validators.len(), 2);
    let bindings = validators[0].dependency_bindings();
    assert!(bindings[0].object_path().steps().is_empty());
    assert_eq!(bindings[1].object_path().steps(), &[NavigationStep::Parent]);
    assert_eq!(bindings[1].property().segments(), &["birthday"]);
}

#[Enum]
enum Link {
    User(#[reference(entity = "example.User", property = "id", path = "../owner")] Id),
}

/// Enum payload references retain slash-separated parent-object navigation.
#[test]
fn test_enum_reference_parent_path() {
    let metadata = TypeMetadata::of::<Link>();
    let reference = metadata.as_enum().expect("enum").variants()[0].fields()[0]
        .reference()
        .expect("reference");
    assert_eq!(
        reference.path().expect("binding path").steps(),
        &[NavigationStep::Parent, NavigationStep::Property("owner")]
    );
}

type Title = String;

#[Model]
struct IntegerParameters {
    #[validator(
        id = "example.integer_bounds",
        params(
            minimum = -170141183460469231731687303715884105728,
            bounds = [-170141183460469231731687303715884105728, 170141183460469231731687303715884105727],
            unsigned = 340282366920938463463374607431768211455,
            hexadecimal = -0x80000000000000000000000000000000i128,
            zero = -0
        )
    )]
    value: String,
}

/// Signed parameter bounds survive macro parsing and runtime metadata emission.
#[test]
fn test_validator_integer_parameters_preserve_full_range() {
    let metadata = TypeMetadata::of::<IntegerParameters>();
    let params = metadata.fields()[0].validators()[0].params();
    assert_eq!(params[0].name(), "minimum");
    assert_eq!(params[0].value(), ValidationArgument::Integer(i128::MIN));
    assert_eq!(
        params[1].value(),
        ValidationArgument::IntegerList(&[i128::MIN, i128::MAX])
    );
    assert_eq!(params[2].value(), ValidationArgument::Unsigned(u128::MAX));
    assert_eq!(params[3].value(), ValidationArgument::Integer(i128::MIN));
    assert_eq!(params[4].value(), ValidationArgument::Integer(0));
}

#[Model]
struct UniqueFields {
    #[unique]
    number: u32,
    #[unique]
    title: Title,
}

/// Uniqueness defaults follow the reflected text capability, including aliases.
#[test]
fn test_unique_defaults_follow_field_capabilities() {
    let metadata = TypeMetadata::of::<UniqueFields>();
    assert!(!metadata.fields()[0].unique().expect("numeric unique").ignore_case());
    assert!(metadata.fields()[1].unique().expect("text unique").ignore_case());
}

#[Model]
struct UniqueGeneric<T> {
    #[unique]
    value: T,
}

/// Generic definitions preserve source options until concrete capabilities
/// exist.
#[test]
fn test_generic_unique_retains_deferred_default() {
    let metadata = TypeMetadata::of::<UniqueGeneric<Title>>();
    assert_eq!(
        metadata.fields()[0].unique().unwrap().effective_ignore_case(),
        Some(true)
    );
    assert_eq!(
        metadata.generic_definition().unwrap().fields()[0]
            .unique()
            .unwrap()
            .effective_ignore_case(),
        None
    );
}
