//! Structured diagnostics produced while binding validation metadata.

use std::fmt;

use qubit_validator::BindError;
use qubit_validator::BindErrorKind;
use qubit_validator::ValidatorId;

use crate::metadata::ModelIdBuf;
use crate::metadata::SelectorPosition;

// qubit-style: allow multiple-public-types

/// Machine-readable validation plan construction failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationBuildErrorKind {
    /// An executable validator could not be bound.
    ValidatorBinding(BindErrorKind),
    /// The erased executor does not support the declared selector position.
    UnsupportedSelectorExecution,
}

/// One validation-plan binding failure with model and declaration context.
pub struct ValidationBuildError {
    model: ModelIdBuf,
    path: Option<String>,
    selector: Option<SelectorPosition>,
    kind: ValidationBuildErrorKind,
    source: Option<BindError>,
}

impl ValidationBuildError {
    pub(crate) fn new(model: ModelIdBuf, source: BindError) -> Self {
        Self {
            model,
            path: None,
            selector: None,
            kind: ValidationBuildErrorKind::ValidatorBinding(source.kind()),
            source: Some(source),
        }
    }

    pub(crate) fn unsupported_selector(model: ModelIdBuf, path: impl Into<String>, selector: SelectorPosition) -> Self {
        Self {
            model,
            path: Some(path.into()),
            selector: Some(selector),
            kind: ValidationBuildErrorKind::UnsupportedSelectorExecution,
            source: None,
        }
    }

    /// Returns the model type name involved in the failure.
    #[must_use]
    pub const fn model(&self) -> &ModelIdBuf {
        &self.model
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

    /// Returns the underlying validator binding error.
    #[must_use]
    pub const fn source_error(&self) -> Option<&BindError> {
        self.source.as_ref()
    }
}

impl fmt::Debug for ValidationBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ValidationBuildError")
            .field("model", &self.model)
            .field("path", &self.path)
            .field("selector", &self.selector)
            .field("kind", &self.kind())
            .field("rule", &self.rule())
            .finish()
    }
}

impl fmt::Display for ValidationBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "validation plan for {} failed: {:?}", self.model, self.kind())?;
        if let Some(path) = &self.path {
            write!(f, " at {path}")?;
        }
        if let Some(selector) = self.selector {
            write!(f, " ({selector:?})")?;
        }
        Ok(())
    }
}

impl std::error::Error for ValidationBuildError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|source| source as &dyn std::error::Error)
    }
}

/// Ordered validation-plan binding failures.
#[derive(Debug)]
pub struct ValidationBuildErrors {
    errors: Vec<ValidationBuildError>,
}

impl ValidationBuildErrors {
    pub(crate) fn from_bind_errors(model: ModelIdBuf, errors: Vec<BindError>) -> Self {
        Self {
            errors: errors
                .into_iter()
                .map(|error| ValidationBuildError::new(model.clone(), error))
                .collect(),
        }
    }

    pub(crate) fn from_errors(mut errors: Vec<ValidationBuildError>) -> Self {
        errors.sort_by(|left, right| {
            left.model
                .cmp(&right.model)
                .then_with(|| left.path.cmp(&right.path))
                .then_with(|| left.selector.cmp(&right.selector))
        });
        Self { errors }
    }

    /// Returns the number of failures.
    #[must_use]
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// Returns whether no failures were recorded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Returns failures in deterministic declaration order.
    #[must_use]
    pub fn as_slice(&self) -> &[ValidationBuildError] {
        &self.errors
    }

    /// Iterates over failures in declaration order.
    pub fn iter(&self) -> impl Iterator<Item = &ValidationBuildError> {
        self.errors.iter()
    }
}

impl std::ops::Index<usize> for ValidationBuildErrors {
    type Output = ValidationBuildError;

    fn index(&self, index: usize) -> &Self::Output {
        &self.errors[index]
    }
}

impl<'a> IntoIterator for &'a ValidationBuildErrors {
    type Item = &'a ValidationBuildError;
    type IntoIter = std::slice::Iter<'a, ValidationBuildError>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.iter()
    }
}

impl fmt::Display for ValidationBuildErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} validation plan binding error(s)", self.errors.len())
    }
}

impl std::error::Error for ValidationBuildErrors {}

impl AsRef<[ValidationBuildError]> for ValidationBuildErrors {
    fn as_ref(&self) -> &[ValidationBuildError] {
        self.as_slice()
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use qubit_validator::BindError;
    use qubit_validator::BindErrorKind;

    use super::ValidationBuildErrors;
    use crate::metadata::ModelIdBuf;

    #[test]
    fn retains_bind_error_context_and_order() {
        let errors = ValidationBuildErrors::from_bind_errors(
            ModelIdBuf::parse("example.Model").expect("valid model ID"),
            vec![BindError::new(BindErrorKind::UnsupportedConstraint)],
        );
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].model().as_str(), "example.Model");
        assert_eq!(
            errors[0].kind(),
            super::ValidationBuildErrorKind::ValidatorBinding(BindErrorKind::UnsupportedConstraint),
        );
        assert!(errors[0].source().is_some());
    }
}
// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================
