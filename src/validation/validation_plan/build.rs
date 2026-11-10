// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Construction APIs for ValidationPlan.

use super::ValidationPlan;
use super::internal::selection_index::SelectionIndex;
use crate::metadata::TypeMetadata;
use crate::validation::ValidationBuildError;
use crate::validation::ValidationBuildErrors;
use crate::validation::ValidationBuildInputs;
use crate::validation::internal::build_budget::BuildBudget;
use crate::validation::internal::build_budget::Counter;
use crate::validation::internal::declaration_walker;
use crate::validation::internal::occurrence_binder;
use crate::validation::standard_constraints;

impl<'registry> ValidationPlan<'registry> {
    /// Binds every executable declaration reachable through the root.
    ///
    /// The argument selects a concrete type. Declarations and the resulting
    /// plan root come from the supplied graph's canonical metadata for that
    /// TypeId, even when the caller holds another overlay of the same type.
    ///
    /// # Parameters
    ///
    /// * `root` - Static metadata identifying the concrete type to plan for.
    /// * `inputs` - Shared graph and borrowed validator registry used for
    ///   binding.
    ///
    /// # Returns
    ///
    /// A plan rooted at the graph's canonical metadata, or the collected
    /// declaration and binding errors.
    ///
    /// # Errors
    /// Returns declaration-scoped diagnostics for unsupported access shapes or
    /// failed rule and dependency binding. No getter is invoked.
    pub fn build(
        root: &'static TypeMetadata,
        inputs: ValidationBuildInputs<'registry, '_>,
    ) -> Result<Self, ValidationBuildErrors> {
        Self::build_with_context(root, inputs, &[])
    }

    /// Binds with nearest-parent-first type context for external dependencies.
    ///
    /// # Parameters
    ///
    /// * `root` - Static metadata identifying the concrete type to plan for.
    /// * `inputs` - Shared graph and borrowed validator registry used for
    ///   binding.
    /// * `ancestors` - Parent types ordered nearest first, used to resolve
    ///   external validator dependencies.
    ///
    /// # Returns
    ///
    /// A plan rooted at the graph's canonical metadata, or the collected
    /// declaration and binding errors.
    ///
    /// # Errors
    /// Returns all independent declaration and binding errors in source order.
    pub fn build_with_context(
        root: &'static TypeMetadata,
        inputs: ValidationBuildInputs<'registry, '_>,
        ancestors: &[&'static TypeMetadata],
    ) -> Result<Self, ValidationBuildErrors> {
        let Some(root) = inputs.graph.model(root.type_id()) else {
            return Err(ValidationBuildErrors::from_errors(vec![
                ValidationBuildError::missing_root(root),
            ]));
        };
        let (occurrences, mut errors, mut budget) =
            declaration_walker::collect_with_budget(root, inputs.graph.as_ref(), BuildBudget::new());
        if errors
            .iter()
            .any(|error| error.kind() == super::super::ValidationBuildErrorKind::BuildLimitExceeded)
        {
            return Err(ValidationBuildErrors::from_errors(errors));
        }
        let (validators, registry_errors) = match standard_constraints::registry(inputs.validators) {
            Ok(result) => result,
            Err(error) => {
                if budget.charge(Counter::Diagnostic).is_err() {
                    errors.push(ValidationBuildError::build_limit(root));
                    return Err(ValidationBuildErrors::from_errors(errors));
                }
                errors.push(ValidationBuildError::new(root, error));
                return Err(ValidationBuildErrors::from_errors(errors));
            }
        };
        let registry_errors_exceed_limit =
            errors.len() + registry_errors.len() > crate::validation::internal::build_budget::MAX_DIAGNOSTICS;
        let remaining = crate::validation::internal::build_budget::MAX_DIAGNOSTICS.saturating_sub(errors.len());
        let mut registry_errors = registry_errors.into_iter();
        for error in registry_errors.by_ref().take(remaining) {
            let _ = budget.charge(Counter::Diagnostic);
            errors.push(ValidationBuildError::new(root, error));
        }
        if registry_errors_exceed_limit {
            errors.push(ValidationBuildError::build_limit(root));
            return Err(ValidationBuildErrors::from_errors(errors));
        }
        let mut bindings = Vec::new();
        for occurrence in occurrences {
            if budget.charge(Counter::Work).is_err() {
                errors.push(ValidationBuildError::build_limit(root));
                return Err(ValidationBuildErrors::from_errors(errors));
            }
            match occurrence_binder::bind(
                &occurrence,
                inputs.graph.as_ref(),
                &validators,
                ancestors,
                crate::validation::internal::build_budget::MAX_BINDINGS - bindings.len(),
            ) {
                Ok(mut compiled) => {
                    for _ in &compiled {
                        if budget.charge(Counter::Binding).is_err() {
                            errors.push(ValidationBuildError::build_limit(root));
                            return Err(ValidationBuildErrors::from_errors(errors));
                        }
                    }
                    bindings.append(&mut compiled)
                }
                Err(mut failures) => {
                    if failures.iter().any(|failure| {
                        failure.kind() == crate::validation::ValidationBuildErrorKind::BuildLimitExceeded
                    }) {
                        errors.push(ValidationBuildError::build_limit(root));
                        return Err(ValidationBuildErrors::from_errors(errors));
                    }
                    let remaining =
                        crate::validation::internal::build_budget::MAX_DIAGNOSTICS.saturating_sub(errors.len());
                    let failures_exceed_limit = failures.len() > remaining;
                    for failure in failures.drain(..).take(remaining) {
                        if budget.charge(Counter::Diagnostic).is_err() {
                            errors.push(ValidationBuildError::build_limit(root));
                            return Err(ValidationBuildErrors::from_errors(errors));
                        }
                        errors.push(failure);
                    }
                    if failures_exceed_limit {
                        errors.push(ValidationBuildError::build_limit(root));
                        return Err(ValidationBuildErrors::from_errors(errors));
                    }
                }
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
        let selection_index = SelectionIndex::new(&bindings);
        Ok(Self {
            root,
            graph: inputs.graph,
            bindings: bindings.into_boxed_slice(),
            selection_index,
            model_rules: Box::new([]),
        })
    }
}
