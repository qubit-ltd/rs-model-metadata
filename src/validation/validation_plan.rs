// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Immutable validation plans.
//!
//! Construction and execution methods are defined in the `build` and `execute`
//! child modules; the plan owns their shared immutable state.

// Defines declaration discovery and fallible plan construction.
mod build;
// Defines instance borrowing, rule execution and bounded report collection.
mod execute;

use crate::metadata::TypeMetadata;
use crate::resolve::ModelGraph;
use crate::validation::internal::field_rule_binding::FieldRuleBinding;
use crate::validation::model_rule_binding::ModelRuleBinding;

/// A read-only plan containing no model instance or getter output.
///
/// Construction discovers declarations at every usage path, binds supported
/// rules, and rejects unsupported execution shapes. It never invokes getters.
/// A successful build does not imply that a particular model instance is valid.
/// Reuse the plan for repeated calls; reports and execution budgets are fresh
/// for each call, and no instance borrow is stored in the plan.
///
/// # Type Parameters
///
/// - `'a`: lifetime of the immutable structural graph and its registry
///   snapshot. The graph must outlive the plan.
///
/// # Examples
///
/// ```
/// use qubit_model_derive::Model;
/// use qubit_model_metadata::metadata::TypeMetadata;
/// use qubit_model_metadata::registry::ModelRegistry;
/// use qubit_model_metadata::resolve::ResolveInputs;
/// use qubit_model_metadata::resolve::StructureResolver;
/// use qubit_model_metadata::validation::ValidationBuildInputs;
/// use qubit_model_metadata::validation::ValidationOptions;
/// use qubit_model_metadata::validation::ValidationPlan;
/// use qubit_reflect::ReflectedRef;
/// use qubit_validator::ValidatorRegistry;
///
/// #[Model]
/// struct Profile { #[text(non_blank)] name: String }
/// # fn main() {
/// let root = TypeMetadata::of::<Profile>();
/// let roots = [root];
/// let models = ModelRegistry::from_static_metadata(&[]).expect("isolated registry");
/// let graph = StructureResolver::new(ResolveInputs { models: &models, roots: &roots })
///     .resolve().expect("valid structure");
/// let validators = ValidatorRegistry::empty();
/// let plan = ValidationPlan::build(root, ValidationBuildInputs { graph: &graph, validators: &validators })
///     .expect("built-in text rules bind without custom registration");
/// let profile = Profile { name: String::new() };
/// let report = plan.validate(ReflectedRef::new(&profile), &ValidationOptions::default())
///     .expect("no infrastructure failure");
/// assert_eq!(report.violations().len(), 1);
/// assert_eq!(report.violations()[0].path().render(), "name");
/// # }
/// ```
#[must_use]
pub struct ValidationPlan<'a> {
    /// Metadata root for the plan.
    root: &'static TypeMetadata,
    /// Resolved graph used to compile and execute paths.
    graph: &'a ModelGraph<'a>,
    /// Field-level validator bindings.
    bindings: Box<[FieldRuleBinding]>,
    /// Model-level validator bindings.
    model_rules: Box<[ModelRuleBinding]>,
}

impl<'a> ValidationPlan<'a> {
    /// Returns the number of bound validator occurrences.
    ///
    /// # Returns
    ///
    /// The sum of field-level and model-level validator occurrences bound
    /// into this plan.
    #[must_use]
    #[inline]
    pub const fn binding_count(&self) -> usize {
        self.bindings.len() + self.model_rules.len()
    }

    /// Returns the canonical graph metadata this plan was built for.
    ///
    /// # Returns
    ///
    /// The static metadata root used to resolve this plan.
    #[must_use]
    #[inline]
    pub const fn root(&self) -> &'static TypeMetadata {
        self.root
    }

    /// Returns the borrowed graph used for both binding and execution.
    ///
    /// # Returns
    ///
    /// The graph retained by this plan; its lifetime is bounded by the plan's
    /// registry snapshot.
    #[must_use = "inspect the graph retained by this plan"]
    #[inline]
    pub const fn graph(&self) -> &'a ModelGraph<'a> {
        self.graph
    }

    /// Adds prepared model-level validators to this plan in iteration order.
    ///
    /// # Parameters
    ///
    /// * `bindings` - Prepared model-level validator occurrences to append.
    ///
    /// # Returns
    ///
    /// This plan with the supplied model-level rules appended after existing
    /// model rules. An empty iterator leaves the existing rules unchanged.
    #[must_use = "use the returned plan containing the added model rules"]
    pub fn with_model_rules(mut self, bindings: impl IntoIterator<Item = ModelRuleBinding>) -> Self {
        let bindings = bindings.into_iter();
        let mut all = Vec::with_capacity(self.model_rules.len() + bindings.size_hint().0);
        all.extend(self.model_rules);
        all.extend(bindings);
        self.model_rules = all.into_boxed_slice();
        self
    }

    /// Returns model-level rules retained by the plan.
    #[must_use]
    #[inline]
    pub(crate) fn model_rules(&self) -> &[ModelRuleBinding] {
        &self.model_rules
    }

    /// Returns the immutable bound occurrences for the executor.
    #[must_use]
    #[inline]
    pub(crate) fn bindings(&self) -> &[FieldRuleBinding] {
        &self.bindings
    }
}
