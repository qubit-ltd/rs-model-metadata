// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! One compiled field occurrence and its bound paths.

use qubit_validator::BoundValidator;
use qubit_validator::ValidatorId;

use super::selector_binding::SelectorBinding;
use super::validation_occurrence::ValidationOccurrence;
use crate::metadata::OnNone;
use crate::property::ItemEqAdapter;
use crate::property::MapLenAdapter;
use crate::validation::compiled_property_path::CompiledPropertyPath;
use crate::validation::standard_constraints::StandardTarget;

/// One declaration bound to an executable validator and compiled paths.
#[derive(Clone, Debug)]
pub(crate) struct FieldRuleBinding {
    /// Original declaration context retained for execution failures.
    pub(crate) context: ValidationOccurrence,
    /// Stable occurrence number used for deterministic reports.
    pub(crate) occurrence: usize,
    /// Runtime validator identifier.
    pub(crate) rule_id: ValidatorId,
    /// Compiled value path supplied to the validator.
    pub(crate) value: CompiledPropertyPath,
    /// Compiled dependency paths supplied to the validator.
    pub(crate) dependencies: Box<[CompiledPropertyPath]>,
    /// Checked execution mechanism for this occurrence.
    pub(crate) execution: FieldExecution,
    /// Behavior when an optional value is absent.
    pub(crate) on_none: OnNone,
    /// Selector metadata for nested collection validation.
    pub(crate) selector: Option<SelectorBinding>,
}

/// Registry-backed validation or metadata-owned sequence equality.
#[derive(Clone, Debug)]
pub(crate) enum FieldExecution {
    /// A prepared registered rule and its checked input projection.
    Registry {
        /// Prepared rule.
        validator: BoundValidator,
        /// Standard target, if this is a built-in constraint.
        target: Option<StandardTarget>,
        /// Checked map count adapter, when needed.
        map_len: Option<MapLenAdapter>,
    },
    /// Exact element comparison adapter for one sequence declaration.
    SequenceUnique {
        /// Typed equality operation generated for the field.
        item_eq: ItemEqAdapter,
    },
}

impl FieldRuleBinding {
    /// Returns the occurrence index in the plan.
    pub(crate) const fn occurrence(&self) -> usize {
        self.occurrence
    }
    /// Returns the validator identifier.
    pub(crate) const fn rule_id(&self) -> ValidatorId {
        self.rule_id
    }
    /// Returns the compiled value path.
    pub(crate) const fn value(&self) -> &CompiledPropertyPath {
        &self.value
    }
    /// Returns compiled dependency paths in declaration order.
    pub(crate) fn dependencies(&self) -> &[CompiledPropertyPath] {
        &self.dependencies
    }
    /// Returns the checked execution mechanism.
    pub(crate) const fn execution(&self) -> &FieldExecution {
        &self.execution
    }
    /// Returns the absent-value policy.
    pub(crate) const fn on_none(&self) -> OnNone {
        self.on_none
    }
    /// Returns the nested selector binding, if any.
    pub(crate) const fn selector(&self) -> Option<&SelectorBinding> {
        self.selector.as_ref()
    }
}
