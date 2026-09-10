// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Standard declaration adapters preserve text units, policies and rule
//! identity.

use qubit_model_derive::Enum;
use qubit_model_derive::Model;
use qubit_model_derive::ModelImpl;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_model_metadata::validation::ValidationBuildErrorKind;
use qubit_model_metadata::validation::ValidationBuildInputs;
use qubit_model_metadata::validation::ValidationOptions;
use qubit_model_metadata::validation::ValidationPlan;
use qubit_reflect::ReflectedRef;
use qubit_validation_rules::registrations;
use qubit_validator::BindErrorKind;
use qubit_validator::ValidationReport;
use qubit_validator::ValidatorRegistry;

#[Model]
struct TextUnits {
    #[text(min_chars = 1, max_chars = 2, min_bytes = 2, max_bytes = 3)]
    value: String,
}

#[Model]
struct CharacterPolicies {
    #[text(allowed_chars = unicode)]
    unicode: String,
    #[text(allowed_chars = printable_unicode)]
    printable_unicode: String,
    #[text(allowed_chars = ascii)]
    ascii: String,
    #[text(allowed_chars = printable_ascii)]
    printable_ascii: String,
    #[text(allowed_chars = code)]
    code: String,
}

#[Model]
struct Formats {
    #[text(format = email)]
    email: String,
    #[text(format = cn_mobile)]
    mobile: String,
    #[text(format = uri)]
    uri: String,
    #[text(format = uuid)]
    uuid: String,
}

#[Model]
struct SequenceCount {
    #[sequence(min_items = 1, max_items = 2)]
    values: Vec<String>,
}

#[Enum]
enum UnsupportedChoice {
    Named {
        #[text(non_blank)]
        name: String,
    },
}

#[Model]
struct IndependentFailures {
    choice: UnsupportedChoice,
    #[validator(id = "example.missing_independent_rule")]
    value: String,
}

#[ModelImpl]
impl SequenceCount {
    pub fn values(&self) -> &[String] {
        &self.values
    }
}

/// Builds through the same explicit graph/registry API used by consumers.
fn validate(root: &'static TypeMetadata, value: ReflectedRef<'_>) -> ValidationReport {
    let roots = [root];
    let graph = StructureResolver::new(ResolveInputs {
        models: ModelRegistry::global(),
        roots: &roots,
    })
    .resolve()
    .expect("standard constraint structure");
    let validators = ValidatorRegistry::empty();
    let plan = ValidationPlan::build(
        root,
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    )
    .expect("standard rule binding");
    plan.validate(value, &ValidationOptions::default())
        .expect("standard rule execution")
}

/// UTF-8 byte bounds must not accidentally become character-count bounds.
#[test]
fn test_text_character_and_byte_bounds_are_independent() {
    let root = TypeMetadata::of::<TextUnits>();
    for text in ["é", "ab", "中"] {
        assert!(
            validate(root, ReflectedRef::new(&TextUnits { value: text.to_owned() })).is_valid(),
            "{text}"
        );
    }
    for (text, expected_rules) in [
        ("a", vec!["qubit.rules.text.byte_length"]),
        ("éé", vec!["qubit.rules.text.byte_length"]),
        ("abc", vec!["qubit.rules.text.char_length"]),
        ("", vec!["qubit.rules.text.char_length", "qubit.rules.text.byte_length"]),
    ] {
        let report = validate(root, ReflectedRef::new(&TextUnits { value: text.to_owned() }));
        let rules: Vec<_> = report.violations().iter().map(|item| item.rule_id().as_str()).collect();
        assert_eq!(rules, expected_rules, "{text:?}");
        assert!(report.violations().iter().all(|item| item.path().render() == "value"));
    }
}

/// The five policies remain distinct when translated to runtime arguments.
#[test]
fn test_character_policies_preserve_unicode_and_printability() {
    let root = TypeMetadata::of::<CharacterPolicies>();
    let valid = CharacterPolicies {
        unicode: "\n中文".to_owned(),
        printable_unicode: "中文".to_owned(),
        ascii: "\nASCII".to_owned(),
        printable_ascii: "ASCII 7".to_owned(),
        code: "Account_7.example-name".to_owned(),
    };
    assert!(validate(root, ReflectedRef::new(&valid)).is_valid());
    let invalid = CharacterPolicies {
        unicode: valid.unicode,
        printable_unicode: "\n".to_owned(),
        ascii: "é".to_owned(),
        printable_ascii: "\n".to_owned(),
        code: "with space".to_owned(),
    };
    let report = validate(root, ReflectedRef::new(&invalid));
    let paths: Vec<_> = report.violations().iter().map(|item| item.path().render()).collect();
    assert_eq!(paths, ["printable_unicode", "ascii", "printable_ascii", "code"]);
    assert!(
        report
            .violations()
            .iter()
            .all(|item| item.rule_id().as_str() == "qubit.rules.text.allowed_chars")
    );
}

/// Each format maps to its own built-in rule, with visible declaration paths.
#[test]
fn test_text_formats_bind_and_report_their_own_rule_identity() {
    let root = TypeMetadata::of::<Formats>();
    let valid = Formats {
        email: "user@example.com".to_owned(),
        mobile: "13800138000".to_owned(),
        uri: "https://example.com/profile".to_owned(),
        uuid: "550e8400-e29b-41d4-a716-446655440000".to_owned(),
    };
    assert!(validate(root, ReflectedRef::new(&valid)).is_valid());
    let invalid = Formats {
        email: "invalid".to_owned(),
        mobile: "invalid".to_owned(),
        uri: "invalid".to_owned(),
        uuid: "invalid".to_owned(),
    };
    let report = validate(root, ReflectedRef::new(&invalid));
    let failures: Vec<_> = report
        .violations()
        .iter()
        .map(|item| (item.path().render(), item.rule_id().as_str()))
        .collect();
    assert_eq!(
        failures,
        [
            ("email".to_owned(), "qubit.rules.text.email_ascii"),
            ("mobile".to_owned(), "qubit.rules.text.china_mobile_structure"),
            ("uri".to_owned(), "qubit.rules.text.uri"),
            ("uuid".to_owned(), "qubit.rules.text.uuid"),
        ]
    );
}

/// Item-count validation reads the exposed slice and includes both boundaries.
#[test]
fn test_sequence_count_uses_the_actual_slice_length() {
    let root = TypeMetadata::of::<SequenceCount>();
    for count in 0..=3 {
        let value = SequenceCount {
            values: vec!["item".to_owned(); count],
        };
        let report = validate(root, ReflectedRef::new(&value));
        assert_eq!(report.is_valid(), (1..=2).contains(&count));
        if !(1..=2).contains(&count) {
            assert_eq!(report.violations().len(), 1);
            assert_eq!(report.violations()[0].path().render(), "values");
            assert_eq!(
                report.violations()[0].rule_id().as_str(),
                "qubit.rules.collection.item_count"
            );
        }
    }
}

/// A supplied registration cannot override the meaning of a standard rule ID.
#[test]
fn test_custom_registry_cannot_override_a_builtin_rule() {
    let root = TypeMetadata::of::<TextUnits>();
    let roots = [root];
    let graph = StructureResolver::new(ResolveInputs {
        models: ModelRegistry::global(),
        roots: &roots,
    })
    .resolve()
    .expect("structure");
    let builtin = registrations()[0];
    let validators = ValidatorRegistry::from_registrations([builtin]).expect("one valid registration");
    let Err(errors) = ValidationPlan::build(
        root,
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    ) else {
        panic!("duplicate built-in ID must fail plan binding");
    };
    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0].kind(),
        ValidationBuildErrorKind::ValidatorBinding(BindErrorKind::InvalidDeclaration)
    );
    assert_eq!(errors[0].root_type_id(), root.type_id());
    assert!(errors[0].source_error().is_some());
}

/// A registry collision must not hide independent structural or binding errors.
#[test]
fn test_builtin_collision_retains_independent_declaration_errors() {
    let root = TypeMetadata::of::<IndependentFailures>();
    let roots = [root, TypeMetadata::of::<UnsupportedChoice>()];
    let models = ModelRegistry::from_metadata(&[]).expect("isolated registry");
    let graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect("valid declaration structure");
    let builtin = registrations()[0];
    let validators = ValidatorRegistry::from_registrations([builtin]).expect("one registration");
    let Err(errors) = ValidationPlan::build(
        root,
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    ) else {
        panic!("independent failures must reject the plan");
    };
    assert_eq!(errors.len(), 3, "registry, unsupported payload, and missing rule");
    assert_eq!(
        errors[0].kind(),
        ValidationBuildErrorKind::ValidatorBinding(BindErrorKind::InvalidDeclaration)
    );
    assert_eq!(errors[0].rule(), Some(builtin.id()));
    assert!(errors[0].constraint_rule_ids().is_empty(), "registration-level error");
    assert_eq!(errors[1].kind(), ValidationBuildErrorKind::UnsupportedExecution);
    assert_eq!(errors[1].path(), Some("choice.Named.name"));
    assert_eq!(
        errors[2].kind(),
        ValidationBuildErrorKind::ValidatorBinding(BindErrorKind::MissingRule)
    );
    assert_eq!(errors[2].path(), Some("value"));
    assert_eq!(errors[2].declared_rule_id(), Some("example.missing_independent_rule"));
    assert!(errors[2].constraint_rule_ids().is_empty(), "custom rule declaration");
}
