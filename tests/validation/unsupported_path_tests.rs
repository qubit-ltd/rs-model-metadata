// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Unsupported paths retain each structural use, including tuple positions.

use std::any::TypeId;
use std::collections::BTreeMap;
use std::collections::HashMap;

use qubit_model_derive::Enum;
use qubit_model_derive::Model;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_model_metadata::validation::ValidationBuildErrorKind;
use qubit_model_metadata::validation::ValidationBuildErrors;
use qubit_model_metadata::validation::ValidationCapabilities;
use qubit_reflect::Reflect;

#[Model]
struct Child {
    #[text(non_blank)]
    name: String,
}

#[Model]
struct TupleUses {
    pair: (Child, (Child, Child)),
}

#[Enum]
enum Choice {
    First {
        #[text(non_blank)]
        name: String,
    },
    Second {
        #[text(non_blank)]
        name: String,
    },
    Tuple(#[text(non_blank)] String),
}

#[Model]
struct Envelope {
    choice: Choice,
}

#[derive(Clone, Eq, Hash, PartialEq, Reflect)]
enum ReflectedChoice {
    First(Child),
    Second { child: Child },
}

#[Model(no_redact, no_display, no_debug, no_serialize, no_deserialize)]
struct ReflectedEnvelope {
    raw: ReflectedChoice,
}

#[Model]
struct Containers {
    list: Vec<Child>,
    array: [Child; 2],
}

type MapAlias = HashMap<String, String>;

#[Model(no_hash)]
struct MissingCollectionAdapters {
    #[map(min_entries = 1)]
    entries: MapAlias,
    #[sequence(unique_items)]
    values: Vec<String>,
}

#[Model(no_redact, no_display, no_debug, no_serialize, no_deserialize)]
struct UnsupportedTimeType {
    #[time(precision = second)]
    date: chrono::NaiveDate,
}

#[Enum]
enum MappedConstraints {
    Text {
        #[text(non_blank, min_chars = 2, max_bytes = 32, allowed_chars = ascii, format = email_ascii)]
        value: String,
    },
    Sequence {
        #[sequence(min_items = 1, unique_items)]
        values: Vec<String>,
    },
    Map {
        #[map(min_entries = 1)]
        values: BTreeMap<String, String>,
    },
}

/// Unsupported shapes retain every mapped rule without fabricating a binding.
#[test]
fn test_unsupported_constraints_retain_complete_rule_mappings() {
    let errors = unsupported(TypeMetadata::of::<MappedConstraints>());
    assert_eq!(errors.len(), 3);
    let text_rules = errors[0].constraint_rule_ids();
    assert_eq!(
        text_rules.iter().map(|id| id.as_str()).collect::<Vec<_>>(),
        [
            "qubit.rules.text.non_blank",
            "qubit.rules.text.char_length",
            "qubit.rules.text.byte_length",
            "qubit.rules.text.allowed_chars",
            "qubit.rules.text.email_ascii",
        ]
    );
    let sequence_rules = errors[1].constraint_rule_ids();
    assert_eq!(
        sequence_rules.iter().map(|id| id.as_str()).collect::<Vec<_>>(),
        ["qubit.rules.collection.item_count", "qubit.rules.collection.unique",]
    );
    let map_rules = errors[2].constraint_rule_ids();
    assert_eq!(
        map_rules.iter().map(|id| id.as_str()).collect::<Vec<_>>(),
        ["qubit.rules.collection.item_count"]
    );
    assert!(errors.iter().all(|error| error.source_error().is_none()));
    assert!(errors.iter().all(|error| error.rule().is_none()), "no rule was bound");
}

/// Checks only the explicit root graph and returns its unsupported
/// declarations.
fn unsupported(root: &'static TypeMetadata) -> ValidationBuildErrors {
    let models = ModelRegistry::from_static_metadata(&[]).expect("isolated registry");
    let roots = [root, TypeMetadata::of::<Child>(), TypeMetadata::of::<Choice>()];
    let graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect("all fixtures are structurally valid");
    let errors = ValidationCapabilities::check(root, &graph)
        .expect_err("each fixture contains unsupported execution declarations");
    assert!(
        errors
            .iter()
            .all(|error| error.kind() == ValidationBuildErrorKind::UnsupportedExecution)
    );
    errors
}

#[test]
fn test_repeated_tuple_models_keep_each_position_in_source_order() {
    let errors = unsupported(TypeMetadata::of::<TupleUses>());
    assert_eq!(errors.len(), 3, "type reachability must not deduplicate static uses");
    assert_eq!(
        errors.iter().map(|error| error.path()).collect::<Vec<_>>(),
        [Some("pair.0.name"), Some("pair.1.0.name"), Some("pair.1.1.name")]
    );
    for (ordinal, error) in errors.iter().enumerate() {
        assert_eq!(error.owner_type_id(), TypeId::of::<Child>());
        assert_eq!(error.occurrence(), Some(ordinal));
        assert_eq!(error.field_location(), errors[0].field_location());
    }
}

#[test]
fn test_enum_paths_distinguish_variants_and_unnamed_fields() {
    let errors = unsupported(TypeMetadata::of::<Envelope>());
    assert_eq!(
        errors.iter().map(|error| error.path()).collect::<Vec<_>>(),
        [
            Some("choice.First.name"),
            Some("choice.Second.name"),
            Some("choice.Tuple.0")
        ]
    );
    for (variant, error) in errors.iter().enumerate() {
        let location = error.field_location().expect("concrete enum payload field");
        assert_eq!(location.variant(), Some(variant));
        assert_eq!(location.index(), 0);
        assert_eq!(error.owner_type_id(), TypeId::of::<Choice>());
    }
}

#[test]
fn test_reflection_only_enum_wrappers_preserve_nested_model_uses() {
    let errors = unsupported(TypeMetadata::of::<ReflectedEnvelope>());
    assert_eq!(
        errors.iter().map(|error| error.path()).collect::<Vec<_>>(),
        [Some("raw.First.0.name"), Some("raw.Second.child.name")]
    );
    assert!(
        errors
            .iter()
            .all(|error| error.owner_type_id() == TypeId::of::<Child>())
    );
}

#[test]
fn test_container_element_paths_do_not_claim_an_instance_index() {
    let errors = unsupported(TypeMetadata::of::<Containers>());
    assert_eq!(
        errors.iter().map(|error| error.path()).collect::<Vec<_>>(),
        [Some("list[].name"), Some("array[].name")]
    );
}

#[test]
fn test_missing_collection_adapters_reject_each_declaration() {
    let errors = unsupported(TypeMetadata::of::<MissingCollectionAdapters>());
    assert_eq!(errors.len(), 2);
    assert_eq!(errors[0].path(), Some("entries"));
    assert_eq!(
        errors[0]
            .constraint_rule_ids()
            .iter()
            .map(|id| id.as_str())
            .collect::<Vec<_>>(),
        ["qubit.rules.collection.item_count"]
    );
    assert_eq!(errors[1].path(), Some("values"));
    assert_eq!(
        errors[1]
            .constraint_rule_ids()
            .iter()
            .map(|id| id.as_str())
            .collect::<Vec<_>>(),
        ["qubit.rules.collection.unique"]
    );
}

#[test]
fn test_unsupported_time_type_is_rejected_at_its_field() {
    let errors = unsupported(TypeMetadata::of::<UnsupportedTimeType>());
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].path(), Some("date"));
    assert_eq!(errors[0].owner_type_id(), TypeId::of::<UnsupportedTimeType>());
}
