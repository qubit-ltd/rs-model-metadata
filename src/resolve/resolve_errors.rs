// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Collected diagnostics from one failed resolution pass.

use core::fmt::Display;
use core::fmt::Formatter;
use core::fmt::Result;
use std::error::Error;

use super::resolve_error::ResolveError;

/// All errors from one complete failed resolution pass.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::TypeMetadata;
/// use qubit_model_metadata::registry::ModelRegistry;
/// use qubit_model_metadata::resolve::{ResolveError, ResolveErrorKind, ResolveErrors, ResolveInputs, StructureResolver};
/// use qubit_reflect::ReflectionContext;
///
/// # mod example {
/// use qubit_model_derive::{Model, Value};
///
/// #[Value]
/// pub struct InvalidValue { nested: PlainModel }
/// #[Model(id = "example.ResolveErrorsPlainModel", eq, hash)]
/// pub struct PlainModel;
/// # }
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let reflection = ReflectionContext::builder().include_linked().build()?;
/// let models = ModelRegistry::from_context(&reflection)?;
/// let roots = [TypeMetadata::of::<example::InvalidValue>()];
/// let errors: ResolveErrors = StructureResolver::new(ResolveInputs { models: &models, roots: &roots })
///     .resolve()
///     .unwrap_err();
/// let first: &ResolveError = &errors.errors()[0];
/// assert_eq!(first.kind(), ResolveErrorKind::InvalidValueClosure);
///
/// # Ok(())
/// # }
/// ```
#[must_use]
#[derive(Debug)]
pub struct ResolveErrors {
    /// All deterministic failures collected during one resolution pass.
    pub(super) errors: Vec<ResolveError>,
}

impl ResolveErrors {
    /// Returns every collected resolution failure.
    ///
    /// # Returns
    ///
    /// A borrowed slice in deterministic diagnostic order; it is empty only
    /// when no failures were collected.
    #[must_use = "inspect the resolution failures"]
    #[inline]
    pub fn errors(&self) -> &[ResolveError] {
        &self.errors
    }

    /// Consumes this collection and returns its failures.
    ///
    /// # Returns
    ///
    /// The owned diagnostics in deterministic order.
    #[inline]
    #[must_use]
    pub fn into_vec(self) -> Vec<ResolveError> {
        self.errors
    }
}

impl Display for ResolveErrors {
    /// Formats the number of collected resolution errors.
    ///
    /// # Parameters
    ///
    /// - `formatter`: Destination for the diagnostic count.
    ///
    /// # Returns
    ///
    /// `Ok(())` when formatting succeeds, or the formatter's error.
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        write!(formatter, "{} model resolution error(s)", self.errors.len())
    }
}

impl Error for ResolveErrors {}

#[cfg(test)]
mod tests {
    use super::ResolveErrors;
    use crate::resolve::ResolveError;
    use crate::resolve::ResolveErrorKind;

    #[test]
    fn test_collected_errors_expose_order_and_owned_values() {
        let errors = ResolveErrors {
            errors: vec![ResolveError::new(
                ResolveErrorKind::InvalidValueClosure,
                None,
                None,
                None,
                None,
                None,
            )],
        };

        assert_eq!(errors.errors().len(), 1);
        assert_eq!(errors.errors()[0].kind(), ResolveErrorKind::InvalidValueClosure);
        assert_eq!(errors.to_string(), "1 model resolution error(s)");
        let owned = errors.into_vec();
        assert_eq!(owned.len(), 1);
        assert_eq!(owned[0].kind(), ResolveErrorKind::InvalidValueClosure);
    }
}
