// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! One failed validation declaration and its diagnostic context.

use std::any::TypeId;
use std::error::Error;
use std::fmt;

use qubit_validator::BindError;
use qubit_validator::BindErrorKind;
use qubit_validator::ValidatorId;

use super::validation_build_error_kind::ValidationBuildErrorKind;
use crate::metadata::ConstraintMetadata;
use crate::metadata::DeclarationLocation;
use crate::metadata::FieldLocation;
use crate::metadata::ModelId;
use crate::metadata::SelectorPosition;
use crate::metadata::TypeMetadata;
use crate::validation::internal::execution_declaration::ExecutionDeclaration;
use crate::validation::internal::validation_occurrence::ValidationOccurrence;
use crate::validation::standard_constraints;

/// One validation-plan binding failure with model and declaration context.
///
/// The original declaration remains available even if its validator has no
/// registration. Unsupported shapes retain source coordinates but have no
/// fabricated validator error in their source chain.
///
/// # Examples
///
/// ```
/// use qubit_model_derive::Enum;
/// use qubit_model_metadata::metadata::TypeMetadata;
/// use qubit_model_metadata::registry::ModelRegistry;
/// use qubit_model_metadata::resolve::ResolveInputs;
/// use qubit_model_metadata::resolve::StructureResolver;
/// use qubit_model_metadata::validation::ValidationBuildErrorKind;
/// use qubit_model_metadata::validation::ValidationCapabilities;
///
/// #[Enum]
/// enum Choice { Named { #[text(non_blank)] name: String } }
/// # fn main() {
/// let root = TypeMetadata::of::<Choice>();
/// let models = ModelRegistry::from_metadata(&[]).expect("isolated registry");
/// let roots = [root];
/// let graph = StructureResolver::new(ResolveInputs { models: &models, roots: &roots })
///     .resolve().expect("enum metadata is structurally valid");
/// let errors = ValidationCapabilities::check(root, &graph)
///     .expect_err("enum payload execution is not supported");
/// assert_eq!(errors[0].kind(), ValidationBuildErrorKind::UnsupportedExecution);
/// assert_eq!(errors[0].field_location().expect("payload field").variant(), Some(0));
/// assert!(errors[0].source_error().is_none());
/// # }
/// ```
#[must_use]
pub struct ValidationBuildError {
    /// Requested concrete validation root, including anonymous models.
    model: &'static TypeMetadata,
    /// Concrete model which declares the failed field.
    owner: &'static TypeMetadata,
    /// Address-independent field identity; absent for root-level failures.
    field_location: Option<FieldLocation>,
    /// Generated source coordinates; absent before visiting a declaration.
    declaration: Option<DeclarationLocation>,
    /// Deterministic declaration ordinal; absent for a root-level failure.
    occurrence: Option<usize>,
    /// Original custom rule ID, whether or not binding found its registry
    /// entry.
    declared_rule_id: Option<&'static str>,
    /// Original standard constraint; absent for custom rules or traversal
    /// edges.
    constraint: Option<&'static ConstraintMetadata>,
    /// Root-relative usage path; absent if construction failed before
    /// traversal.
    path: Option<String>,
    /// Collection position, absent for a whole-field declaration.
    selector: Option<SelectorPosition>,
    /// Stable classification independent of the rendered message.
    kind: ValidationBuildErrorKind,
    /// Original binder failure; unsupported shapes and missing roots have none.
    source: Option<BindError>,
}

impl ValidationBuildError {
    /// Wraps a shared validator binding error with model context.
    pub(crate) fn new(model: &'static TypeMetadata, source: BindError) -> Self {
        Self {
            model,
            owner: model,
            field_location: None,
            declaration: None,
            occurrence: None,
            declared_rule_id: None,
            constraint: None,
            path: None,
            selector: None,
            kind: ValidationBuildErrorKind::ValidatorBinding(source.kind()),
            source: Some(source),
        }
    }

    /// Builds a diagnostic at the exact declaration being compiled.
    pub(crate) fn at_occurrence(occurrence: &ValidationOccurrence, source: BindError) -> Self {
        let mut error = Self::new(occurrence.root, source);
        error.owner = occurrence.owner;
        error.field_location = occurrence.field.location();
        error.declaration = Some(match occurrence.selector {
            Some(selector) => occurrence.field.declaration().with_selector(selector),
            None => *occurrence.field.declaration(),
        });
        error.path = Some(
            occurrence
                .unsupported_path
                .clone()
                .unwrap_or_else(|| occurrence.segments.join(".")),
        );
        error.selector = occurrence.selector;
        error.occurrence = Some(occurrence.ordinal);
        error.declared_rule_id = occurrence.declared_rule_id();
        error.constraint = match occurrence.declaration {
            ExecutionDeclaration::Constraint(value) => Some(value),
            _ => None,
        };
        error
    }

    /// Records a declaration which cannot be executed without losing semantics.
    pub(crate) fn unsupported(occurrence: &ValidationOccurrence) -> Self {
        let mut error = Self::at_occurrence(occurrence, BindError::new(BindErrorKind::UnsupportedConstraint));
        error.kind = ValidationBuildErrorKind::UnsupportedExecution;
        error.source = None;
        error
    }

    /// Rejects a caller root not owned by the explicitly supplied graph.
    pub(crate) fn missing_root(root: &'static TypeMetadata) -> Self {
        let mut error = Self::new(root, BindError::new(BindErrorKind::UnreadablePath));
        error.kind = ValidationBuildErrorKind::RootNotInGraph;
        error.source = None;
        error
    }

    /// Returns the concrete validation root identity.
    #[must_use]
    pub fn root_type_id(&self) -> TypeId {
        self.model.type_id()
    }

    /// Returns the concrete type declaring the failed occurrence.
    #[must_use]
    pub fn owner_type_id(&self) -> TypeId {
        self.owner.type_id()
    }

    /// Returns the original concrete field identity, when this is a field rule.
    #[must_use]
    pub const fn field_location(&self) -> Option<FieldLocation> {
        self.field_location
    }

    /// Returns source coordinates, including selector and variant information.
    #[must_use]
    pub const fn declaration(&self) -> Option<DeclarationLocation> {
        self.declaration
    }

    /// Returns the source occurrence ordinal within the validation root.
    #[must_use]
    pub const fn occurrence(&self) -> Option<usize> {
        self.occurrence
    }

    /// Returns the original declared user rule ID, including for missing rules.
    #[must_use]
    pub const fn declared_rule_id(&self) -> Option<&'static str> {
        self.declared_rule_id
    }

    /// Returns the original standard constraint, when this was not a user rule.
    #[must_use]
    pub const fn constraint(&self) -> Option<&'static ConstraintMetadata> {
        self.constraint
    }

    /// Returns the validation root's stable model identifier, or `None` when
    /// the root is anonymous. This is independent of the declaration owner's
    /// identity for a nested failure.
    #[must_use]
    pub const fn model(&self) -> Option<ModelId> {
        self.model.model_id()
    }

    /// Returns the exact Rust identity even for an anonymous root.
    #[must_use]
    pub fn type_id(&self) -> TypeId {
        self.model.type_id()
    }

    /// Returns the validation root's Rust type name independently of
    /// registration. For a nested declaration, use [`Self::owner_type_id`]
    /// to distinguish its owner from this root.
    #[must_use]
    pub fn type_name(&self) -> &'static str {
        self.model.type_name()
    }

    /// Returns the declaration path, when available.
    #[must_use]
    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    /// Returns the selected collection position, when available.
    #[must_use]
    pub const fn selector(&self) -> Option<SelectorPosition> {
        self.selector
    }

    /// Returns the shared validator binding error kind.
    #[must_use]
    pub const fn kind(&self) -> ValidationBuildErrorKind {
        self.kind
    }

    /// Returns the rule identifier, when binding reached a concrete rule.
    #[must_use]
    pub const fn rule(&self) -> Option<ValidatorId> {
        match &self.source {
            Some(source) => source.rule_id(),
            None => None,
        }
    }

    /// Returns all known rule IDs mapped from the original standard constraint
    /// in binding order, even when its access shape prevents execution.
    ///
    /// Returns an empty vector for custom declarations, root-level errors, or
    /// constraints with no known backend mapping. This does not bind a rule or
    /// create a validator source; [`Self::rule`] still identifies only a rule
    /// reached by the binder. Compound constraints can map to several IDs.
    #[must_use]
    pub fn constraint_rule_ids(&self) -> Vec<ValidatorId> {
        self.constraint.map_or_else(Vec::new, standard_constraints::rule_ids)
    }

    /// Returns the underlying validator binding error, or `None` for an
    /// unsupported shape or a root missing from the graph. This is the same
    /// error exposed by [`Error::source`].
    #[must_use]
    pub const fn source_error(&self) -> Option<&BindError> {
        self.source.as_ref()
    }
}

impl fmt::Debug for ValidationBuildError {
    /// Formats identities and declaration coordinates without expanding the
    /// metadata graph or inspecting model instances.
    ///
    /// # Errors
    ///
    /// Returns the formatter's error if writing diagnostics fails.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ValidationBuildError")
            .field("root", &self.model.type_name())
            .field("owner", &self.owner.type_name())
            .field("declaration", &self.declaration)
            .field("occurrence", &self.occurrence)
            .field("declared_rule_id", &self.declared_rule_id)
            .field("path", &self.path)
            .field("selector", &self.selector)
            .field("kind", &self.kind())
            .field("rule", &self.rule())
            .field("constraint_rule_ids", &self.constraint_rule_ids())
            .finish()
    }
}

impl fmt::Display for ValidationBuildError {
    /// Formats the root, occurrence path, selector, original rule ID and cause
    /// when each is available.
    ///
    /// # Errors
    ///
    /// Returns the formatter's error if writing diagnostics fails.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "validation plan for {} failed: {:?}", self.type_name(), self.kind())?;
        if let Some(path) = &self.path {
            write!(f, " at {path}")?;
        }
        if let Some(selector) = self.selector {
            write!(f, " ({selector:?})")?;
        }
        if let Some(id) = self.declared_rule_id {
            write!(f, " [rule {id}]")?;
        }
        if let Some(source) = &self.source {
            write!(f, ": {source}")?;
        }
        Ok(())
    }
}

impl Error for ValidationBuildError {
    /// Borrows the binder's typed error; structural rejections have no
    /// fabricated binder cause.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(|source| source as &dyn Error)
    }
}
