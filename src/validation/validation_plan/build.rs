// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Construction APIs for ValidationPlan.

use super::ValidationPlan;
use crate::metadata::TypeMetadata;
use crate::validation::ValidationBuildError;
use crate::validation::ValidationBuildErrors;
use crate::validation::ValidationBuildInputs;
use crate::validation::internal::declaration_walker;
use crate::validation::internal::occurrence_binder;
use crate::validation::standard_constraints;

impl<'a> ValidationPlan<'a> {
    /// Binds every executable declaration reachable through the root.
    ///
    /// The argument selects a concrete type. Declarations and the resulting
    /// plan root come from the supplied graph's canonical metadata for that
    /// TypeId, even when the caller holds another overlay of the same type.
    ///
    /// # Errors
    /// Returns declaration-scoped diagnostics for unsupported access shapes or
    /// failed rule and dependency binding. No getter is invoked.
    pub fn build(
        root: &'static TypeMetadata,
        inputs: ValidationBuildInputs<'a>,
    ) -> Result<Self, ValidationBuildErrors> {
        Self::build_with_context(root, inputs, &[])
    }

    /// Binds with nearest-parent-first type context for external dependencies.
    ///
    /// # Errors
    /// Returns all independent declaration and binding errors in source order.
    pub fn build_with_context(
        root: &'static TypeMetadata,
        inputs: ValidationBuildInputs<'a>,
        ancestors: &[&'static TypeMetadata],
    ) -> Result<Self, ValidationBuildErrors> {
        let Some(root) = inputs.graph.model(root.type_id()) else {
            return Err(ValidationBuildErrors::from_errors(vec![
                ValidationBuildError::missing_root(root),
            ]));
        };
        let (occurrences, mut errors) = declaration_walker::collect(root, inputs.graph);
        let (validators, registry_errors) = match standard_constraints::registry(inputs.validators) {
            Ok(result) => result,
            Err(error) => {
                errors.push(ValidationBuildError::new(root, error));
                return Err(ValidationBuildErrors::from_errors(errors));
            }
        };
        errors.extend(
            registry_errors
                .into_iter()
                .map(|error| ValidationBuildError::new(root, error)),
        );
        let mut bindings = Vec::new();
        for occurrence in occurrences {
            match occurrence_binder::bind(&occurrence, inputs.graph, &validators, ancestors) {
                Ok(mut compiled) => bindings.append(&mut compiled),
                Err(mut failures) => errors.append(&mut failures),
            }
        }
        if !errors.is_empty() {
            return Err(ValidationBuildErrors::from_errors(errors));
        }
        // A constraint can expand into several runtime rules, each with its own
        // execution occurrence even though its source declaration is shared.
        for (index, binding) in bindings.iter_mut().enumerate() {
            binding.occurrence = index;
        }
        Ok(Self {
            root,
            graph: inputs.graph,
            bindings: bindings.into_boxed_slice(),
            model_rules: Box::new([]),
        })
    }
}
