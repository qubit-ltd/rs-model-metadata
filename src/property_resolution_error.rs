// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Failures when resolving effective model properties.

use std::sync::Arc;

use qubit_reflect::capability::CapabilityError;
use qubit_reflect::error::ContextError;
use thiserror::Error;

use crate::metadata::PropertyBuildErrors;

/// Distinguishes unavailable reflection from invalid property declarations.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::PropertyResolutionError;
/// fn is_assembly_failure(error: &PropertyResolutionError) -> bool {
///     matches!(error, PropertyResolutionError::Assembly(_))
/// }
/// ```
#[must_use]
#[derive(Clone, Debug, Error)]
pub enum PropertyResolutionError {
    /// Intrinsic capabilities cannot form a valid set for the property owner.
    #[error("property capability resolution failed: {0}")]
    Capability(Box<CapabilityError>),
    /// The process-wide reflection snapshot could not be initialized.
    #[error("reflection registry error: {0}")]
    Reflection(#[from] ContextError),
    /// Linked field and method declarations cannot form valid properties.
    #[error("property assembly error: {0}")]
    Assembly(#[from] Arc<PropertyBuildErrors>),
}

impl From<CapabilityError> for PropertyResolutionError {
    fn from(error: CapabilityError) -> Self {
        Self::Capability(Box::new(error))
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::sync::Arc;

    use qubit_reflect::capability::CapabilityError;
    use qubit_reflect::error::ContextError;

    use super::PropertyResolutionError;
    use crate::metadata::PropertyBuildErrors;

    #[test]
    fn reflection_conversion_keeps_the_context_error() {
        let cause = ContextError::new("registry is unavailable");
        let error = PropertyResolutionError::from(cause.clone());

        let PropertyResolutionError::Reflection(retained) = &error else {
            panic!("context error should become the reflection variant");
        };
        assert_eq!(retained, &cause);
        assert_eq!(error.to_string(), "reflection registry error: registry is unavailable");
        assert_eq!(error.source().expect("retained source").to_string(), cause.to_string());
    }

    #[test]
    fn capability_conversion_keeps_the_capability_failure() {
        let cause = CapabilityError::Context(ContextError::new("context lookup failed"));
        let error = PropertyResolutionError::from(cause.clone());

        let PropertyResolutionError::Capability(retained) = &error else {
            panic!("capability error should become the capability variant");
        };
        assert_eq!(retained.as_ref(), &cause);
        assert_eq!(
            error.to_string(),
            "property capability resolution failed: reflection context failed: context lookup failed"
        );
    }

    #[test]
    fn assembly_variant_keeps_shared_build_errors_when_cloned() {
        let cause = Arc::new(PropertyBuildErrors::new(Vec::new()));
        let error = PropertyResolutionError::Assembly(Arc::clone(&cause));
        let clone = error.clone();

        let PropertyResolutionError::Assembly(retained) = clone else {
            panic!("assembly error should keep its variant when cloned");
        };
        assert!(Arc::ptr_eq(&retained, &cause));
        assert_eq!(error.to_string(), "property assembly error: ");
    }
}
