//! Structured diagnostics produced while binding validation metadata.

use std::fmt;

use qubit_validator::{BindError, BindErrorKind, ValidatorId};

use crate::SelectorPosition;

/// One validation-plan binding failure with model and declaration context.
pub struct ValidationBuildError {
    model: String,
    path: Option<String>,
    selector: Option<SelectorPosition>,
    source: BindError,
}

impl ValidationBuildError {
    pub(crate) fn new(model: impl Into<String>, source: BindError) -> Self {
        Self {
            model: model.into(),
            path: None,
            selector: None,
            source,
        }
    }

    /// Adds the metadata path that caused this failure.
    #[must_use]
    pub(crate) fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Adds the collection position selected by this failure.
    #[must_use]
    pub(crate) fn with_selector(mut self, selector: SelectorPosition) -> Self {
        self.selector = Some(selector);
        self
    }

    /// Returns the model type name involved in the failure.
    #[must_use]
    pub fn model(&self) -> &str {
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
    pub const fn kind(&self) -> BindErrorKind {
        self.source.kind()
    }

    /// Returns the rule identifier, when binding reached a concrete rule.
    #[must_use]
    pub const fn rule(&self) -> Option<ValidatorId> {
        self.source.rule_id()
    }

    /// Returns the underlying validator binding error.
    #[must_use]
    pub const fn source_error(&self) -> &BindError {
        &self.source
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
        write!(
            f,
            "validation plan for {} failed: {}",
            self.model,
            self.kind()
        )?;
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
        Some(&self.source)
    }
}

/// Ordered validation-plan binding failures.
#[derive(Debug)]
pub struct ValidationBuildErrors {
    errors: Vec<ValidationBuildError>,
}

impl ValidationBuildErrors {
    pub(crate) fn from_bind_errors(model: &str, errors: Vec<BindError>) -> Self {
        Self {
            errors: errors
                .into_iter()
                .map(|e| ValidationBuildError::new(model, e))
                .collect(),
        }
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
    use super::*;

    #[test]
    fn retains_bind_error_context_and_order() {
        let errors = ValidationBuildErrors::from_bind_errors(
            "example.Model",
            vec![BindError::new(BindErrorKind::UnsupportedConstraint)],
        );
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].model(), "example.Model");
        assert_eq!(errors[0].kind(), BindErrorKind::UnsupportedConstraint);
        assert!(errors[0].source().is_some());
    }
}
