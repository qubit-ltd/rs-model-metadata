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
use std::collections::BTreeSet;
use std::collections::HashMap;

use qubit_model_derive::Enum;
use qubit_model_derive::Model;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_model_metadata::validation::ConstraintRuleRef;
use qubit_model_metadata::validation::ValidationBuildErrorKind;
use qubit_model_metadata::validation::ValidationBuildErrors;
use qubit_model_metadata::validation::ValidationBuildInputs;
use qubit_model_metadata::validation::ValidationCapabilities;
use qubit_model_metadata::validation::ValidationPlan;
use qubit_reflect::Reflect;
use qubit_reflect::registry::RegistrySnapshotBuilder;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorRegistry;

#[Model]
#[derive(Ord, PartialOrd)]
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

#[derive(Clone, Eq, Hash, PartialEq, Reflect)]
struct ReflectedWrapper {
    #[reflect(rename = "lookup_child")]
    child: Child,
    sibling: Child,
}

#[Model(no_redact, no_display, no_debug, no_serialize, no_deserialize)]
struct ReflectedStructEnvelope {
    raw: ReflectedWrapper,
}

#[derive(Clone, Eq, Hash, PartialEq, Reflect)]
struct RawTuple(Child);

#[Model(no_redact, no_display, no_debug, no_serialize, no_deserialize)]
struct ReflectedTupleEnvelope {
    wrapper: RawTuple,
}

#[derive(Clone, Eq, Hash, PartialEq, Reflect)]
enum AliasedChoice {
    #[reflect(rename = "lookup_variant")]
    Named {
        #[reflect(rename = "lookup_child")]
        child: Child,
    },
}

#[Model(no_redact, no_display, no_debug, no_serialize, no_deserialize)]
struct AliasedEnvelope {
    raw: AliasedChoice,
}

#[derive(Clone, Eq, Hash, PartialEq, Reflect)]
struct RecursiveWrapper {
    next: Option<Box<RecursiveWrapper>>,
    child: Child,
}

#[Model(no_redact, no_display, no_debug, no_serialize, no_deserialize)]
struct RecursiveEnvelope {
    raw: RecursiveWrapper,
}

#[Model]
struct EmptyChild {
    name: String,
}

#[derive(Clone, Eq, Hash, PartialEq, Reflect)]
struct EmptyWrapper {
    child: EmptyChild,
}

#[Model(no_redact, no_display, no_debug, no_serialize, no_deserialize)]
struct EmptyEnvelope {
    raw: EmptyWrapper,
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

#[Model]
struct MapUses {
    entries: BTreeMap<Child, Child>,
}

#[Model]
struct SetAndSliceUses {
    values: BTreeSet<Child>,
    elements: Box<[Child]>,
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
    let text_rules = errors[0].constraint_rules();
    assert_eq!(
        text_rules,
        [
            ConstraintRuleRef::Registry(ValidatorId::new("qubit.rules.text.non_blank")),
            ConstraintRuleRef::Registry(ValidatorId::new("qubit.rules.text.char_length")),
            ConstraintRuleRef::Registry(ValidatorId::new("qubit.rules.text.byte_length")),
            ConstraintRuleRef::Registry(ValidatorId::new("qubit.rules.text.allowed_chars")),
            ConstraintRuleRef::Registry(ValidatorId::new("qubit.rules.text.email_ascii")),
        ]
    );
    let sequence_rules = errors[1].constraint_rules();
    assert_eq!(
        sequence_rules,
        [
            ConstraintRuleRef::Registry(ValidatorId::new("qubit.rules.collection.item_count")),
            ConstraintRuleRef::ModelIntrinsic(ValidatorId::new("qubit.rules.collection.unique")),
        ]
    );
    let map_rules = errors[2].constraint_rules();
    assert_eq!(
        map_rules,
        [ConstraintRuleRef::Registry(ValidatorId::new(
            "qubit.rules.collection.item_count"
        ))]
    );
    assert_eq!(sequence_rules[0].registry_id(), Some(sequence_rules[0].id()));
    assert_eq!(sequence_rules[1].registry_id(), None);
    assert!(errors.iter().all(|error| error.source_error().is_none()));
    assert!(errors.iter().all(|error| error.rule().is_none()), "no rule was bound");
}

/// Checks only the explicit root graph and returns its unsupported
/// declarations.
fn unsupported(root: &'static TypeMetadata) -> ValidationBuildErrors {
    let reflection = RegistrySnapshotBuilder::new()
        .build()
        .expect("fresh reflection snapshot");
    let models = ModelRegistry::from_reflect_registry(&reflection).expect("explicit reflection model registry");
    let roots = [root];
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
    let validators = ValidatorRegistry::empty();
    let plan_errors = match ValidationPlan::build(
        root,
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    ) {
        Err(errors) => errors,
        Ok(_) => panic!("unsupported declarations cannot become a successful zero-binding plan"),
    };
    assert_eq!(plan_errors.len(), errors.len());
    for (capability, plan) in errors.iter().zip(plan_errors.iter()) {
        assert_eq!(plan.kind(), capability.kind());
        assert_eq!(plan.path(), capability.path());
        assert_eq!(plan.owner_type_id(), capability.owner_type_id());
        assert_eq!(plan.constraint_rules(), capability.constraint_rules());
    }
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
fn test_unsupported_map_preserves_key_before_value_uses() {
    let errors = unsupported(TypeMetadata::of::<MapUses>());
    assert_eq!(
        errors.iter().map(|error| error.path()).collect::<Vec<_>>(),
        [Some("entries[key].name"), Some("entries[value].name"),]
    );
    assert_eq!(errors[0].occurrence(), Some(0));
    assert_eq!(errors[1].occurrence(), Some(1));
    assert!(
        errors
            .iter()
            .all(|error| error.owner_type_id() == TypeId::of::<Child>())
    );
}

#[test]
fn test_unsupported_set_and_slice_preserve_element_type_suffixes() {
    let errors = unsupported(TypeMetadata::of::<SetAndSliceUses>());
    assert_eq!(
        errors.iter().map(|error| error.path()).collect::<Vec<_>>(),
        [Some("values[].name"), Some("elements[].name"),]
    );
    assert!(
        errors
            .iter()
            .all(|error| error.owner_type_id() == TypeId::of::<Child>())
    );
}

#[test]
fn test_missing_collection_adapters_reject_each_declaration() {
    let errors = unsupported(TypeMetadata::of::<MissingCollectionAdapters>());
    assert_eq!(errors.len(), 2);
    assert_eq!(errors[0].path(), Some("entries"));
    assert_eq!(
        errors[0].constraint_rules(),
        [ConstraintRuleRef::Registry(ValidatorId::new(
            "qubit.rules.collection.item_count"
        ))]
    );
    assert_eq!(errors[1].path(), Some("values"));
    assert_eq!(
        errors[1].constraint_rules(),
        [ConstraintRuleRef::ModelIntrinsic(ValidatorId::new(
            "qubit.rules.collection.unique"
        ))]
    );
}

#[test]
fn test_unsupported_time_type_is_rejected_at_its_field() {
    let errors = unsupported(TypeMetadata::of::<UnsupportedTimeType>());
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].path(), Some("date"));
    assert_eq!(errors[0].owner_type_id(), TypeId::of::<UnsupportedTimeType>());
}

#[test]
fn test_unsupported_raw_struct_preserves_sibling_uses_and_query_alias() {
    let errors = unsupported(TypeMetadata::of::<ReflectedStructEnvelope>());
    assert_eq!(errors.len(), 2);
    assert_eq!(
        errors.iter().map(|error| error.path()).collect::<Vec<_>>(),
        [Some("raw.lookup_child.name"), Some("raw.sibling.name"),]
    );
    for (ordinal, error) in errors.iter().enumerate() {
        assert_eq!(error.owner_type_id(), TypeId::of::<Child>());
        assert_eq!(error.occurrence(), Some(ordinal));
        assert_eq!(
            error.constraint_rules(),
            [ConstraintRuleRef::Registry(ValidatorId::new(
                "qubit.rules.text.non_blank"
            ))]
        );
        assert!(error.source_error().is_none());
    }
}

#[test]
fn test_unsupported_raw_positional_struct_preserves_numeric_suffix() {
    let root = TypeMetadata::of::<ReflectedTupleEnvelope>();
    let errors = unsupported(root);
    assert_eq!(errors.len(), 1);
    let error = &errors[0];
    assert_eq!(error.path(), Some("wrapper.0.name"));
    assert_eq!(error.root_type_id(), root.type_id());
    assert_eq!(error.owner_type_id(), TypeId::of::<Child>());
    assert_eq!(
        error.field_location(),
        TypeMetadata::of::<Child>().fields()[0].location()
    );
    assert_eq!(error.occurrence(), Some(0));
    assert_eq!(
        error.constraint_rules(),
        [ConstraintRuleRef::Registry(ValidatorId::new(
            "qubit.rules.text.non_blank"
        ))]
    );
}

#[test]
fn test_unsupported_raw_enum_preserves_variant_source_name_and_field_query_alias() {
    let errors = unsupported(TypeMetadata::of::<AliasedEnvelope>());
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].path(), Some("raw.Named.lookup_child.name"));
    assert_eq!(errors[0].owner_type_id(), TypeId::of::<Child>());
}

#[test]
fn test_unsupported_recursive_raw_wrapper_terminates_without_losing_work() {
    let errors = unsupported(TypeMetadata::of::<RecursiveEnvelope>());
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].path(), Some("raw.child.name"));
    assert_eq!(errors[0].owner_type_id(), TypeId::of::<Child>());
}

#[test]
fn test_unsupported_shapes_without_execution_work_remain_acceptable() {
    let reflection = RegistrySnapshotBuilder::new()
        .build()
        .expect("fresh reflection snapshot");
    let models = ModelRegistry::from_reflect_registry(&reflection).expect("explicit reflection model registry");
    let root = TypeMetadata::of::<EmptyEnvelope>();
    let roots = [root];
    let graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect("wrapper structure is valid");
    assert!(
        graph.model(TypeId::of::<EmptyChild>()).is_some(),
        "empty Child still has a discoverable capability"
    );
    ValidationCapabilities::check(root, &graph).expect("empty declarations require no execution adapter");
    let validators = ValidatorRegistry::empty();
    let plan = ValidationPlan::build(
        root,
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    )
    .expect("wrapper without work is a valid empty plan");
    assert_eq!(plan.binding_count(), 0);
}

#[test]
fn test_unsupported_metadata_only_graph_does_not_import_reflection_capabilities() {
    let models = ModelRegistry::from_static_metadata(&[]).expect("metadata-only registry has no reflection snapshot");
    let root = TypeMetadata::of::<ReflectedStructEnvelope>();
    let roots = [root];
    let graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect("explicit root metadata supplies only its own model facts");
    assert!(
        graph.model(TypeId::of::<Child>()).is_none(),
        "Child capability cannot be imported from another snapshot"
    );
    ValidationCapabilities::check(root, &graph).expect("this explicit graph contains no child execution declarations");
    let validators = ValidatorRegistry::empty();
    let plan = ValidationPlan::build(
        root,
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    )
    .expect("no work is known to this metadata-only graph");
    assert_eq!(plan.binding_count(), 0);

    let roots = [root, TypeMetadata::of::<Child>()];
    let graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect("explicit Child metadata is an allowed metadata-only discovery source");
    let errors = ValidationCapabilities::check(root, &graph)
        .expect_err("explicitly known Child rules require unsupported wrapper access");
    assert_eq!(errors.len(), 2);
    assert!(
        errors
            .iter()
            .all(|error| error.kind() == ValidationBuildErrorKind::UnsupportedExecution)
    );
    assert_eq!(
        errors.iter().map(|error| error.path()).collect::<Vec<_>>(),
        [Some("raw.lookup_child.name"), Some("raw.sibling.name"),]
    );
    let plan_errors = match ValidationPlan::build(
        root,
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    ) {
        Err(errors) => errors,
        Ok(_) => panic!("explicit metadata-only Child rules require unsupported wrapper access"),
    };
    assert_eq!(plan_errors.len(), errors.len());
    for (capability, plan) in errors.iter().zip(plan_errors.iter()) {
        assert_eq!(plan.kind(), ValidationBuildErrorKind::UnsupportedExecution);
        assert_eq!(plan.path(), capability.path());
        assert_eq!(plan.owner_type_id(), TypeId::of::<Child>());
        assert_eq!(plan.field_location(), capability.field_location());
        assert_eq!(plan.constraint_rules(), capability.constraint_rules());
        assert_eq!(
            plan.constraint_rules(),
            [ConstraintRuleRef::Registry(ValidatorId::new(
                "qubit.rules.text.non_blank"
            ))]
        );
    }
}
