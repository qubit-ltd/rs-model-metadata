// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Execution failures with declaration context and a partial report.

use std::any::TypeId;
use std::error::Error;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

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
/// returns the [`ExecutionError`]; the standard source chain ends there.
/// When the error retains an owned underlying cause, trusted diagnostic code
/// can inspect it through [`ExecutionError::trusted_source`] without exposing
/// it to ordinary error-chain formatting.
///
/// # Examples
///
/// ```
/// use std::error::Error;
/// use std::sync::Arc;
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
/// let models = ModelRegistry::from_static_metadata(&[]).expect("isolated registry");
/// let graph = Arc::new(StructureResolver::new(ResolveInputs { models: &models, roots: &roots })
///     .resolve().expect("valid structure"));
/// let validators = ValidatorRegistry::empty();
/// let plan = ValidationPlan::build(root, ValidationBuildInputs { graph: Arc::clone(&graph), validators: &validators })
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
    /// Original execution failure, with an explicitly accessible trusted
    /// source; standard Error::source stops at this error.
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
    ///
    /// # Parameters
    ///
    /// * `error` - The infrastructure failure that stopped validation.
    /// * `partial_report` - Results recorded before execution stopped.
    ///
    /// # Returns
    ///
    /// Returns an error with no model or field context attached.
    #[must_use = "the validation error must be retained and handled"]
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
    ///
    /// # Parameters
    ///
    /// * `root` - The model whose validation was being performed.
    /// * `occurrence` - The selected model-rule occurrence, if one exists.
    ///
    /// # Returns
    ///
    /// Returns this error with root identity and optional occurrence attached.
    #[must_use = "the enriched model validation error must be retained"]
    pub(crate) fn at_model(mut self, root: &'static TypeMetadata, occurrence: Option<usize>) -> Self {
        self.root_type_id = Some(root.type_id());
        self.occurrence = occurrence;
        self
    }

    /// Associates a field failure with its source and optional dependency
    /// paths.
    ///
    /// # Parameters
    ///
    /// * `context` - The declaration occurrence that failed.
    /// * `occurrence` - The concrete runtime occurrence number.
    /// * `dependency` - The declared dependency involved in the failure, if
    ///   any.
    ///
    /// # Returns
    ///
    /// Returns this error with root, field, occurrence, and dependency context.
    #[must_use = "the enriched field validation error must be retained"]
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
    ///
    /// # Returns
    ///
    /// Returns the execution error that stopped validation.
    #[must_use = "the execution failure should be inspected"]
    #[inline]
    pub const fn error(&self) -> &ExecutionError {
        &self.error
    }

    /// Returns the report collected before the failure.
    ///
    /// # Returns
    ///
    /// Returns the partial report accumulated before execution stopped.
    #[must_use = "the partial validation report should be inspected"]
    #[inline]
    pub const fn partial_report(&self) -> &ValidationReport {
        &self.partial_report
    }

    /// Returns the associated validation root identity, including anonymous
    /// models; `None` means no root context was attached.
    ///
    /// # Returns
    ///
    /// Returns the root type identity, or `None` if no root was associated.
    #[must_use = "the optional root type identity should be checked"]
    #[inline]
    pub const fn root_type_id(&self) -> Option<TypeId> {
        self.root_type_id
    }

    /// Returns the concrete owner of the failed field or model rule.
    ///
    /// # Returns
    ///
    /// Returns the field owner when field context exists, otherwise the root
    /// model identity; returns `None` when neither is available.
    #[must_use = "the optional owner type identity should be checked"]
    #[inline]
    pub fn owner_type_id(&self) -> Option<TypeId> {
        self.context
            .as_ref()
            .map(|context| context.owner.type_id())
            .or(self.root_type_id)
    }

    /// Returns the unique execution occurrence, including model rules.
    /// Returns `None` when execution failed before selecting an occurrence,
    /// such as a root input type mismatch.
    ///
    /// # Returns
    ///
    /// Returns the selected execution occurrence, or `None` when failure
    /// occurred before an occurrence was selected.
    #[must_use = "the optional execution occurrence should be checked"]
    #[inline]
    pub const fn occurrence(&self) -> Option<usize> {
        self.occurrence
    }

    /// Returns the original field identity, absent for model-level rules.
    ///
    /// # Returns
    ///
    /// Returns the field coordinates when this failure belongs to a field rule.
    #[must_use = "the optional field location should be checked"]
    #[inline]
    pub fn field_location(&self) -> Option<FieldLocation> {
        self.context.as_ref().and_then(|context| context.field.location())
    }

    /// Returns source coordinates for the field and selected collection
    /// position.
    ///
    /// # Returns
    ///
    /// Returns declaration coordinates enriched with selector position when
    /// present, or `None` for a model-level failure.
    #[must_use = "the optional declaration location should be checked"]
    #[inline]
    pub fn declaration(&self) -> Option<DeclarationLocation> {
        self.context.as_ref().map(|context| match context.selector {
            Some(selector) => context.field.declaration().with_selector(selector),
            None => *context.field.declaration(),
        })
    }

    /// Returns the declared custom rule ID, if the failure came from one.
    ///
    /// # Returns
    ///
    /// Returns the custom rule identifier, or `None` for non-custom rules.
    #[must_use = "the optional declared rule identifier should be checked"]
    #[inline]
    pub fn declared_rule_id(&self) -> Option<&'static str> {
        self.context.as_ref().and_then(|context| context.declared_rule_id())
    }

    /// Returns dependency object navigation separately from property selection.
    ///
    /// # Returns
    ///
    /// Returns the dependency's object navigation, or `None` when the failure
    /// is unrelated to a dependency.
    #[must_use = "the optional dependency object path should be checked"]
    #[inline]
    pub fn dependency_object_path(&self) -> Option<ObjectPath> {
        self.dependency.as_ref().map(|binding| binding.object_path())
    }

    /// Returns the property selected after dependency object navigation.
    ///
    /// # Returns
    ///
    /// Returns the dependency property path, or `None` when no dependency
    /// context was attached.
    #[must_use = "the optional dependency property path should be checked"]
    #[inline]
    pub fn dependency_property_path(&self) -> Option<PropertyPath<'static>> {
        self.dependency.as_ref().map(|binding| binding.property())
    }

    /// Consumes the error and returns the original failure and partial report.
    ///
    /// # Returns
    ///
    /// Returns the execution error and accumulated report, moving both out of
    /// this wrapper.
    #[must_use = "the execution error and partial report should be handled"]
    pub fn into_parts(self) -> (ExecutionError, ValidationReport) {
        (*self.error, *self.partial_report)
    }
}

impl Debug for ModelValidationError {
    /// Formats the error's structured context for debugging.
    ///
    /// # Parameters
    ///
    /// * `formatter` - The destination formatter.
    ///
    /// # Returns
    ///
    /// Returns the formatter result after writing the debug representation.
    ///
    /// # Errors
    ///
    /// Returns the formatter's error if the destination rejects output.
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
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

impl Display for ModelValidationError {
    /// Formats the underlying execution error for ordinary display.
    ///
    /// # Parameters
    ///
    /// * `formatter` - The destination formatter.
    ///
    /// # Returns
    ///
    /// Returns the result from formatting the underlying execution error.
    ///
    /// # Errors
    ///
    /// Returns the formatter's error if the destination rejects output.
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(self.error.as_ref(), formatter)
    }
}

impl Error for ModelValidationError {
    /// Exposes the execution failure as the standard error source.
    ///
    /// # Returns
    ///
    /// Returns the wrapped execution error; ordinary source traversal stops at
    /// that error, while trusted diagnostics may inspect its retained cause.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.error.as_ref())
    }
}
