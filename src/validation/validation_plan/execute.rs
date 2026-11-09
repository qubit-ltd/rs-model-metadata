// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Execution APIs sharing one report accumulator and one work budget.

use qubit_reflect::value::Local;
use qubit_reflect::value::ValueForRef;
use qubit_validation_rules::collection::UniquePairs;
use qubit_validator::BoundValidationContext;
use qubit_validator::ExecutionError;
use qubit_validator::ExecutionErrorKind;
use qubit_validator::PathSegment;
use qubit_validator::SkipReason;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationPath;
use qubit_validator::ValidationReport;
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorId;
use qubit_validator::Violation;
use qubit_validator::ViolationCode;
use qubit_validator::ViolationParam;

use super::ValidationPlan;
use super::internal::selection_mask::SelectionMask;
use crate::metadata::OnNone;
use crate::metadata::PropertyBorrowed;
use crate::metadata::PropertyOwned;
use crate::metadata::PropertyValue;
use crate::metadata::SelectorPosition;
use crate::property::ItemEqAdapter;
use crate::property::OptionalOps;
use crate::property::PropertyTypeMismatch;
use crate::resolve::ModelGraph;
use crate::validation::ModelValidationError;
use crate::validation::ValidationOptions;
use crate::validation::internal::execution_budget::ExecutionBudget;
use crate::validation::internal::execution_failure::ExecutionFailure;
use crate::validation::internal::field_rule_binding::FieldExecution;
use crate::validation::internal::field_rule_binding::FieldRuleBinding;
use crate::validation::internal::path_reader;
use crate::validation::internal::path_reader::path_for;
use crate::validation::internal::report_accumulator::ReportAccumulator;
use crate::validation::standard_constraints::StandardTarget;
use crate::validation::validation_options::FieldPath;
use crate::validation::validation_options::ValidationSelection;

impl<'a> ValidationPlan<'a> {
    /// Executes selected occurrences under one stopping policy and work budget.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Concrete root type that can be borrowed for local reflection and
    ///   inspected through `Any`.
    ///
    /// # Parameters
    ///
    /// * `value` - Borrowed root instance whose type must match this plan.
    /// * `options` - Selection, stopping policy and execution budgets for this
    ///   validation call.
    ///
    /// # Returns
    ///
    /// A report containing the selected rule outcomes, or the infrastructure
    /// error together with the partial report.
    ///
    /// # Errors
    /// Returns the original infrastructure failure and the partial report.
    /// The borrowed root must preserve its concrete `Any` view for typed
    /// validators; otherwise execution reports an adapter contract violation.
    #[inline]
    pub fn validate<T: ValueForRef<Local> + std::any::Any>(
        &self,
        value: &T,
        options: &ValidationOptions,
    ) -> Result<ValidationReport, ModelValidationError> {
        self.validate_with_context(value, &[], options)
    }

    /// Executes validation for a root value whose concrete type is erased.
    ///
    /// # Parameters
    ///
    /// * `value` - Borrowed root instance whose type must match this plan.
    /// * `options` - Selection, stopping policy and execution budgets for this
    ///   validation call.
    ///
    /// # Returns
    ///
    /// A report containing the selected rule outcomes, or the infrastructure
    /// error together with the partial report.
    ///
    /// # Errors
    /// Returns the original infrastructure failure and the partial report.
    /// Non-text typed validation requires a dynamic borrow that preserves its
    /// concrete `Any` view; a view without one produces an adapter contract
    /// violation.
    #[inline]
    pub fn validate_dynamic(
        &self,
        value: PropertyBorrowed<'_>,
        options: &ValidationOptions,
    ) -> Result<ValidationReport, ModelValidationError> {
        self.validate_dynamic_with_context(value, &[], options)
    }

    /// Executes with nearest-parent-first containing-object borrows.
    ///
    /// # Type Parameters
    ///
    /// * `'value` - Shared lifetime of the root and ancestor borrows.
    /// * `T` - Concrete root type that can be borrowed for local reflection and
    ///   inspected through `Any`.
    ///
    /// # Parameters
    ///
    /// * `value` - Borrowed root instance whose type must match this plan.
    /// * `ancestors` - Containing objects ordered from nearest parent outward,
    ///   available to dependency readers.
    /// * `options` - Selection, stopping policy and execution budgets for this
    ///   validation call.
    ///
    /// # Returns
    ///
    /// A report containing the selected rule outcomes, or the infrastructure
    /// error together with the partial report.
    ///
    /// # Errors
    /// Returns typed source context for failed reads or rule invocations, or a
    /// traversal-limit error before the operation that would exceed its budget.
    /// Non-text typed validation requires a dynamic borrow that preserves its
    /// concrete `Any` view; a view without one produces an adapter contract
    /// violation.
    #[inline]
    pub fn validate_with_context<'value, T: ValueForRef<Local> + std::any::Any>(
        &self,
        value: &'value T,
        ancestors: &[PropertyBorrowed<'value>],
        options: &ValidationOptions,
    ) -> Result<ValidationReport, ModelValidationError> {
        self.validate_dynamic_with_context(PropertyBorrowed::new_any(value), ancestors, options)
    }

    /// Executes a dynamic root value against the selected rules and shared
    /// execution budget.
    ///
    /// The public `validate_dynamic` and `validate_with_context` flows provide
    /// the root and ancestor borrows accepted here.
    ///
    /// # Parameters
    ///
    /// * `value` - Dynamic borrow of the root instance described by this plan.
    /// * `ancestors` - Containing objects ordered from nearest parent outward.
    /// * `options` - Selection, stopping policy, and execution budgets.
    ///
    /// # Returns
    ///
    /// A report of selected outcomes, or an error retaining the partial report.
    ///
    /// # Errors
    ///
    /// Returns an input type or selection error, a typed execution failure, or
    /// a traversal-limit error. Typed validation also requires the dynamic
    /// borrow to preserve its concrete `Any` view.
    fn validate_dynamic_with_context<'value>(
        &self,
        value: PropertyBorrowed<'value>,
        ancestors: &[PropertyBorrowed<'value>],
        options: &ValidationOptions,
    ) -> Result<ValidationReport, ModelValidationError> {
        if value.clone().type_id() != self.root().type_id() {
            return Err(ModelValidationError::new(
                ExecutionError::new(ExecutionErrorKind::InputTypeMismatch),
                ReportAccumulator::new(options).into_report(),
            )
            .at_model(self.root(), None));
        }
        if let Some(path) = unmatched_selection(options.selection(), self.model_rules().len(), self.bindings()) {
            let error = ExecutionError::new(ExecutionErrorKind::InvalidSelection)
                .with_path(ValidationPath::root())
                .with_trusted_source(path);
            return Err(
                ModelValidationError::new(error, ReportAccumulator::new(options).into_report())
                    .at_model(self.root(), None),
            );
        }
        let mut report = ReportAccumulator::new(options);
        let mut budget = ExecutionBudget::new(options);
        let model_rule_count = self.model_rules().len();
        let model_path = ValidationPath::root();
        let mut mask = SelectionMask::new(options.selection(), model_rule_count, self.bindings());
        for (occurrence, binding) in self.model_rules().iter().enumerate() {
            if report.stopped() {
                return Ok(report.into_report());
            }
            let path = model_path.clone();
            if !mask.selected(occurrence) {
                continue;
            }
            let result = (|| {
                let input = reflected_value(&value)?;
                let context = BoundValidationContext::new_with_paths(&[], &[])?;
                budget.invoke(0, false)?;
                let outcome = binding.validate(input, &context)?;
                let has_more_work = mask.later_selected(occurrence);
                report.accept(occurrence, &path, outcome, has_more_work).map(|_| ())
            })();
            if let Err(error) = result {
                return Err(
                    ModelValidationError::new(error.with_rule(binding.rule_id()), report.into_report())
                        .at_model(self.root(), Some(occurrence)),
                );
            }
        }
        for (binding_index, binding) in self.bindings().iter().enumerate() {
            if report.stopped() {
                break;
            }
            let mask_index = model_rule_count + binding_index;
            if !mask.selected(mask_index) {
                continue;
            }
            let path = mask
                .take_field_path(mask_index)
                .unwrap_or_else(|| path_for(binding.value()));
            let occurrence = self.model_rules().len() + binding.occurrence();
            let has_more_work = mask.later_selected(mask_index);
            if let Err(failure) = execute_field(
                binding,
                occurrence,
                value.clone(),
                ancestors,
                self.graph(),
                &mut budget,
                &mut report,
                has_more_work,
            ) {
                let mut error = (*failure.error).with_rule(binding.rule_id());
                if error.path().as_segments().is_empty() {
                    error = error.with_path(path);
                }
                return Err(ModelValidationError::new(error, report.into_report()).at_field(
                    &binding.context,
                    occurrence,
                    failure.dependency,
                ));
            }
        }
        Ok(report.into_report())
    }
}

/// Returns the first requested path that has no corresponding bound rule.
///
/// Model-level occurrences match the empty path. A field selection must not
/// silently succeed when it is empty or names an existing field without a
/// validation rule.
///
/// # Parameters
///
/// * `selection` - Requested model or field paths to match against bound rules.
/// * `model_rule_count` - Number of model-level rules, which match the empty
///   path.
/// * `bindings` - Compiled field rules available to satisfy field selections.
///
/// # Returns
///
/// The first unmatched field path, or `None` when every requested path has a
/// corresponding model or field rule.
fn unmatched_selection(
    selection: &ValidationSelection,
    model_rule_count: usize,
    bindings: &[FieldRuleBinding],
) -> Option<FieldPath> {
    let ValidationSelection::Fields(fields) = selection else {
        return None;
    };
    if fields.is_empty() {
        return Some(FieldPath::from_segments(std::iter::empty::<String>()));
    }
    fields
        .iter()
        .find(|field| {
            let matches_model = model_rule_count > 0 && field.segments().is_empty();
            !matches_model
                && !bindings
                    .iter()
                    .any(|binding| field_matches(field, &path_for(binding.value())))
        })
        .cloned()
}

/// Reads a value before its dependencies and executes one compiled field rule.
///
/// Returns `Ok(())` when the field has no runtime value or its outcome is
/// accepted. Read, budget, validator, and report failures retain their field
/// and dependency context in `ExecutionFailure`.
///
/// # Parameters
///
/// * `binding` - Compiled rule and field metadata used to read and validate
///   this occurrence.
/// * `occurrence` - Report index assigned to this field occurrence.
/// * `root` - Borrowed root model instance from which the field value is read.
/// * `ancestors` - Containing-object borrows available to dependency readers.
/// * `graph` - Resolved model graph providing registry and reflection context.
/// * `budget` - Shared execution budget charged for reads and validator calls.
/// * `report` - Accumulator receiving the field outcome and any violations.
/// * `has_more_work` - Whether later selected work remains for stop-policy
///   handling.
#[allow(clippy::too_many_arguments)]
fn execute_field<'value>(
    binding: &FieldRuleBinding,
    occurrence: usize,
    root: PropertyBorrowed<'value>,
    ancestors: &[PropertyBorrowed<'value>],
    graph: &ModelGraph<'_>,
    budget: &mut ExecutionBudget<'_>,
    report: &mut ReportAccumulator<'_>,
    has_more_work: bool,
) -> Result<(), ExecutionFailure> {
    let context = graph
        .registry()
        .context()
        .expect("validation plans require the graph's reflection context");
    let path = path_for(binding.value());
    let Some(value) = path_reader::read(context, binding.value(), root.clone(), 0, budget)? else {
        return Ok(());
    };
    let missing_optional_path = matches!(&value, PropertyValue::OptionalBorrowed(None));
    let direct_optional_scalar = binding.value().steps().last().is_some_and(|step| {
        let property = step.property();
        property.getter().is_none() && property.optional_ops().is_some()
    }) && matches!(
        binding.execution(),
        FieldExecution::Registry {
            target: Some(StandardTarget::Value),
            ..
        }
    );
    let projected_scalar = if direct_optional_scalar && !missing_optional_path {
        let descriptor = binding
            .value()
            .steps()
            .last()
            .and_then(|step| step.property().optional_ops())
            .ok_or_else(|| ExecutionError::new(ExecutionErrorKind::AdapterContractViolation).with_path(path.clone()))?;
        Some(
            optional_scalar_value(&value, descriptor).map_err(|_| {
                ExecutionError::new(ExecutionErrorKind::AdapterContractViolation).with_path(path.clone())
            })?,
        )
    } else {
        None
    };
    if missing_optional_path || projected_scalar.as_ref().is_some_and(Option::is_none) {
        if !binding.value().is_optional() {
            return Err(ExecutionError::new(ExecutionErrorKind::AdapterContractViolation).into());
        }
        let outcome = if binding.on_none() == OnNone::Reject {
            ValidationOutcome::Invalid(vec![Violation::new(
                binding.rule_id(),
                ViolationCode::new("value.required"),
            )])
        } else {
            ValidationOutcome::Skipped {
                reason: SkipReason::MissingOptional,
                prerequisites: Vec::new(),
            }
        };
        report.accept(occurrence, &path, outcome, has_more_work).map(|_| ())?;
        return Ok(());
    }
    if let FieldExecution::SequenceUnique { item_eq } = binding.execution() {
        return execute_unique(
            binding.rule_id(),
            occurrence,
            value,
            &path,
            *item_eq,
            budget,
            report,
            has_more_work,
        )
        .map_err(Into::into);
    }
    if let Some(selector) = binding.selector() {
        return execute_selected_members(
            binding,
            occurrence,
            value,
            &path,
            selector.position(),
            selector.items(),
            budget,
            report,
            has_more_work,
        )
        .map_err(Into::into);
    }
    execute_registry_field(
        binding,
        occurrence,
        value,
        projected_scalar,
        path,
        root,
        ancestors,
        graph,
        budget,
        report,
        has_more_work,
    )
}

/// Resolves registry dependencies, builds validator input, and records its
/// outcome.
///
/// # Parameters
///
/// * `binding` - Compiled registry rule, field path, and dependency metadata.
/// * `occurrence` - Report index assigned to this field occurrence.
/// * `value` - Property value read for the field, including collection views.
/// * `projected_scalar` - Borrowed scalar projected from a direct optional
///   field, when applicable.
/// * `path` - Field path used to attach context to execution failures.
/// * `root` - Borrowed root model instance used for dependency reads.
/// * `ancestors` - Containing-object borrows available to dependency readers.
/// * `graph` - Resolved model graph providing registry and reflection context.
/// * `budget` - Shared execution budget charged for dependency reads and
///   validator calls.
/// * `report` - Accumulator receiving the validator outcome and any violations.
/// * `has_more_work` - Whether later selected work remains for stop-policy
///   handling.
#[allow(clippy::too_many_arguments)]
fn execute_registry_field<'value>(
    binding: &FieldRuleBinding,
    occurrence: usize,
    value: PropertyValue<'_>,
    projected_scalar: Option<Option<PropertyBorrowed<'value>>>,
    path: ValidationPath,
    root: PropertyBorrowed<'value>,
    ancestors: &[PropertyBorrowed<'value>],
    graph: &ModelGraph<'_>,
    budget: &mut ExecutionBudget<'_>,
    report: &mut ReportAccumulator<'_>,
    has_more_work: bool,
) -> Result<(), ExecutionFailure> {
    let FieldExecution::Registry {
        validator,
        target,
        map_len,
    } = binding.execution()
    else {
        return Err(ExecutionError::new(ExecutionErrorKind::AdapterContractViolation).into());
    };
    let (dependencies, paths) = path_reader::dependencies(binding.dependencies(), root, ancestors, graph, budget)?;
    let values: Vec<_> = dependencies
        .iter()
        .map(property_value)
        .collect::<Result<_, _>>()
        .map_err(|error: ExecutionError| error.with_path(path.clone()))?;
    // The plan binder matched dependencies by signature name, so this ordered
    // fast path receives values in the validator's declared order.
    let context = BoundValidationContext::new_with_paths(&values, &paths)?;
    let count = match &value {
        PropertyValue::BorrowedSlice(values) => Some(values.len()),
        _ => None,
    };
    let map_count = if *target == Some(StandardTarget::MapCount) {
        let Some(map_len) = map_len else {
            return Err(ExecutionError::new(ExecutionErrorKind::AdapterContractViolation).into());
        };
        let count = map_len(&value).map_err(|error| {
            ExecutionError::new(ExecutionErrorKind::PropertyReadFailed)
                .with_trusted_source(error)
                .with_path(path.clone())
        })?;
        let Some(count) = count else {
            let declared_optional = binding.value().is_optional()
                || binding
                    .value()
                    .steps()
                    .last()
                    .and_then(|step| step.property().optional_ops())
                    .is_some();
            if !declared_optional {
                return Err(ExecutionError::new(ExecutionErrorKind::AdapterContractViolation)
                    .with_path(path)
                    .into());
            }
            report
                .accept(
                    occurrence,
                    &path,
                    ValidationOutcome::Skipped {
                        reason: SkipReason::MissingOptional,
                        prerequisites: Vec::new(),
                    },
                    has_more_work,
                )
                .map(|_| ())?;
            return Ok(());
        };
        Some(count)
    } else {
        None
    };
    let input = match (*target, &value) {
        (Some(StandardTarget::SequenceCount), PropertyValue::BorrowedSlice(_)) => {
            ValidationValue::Typed(count.as_ref().expect("slice count"))
        }
        (Some(StandardTarget::MapCount), PropertyValue::Borrowed(_) | PropertyValue::OptionalBorrowed(Some(_))) => {
            ValidationValue::Typed(map_count.as_ref().expect("map count"))
        }
        (Some(StandardTarget::MapCount), _) => {
            return Err(ExecutionError::new(ExecutionErrorKind::PropertyReadFailed)
                .with_path(path)
                .into());
        }
        (_, PropertyValue::BorrowedSlice(_)) => {
            return Err(ExecutionError::new(ExecutionErrorKind::PropertyReadFailed).into());
        }
        _ => projected_scalar
            .as_ref()
            .and_then(Option::as_ref)
            .map(reflected_value)
            .transpose()
            .map_err(|error| error.with_path(path.clone()))?
            .map_or_else(|| property_value(&value), Ok)?,
    };
    budget.invoke(path.as_segments().len(), false)?;
    let outcome = validator.validate(input, &context).map_err(|error| {
        let dependency = error.dependency().and_then(|name| {
            binding
                .dependencies()
                .iter()
                .filter_map(|path| path.dependency())
                .find(|dependency| dependency.name() == name)
        });
        let error = if dependency.is_some() {
            error
        } else {
            prefix_error(error, &path)
        };
        ExecutionFailure {
            error: Box::new(error),
            dependency,
        }
    })?;
    report.accept(occurrence, &path, outcome, has_more_work).map(|_| ())?;
    Ok(())
}

/// Compares borrowed elements in deterministic pair order, reserving every
/// read and comparison before calling the checked adapter.
///
/// # Parameters
///
/// * `binding` - Compiled rule metadata used to identify duplicate-item
///   violations.
/// * `occurrence` - Report index assigned to this field occurrence.
/// * `value` - Property value expected to contain a borrowed sequence.
/// * `path` - Field path used to attach read or comparison failures.
/// * `item_eq` - Checked adapter for comparing two borrowed sequence items.
/// * `budget` - Shared execution budget charged for reads and comparisons.
/// * `report` - Accumulator receiving the valid outcome or duplicate violation.
/// * `has_more_work` - Whether later selected work remains for stop-policy
///   handling.
#[allow(clippy::too_many_arguments)]
fn execute_unique(
    rule_id: ValidatorId,
    occurrence: usize,
    value: PropertyValue<'_>,
    path: &ValidationPath,
    item_eq: ItemEqAdapter,
    budget: &mut ExecutionBudget<'_>,
    report: &mut ReportAccumulator<'_>,
    has_more_work: bool,
) -> Result<(), ExecutionError> {
    let PropertyValue::BorrowedSlice(values) = value else {
        return Err(ExecutionError::new(ExecutionErrorKind::PropertyReadFailed).with_path(path.clone()));
    };
    budget
        .invoke(path.as_segments().len(), false)
        .map_err(|error| error.with_path(path.clone()))?;
    let mut current_second = None;
    let mut second_path = path.clone();
    for (first, second) in UniquePairs::new(values.len()) {
        if current_second != Some(second) {
            current_second = Some(second);
            second_path = path.clone().with_index(second);
        }
        let first_path = path.clone().with_index(first);
        budget
            .read(first_path.as_segments().len())
            .map_err(|error| error.with_path(first_path.clone()))?;
        let Some(first_value) = values.get(first) else {
            return Err(ExecutionError::new(ExecutionErrorKind::PropertyReadFailed).with_path(first_path));
        };
        budget
            .read(second_path.as_segments().len())
            .map_err(|error| error.with_path(second_path.clone()))?;
        let Some(second_value) = values.get(second) else {
            return Err(ExecutionError::new(ExecutionErrorKind::PropertyReadFailed).with_path(second_path.clone()));
        };
        budget
            .compare(second_path.as_segments().len())
            .map_err(|error| error.with_path(second_path.clone()))?;
        let equal = item_eq(first_value, second_value).map_err(|error| {
            ExecutionError::new(ExecutionErrorKind::PropertyReadFailed)
                .with_trusted_source(error)
                .with_path(second_path.clone())
        })?;
        if equal {
            let violation = Violation::new(rule_id, ViolationCode::new("collection.duplicate_item"))
                .with_path(ValidationPath::root().with_index(second))
                .with_param("first_index", ViolationParam::Unsigned(first as u128));
            report.accept(
                occurrence,
                path,
                ValidationOutcome::Invalid(vec![violation]),
                has_more_work,
            )?;
            return Ok(());
        }
    }
    report.accept(occurrence, path, ValidationOutcome::Valid, has_more_work)?;
    Ok(())
}

/// Executes selected sequence or map members, checking budgets before each
/// member is exposed to the validator.
///
/// # Parameters
///
/// * `binding` - Compiled registry rule and dependency metadata for each
///   selected member.
/// * `occurrence` - Report index assigned to this field occurrence.
/// * `value` - Property value containing the sequence or map to project.
/// * `path` - Field path used as the base for member paths and failures.
/// * `position` - Selector position choosing sequence elements, map keys, or
///   map values.
/// * `items` - Checked adapter that projects the selected collection members.
/// * `budget` - Shared execution budget charged before each member read and
///   validator call.
/// * `report` - Accumulator receiving member outcomes and any violations.
/// * `has_more_work` - Whether work beyond the selected members remains for
///   stop-policy handling.
#[allow(clippy::too_many_arguments)]
fn execute_selected_members(
    binding: &FieldRuleBinding,
    occurrence: usize,
    value: PropertyValue<'_>,
    path: &ValidationPath,
    position: SelectorPosition,
    items: crate::property::CollectionItemsAdapter,
    budget: &mut ExecutionBudget<'_>,
    report: &mut ReportAccumulator<'_>,
    has_more_work: bool,
) -> Result<(), ExecutionError> {
    let FieldExecution::Registry { validator, .. } = binding.execution() else {
        return Err(ExecutionError::new(ExecutionErrorKind::AdapterContractViolation).with_path(path.clone()));
    };
    execute_selected_members_with_validator(
        occurrence,
        value,
        path,
        position,
        items,
        validator,
        budget,
        report,
        has_more_work,
    )
}

/// Executes selected members with an already-bound validator.
///
/// The private split allows the budget behavior to be tested without
/// constructing the full validation plan and registry hierarchy.
///
/// # Parameters
///
/// * `occurrence` - Report index assigned to this field occurrence.
/// * `value` - Property value containing the sequence or map to project.
/// * `path` - Field path used as the base for member paths and failures.
/// * `position` - Selector position choosing sequence or map members.
/// * `items` - Adapter that projects the selected collection members.
/// * `validator` - Bound rule applied to each selected member.
/// * `budget` - Shared execution budget charged before each member read and
///   validator call.
/// * `report` - Accumulator receiving member outcomes and violations.
/// * `has_more_work` - Whether work beyond the selected members remains for
///   stop-policy handling.
///
/// # Errors
///
/// Returns an execution error when an adapter, budget, or validator fails.
#[allow(clippy::too_many_arguments)]
fn execute_selected_members_with_validator(
    occurrence: usize,
    value: PropertyValue<'_>,
    path: &ValidationPath,
    position: SelectorPosition,
    items: crate::property::CollectionItemsAdapter,
    validator: &qubit_validator::BoundValidator,
    budget: &mut ExecutionBudget<'_>,
    report: &mut ReportAccumulator<'_>,
    has_more_work: bool,
) -> Result<(), ExecutionError> {
    let mut values = items(&value, position).map_err(|error| {
        ExecutionError::new(ExecutionErrorKind::PropertyReadFailed)
            .with_trusted_source(error)
            .with_path(path.clone())
    })?;
    let Some(mut values) = values.take() else {
        return Ok(());
    };
    let context = BoundValidationContext::new_with_paths(&[], &[])?;
    let mut index = 0_usize;
    loop {
        if report.stopped() {
            break;
        }
        if values.remaining_len() == Some(0) {
            break;
        }
        let member_path = match position {
            SelectorPosition::Element => path.clone().with_index(index),
            SelectorPosition::MapKey => path.clone().with_map_entry(index).with_map_key(),
            SelectorPosition::MapValue => path.clone().with_map_entry(index).with_map_value(),
        };
        let depth = member_path.as_segments().len();
        // Charge before `next()`: an unknown-length iterator can only reveal
        // exhaustion through a probe, which reserves one possible member.
        budget
            .read(depth)
            .map_err(|error| error.with_path(member_path.clone()))?;
        budget
            .invoke(depth, true)
            .map_err(|error| error.with_path(member_path.clone()))?;
        let Some(element) = values.next() else {
            break;
        };
        let member_has_more = values.remaining_len().is_none_or(|remaining| remaining > 0);
        let outcome = validator
            .validate(
                reflected_value(&element).map_err(|error| error.with_path(member_path.clone()))?,
                &context,
            )
            .map_err(|error| prefix_error(error, &member_path))?;
        report
            .accept(occurrence, &member_path, outcome, member_has_more || has_more_work)
            .map_err(|error| error.with_path(member_path))?;
        index += 1;
    }
    Ok(())
}

/// Prefixes a validator's relative error path without discarding its source.
///
/// # Parameters
///
/// * `error` - Validator failure whose relative path is extended.
/// * `prefix` - Field or member path prepended to the validator's path.
fn prefix_error(error: ExecutionError, prefix: &ValidationPath) -> ExecutionError {
    let path =
        prefix
            .as_segments()
            .iter()
            .chain(error.path().as_segments())
            .fold(ValidationPath::root(), |path, segment| match segment {
                PathSegment::Field(field) => path.with_field(field),
                PathSegment::Index(index) => path.with_index(*index),
                PathSegment::MapEntry(index) => path.with_map_entry(*index),
                PathSegment::MapKey => path.with_map_key(),
                PathSegment::MapValue => path.with_map_value(),
                _ => path,
            });
    error.with_path(path)
}

/// Converts a reflected borrow into the validator input abstraction.
///
/// Strings become text inputs; other values require a dynamic view that
/// exposes the concrete value through `Any`.
///
/// # Parameters
///
/// * `value` - Reflected borrow converted to the validator's text or typed
///   input form.
fn reflected_value<'a>(value: &'a PropertyBorrowed<'_>) -> Result<ValidationValue<'a>, ExecutionError> {
    if let Some(text) = value.downcast_ref::<String>() {
        return Ok(ValidationValue::Text(text.as_str()));
    }
    if let Some(text) = value.downcast_ref::<str>() {
        return Ok(ValidationValue::Text(text));
    }
    value
        .as_any()
        .map(ValidationValue::Typed)
        .ok_or_else(|| ExecutionError::new(ExecutionErrorKind::AdapterContractViolation))
}

/// Converts a property adapter result into a validator input abstraction.
///
/// Borrowed and owned values use the same text-versus-typed conversion. An
/// absent optional value or a borrowed slice becomes `Missing`.
///
/// # Parameters
///
/// * `value` - Borrowed or owned property adapter result converted for a
///   validator.
fn property_value<'a>(value: &'a PropertyValue<'_>) -> Result<ValidationValue<'a>, ExecutionError> {
    match value {
        PropertyValue::Borrowed(value) => reflected_value(value),
        PropertyValue::OptionalBorrowed(Some(value)) => reflected_value(value),
        PropertyValue::OptionalBorrowed(None) => Ok(ValidationValue::Missing),
        PropertyValue::Owned(value) => owned_value(value),
        PropertyValue::BorrowedSlice(_) => Ok(ValidationValue::Missing),
    }
}

/// Borrows the contained scalar from a reflected, field-backed `Option<T>`.
/// A non-borrowed adapter shape returns `PropertyTypeMismatch`; `Ok(None)`
/// means the optional field is absent. No value is cloned or formatted.
///
/// # Parameters
///
/// * `value` - Property adapter result expected to contain a borrowed optional
///   field.
/// * `optional` - Operations descriptor used to project the borrowed inner
///   scalar.
fn optional_scalar_value<'a>(
    value: &PropertyValue<'a>,
    optional: OptionalOps,
) -> Result<Option<PropertyBorrowed<'a>>, PropertyTypeMismatch> {
    let PropertyValue::Borrowed(value) = value else {
        return Err(PropertyTypeMismatch {
            expected: optional.element_type_id(),
            actual: match value {
                PropertyValue::Borrowed(value) | PropertyValue::OptionalBorrowed(Some(value)) => {
                    value.clone().type_id()
                }
                PropertyValue::OptionalBorrowed(None) => std::any::TypeId::of::<()>(),
                PropertyValue::Owned(value) => value.type_id(),
                PropertyValue::BorrowedSlice(_) => std::any::TypeId::of::<()>(),
            },
        });
    };
    optional.project_ref(value.clone())
}

/// Converts an owned reflected value into a validator input abstraction.
///
/// Owned strings become text inputs; other values remain typed borrows from
/// the owner and are not cloned.
///
/// # Parameters
///
/// * `value` - Owned reflected property value converted for a validator.
fn owned_value<'a>(value: &'a PropertyOwned) -> Result<ValidationValue<'a>, ExecutionError> {
    if let Some(text) = value.downcast_ref::<String>() {
        return Ok(ValidationValue::Text(text.as_str()));
    }
    Ok(ValidationValue::Typed(value.as_any()))
}

/// Matches the complete field-name sequence, ignoring collection indices.
///
/// # Parameters
///
/// * `field` - Requested field path containing field names only.
/// * `path` - Compiled rule path, which may also contain collection indices.
///
/// # Returns
///
/// `true` when the field names match after collection indices are ignored.
#[must_use]
pub(super) fn field_matches(field: &FieldPath, path: &ValidationPath) -> bool {
    field
        .segments()
        .iter()
        .map(String::as_str)
        .eq(path.as_segments().iter().filter_map(|segment| match segment {
            PathSegment::Field(name) => Some(*name),
            _ => None,
        }))
}

#[cfg(test)]
mod any_view_tests {
    use qubit_reflect::Reflect;
    use qubit_reflect::value::DynamicOwned;
    use qubit_reflect::value::DynamicRef;
    use qubit_reflect::value::Local;
    use qubit_validator::ExecutionErrorKind;

    use super::owned_value;
    use super::reflected_value;

    #[Reflect]
    struct NonTextValue;

    #[test]
    fn test_reflected_value_rejects_dynamic_view_without_any() {
        let value = NonTextValue;
        let dynamic = DynamicRef::<Local>::new(&value);

        let error = reflected_value(&dynamic).expect_err("the view has no concrete Any reference");

        assert_eq!(error.kind(), ExecutionErrorKind::AdapterContractViolation);
    }

    #[test]
    fn test_reflected_value_preserves_any_view_when_available() {
        let value = NonTextValue;
        let dynamic = DynamicRef::<Local>::new_any(&value);

        let qubit_validator::ValidationValue::Typed(actual) =
            reflected_value(&dynamic).expect("new_any retains the concrete value view")
        else {
            panic!("non-text values remain typed inputs");
        };
        assert!(actual.is::<NonTextValue>());
    }

    #[test]
    fn test_owned_value_preserves_non_text_typed_input() {
        let value = DynamicOwned::<Local>::new(NonTextValue);

        let input = owned_value(&value).expect("owned reflected values expose their concrete Any view");

        let qubit_validator::ValidationValue::Typed(actual) = input else {
            panic!("non-text owned values remain typed inputs");
        };
        assert!(actual.is::<NonTextValue>());
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::sync::Arc;

    use qubit_reflect::value::DynamicRef;
    use qubit_reflect::value::Local;
    use qubit_validator::BoundValidationContext;
    use qubit_validator::BoundValidator;
    use qubit_validator::ExecutionError;
    use qubit_validator::ExecutionErrorKind;
    use qubit_validator::InputType;
    use qubit_validator::PreparedOutcome;
    use qubit_validator::PreparedValidator;
    use qubit_validator::ValidationPath;
    use qubit_validator::ValidationValue;
    use qubit_validator::ValidatorId;

    use super::super::internal::selection_mask::SelectionMask;
    use super::execute_selected_members_with_validator;
    use super::execute_unique;
    use super::field_matches;
    use super::optional_scalar_value;
    use super::prefix_error;
    use super::property_value;
    use super::unmatched_selection;
    use crate::metadata::BorrowedCollection;
    use crate::metadata::BorrowedPropertySlice;
    use crate::metadata::PropertyOwned;
    use crate::metadata::PropertyValue;
    use crate::metadata::SelectorPosition;
    use crate::property::CollectionItemsAdapter;
    use crate::property::ItemEqAdapter;
    use crate::property::OptionalOps;
    use crate::validation::ValidationOptions;
    use crate::validation::internal::execution_budget::ExecutionBudget;
    use crate::validation::internal::report_accumulator::ReportAccumulator;

    thread_local! {
        static UNKNOWN_NEXT_CALLS: Cell<usize> = const { Cell::new(0) };
        static SELECTOR_RULE_CALLS: Cell<usize> = const { Cell::new(0) };
    }

    struct UnknownLen<I>(I);
    impl<I: Iterator> Iterator for UnknownLen<I> {
        type Item = I::Item;
        fn next(&mut self) -> Option<Self::Item> {
            UNKNOWN_NEXT_CALLS.with(|calls| calls.set(calls.get() + 1));
            self.0.next()
        }
        fn size_hint(&self) -> (usize, Option<usize>) {
            (0, None)
        }
    }

    fn unknown_len_items<'a>(
        value: &PropertyValue<'a>,
        _position: SelectorPosition,
    ) -> Result<Option<BorrowedCollection<'a>>, crate::metadata::PropertyAccessError> {
        let PropertyValue::Borrowed(value) = value else {
            return Err(crate::metadata::PropertyAccessError::NotReadable);
        };
        let values = value
            .clone()
            .downcast::<Vec<u32>>()
            .map_err(|_| crate::metadata::PropertyAccessError::NotReadable)?;
        Ok(Some(BorrowedCollection::new(UnknownLen(
            values.iter().map(DynamicRef::<Local>::new_any),
        ))))
    }

    fn exact_len_items<'a>(
        value: &PropertyValue<'a>,
        _position: SelectorPosition,
    ) -> Result<Option<BorrowedCollection<'a>>, crate::metadata::PropertyAccessError> {
        let PropertyValue::Borrowed(value) = value else {
            return Err(crate::metadata::PropertyAccessError::NotReadable);
        };
        let values = value
            .clone()
            .downcast::<Vec<u32>>()
            .map_err(|_| crate::metadata::PropertyAccessError::NotReadable)?;
        Ok(Some(BorrowedCollection::new(
            values.iter().map(DynamicRef::<Local>::new_any),
        )))
    }

    struct CountAndFailLast;
    impl PreparedValidator for CountAndFailLast {
        fn input_type(&self) -> InputType {
            InputType::of::<u32>()
        }
        fn dependency_specs(&self) -> &'static [qubit_validator::DependencySpec] {
            &[]
        }
        fn validate(
            &self,
            value: ValidationValue<'_>,
            _context: &BoundValidationContext<'_>,
        ) -> Result<PreparedOutcome, ExecutionError> {
            SELECTOR_RULE_CALLS.with(|calls| calls.set(calls.get() + 1));
            let ValidationValue::Typed(value) = value else {
                panic!("typed input expected")
            };
            if value.downcast_ref::<u32>() == Some(&2) {
                Ok(PreparedOutcome::Invalid(vec![qubit_validator::ViolationDraft::new(
                    qubit_validator::ViolationCode::new("test.last"),
                )]))
            } else {
                Ok(PreparedOutcome::valid())
            }
        }
    }

    fn run_unknown_matrix(
        source: &[u32],
        adapter: CollectionItemsAdapter,
        options: &ValidationOptions,
    ) -> Result<qubit_validator::ValidationReport, Box<(ExecutionError, qubit_validator::ValidationReport)>> {
        UNKNOWN_NEXT_CALLS.with(|calls| calls.set(0));
        SELECTOR_RULE_CALLS.with(|calls| calls.set(0));
        let validator =
            BoundValidator::try_from_prepared::<u32>(ValidatorId::new("test.selector"), Arc::new(CountAndFailLast))
                .expect("validator input matches selector member type");
        let collection = source.to_vec();
        let value = PropertyValue::Borrowed(DynamicRef::<Local>::new(&collection));
        let path = ValidationPath::root().with_field("values");
        let mut budget = ExecutionBudget::new(options);
        let mut report = ReportAccumulator::new(options);
        if let Err(error) = execute_selected_members_with_validator(
            0,
            value,
            &path,
            SelectorPosition::Element,
            adapter,
            &validator,
            &mut budget,
            &mut report,
            false,
        ) {
            return Err(Box::new((error, report.into_report())));
        }
        Ok(report.into_report())
    }

    fn unused_item_comparison(
        left: crate::metadata::PropertyBorrowed<'_>,
        right: crate::metadata::PropertyBorrowed<'_>,
    ) -> Result<bool, crate::metadata::PropertyAccessError> {
        let left = left.downcast_ref::<u32>();
        let right = right.downcast_ref::<u32>();
        Ok(left.zip(right).is_some_and(|(left, right)| left == right))
    }

    fn failing_item_comparison(
        _left: crate::metadata::PropertyBorrowed<'_>,
        _right: crate::metadata::PropertyBorrowed<'_>,
    ) -> Result<bool, crate::metadata::PropertyAccessError> {
        Err(crate::metadata::PropertyAccessError::NotReadable)
    }
    use crate::validation::validation_options::FieldPath;
    use crate::validation::validation_options::ValidationSelection;

    #[test]
    fn test_field_matching_ignores_indices_without_allocating_a_field_list() {
        assert!(field_matches(
            &FieldPath::from_segments(["items", "name"]),
            &ValidationPath::root()
                .with_field("items")
                .with_index(2)
                .with_field("name"),
        ));
        assert!(!field_matches(
            &FieldPath::from_segments(["items", "name"]),
            &ValidationPath::root()
                .with_field("items")
                .with_index(2)
                .with_field("other"),
        ));
    }

    #[test]
    fn test_selection_mask_tracks_all_and_explicit_root_selection() {
        let mut all = SelectionMask::new(&ValidationSelection::All, 1, &[]);
        assert!(all.selected(0));
        assert!(!all.later_selected(0));
        assert!(all.take_field_path(0).is_none());

        let mut fields = SelectionMask::new(
            &ValidationSelection::Fields(vec![FieldPath::from_segments([] as [&str; 0])]),
            1,
            &[],
        );
        assert!(fields.selected(0));
        assert!(!fields.later_selected(0));
        assert!(fields.take_field_path(0).is_none());
    }

    #[test]
    fn test_unmatched_selection_requires_a_bound_field_or_model_rule() {
        assert!(unmatched_selection(&ValidationSelection::All, 0, &[]).is_none());

        let empty = ValidationSelection::Fields(Vec::new());
        assert!(
            unmatched_selection(&empty, 0, &[])
                .expect("an empty field selection has no rule")
                .segments()
                .is_empty()
        );

        let root = ValidationSelection::Fields(vec![FieldPath::from_segments([] as [&str; 0])]);
        assert!(unmatched_selection(&root, 1, &[]).is_none());
        assert!(unmatched_selection(&root, 0, &[]).is_some());
    }

    #[test]
    fn test_property_value_treats_absent_optional_and_slices_as_missing() {
        assert!(matches!(
            property_value(&PropertyValue::OptionalBorrowed(None))
                .expect("an absent optional is a missing validator input"),
            qubit_validator::ValidationValue::Missing
        ));

        let values = ["first".to_owned(), "second".to_owned()];
        let slice = BorrowedPropertySlice::new(&values);
        assert!(matches!(
            property_value(&PropertyValue::BorrowedSlice(slice))
                .expect("a collection slice is not a scalar validator input"),
            qubit_validator::ValidationValue::Missing
        ));
    }

    #[test]
    fn test_optional_scalar_projection_preserves_presence_and_borrow() {
        let present = Some(String::from("present"));
        let projected = optional_scalar_value(
            &PropertyValue::Borrowed(DynamicRef::<Local>::new(&present)),
            OptionalOps::for_element::<String>(),
        )
        .expect("matching Option<String> adapter projects successfully")
        .expect("present optional projects a borrow");
        assert_eq!(projected.downcast_ref::<String>().map(String::as_str), Some("present"));

        let absent: Option<String> = None;
        assert!(
            optional_scalar_value(
                &PropertyValue::Borrowed(DynamicRef::<Local>::new(&absent)),
                OptionalOps::for_element::<String>(),
            )
            .expect("matching absent optional is not a type error")
            .is_none()
        );
    }

    #[test]
    fn test_prefix_error_composes_field_index_and_map_segments() {
        let relative = ValidationPath::root()
            .with_field("item")
            .with_index(2)
            .with_map_entry(1)
            .with_map_value();
        let error = ExecutionError::new(ExecutionErrorKind::PropertyReadFailed).with_path(relative);
        let prefix = ValidationPath::root().with_field("records");

        let error = prefix_error(error, &prefix);

        assert_eq!(
            error.path(),
            &ValidationPath::root()
                .with_field("records")
                .with_field("item")
                .with_index(2)
                .with_map_entry(1)
                .with_map_value()
        );
    }

    #[test]
    fn test_unique_execution_accepts_an_empty_borrowed_sequence() {
        let values: [u32; 0] = [];
        let options = ValidationOptions::default();
        let mut budget = ExecutionBudget::new(&options);
        let mut report = ReportAccumulator::new(&options);
        let path = ValidationPath::root().with_field("values");

        execute_unique(
            ValidatorId::new("test.unique"),
            0,
            PropertyValue::BorrowedSlice(BorrowedPropertySlice::new(&values)),
            &path,
            unused_item_comparison as ItemEqAdapter,
            &mut budget,
            &mut report,
            false,
        )
        .expect("an empty sequence has no duplicate pair");

        assert!(report.into_report().is_valid());
    }

    #[test]
    fn test_unique_execution_rejects_non_slice_property_values_with_path() {
        let options = ValidationOptions::default();
        let mut budget = ExecutionBudget::new(&options);
        let mut report = ReportAccumulator::new(&options);
        let path = ValidationPath::root().with_field("values");

        let error = execute_unique(
            ValidatorId::new("test.unique"),
            0,
            PropertyValue::Owned(PropertyOwned::new(42_u32)),
            &path,
            unused_item_comparison as ItemEqAdapter,
            &mut budget,
            &mut report,
            false,
        )
        .expect_err("a scalar getter cannot satisfy a sequence uniqueness rule");

        assert_eq!(error.kind(), ExecutionErrorKind::PropertyReadFailed);
        assert_eq!(error.path(), &path);
    }

    #[test]
    fn test_unique_execution_reports_duplicate_indices() {
        let values = [3_u32, 5, 3];
        let options = ValidationOptions::default();
        let mut budget = ExecutionBudget::new(&options);
        let mut report = ReportAccumulator::new(&options);
        let path = ValidationPath::root().with_field("values");

        execute_unique(
            ValidatorId::new("test.unique"),
            0,
            PropertyValue::BorrowedSlice(BorrowedPropertySlice::new(&values)),
            &path,
            unused_item_comparison as ItemEqAdapter,
            &mut budget,
            &mut report,
            false,
        )
        .expect("duplicate values are reported as a validation outcome");

        let report = report.into_report();
        assert!(!report.is_valid());
        assert_eq!(report.violations().len(), 1);
        let violation = &report.violations()[0];
        assert_eq!(violation.code().as_str(), "collection.duplicate_item");
        assert_eq!(
            violation.path(),
            &ValidationPath::root().with_field("values").with_index(2)
        );
    }

    #[test]
    fn test_unique_execution_accepts_distinct_values() {
        let values = [3_u32, 5, 8];
        let options = ValidationOptions::default();
        let mut budget = ExecutionBudget::new(&options);
        let mut report = ReportAccumulator::new(&options);
        let path = ValidationPath::root().with_field("values");

        execute_unique(
            ValidatorId::new("test.unique"),
            0,
            PropertyValue::BorrowedSlice(BorrowedPropertySlice::new(&values)),
            &path,
            unused_item_comparison as ItemEqAdapter,
            &mut budget,
            &mut report,
            false,
        )
        .expect("distinct values finish the uniqueness rule");

        assert!(report.into_report().is_valid());
    }

    #[test]
    fn test_unique_execution_preserves_comparison_failure_path() {
        let values = [3_u32, 5];
        let options = ValidationOptions::default();
        let mut budget = ExecutionBudget::new(&options);
        let mut report = ReportAccumulator::new(&options);
        let path = ValidationPath::root().with_field("values");

        let error = execute_unique(
            ValidatorId::new("test.unique"),
            0,
            PropertyValue::BorrowedSlice(BorrowedPropertySlice::new(&values)),
            &path,
            failing_item_comparison as ItemEqAdapter,
            &mut budget,
            &mut report,
            false,
        )
        .expect_err("a failing item adapter stops uniqueness comparison");

        assert_eq!(error.kind(), ExecutionErrorKind::PropertyReadFailed);
        assert_eq!(error.path(), &path.with_index(1));
    }

    #[test]
    fn test_unique_execution_reports_comparison_budget_exhaustion() {
        let values = [3_u32, 5, 8];
        let options = ValidationOptions::builder()
            .max_comparisons(std::num::NonZeroUsize::new(1).expect("positive comparison limit"))
            .build();
        let mut budget = ExecutionBudget::new(&options);
        let mut report = ReportAccumulator::new(&options);
        let path = ValidationPath::root().with_field("values");

        let error = execute_unique(
            ValidatorId::new("test.unique"),
            0,
            PropertyValue::BorrowedSlice(BorrowedPropertySlice::new(&values)),
            &path,
            unused_item_comparison as ItemEqAdapter,
            &mut budget,
            &mut report,
            false,
        )
        .expect_err("the second comparison exceeds the configured budget");

        assert_eq!(error.kind(), ExecutionErrorKind::TraversalLimit);
        assert_eq!(error.path(), &path.with_index(2));
    }

    fn matrix_options(nodes: usize, comparisons: usize) -> ValidationOptions {
        ValidationOptions::builder()
            .max_nodes(std::num::NonZeroUsize::new(nodes).unwrap())
            .max_comparisons(std::num::NonZeroUsize::new(comparisons).unwrap())
            .build()
    }

    #[test]
    fn test_unknown_length_selector_budget_boundary_matrix() {
        let report = run_unknown_matrix(&[], unknown_len_items, &matrix_options(3, 1))
            .expect("empty unknown iterator exhaustion probe fits");
        assert!(report.is_valid());
        assert!(!report.is_truncated());
        assert_eq!(UNKNOWN_NEXT_CALLS.with(Cell::get), 1);
        assert_eq!(SELECTOR_RULE_CALLS.with(Cell::get), 0);
        let (error, partial) = *run_unknown_matrix(&[], unknown_len_items, &matrix_options(2, 1))
            .expect_err("empty unknown iterator cannot reserve an exhaustion probe");
        assert_eq!(error.kind(), ExecutionErrorKind::TraversalLimit);
        assert!(partial.is_valid());
        assert!(!partial.is_truncated());
        assert_eq!(UNKNOWN_NEXT_CALLS.with(Cell::get), 0);
        assert_eq!(SELECTOR_RULE_CALLS.with(Cell::get), 0);
        let depth_limited = ValidationOptions::builder()
            .max_depth(std::num::NonZeroUsize::new(1).unwrap())
            .build();
        let (error, partial) = *run_unknown_matrix(&[1], unknown_len_items, &depth_limited)
            .expect_err("member depth is checked before the iterator is probed");
        assert_eq!(error.kind(), ExecutionErrorKind::TraversalLimit);
        assert!(partial.is_valid());
        assert_eq!(UNKNOWN_NEXT_CALLS.with(Cell::get), 0);
        assert_eq!(SELECTOR_RULE_CALLS.with(Cell::get), 0);

        let (error, partial) = *run_unknown_matrix(&[1], unknown_len_items, &matrix_options(3, 2))
            .expect_err("one item fits, but the required completion probe has no node budget");
        assert_eq!(error.kind(), ExecutionErrorKind::TraversalLimit);
        assert!(partial.is_valid());
        assert!(!partial.is_truncated());
        assert_eq!(UNKNOWN_NEXT_CALLS.with(Cell::get), 1);
        assert_eq!(SELECTOR_RULE_CALLS.with(Cell::get), 1);
        let report = run_unknown_matrix(&[1], exact_len_items, &matrix_options(3, 1))
            .expect("exact size avoids the completion probe");
        assert!(report.is_valid());
        assert_eq!(SELECTOR_RULE_CALLS.with(Cell::get), 1);

        for options in [matrix_options(3, 2), matrix_options(10, 1)] {
            let (error, partial) = *run_unknown_matrix(&[1, 2], unknown_len_items, &options)
                .expect_err("second possible member exceeds node or comparison budget");
            assert_eq!(error.kind(), ExecutionErrorKind::TraversalLimit);
            assert!(partial.is_valid());
            assert!(!partial.is_truncated());
            assert_eq!(UNKNOWN_NEXT_CALLS.with(Cell::get), 1);
            assert_eq!(SELECTOR_RULE_CALLS.with(Cell::get), 1);
        }

        let (error, partial) = *run_unknown_matrix(&[1, 2], unknown_len_items, &matrix_options(5, 2))
            .expect_err("two member reservations leave no budget to prove exhaustion");
        assert_eq!(error.kind(), ExecutionErrorKind::TraversalLimit);
        assert_eq!(partial.violations().len(), 1);
        assert!(!partial.is_truncated());
        assert_eq!(UNKNOWN_NEXT_CALLS.with(Cell::get), 2);
        assert_eq!(SELECTOR_RULE_CALLS.with(Cell::get), 2);
        let report = run_unknown_matrix(&[1, 2], exact_len_items, &matrix_options(5, 2))
            .expect("exact-size iterator completes at the same member budget");
        assert_eq!(report.violations().len(), 1);
        assert!(!report.is_truncated());
        assert_eq!(SELECTOR_RULE_CALLS.with(Cell::get), 2);

        let options = ValidationOptions::builder()
            .mode(crate::validation::ValidationMode::FailFast)
            .max_nodes(std::num::NonZeroUsize::new(20).unwrap())
            .max_comparisons(std::num::NonZeroUsize::new(10).unwrap())
            .build();
        let report = run_unknown_matrix(&[1, 2], unknown_len_items, &options)
            .expect("FailFast returns a partial report rather than an error");
        assert_eq!(UNKNOWN_NEXT_CALLS.with(Cell::get), 2);
        assert_eq!(SELECTOR_RULE_CALLS.with(Cell::get), 2);
        assert_eq!(report.violations().len(), 1);
        assert!(report.is_truncated());
        let report = run_unknown_matrix(&[1, 2], exact_len_items, &options)
            .expect("exact size identifies the failed item as the last member");
        assert_eq!(SELECTOR_RULE_CALLS.with(Cell::get), 2);
        assert_eq!(report.violations().len(), 1);
        assert!(!report.is_truncated());
    }
}
