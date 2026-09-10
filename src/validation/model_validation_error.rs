// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Execution failures with declaration context and a partial report.

use std::any::TypeId;

use qubit_validator::ExecutionError;
use qubit_validator::ValidationReport;

use crate::metadata::DeclarationLocation;
use crate::metadata::DependencyBindingMetadata;
use crate::metadata::FieldLocation;
use crate::metadata::ObjectPath;
use crate::metadata::PropertyPath;
use crate::metadata::TypeMetadata;
use crate::validation::internal::validation_occurrence::ValidationOccurrence;

/// An infrastructure failure with its original cause and collected report.
///
/// Field failures retain both the validation root and the concrete declaration
/// owner. Dependency object navigation remains separate from property
/// selection.
///
/// Inspect [`Self::partial_report`] before discarding the error: earlier rules
/// may already have reported violations or skips. An infrastructure failure
/// does not mean the instance passed validation. [`std::error::Error::source`]
/// returns the original [`ExecutionError`], whose source may retain an adapter
/// or validator failure.
///
/// # Examples
///
/// ```
/// use std::error::Error;
/// use qubit_model_derive::Model;
/// use qubit_model_metadata::metadata::TypeMetadata;
/// use qubit_model_metadata::registry::ModelRegistry;
/// use qubit_model_metadata::resolve::ResolveInputs;
/// use qubit_model_metadata::resolve::StructureResolver;
/// use qubit_model_metadata::validation::ValidationBuildInputs;
/// use qubit_model_metadata::validation::ValidationOptions;
/// use qubit_model_metadata::validation::ValidationPlan;
/// use qubit_reflect::ReflectedRef;
/// use qubit_validator::ExecutionError;
/// use qubit_validator::ExecutionErrorKind;
/// use qubit_validator::ValidatorRegistry;
///
/// #[Model]
/// struct Profile;
/// # fn main() {
/// let root = TypeMetadata::of::<Profile>();
/// let roots = [root];
/// let models = ModelRegistry::from_metadata(&[]).expect("isolated registry");
/// let graph = StructureResolver::new(ResolveInputs { models: &models, roots: &roots })
///     .resolve().expect("valid structure");
/// let validators = ValidatorRegistry::empty();
/// let plan = ValidationPlan::build(root, ValidationBuildInputs { graph: &graph, validators: &validators })
///     .expect("empty model plan");
/// let error = plan.validate(ReflectedRef::new(&42_u32), &ValidationOptions::default())
///     .expect_err("the instance has the wrong root type");
/// assert_eq!(error.error().kind(), ExecutionErrorKind::InputTypeMismatch);
/// assert!(error.partial_report().violations().is_empty());
/// assert!(error.occurrence().is_none());
/// assert!(error.source().expect("execution cause").is::<ExecutionError>());
/// # }
/// ```
#[must_use]
pub struct ModelValidationError {
    /// Original execution failure, including its typed source chain.
    error: Box<ExecutionError>,
    /// Report collected before execution stopped.
    partial_report: Box<ValidationReport>,
    /// Root model identity, including for model-level failures.
    root_type_id: Option<TypeId>,
    /// Unique runtime occurrence, absent for pre-execution root checks.
    occurrence: Option<usize>,
    /// Original field declaration, when a field rule failed.
    context: Option<Box<ValidationOccurrence>>,
    /// Original dependency paths, when a dependency read failed.
    dependency: Option<Box<DependencyBindingMetadata>>,
}

impl ModelValidationError {
    /// Retains an execution failure and all results collected before it.
    pub(crate) fn new(error: ExecutionError, partial_report: ValidationReport) -> Self {
        Self {
            error: Box::new(error),
            partial_report: Box::new(partial_report),
            root_type_id: None,
            occurrence: None,
            context: None,
            dependency: None,
        }
    }

    /// Associates a model-level or pre-execution failure with its root.
    pub(crate) fn at_model(mut self, root: &'static TypeMetadata, occurrence: Option<usize>) -> Self {
        self.root_type_id = Some(root.type_id());
        self.occurrence = occurrence;
        self
    }

    /// Associates a field failure with its source and optional dependency
    /// paths.
    pub(crate) fn at_field(
        mut self,
        context: &ValidationOccurrence,
        occurrence: usize,
        dependency: Option<DependencyBindingMetadata>,
    ) -> Self {
        self.root_type_id = Some(context.root.type_id());
        self.occurrence = Some(occurrence);
        self.context = Some(Box::new(context.clone()));
        self.dependency = dependency.map(Box::new);
        self
    }

    /// Returns the original execution failure.
    #[must_use]
    pub const fn error(&self) -> &ExecutionError {
        &self.error
    }

    /// Returns the report collected before the failure.
    #[must_use]
    pub const fn partial_report(&self) -> &ValidationReport {
        &self.partial_report
    }

    /// Returns the associated validation root identity, including anonymous
    /// models; `None` means no root context was attached.
    #[must_use]
    pub const fn root_type_id(&self) -> Option<TypeId> {
        self.root_type_id
    }

    /// Returns the concrete owner of the failed field or model rule.
    #[must_use]
    pub fn owner_type_id(&self) -> Option<TypeId> {
        self.context
            .as_ref()
            .map(|context| context.owner.type_id())
            .or(self.root_type_id)
    }

    /// Returns the unique execution occurrence, including model rules.
    /// Returns `None` when execution failed before selecting an occurrence,
    /// such as a root input type mismatch.
    #[must_use]
    pub const fn occurrence(&self) -> Option<usize> {
        self.occurrence
    }

    /// Returns the original field identity, absent for model-level rules.
    #[must_use]
    pub fn field_location(&self) -> Option<FieldLocation> {
        self.context.as_ref().and_then(|context| context.field.location())
    }

    /// Returns source coordinates for the field and selected collection
    /// position.
    #[must_use]
    pub fn declaration(&self) -> Option<DeclarationLocation> {
        self.context.as_ref().map(|context| match context.selector {
            Some(selector) => context.field.declaration().with_selector(selector),
            None => *context.field.declaration(),
        })
    }

    /// Returns the declared custom rule ID, if the failure came from one.
    #[must_use]
    pub fn declared_rule_id(&self) -> Option<&'static str> {
        self.context.as_ref().and_then(|context| context.declared_rule_id())
    }

    /// Returns dependency object navigation separately from property selection.
    #[must_use]
    pub fn dependency_object_path(&self) -> Option<ObjectPath> {
        self.dependency.as_ref().map(|binding| binding.object_path())
    }

    /// Returns the property selected after dependency object navigation.
    #[must_use]
    pub fn dependency_property_path(&self) -> Option<PropertyPath<'static>> {
        self.dependency.as_ref().map(|binding| binding.property())
    }

    /// Consumes the error and returns the original failure and partial report.
    #[must_use]
    pub fn into_parts(self) -> (ExecutionError, ValidationReport) {
        (*self.error, *self.partial_report)
    }
}

impl std::fmt::Debug for ModelValidationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ModelValidationError")
            .field("error", &self.error)
            .field("root_type_id", &self.root_type_id)
            .field("occurrence", &self.occurrence)
            .field("context", &self.context)
            .field("partial_report", &self.partial_report)
            .finish()
    }
}

impl std::fmt::Display for ModelValidationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(formatter)
    }
}

impl std::error::Error for ModelValidationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.error.as_ref())
    }
}
