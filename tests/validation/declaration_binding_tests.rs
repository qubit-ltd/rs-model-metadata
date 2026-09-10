// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Every executable declaration must be bound or explicitly rejected.

use std::any::TypeId;
use std::error::Error;
use std::ptr::eq;
use std::sync::Arc;

use qubit_model_derive::Enum;
use qubit_model_derive::Model;
use qubit_model_derive::ModelImpl;
use qubit_model_metadata::metadata::ConstraintMetadata;
use qubit_model_metadata::metadata::SelectorPosition;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_model_metadata::validation::ValidationBuildError;
use qubit_model_metadata::validation::ValidationBuildErrorKind;
use qubit_model_metadata::validation::ValidationBuildInputs;
use qubit_model_metadata::validation::ValidationCapabilities;
use qubit_model_metadata::validation::ValidationOptions;
use qubit_model_metadata::validation::ValidationPlan;
use qubit_reflect::ReflectedRef;
use qubit_validator::BindError;
use qubit_validator::BindErrorKind;
use qubit_validator::BoundValidationContext;
use qubit_validator::ExecutionError;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::PreparedValidator;
use qubit_validator::RegistrationSource;
use qubit_validator::RuleOutcome;
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorDescriptor;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorRegistration;
use qubit_validator::ValidatorRegistry;
use qubit_validator::ValidatorSignature;
use qubit_validator::Violation;
use qubit_validator::ViolationCode;

#[Model(id = "binding.Child")]
struct Child {
    #[element(validator(id = "binding.missing"))]
    values: Vec<String>,
}

#[ModelImpl]
impl Child {
    pub fn values(&self) -> &[String] {
        &self.values
    }
}

#[Model(id = "binding.Parent")]
struct Parent {
    child: Child,
}

#[Enum(id = "binding.Choice")]
enum Choice {
    Named {
        #[text(non_blank)]
        name: String,
    },
}

/// Checks the same missing rule from an arbitrary concrete root.
fn assert_missing_rule(root: &'static TypeMetadata) {
    let models = ModelRegistry::try_global().expect("valid registrations");
    let graph = StructureResolver::new(ResolveInputs { models, roots: &[] })
        .resolve()
        .expect("valid structure");
    let validators = ValidatorRegistry::empty();
    let Err(errors) = ValidationPlan::build(
        root,
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    ) else {
        panic!(
            "unregistered element validator must fail binding for {}",
            root.type_name()
        );
    };
    assert!(
        errors
            .iter()
            .any(|error| error.kind() == ValidationBuildErrorKind::ValidatorBinding(BindErrorKind::MissingRule))
    );
    assert_eq!(errors[0].declared_rule_id(), Some("binding.missing"));
    assert_eq!(errors[0].root_type_id(), root.type_id());
    assert_eq!(errors[0].owner_type_id(), TypeId::of::<Child>());
    assert_eq!(errors[0].field_location().unwrap().owner(), TypeId::of::<Child>());
    assert_eq!(errors[0].selector(), Some(SelectorPosition::Element));
    assert_eq!(
        errors[0].path(),
        Some(if root.type_id() == TypeId::of::<Child>() {
            "values"
        } else {
            "child.values"
        })
    );
    assert!(errors[0].declaration().unwrap().line.is_some());
    assert_eq!(errors[0].model(), root.model_id());
    assert_eq!(errors[0].type_id(), root.type_id());
    assert_eq!(errors[0].type_name(), root.type_name());
    let cause = errors[0]
        .source()
        .and_then(|cause| cause.downcast_ref::<BindError>())
        .expect("typed binder cause");
    assert_eq!(cause.kind(), BindErrorKind::MissingRule);
    assert!(eq(cause, errors[0].source_error().expect("same binder cause")));
    let display = errors[0].to_string();
    assert!(display.contains(root.type_name()));
    assert!(display.contains(errors[0].path().expect("declaration path")));
    assert!(display.contains("binding.missing"));
    assert!(display.contains("Element"));
    let debug = format!("{:?}", errors[0]);
    assert!(debug.contains(root.type_name()));
    assert!(debug.contains("binding.missing"));
    assert!(
        !debug.contains("TypeMetadata {"),
        "diagnostics must not expand metadata graphs"
    );
}

#[test]
fn test_root_selector_requires_registered_rule() {
    assert_missing_rule(TypeMetadata::of::<Child>());
}

#[test]
fn test_nested_selector_requires_registered_rule() {
    assert_missing_rule(TypeMetadata::of::<Parent>());
}

#[test]
fn test_enum_payload_constraint_is_not_silently_omitted() {
    let models = ModelRegistry::try_global().expect("valid registrations");
    let graph = StructureResolver::new(ResolveInputs { models, roots: &[] })
        .resolve()
        .expect("valid structure");
    let validators = ValidatorRegistry::empty();
    let result = ValidationPlan::build(
        TypeMetadata::of::<Choice>(),
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    );
    assert!(
        result.is_err(),
        "constrained enum payload must not produce an empty valid plan"
    );
}

/// Rejects every selected string to make nested execution observable.
struct Reject;
impl PreparedValidator for Reject {
    fn validate(
        &self,
        value: ValidationValue<'_>,
        _: &BoundValidationContext<'_>,
    ) -> Result<RuleOutcome, ExecutionError> {
        assert!(value.as_text().is_some());
        Ok(RuleOutcome::Invalid(vec![Violation::new(
            ValidatorId::new("binding.missing"),
            ViolationCode::new("bad"),
        )]))
    }
}
fn prepare_reject(_: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(Arc::new(Reject))
}
static SIGNATURES: &[ValidatorSignature] = &[ValidatorSignature::new(InputType::Text, &[], prepare_reject)];
static DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(SIGNATURES);
static REGISTRATION: ValidatorRegistration = ValidatorRegistration::new(
    ValidatorId::new("binding.missing"),
    &DESCRIPTOR,
    RegistrationSource::new("binding-tests", "nested", file!(), line!()),
);

#[Model(id = "binding.OptionalParent")]
struct OptionalParent {
    child: Option<Child>,
}
#[ModelImpl]
impl OptionalParent {
    pub fn child(&self) -> Option<&Child> {
        self.child.as_ref()
    }
}
#[Model(id = "binding.Pair")]
struct Pair {
    left: Child,
    right: Child,
}

#[test]
fn test_nested_selectors_execute_at_each_usage_path_and_skip_missing_optional() {
    let models = ModelRegistry::try_global().unwrap();
    let graph = StructureResolver::new(ResolveInputs { models, roots: &[] })
        .resolve()
        .unwrap();
    let validators = ValidatorRegistry::from_registrations([REGISTRATION]).unwrap();
    let plan = ValidationPlan::build(
        TypeMetadata::of::<Pair>(),
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    )
    .unwrap();
    let pair = Pair {
        left: Child {
            values: vec!["a".into()],
        },
        right: Child {
            values: vec!["b".into()],
        },
    };
    let report = plan
        .validate(ReflectedRef::new(&pair), &ValidationOptions::default())
        .unwrap();
    assert_eq!(
        report
            .violations()
            .iter()
            .map(|value| value.path().render())
            .collect::<Vec<_>>(),
        ["left.values[0]", "right.values[0]"]
    );
    let plan = ValidationPlan::build(
        TypeMetadata::of::<OptionalParent>(),
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    )
    .unwrap();
    let missing = OptionalParent { child: None };
    let report = plan
        .validate(ReflectedRef::new(&missing), &ValidationOptions::default())
        .unwrap();
    assert!(report.violations().is_empty());
    let present = OptionalParent {
        child: Some(Child {
            values: vec!["a".into(), "b".into()],
        }),
    };
    let report = plan
        .validate(ReflectedRef::new(&present), &ValidationOptions::default())
        .unwrap();
    assert_eq!(
        report
            .violations()
            .iter()
            .map(|value| value.path().render())
            .collect::<Vec<_>>(),
        ["child.values[0]", "child.values[1]"]
    );
}

#[test]
fn test_capability_check_does_not_require_a_registered_validator() {
    let models = ModelRegistry::try_global().unwrap();
    let graph = StructureResolver::new(ResolveInputs { models, roots: &[] })
        .resolve()
        .unwrap();
    ValidationCapabilities::check(TypeMetadata::of::<Parent>(), &graph).unwrap();
    let errors = ValidationCapabilities::check(TypeMetadata::of::<Choice>(), &graph).unwrap_err();
    assert_eq!(errors[0].kind(), ValidationBuildErrorKind::UnsupportedExecution);
    assert_eq!(errors[0].field_location().unwrap().variant(), Some(0));
    assert_eq!(errors[0].field_location().unwrap().index(), 0);
}

#[test]
fn test_explicit_root_not_in_graph_is_rejected() {
    let models = ModelRegistry::from_metadata(&[]).unwrap();
    let graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &[],
    })
    .resolve()
    .unwrap();
    let errors = ValidationCapabilities::check(TypeMetadata::of::<Child>(), &graph).unwrap_err();
    assert_eq!(errors[0].kind(), ValidationBuildErrorKind::RootNotInGraph);
    assert!(errors[0].source().is_none());
    assert!(errors[0].path().is_none());
    assert!(errors[0].to_string().contains("RootNotInGraph"));
}

#[Model]
struct QuietCycle {
    next: Option<Box<QuietCycle>>,
}
#[Model]
struct LoudCycle {
    next: Option<Box<LoudCycle>>,
    #[text(non_blank)]
    name: String,
}
#[Enum]
enum QuietChoice {
    Empty,
    Payload(String),
}
#[Model]
struct Container {
    items: Vec<Child>,
}

#[Model]
struct TupleContainer {
    pair: (Child, String),
}

#[Model]
struct OptionalElements {
    #[element(validator(id = "binding.missing"))]
    values: Vec<Option<String>>,
}

#[ModelImpl]
impl OptionalElements {
    pub fn values(&self) -> &[Option<String>] {
        &self.values
    }
}

#[Model(no_redact, no_display, no_debug, no_serialize, no_deserialize)]
struct PointerElements {
    #[element(validator(id = "binding.missing"))]
    values: Vec<Arc<String>>,
}

#[ModelImpl]
impl PointerElements {
    pub fn values(&self) -> &[Arc<String>] {
        &self.values
    }
}

#[test]
fn test_tuple_contained_declarations_are_discovered_and_rejected() {
    let root = TypeMetadata::of::<TupleContainer>();
    let roots = [root];
    let graph = StructureResolver::new(ResolveInputs {
        models: ModelRegistry::try_global().expect("registered declarations"),
        roots: &roots,
    })
    .resolve()
    .expect("tuple structure is representable");
    let errors = ValidationCapabilities::check(root, &graph)
        .expect_err("tuple-contained execution must not be silently omitted");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].kind(), ValidationBuildErrorKind::UnsupportedExecution);
    assert_eq!(errors[0].owner_type_id(), TypeId::of::<Child>());
    assert_eq!(errors[0].declared_rule_id(), Some("binding.missing"));
}

#[test]
fn test_selector_value_targets_require_an_actual_element_unwrap_adapter() {
    for root in [
        TypeMetadata::of::<OptionalElements>(),
        TypeMetadata::of::<PointerElements>(),
    ] {
        let roots = [root];
        let graph = StructureResolver::new(ResolveInputs {
            models: ModelRegistry::try_global().expect("registered declarations"),
            roots: &roots,
        })
        .resolve()
        .expect("wrapper element structure is representable");
        let errors =
            ValidationCapabilities::check(root, &graph).expect_err("slice elements cannot be implicitly unwrapped");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind(), ValidationBuildErrorKind::UnsupportedExecution);
        assert_eq!(errors[0].selector(), Some(SelectorPosition::Element));
        assert_eq!(errors[0].path(), Some("values"));
    }
}
#[Model]
struct WithoutSlice {
    #[element(validator(id = "binding.missing"))]
    values: Vec<String>,
}
#[Model]
struct OwnedParent {
    child: Child,
}
#[ModelImpl]
impl OwnedParent {
    pub fn child(&self) -> Child {
        panic!("capability checks must not invoke getters")
    }
}
#[Model]
struct BorrowedOption {
    child: Option<Child>,
}
#[ModelImpl]
impl BorrowedOption {
    pub fn child(&self) -> &Option<Child> {
        panic!("capability checks must not invoke getters")
    }
}
#[Model]
struct Repeated {
    #[validator(id = "binding.missing")]
    #[validator(id = "binding.missing")]
    name: String,
}

#[test]
fn test_cycles_and_payloads_without_executable_work_are_allowed() {
    let models = ModelRegistry::try_global().unwrap();
    for root in [TypeMetadata::of::<QuietCycle>(), TypeMetadata::of::<QuietChoice>()] {
        let roots = [root];
        let graph = StructureResolver::new(ResolveInputs { models, roots: &roots })
            .resolve()
            .unwrap();
        ValidationCapabilities::check(root, &graph).unwrap();
    }
}

#[test]
fn test_unsupported_recursive_container_and_adapter_paths_fail_before_execution() {
    let models = ModelRegistry::try_global().unwrap();
    for root in [
        TypeMetadata::of::<LoudCycle>(),
        TypeMetadata::of::<Container>(),
        TypeMetadata::of::<WithoutSlice>(),
        TypeMetadata::of::<OwnedParent>(),
        TypeMetadata::of::<BorrowedOption>(),
    ] {
        let roots = [root];
        let graph = StructureResolver::new(ResolveInputs { models, roots: &roots })
            .resolve()
            .unwrap();
        let errors = ValidationCapabilities::check(root, &graph).expect_err("unsupported execution shape");
        assert!(
            errors
                .iter()
                .any(|error| error.kind() == ValidationBuildErrorKind::UnsupportedExecution),
            "{}: {errors:?}",
            root.type_name()
        );
        assert!(errors.iter().all(|error| error.field_location().is_some()));
    }
}

#[test]
fn test_duplicate_rule_ids_remain_distinct_occurrences() {
    let models = ModelRegistry::try_global().unwrap();
    let root = TypeMetadata::of::<Repeated>();
    let roots = [root];
    let graph = StructureResolver::new(ResolveInputs { models, roots: &roots })
        .resolve()
        .unwrap();
    let validators = ValidatorRegistry::empty();
    let Err(errors) = ValidationPlan::build(
        root,
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    ) else {
        panic!("both declarations require the missing registration")
    };
    assert_eq!(errors.len(), 2);
    assert_eq!(errors[0].declared_rule_id(), errors[1].declared_rule_id());
    assert_ne!(errors[0].occurrence(), errors[1].occurrence());
    let validators = ValidatorRegistry::from_registrations([REGISTRATION]).unwrap();
    let plan = ValidationPlan::build(
        root,
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    )
    .unwrap();
    assert_eq!(plan.binding_count(), 2);
    let report = plan
        .validate(
            ReflectedRef::new(&Repeated { name: "a".into() }),
            &ValidationOptions::default(),
        )
        .unwrap();
    assert_eq!(report.violations().len(), 2);
}

#[Model(no_redact, no_display, no_debug, no_serialize, no_deserialize)]
struct Mixed {
    #[validator(id = "binding.missing")]
    z_missing: String,
    #[time(precision = second)]
    a_time: Option<chrono::DateTime<chrono::Utc>>,
}
#[Model]
struct Counted {
    #[sequence(min_items = 1)]
    values: Vec<String>,
}
#[ModelImpl]
impl Counted {
    pub fn values(&self) -> &[String] {
        &self.values
    }
}
#[Model]
struct UnreadableCount {
    #[sequence(min_items = 1)]
    values: Vec<String>,
}
#[Model]
struct SelectorConstraint {
    #[element(text(non_blank))]
    values: Vec<String>,
}
#[ModelImpl]
impl SelectorConstraint {
    pub fn values(&self) -> &[String] {
        &self.values
    }
}

#[test]
fn test_build_aggregates_binding_and_unsupported_errors_in_source_order() {
    let models = ModelRegistry::try_global().unwrap();
    let root = TypeMetadata::of::<Mixed>();
    let roots = [root];
    let graph = StructureResolver::new(ResolveInputs { models, roots: &roots })
        .resolve()
        .unwrap();
    let validators = ValidatorRegistry::empty();
    let Err(errors) = ValidationPlan::build(
        root,
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    ) else {
        panic!("both declarations must fail")
    };
    assert_eq!(errors.len(), 2);
    assert_eq!(errors[0].path(), Some("z_missing"));
    assert_eq!(
        errors[0].kind(),
        ValidationBuildErrorKind::ValidatorBinding(BindErrorKind::MissingRule)
    );
    assert_eq!(errors[1].path(), Some("a_time"));
    assert_eq!(errors[1].kind(), ValidationBuildErrorKind::UnsupportedExecution);
    assert!(matches!(errors[1].constraint(), Some(ConstraintMetadata::Time(_))));
    assert!(
        !errors.is_empty(),
        "a rejected plan exposes its independent diagnostics"
    );
    assert_eq!(errors.to_string(), "2 validation plan binding error(s)");

    // A consumer can pass the same ordered diagnostics to a slice-based
    // renderer or inspect them through borrowed collection iteration.
    let diagnostics: &[ValidationBuildError] = errors.as_ref();
    assert_eq!(diagnostics[0].declared_rule_id(), Some("binding.missing"));
    let paths: Vec<_> = (&errors).into_iter().map(ValidationBuildError::path).collect();
    assert_eq!(paths, [Some("z_missing"), Some("a_time")]);
}

#[test]
fn test_count_constraints_require_slice_adapter_and_selector_constraints_are_explicitly_rejected() {
    let models = ModelRegistry::try_global().unwrap();
    for root in [
        TypeMetadata::of::<UnreadableCount>(),
        TypeMetadata::of::<SelectorConstraint>(),
    ] {
        let roots = [root];
        let graph = StructureResolver::new(ResolveInputs { models, roots: &roots })
            .resolve()
            .unwrap();
        let errors = ValidationCapabilities::check(root, &graph).unwrap_err();
        assert_eq!(errors[0].kind(), ValidationBuildErrorKind::UnsupportedExecution);
        if root.type_id() == TypeId::of::<SelectorConstraint>() {
            assert_eq!(errors[0].selector(), Some(SelectorPosition::Element));
        }
    }
    let root = TypeMetadata::of::<Counted>();
    let roots = [root];
    let graph = StructureResolver::new(ResolveInputs { models, roots: &roots })
        .resolve()
        .unwrap();
    ValidationCapabilities::check(root, &graph).unwrap();
    let validators = ValidatorRegistry::empty();
    let plan = ValidationPlan::build(
        root,
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    )
    .unwrap();
    let report = plan
        .validate(
            ReflectedRef::new(&Counted { values: vec![] }),
            &ValidationOptions::default(),
        )
        .unwrap();
    assert_eq!(report.violations().len(), 1);
    assert_eq!(report.violations()[0].path().render(), "values");
}
