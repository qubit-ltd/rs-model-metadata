// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::dependency_binding_metadata::DependencyBindingMetadata;
use super::named_validation_argument::NamedValidationArgument;
use super::on_none::OnNone;
use super::target_mode::TargetMode;
use crate::relation::PropertyPath;

/// One declared validator occurrence.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::{OnNone, TargetMode, ValidatorMetadata};
///
/// let validator = ValidatorMetadata::new("example.required", &[], &[]);
/// assert_eq!(validator.target(), TargetMode::Value);
/// assert_eq!(validator.on_none(), OnNone::Skip);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct ValidatorMetadata {
    /// The stable identifier used to resolve the validator registration.
    declared_id: &'static str,
    /// Static arguments passed to the resolved validator.
    params: &'static [NamedValidationArgument<'static>],
    /// Property paths that must be available to the validator.
    depends_on: &'static [PropertyPath<'static>],
    /// Named dependency slots for the typed validator contract.
    dependency_bindings: &'static [DependencyBindingMetadata],
    /// Value shape selected for the occurrence.
    target: TargetMode,
    /// Behavior for an absent expanded value.
    on_none: OnNone,
}

impl ValidatorMetadata {
    /// Creates a validator occurrence without resolving a runtime registry.
    /// Uses value expansion and skips absent optional values.
    ///
    /// # Parameters
    ///
    /// * `declared_id` - Non-empty stable validator registration ID.
    /// * `params` - Static typed arguments in declaration order.
    /// * `depends_on` - Static property dependencies in declaration order.
    ///
    /// # Returns
    ///
    /// A value-target validator occurrence that skips absent optional values.
    ///
    /// # Panics
    /// Panics if `declared_id` is empty.
    #[inline]
    #[must_use]
    pub const fn new(
        declared_id: &'static str,
        params: &'static [NamedValidationArgument<'static>],
        depends_on: &'static [PropertyPath<'static>],
    ) -> Self {
        assert!(!declared_id.is_empty(), "validator ID cannot be empty");
        Self {
            declared_id,
            params,
            depends_on,
            dependency_bindings: &[],
            target: TargetMode::Value,
            on_none: OnNone::Skip,
        }
    }

    /// Creates a validator occurrence with named dependency bindings.
    ///
    /// # Parameters
    ///
    /// * `declared_id` - Non-empty stable validator registration ID.
    /// * `params` - Static typed arguments in declaration order.
    /// * `depends_on` - Static property dependencies in declaration order.
    /// * `dependency_bindings` - Named slots mapped to property paths.
    /// * `target` - Whether validation receives expanded values or containers.
    /// * `on_none` - Policy for absent expanded optional values.
    ///
    /// # Returns
    ///
    /// A validator occurrence with its explicit dependency and optional-value
    /// policies.
    ///
    /// # Panics
    /// Panics if the ID is empty, dependency slot names repeat, or container
    /// input is combined with rejection of absent expanded values. Individual
    /// bindings already require nonempty slot names and property paths.
    #[must_use]
    pub const fn new_bound(
        declared_id: &'static str,
        params: &'static [NamedValidationArgument<'static>],
        depends_on: &'static [PropertyPath<'static>],
        dependency_bindings: &'static [DependencyBindingMetadata],
        target: TargetMode,
        on_none: OnNone,
    ) -> Self {
        assert!(!declared_id.is_empty(), "validator ID cannot be empty");
        assert!(
            !(matches!(target, TargetMode::Container) && matches!(on_none, OnNone::Reject)),
            "container validators cannot reject missing expanded values",
        );
        assert_unique_dependency_names(dependency_bindings);
        Self {
            declared_id,
            params,
            depends_on,
            dependency_bindings,
            target,
            on_none,
        }
    }

    /// Returns the nonempty source ID. Full identifier grammar validation
    /// belongs to the declaration producer or consuming registry.
    ///
    /// # Returns
    ///
    /// The static validator registration ID.
    #[must_use]
    #[inline]
    pub const fn declared_id(&self) -> &'static str {
        self.declared_id
    }

    /// Returns parameters in source order.
    ///
    /// # Returns
    ///
    /// Static named arguments in the order declared by the model.
    #[must_use]
    #[inline]
    pub const fn params(&self) -> &'static [NamedValidationArgument<'static>] {
        self.params
    }

    /// Returns dependency paths in source order.
    ///
    /// # Returns
    ///
    /// Static property dependencies in declaration order.
    #[must_use]
    #[inline]
    pub const fn depends_on(&self) -> &'static [PropertyPath<'static>] {
        self.depends_on
    }

    /// Returns named dependency bindings in declaration order.
    ///
    /// # Returns
    ///
    /// Static named slots and their source property bindings.
    #[must_use]
    #[inline]
    pub const fn dependency_bindings(&self) -> &'static [DependencyBindingMetadata] {
        self.dependency_bindings
    }

    /// Returns the selected validator value shape.
    ///
    /// # Returns
    ///
    /// Whether the validator receives expanded values or declared containers.
    #[must_use]
    #[inline]
    pub const fn target(&self) -> TargetMode {
        self.target
    }

    /// Returns the behavior for an absent expanded value.
    ///
    /// # Returns
    ///
    /// Whether the occurrence is skipped or rejected when the value is absent.
    #[must_use]
    #[inline]
    pub const fn on_none(&self) -> OnNone {
        self.on_none
    }
}

/// Rejects repeated names while preserving declaration-order validation.
///
/// # Parameters
///
/// * `bindings` - Dependency bindings to validate in source order.
const fn assert_unique_dependency_names(bindings: &[DependencyBindingMetadata]) {
    let mut index = 0;
    while index < bindings.len() {
        let binding = bindings[index];
        assert!(
            !contains_dependency_name(bindings, index, binding.name()),
            "validator dependency names must be unique",
        );
        index += 1;
    }
}

/// Reports whether an earlier dependency binding already uses `name`.
///
/// # Parameters
///
/// * `bindings` - Dependency bindings already declared for the validator.
/// * `end` - Exclusive prefix length to inspect.
/// * `name` - Candidate name to compare with that prefix.
///
/// # Returns
///
/// `true` when a binding in the prefix already uses `name`.
#[must_use]
const fn contains_dependency_name(bindings: &[DependencyBindingMetadata], end: usize, name: &str) -> bool {
    let mut index = 0;
    while index < end {
        if same_str(bindings[index].name(), name) {
            return true;
        }
        index += 1;
    }
    false
}

/// Compares two string slices without relying on non-const string equality.
///
/// # Parameters
///
/// * `left` - First byte sequence to compare.
/// * `right` - Second byte sequence to compare.
///
/// # Returns
///
/// `true` when both strings contain the same bytes in the same order.
#[must_use]
const fn same_str(left: &str, right: &str) -> bool {
    let left = left.as_bytes();
    let right = right.as_bytes();
    if left.len() != right.len() {
        return false;
    }
    let mut index = 0;
    while index < left.len() {
        if left[index] != right[index] {
            return false;
        }
        index += 1;
    }
    true
}
