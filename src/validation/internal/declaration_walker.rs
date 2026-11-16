// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Complete declaration discovery over a finite concrete type graph.

use std::any::TypeId;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::ControlFlow;

use qubit_reflect::ReflectionContext;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::descriptor::GenericExpressionArgumentKind;
use qubit_reflect::descriptor::TypeExpression;
use qubit_reflect::descriptor::TypeExpressionKind;

use super::build_budget::BuildBudget;
use super::build_budget::Counter;
use super::execution_declaration::ExecutionDeclaration;
use super::validation_occurrence::ValidationOccurrence;
use crate::metadata::AllowedChars;
use crate::metadata::ConstraintMetadata;
use crate::metadata::EnumVariantMetadata;
use crate::metadata::FieldMetadata;
use crate::metadata::SelectorMetadata;
use crate::metadata::SelectorPosition;
use crate::metadata::TypeMetadata;
use crate::resolve::ModelGraph;
use crate::structure::StructuralEdgeKind;
use crate::structure::children::try_children;
use crate::validation::ValidationBuildError;
use crate::validation::ValidationBuildLimit;
use crate::validation::ValidationBuildLimitKind;

/// Collects supported occurrences and all structural execution failures.
///
/// # Parameters
///
/// * `root` - Metadata for the requested root model.
/// * `graph` - The resolved model graph used to find reachable declarations.
///
/// # Returns
///
/// Returns supported validation occurrences in traversal order together with
/// structural build errors. If the root is absent from `graph`, the occurrence
/// list is empty and the error list contains a missing-root error.
///
/// # Panics
///
/// Panics if the resolved graph no longer retains its reflection context.
/// Public `ModelRegistry` constructors retain their reflection context; this
/// condition indicates an internally constructed graph with an absent context.
#[must_use]
pub(crate) fn collect_with_budget(
    root: &'static TypeMetadata,
    graph: &ModelGraph<'_>,
    mut budget: BuildBudget,
) -> (
    Vec<ValidationOccurrence>,
    Vec<ValidationBuildError>,
    BuildBudget,
) {
    let context = graph
        .registry()
        .context()
        .expect("resolved validation graphs retain their reflection context");
    let Some(root) = graph.model(root.type_id()) else {
        return (
            Vec::new(),
            vec![ValidationBuildError::missing_root(root)],
            budget,
        );
    };
    let work = match reachable_work(root, graph, context, &mut budget) {
        Ok(work) => work,
        Err(detail) => {
            return (
                Vec::new(),
                vec![ValidationBuildError::at_build_limit(root, detail)],
                budget,
            );
        }
    };
    let mut occurrences = Vec::new();
    let mut errors = Vec::new();
    let mut ordinal = 0;
    if let Err(detail) = walk(
        root,
        root,
        &[],
        None,
        context,
        graph,
        &work,
        &mut HashSet::new(),
        false,
        &mut ordinal,
        &mut occurrences,
        &mut errors,
        &mut budget,
        0,
    ) {
        occurrences.clear();
        errors.push(ValidationBuildError::at_build_limit(root, detail));
    }
    (occurrences, errors, budget)
}

/// Computes executable-work reachability from the requested root by monotone
/// propagation, including cycles.
///
/// Structural discovery scans only models reachable from `root`. When a field
/// needs expression fallback, name matching still examines the graph to retain
/// its existing unique-name semantics.
///
/// # Parameters
///
/// * `root` - The canonical root model whose structural closure is examined.
/// * `graph` - The resolved graph used to find nested models and preserve
///   expression-fallback name resolution.
/// * `context` - The reflection context used to resolve concrete field
///   descriptors.
///
/// # Returns
///
/// Returns the set of model types whose reachable declarations require
/// executable work.
fn reachable_work(
    root: &'static TypeMetadata,
    graph: &ModelGraph<'_>,
    context: &ReflectionContext,
    budget: &mut BuildBudget,
) -> Result<HashSet<TypeId>, ValidationBuildLimit> {
    let mut work = HashSet::new();
    let mut predecessors: HashMap<TypeId, Vec<TypeId>> = HashMap::new();
    let mut discovered = HashSet::from([root.type_id()]);
    let mut pending = vec![root];
    while let Some(model) = pending.pop() {
        budget.charge(Counter::Work)?;
        for field in fields(model) {
            budget.charge(Counter::Work)?;
            if has_declarations(field, budget)? {
                work.insert(model.type_id());
            }
            if field.is_opaque() || field.reference().is_some() {
                continue;
            }
            for (target, _, _) in nested_models(
                model
                    .field_descriptor_in(field, context)
                    .and_then(|field| field.find_concrete_type()),
                field.type_expression_in(context),
                model,
                graph,
                budget,
            )? {
                budget.charge(Counter::Work)?;
                predecessors
                    .entry(target.type_id())
                    .or_default()
                    .push(model.type_id());
                if discovered.insert(target.type_id()) {
                    pending.push(target);
                }
            }
        }
    }
    let mut pending: Vec<_> = work.iter().copied().collect();
    while let Some(id) = pending.pop() {
        budget.charge(Counter::Work)?;
        for predecessor in predecessors.get(&id).into_iter().flatten() {
            budget.charge(Counter::Work)?;
            if work.insert(*predecessor) {
                pending.push(*predecessor);
            }
        }
    }
    Ok(work)
}

/// Visits usage paths in source order; only the active path deduplicates types.
///
/// # Parameters
///
/// * `root` - The requested root model retained on each discovered occurrence.
/// * `owner` - The model whose fields are visited in this step.
/// * `prefix` - The access path accumulated from the root to `owner`.
/// * `diagnostic_prefix` - A rendered ancestor path for unsupported
///   diagnostics, when available.
/// * `context` - The reflection context used to resolve field descriptors.
/// * `graph` - The resolved model graph used to discover nested model types.
/// * `work` - Model types known to reach declarations that require execution.
/// * `active` - Model types on the current traversal path, used to stop cycles.
/// * `inherited_unsupported` - Whether an ancestor already makes descendants
///   unsupported.
/// * `ordinal` - The next source-order index assigned to a declaration
///   occurrence.
/// * `occurrences` - The output vector receiving supported declaration
///   occurrences.
/// * `errors` - The output vector receiving unsupported structural occurrences.
#[allow(clippy::too_many_arguments)]
fn walk(
    root: &'static TypeMetadata,
    owner: &'static TypeMetadata,
    prefix: &[&'static str],
    diagnostic_prefix: Option<&str>,
    context: &ReflectionContext,
    graph: &ModelGraph<'_>,
    work: &HashSet<TypeId>,
    active: &mut HashSet<TypeId>,
    inherited_unsupported: bool,
    ordinal: &mut usize,
    occurrences: &mut Vec<ValidationOccurrence>,
    errors: &mut Vec<ValidationBuildError>,
    budget: &mut BuildBudget,
    depth: usize,
) -> Result<(), ValidationBuildLimit> {
    budget.enter(depth)?;
    active.insert(owner.type_id());
    for field in fields(owner) {
        budget.charge(Counter::Work)?;
        let enum_variant = enum_variant_for_field(owner, field);
        let mut segments = prefix.to_vec();
        let transparent_unnamed = is_transparent_unnamed_field(owner, field);
        segments.push(field.name().unwrap_or(if transparent_unnamed {
            "0"
        } else {
            "<unnamed>"
        }));
        let unsupported = inherited_unsupported
            || (owner.as_enum().is_some()
                && (!enum_variant.is_some_and(|variant| {
                    owner
                        .descriptor_in(context)
                        .variants()
                        .iter()
                        .any(|actual| actual.index() == variant.index())
                }) || owner.field_descriptor_in(field, context).is_none()))
            || (field.name().is_none() && !transparent_unnamed && enum_variant.is_none());
        let unsupported_path = (unsupported || enum_variant.is_some()).then(|| {
            let mut path = diagnostic_prefix.map_or_else(|| prefix.join("."), str::to_owned);
            if let Some(variant) = enum_variant {
                append_segment(&mut path, variant.rust_name());
            }
            append_segment(
                &mut path,
                &field
                    .name()
                    .map_or_else(|| field.index().to_string(), str::to_owned),
            );
            path
        });
        collect_field_declarations(
            root,
            owner,
            field,
            enum_variant,
            &segments,
            unsupported_path.as_deref(),
            unsupported,
            ordinal,
            occurrences,
            errors,
            budget,
        )?;
        if field.is_opaque() || field.reference().is_some() {
            continue;
        }
        walk_nested_fields(
            root,
            owner,
            field,
            enum_variant,
            prefix,
            diagnostic_prefix,
            &segments,
            unsupported_path.as_deref(),
            unsupported,
            context,
            graph,
            work,
            active,
            ordinal,
            occurrences,
            errors,
            budget,
            depth,
        )?;
    }
    active.remove(&owner.type_id());
    Ok(())
}

/// Descends through a field's executable nested models in structural order.
///
/// # Parameters
///
/// * `root` - The requested root model retained on descendant occurrences.
/// * `owner` - The model that owns the field being traversed.
/// * `field` - The field whose structural wrappers may contain nested models.
/// * `enum_variant` - The enum variant containing `field`, if the owner is an
///   enum.
/// * `prefix` - The access path to `owner` before this field.
/// * `diagnostic_prefix` - A rendered ancestor path for unsupported
///   diagnostics, when present.
/// * `segments` - The access path extended through the current field.
/// * `unsupported_path` - The rendered path already associated with this
///   unsupported field, if any.
/// * `unsupported` - Whether the current field or an ancestor has unsupported
///   structure.
/// * `context` - The reflection context used to resolve the field's concrete
///   descriptor.
/// * `graph` - The resolved model graph used to find reachable nested
///   declarations.
/// * `work` - Model types known to require executable validation work.
/// * `active` - Model types on the current traversal path, used to detect
///   cycles.
/// * `ordinal` - The next source-order index assigned to a declaration
///   occurrence.
/// * `occurrences` - The output vector receiving supported declaration
///   occurrences.
/// * `errors` - The output vector receiving unsupported structural occurrences.
#[allow(clippy::too_many_arguments)]
fn walk_nested_fields(
    root: &'static TypeMetadata,
    owner: &'static TypeMetadata,
    field: &'static FieldMetadata,
    enum_variant: Option<&'static EnumVariantMetadata>,
    prefix: &[&'static str],
    diagnostic_prefix: Option<&str>,
    segments: &[&'static str],
    unsupported_path: Option<&str>,
    unsupported: bool,
    context: &ReflectionContext,
    graph: &ModelGraph<'_>,
    work: &HashSet<TypeId>,
    active: &mut HashSet<TypeId>,
    ordinal: &mut usize,
    occurrences: &mut Vec<ValidationOccurrence>,
    errors: &mut Vec<ValidationBuildError>,
    budget: &mut BuildBudget,
    depth: usize,
) -> Result<(), ValidationBuildLimit> {
    let field_descriptor = owner
        .field_descriptor_in(field, context)
        .and_then(|field| field.find_concrete_type());
    for (target, suffix, access_segments) in nested_models(
        field_descriptor,
        field.type_expression_in(context),
        owner,
        graph,
        budget,
    )? {
        if !work.contains(&target.type_id()) {
            continue;
        }
        let direct = suffix.is_empty()
            && direct_model(field_descriptor, graph, budget)?
                .is_some_and(|model| model.type_id() == target.type_id());
        let mut nested_segments = segments.to_vec();
        if let Some(access_segments) = access_segments {
            nested_segments.extend(access_segments);
        }
        let nested_path = (unsupported || owner.as_enum().is_some() || !direct).then(|| {
            let mut path = unsupported_path.map_or_else(
                || {
                    let mut path =
                        diagnostic_prefix.map_or_else(|| prefix.join("."), str::to_owned);
                    if let Some(variant) = enum_variant {
                        append_segment(&mut path, variant.rust_name());
                    }
                    append_segment(
                        &mut path,
                        &field
                            .name()
                            .map_or_else(|| field.index().to_string(), str::to_owned),
                    );
                    path
                },
                str::to_owned,
            );
            path.push_str(&suffix);
            path
        });
        if active.contains(&target.type_id()) {
            let occurrence = ValidationOccurrence {
                root,
                owner,
                field,
                enum_variant: None,
                segments: nested_segments.clone(),
                unsupported_path: nested_path,
                selector: None,
                ordinal: *ordinal,
                declaration: ExecutionDeclaration::Traversal,
            };
            *ordinal += 1;
            if errors.len() >= super::build_budget::MAX_DIAGNOSTICS {
                return Err(ValidationBuildLimit::new(
                    ValidationBuildLimitKind::Diagnostics,
                    super::build_budget::MAX_DIAGNOSTICS,
                ));
            }
            budget.charge(Counter::Diagnostic)?;
            errors.push(ValidationBuildError::unsupported(&occurrence));
            continue;
        }
        walk(
            root,
            target,
            &nested_segments,
            nested_path.as_deref(),
            context,
            graph,
            work,
            active,
            unsupported || owner.as_enum().is_some() || !direct,
            ordinal,
            occurrences,
            errors,
            budget,
            depth + 1,
        )?;
    }
    Ok(())
}

/// Records this field's declarations in their original metadata order.
///
/// # Parameters
///
/// * `root` - The requested root model retained on each recorded occurrence.
/// * `owner` - The model that owns the declaration-bearing field.
/// * `field` - The field whose constraints and validators are recorded.
/// * `enum_variant` - The enum variant containing the field, if applicable.
/// * `segments` - The access path from the root to the field.
/// * `unsupported_path` - The rendered unsupported path to attach to
///   occurrences, if present.
/// * `unsupported` - Whether the field must be reported as structurally
///   unsupported.
/// * `ordinal` - The next source-order index assigned to a declaration
///   occurrence.
/// * `occurrences` - The output vector receiving supported declaration
///   occurrences.
/// * `errors` - The output vector receiving unsupported structural occurrences.
#[allow(clippy::too_many_arguments)]
fn collect_field_declarations(
    root: &'static TypeMetadata,
    owner: &'static TypeMetadata,
    field: &'static FieldMetadata,
    enum_variant: Option<&'static EnumVariantMetadata>,
    segments: &[&'static str],
    unsupported_path: Option<&str>,
    unsupported: bool,
    ordinal: &mut usize,
    occurrences: &mut Vec<ValidationOccurrence>,
    errors: &mut Vec<ValidationBuildError>,
    budget: &mut BuildBudget,
) -> Result<(), ValidationBuildLimit> {
    for_each_declaration(field, budget, |budget, selector, declaration| {
        budget.charge(Counter::Work)?;
        budget.charge(Counter::Occurrence)?;
        let occurrence = ValidationOccurrence {
            root,
            owner,
            field,
            enum_variant,
            segments: segments.to_vec(),
            unsupported_path: unsupported_path.map(str::to_owned),
            selector,
            ordinal: *ordinal,
            declaration,
        };
        *ordinal += 1;
        if unsupported {
            if errors.len() >= super::build_budget::MAX_DIAGNOSTICS {
                return Err(ValidationBuildLimit::new(
                    ValidationBuildLimitKind::Diagnostics,
                    super::build_budget::MAX_DIAGNOSTICS,
                ));
            }
            budget.charge(Counter::Diagnostic)?;
            errors.push(ValidationBuildError::unsupported(&occurrence));
        } else {
            occurrences.push(occurrence);
        }
        Ok(())
    })
}

/// Resolves the concrete enum variant associated with a payload field.
///
/// # Parameters
///
/// * `owner` - The model metadata whose enum variants are searched.
/// * `field` - The payload field whose recorded variant index is resolved.
///
/// # Returns
///
/// Returns the variant containing `field`, or `None` when the field has no
/// variant location or `owner` is not an enum.
fn enum_variant_for_field(
    owner: &'static TypeMetadata,
    field: &'static FieldMetadata,
) -> Option<&'static EnumVariantMetadata> {
    let variant_index = field.location()?.variant()?;
    owner
        .as_enum()?
        .variants()
        .iter()
        .find(|variant| variant.index() == variant_index)
}

/// Accepts only the sole anonymous field declared by a transparent value.
///
/// `owner` supplies the role declaration and `field` is the visited field.
/// Returns `true` only for the exact anonymous field at index zero. Other
/// tuple fields remain unsupported.
///
/// # Parameters
///
/// * `owner` - The value metadata that may declare a transparent field.
/// * `field` - The candidate field checked against that declaration.
///
/// # Returns
///
/// Returns whether `field` is the transparent value's sole anonymous field.
#[must_use]
pub(crate) fn is_transparent_unnamed_field(owner: &TypeMetadata, field: &FieldMetadata) -> bool {
    owner
        .as_value()
        .and_then(|value| value.transparent_field())
        .is_some_and(|inner| {
            std::ptr::eq(inner, field) && field.index() == 0 && field.name().is_none()
        })
}

/// Returns struct fields or enum payload fields in declaration order.
///
///
/// # Parameters
///
/// * `model` - The metadata whose struct fields or enum payload fields are
///   returned.
///
/// # Returns
///
/// Returns all fields of `model` in declaration order.
fn fields(model: &'static TypeMetadata) -> Box<dyn Iterator<Item = &'static FieldMetadata>> {
    match model.as_enum() {
        Some(value) => Box::new(value.variants().iter().flat_map(|variant| variant.fields())),
        None => Box::new(model.fields().iter()),
    }
}

/// Enumerates actual executable declarations without mistaking empty selectors
/// for work.
///
///
/// # Parameters
///
/// * `field` - The metadata whose direct and selector declarations are
///   enumerated.
///
/// # Returns
///
/// Returns each executable constraint and validator with its optional selector
/// position, preserving metadata order.
/// Visits executable declarations on a field without collecting them first.
///
/// # Parameters
///
/// - `field`: Field whose declarations are enumerated in metadata order.
/// - `budget`: Shared build budget charged as declarations are inspected.
/// - `visit`: Callback receiving each declaration and its optional selector.
///
/// # Returns
///
/// Returns `Ok(())` after every declaration has been visited.
///
/// # Errors
///
/// Returns `Err(())` when the callback or a budget check stops traversal.
fn for_each_declaration(
    field: &'static FieldMetadata,
    budget: &mut BuildBudget,
    mut visit: impl FnMut(
        &mut BuildBudget,
        Option<SelectorPosition>,
        ExecutionDeclaration,
    ) -> Result<(), ValidationBuildLimit>,
) -> Result<(), ValidationBuildLimit> {
    for constraint in field.constraints() {
        budget.charge(Counter::Work)?;
        if constraint_has_work(constraint) {
            visit(budget, None, ExecutionDeclaration::Constraint(constraint))?;
        }
        for selector in selectors(constraint) {
            budget.charge(Counter::Work)?;
            for constraint in selector.constraints() {
                budget.charge(Counter::Work)?;
                if constraint_has_work(constraint) {
                    visit(
                        budget,
                        Some(selector.position()),
                        ExecutionDeclaration::Constraint(constraint),
                    )?;
                }
            }
            for value in selector.validators() {
                budget.charge(Counter::Work)?;
                visit(
                    budget,
                    Some(selector.position()),
                    ExecutionDeclaration::Validator(value),
                )?;
            }
        }
    }
    for value in field.validators() {
        budget.charge(Counter::Work)?;
        visit(budget, None, ExecutionDeclaration::Validator(value))?;
    }
    Ok(())
}

/// Checks whether a field contains executable declarations.
///
/// # Parameters
///
/// - `field`: Field whose constraints and validators are inspected.
/// - `budget`: Shared build budget charged for each inspected declaration.
///
/// # Returns
///
/// Returns `Ok(true)` when executable work is present, or `Ok(false)` when
/// the field contains no executable declaration.
///
/// # Errors
///
/// Returns `Err(())` when inspecting the field exceeds the work budget.
fn has_declarations(
    field: &'static FieldMetadata,
    budget: &mut BuildBudget,
) -> Result<bool, ValidationBuildLimit> {
    for constraint in field.constraints() {
        budget.charge(Counter::Work)?;
        if constraint_has_work(constraint) {
            return Ok(true);
        }
        for selector in selectors(constraint) {
            budget.charge(Counter::Work)?;
            for constraint in selector.constraints() {
                budget.charge(Counter::Work)?;
                if constraint_has_work(constraint) {
                    return Ok(true);
                }
            }
            if !selector.validators().is_empty() {
                budget.charge(Counter::Work)?;
                return Ok(true);
            }
        }
    }
    if !field.validators().is_empty() {
        budget.charge(Counter::Work)?;
        return Ok(true);
    }
    Ok(false)
}

/// Returns nested selector metadata in key-before-value declaration order.
///
///
/// # Parameters
///
/// * `constraint` - The constraint whose nested selector metadata is inspected.
///
/// # Returns
///
/// Returns selectors belonging to a sequence or map constraint; other
/// constraint kinds produce an empty vector.
fn selectors(constraint: &'static ConstraintMetadata) -> Vec<&'static SelectorMetadata> {
    match constraint {
        ConstraintMetadata::Sequence(value) => value.element().into_iter().collect(),
        ConstraintMetadata::Map(value) => value.key().into_iter().chain(value.value()).collect(),
        _ => Vec::new(),
    }
}

/// Reports whether the outer constraint requires an execution operation.
///
/// # Parameters
///
/// * `constraint` - The configured limits or checks that determine whether
///   execution work is needed.
#[must_use]
fn constraint_has_work(constraint: &ConstraintMetadata) -> bool {
    match constraint {
        ConstraintMetadata::Text(value) => {
            value.is_non_blank()
                || value.min_chars().is_some()
                || value.max_chars().is_some()
                || value.min_bytes().is_some()
                || value.max_bytes().is_some()
                || value.allowed_chars() != AllowedChars::Unicode
                || value.format().is_some()
        }
        ConstraintMetadata::Sequence(value) => {
            value.min_items().is_some() || value.max_items().is_some() || value.unique_items()
        }
        ConstraintMetadata::Map(value) => {
            value.min_entries().is_some() || value.max_entries().is_some()
        }
        ConstraintMetadata::Decimal(_) | ConstraintMetadata::Time(_) => true,
    }
}

/// Resolves a model through optional or smart-pointer wrappers only.
///
/// # Returns
///
/// Returns the first model found through the descriptor or its supported
/// wrappers, or `None` when no model is registered along that path.
///
/// # Parameters
///
/// * `descriptor` - The optional concrete type descriptor followed through
///   supported wrappers.
/// * `graph` - The resolved graph used to identify registered model types.
fn direct_model<'ctx>(
    mut descriptor: Option<TypeDescriptor<'ctx>>,
    graph: &ModelGraph<'_>,
    budget: &mut BuildBudget,
) -> Result<Option<&'static TypeMetadata>, ValidationBuildLimit> {
    let mut depth = 0;
    while let Some(current) = descriptor {
        budget.check_depth(depth)?;
        budget.charge(Counter::Work)?;
        if let Some(model) = graph.model(current.type_id()) {
            return Ok(Some(model));
        }
        let is_wrapper = matches!(
            current.kind(),
            qubit_reflect::descriptor::TypeKind::Optional(_)
                | qubit_reflect::descriptor::TypeKind::SmartPointer(_)
        );
        descriptor = is_wrapper
            .then(|| {
                current
                    .generic_arguments()
                    .get_at(0)?
                    .type_argument()
                    .ok()?
                    .find_concrete_type()
            })
            .flatten();
        depth += 1;
    }
    Ok(None)
}

/// Appends one field, variant or tuple index to a dot-separated diagnostic
/// path.
///
/// # Parameters
///
/// * `path` - The diagnostic path extended in place, with a separator when
///   nonempty.
/// * `segment` - The field, variant, or tuple index name appended to the path.
fn append_segment(path: &mut String, segment: &str) {
    if !path.is_empty() {
        path.push('.');
    }
    path.push_str(segment);
}

/// Renders `kind` as a diagnostic suffix from its retained structural facts.
///
/// `kind` supplies source positions rather than executable property steps.
/// Named fields use their retained query names to preserve aliases; enum
/// variants retain their source Rust names. Unnamed positions remain numeric
/// diagnostic suffixes. Transparent wrappers return an empty suffix, and
/// collection elements identify an element type without a runtime index.
///
///
/// # Parameters
///
/// * `kind` - The structural edge whose source-level path suffix is rendered.
///
/// # Returns
///
/// Returns the diagnostic suffix represented by `kind`.
fn structural_suffix(kind: StructuralEdgeKind) -> String {
    match kind {
        StructuralEdgeKind::OptionalElement | StructuralEdgeKind::PointerPointee => String::new(),
        StructuralEdgeKind::SequenceElement
        | StructuralEdgeKind::SetElement
        | StructuralEdgeKind::ArrayElement
        | StructuralEdgeKind::SliceElement => "[]".to_owned(),
        StructuralEdgeKind::MapKey => "[key]".to_owned(),
        StructuralEdgeKind::MapValue => "[value]".to_owned(),
        StructuralEdgeKind::TupleElement { index } => format!(".{index}"),
        StructuralEdgeKind::StructField {
            index,
            name,
            query_name,
        } => {
            let name = query_name
                .or(name)
                .map_or_else(|| index.to_string(), str::to_owned);
            format!(".{name}")
        }
        StructuralEdgeKind::EnumVariantField {
            variant_name,
            field_index,
            field_name,
            query_name,
            ..
        } => {
            let name = query_name
                .or(field_name)
                .map_or_else(|| field_index.to_string(), str::to_owned);
            format!(".{variant_name}.{name}")
        }
    }
}

/// One discovered nested model and its source access-path information.
type NestedModelUsage = (&'static TypeMetadata, String, Option<Vec<&'static str>>);

/// Finds every model usage behind structural wrappers without expanding model
/// bodies. Returned suffixes distinguish static uses of the same target type.
/// Only active ancestors suppress cycles; siblings are never deduplicated.
///
///
/// # Parameters
///
/// * `descriptor` - A concrete field type descriptor, if reflection resolved
///   one.
/// * `expression` - The source type expression used when the descriptor finds
///   no nested model.
/// * `owner` - The model that owns the field represented by the inputs.
/// * `graph` - The resolved model graph used to match discovered types to
///   metadata.
///
/// # Returns
///
/// Returns each reachable model with its diagnostic suffix and, when available,
/// its field access path.
fn nested_models<'ctx>(
    descriptor: Option<TypeDescriptor<'ctx>>,
    expression: Option<TypeExpression<'ctx>>,
    owner: &'static TypeMetadata,
    graph: &ModelGraph<'_>,
    budget: &mut BuildBudget,
) -> Result<Vec<NestedModelUsage>, ValidationBuildLimit> {
    // Enter/leave frames keep traversal finite without recursive Rust calls.
    let mut pending: Vec<_> = descriptor
        .into_iter()
        .map(|value| (value, String::new(), Some(Vec::new()), true, 0usize))
        .collect();
    let mut active = HashSet::new();
    let mut targets = Vec::new();
    if !pending.is_empty() {
        budget.charge(Counter::Work)?;
    }
    while let Some((current, path, access_segments, entering, depth)) = pending.pop() {
        budget.check_depth(depth)?;
        if !entering {
            active.remove(&current.type_id());
            continue;
        }
        if !active.insert(current.type_id()) {
            continue;
        }
        if let Some(model) = graph.model(current.type_id()) {
            targets.push((model, path, access_segments));
            active.remove(&current.type_id());
            continue;
        }
        pending.push((current, String::new(), None, false, depth));
        let first_child = pending.len();
        let child_result = try_children(current, |edge| -> ControlFlow<ValidationBuildLimit> {
            if let Err(detail) = budget.check_depth(depth.saturating_add(1)) {
                return ControlFlow::Break(detail);
            }
            if let Err(detail) = budget.charge(Counter::Work) {
                return ControlFlow::Break(detail);
            }
            let suffix = structural_suffix(edge.kind);
            let access_segments = match (access_segments.as_ref(), edge.kind) {
                (
                    Some(segments),
                    StructuralEdgeKind::StructField {
                        name, query_name, ..
                    },
                ) => query_name.or(name).map(|name| {
                    let mut next = segments.clone();
                    next.push(name);
                    next
                }),
                (
                    Some(segments),
                    StructuralEdgeKind::OptionalElement | StructuralEdgeKind::PointerPointee,
                ) => Some(segments.clone()),
                _ => None,
            };
            pending.push((
                edge.target,
                format!("{path}{suffix}"),
                access_segments,
                true,
                depth + 1,
            ));
            ControlFlow::Continue(())
        });
        if let ControlFlow::Break(detail) = child_result {
            return Err(detail);
        }
        pending[first_child..].reverse();
    }
    // Descriptor traversal is authoritative when it discovers a nested model:
    // its edges carry the exact source shape and suffix. Expression traversal
    // fills gaps only when this context cannot resolve any model child.
    // Running both unconditionally can mistake generic arguments inside an
    // already-resolved wrapper for additional paths and emit duplicate or
    // malformed declarations.
    if targets.is_empty()
        && let Some(expression) = expression
    {
        expression_models(expression, owner, graph, "", &mut targets, budget, 0)?;
    }
    Ok(targets)
}

/// Retains model usages found in source expressions when a context has no
/// registered concrete field descriptor for the owning type.
///
/// # Parameters
///
/// * `expression` - The type expression recursively searched for registered
///   model uses.
/// * `owner` - The model whose type expression is being interpreted, including
///   `Self` references.
/// * `graph` - The resolved model graph used to resolve matching type names.
/// * `suffix` - The diagnostic suffix accumulated from surrounding wrappers.
/// * `output` - The vector receiving each discovered model use and its access
///   information.
fn expression_models(
    expression: TypeExpression<'_>,
    owner: &'static TypeMetadata,
    graph: &ModelGraph<'_>,
    suffix: &str,
    output: &mut Vec<NestedModelUsage>,
    budget: &mut BuildBudget,
    depth: usize,
) -> Result<(), ValidationBuildLimit> {
    budget.check_depth(depth)?;
    budget.charge(Counter::Work)?;
    match expression.kind() {
        TypeExpressionKind::SelfType => {
            if let Some(model) = graph.model(owner.type_id()) {
                output.push((model, suffix.to_owned(), None));
            }
        }
        TypeExpressionKind::Named(named) => {
            let path = named.path().trim();
            let owner_name = owner.type_name();
            let owner_short_name = owner_name.rsplit("::").next().unwrap_or(owner_name);
            if (path == owner_name || path == owner_short_name)
                && let Some(model) = graph.model(owner.type_id())
            {
                output.push((model, suffix.to_owned(), None));
                return Ok(());
            }
            let mut matching_model = None;
            let mut match_count = 0usize;
            for model in graph.models() {
                budget.charge(Counter::Work)?;
                if model.type_name() == path {
                    matching_model = Some(model);
                    match_count += 1;
                    if match_count > 1 {
                        break;
                    }
                }
            }
            if match_count == 1
                && let Some(model) = matching_model
            {
                output.push((model, suffix.to_owned(), None));
                return Ok(());
            }
            let tail = path.rsplit("::").next().unwrap_or(path);
            for (position, argument) in named.arguments().iter().enumerate() {
                let GenericExpressionArgumentKind::Type(nested) = argument.kind() else {
                    continue;
                };
                let nested_suffix = match tail {
                    "Vec" | "VecDeque" | "HashSet" | "BTreeSet" | "BinaryHeap" => {
                        format!("{suffix}[]")
                    }
                    "HashMap" | "BTreeMap" => {
                        format!(
                            "{suffix}{}",
                            if position == 0 { "[key]" } else { "[value]" }
                        )
                    }
                    _ => suffix.to_owned(),
                };
                expression_models(
                    nested,
                    owner,
                    graph,
                    &nested_suffix,
                    output,
                    budget,
                    depth + 1,
                )?;
            }
        }
        TypeExpressionKind::Tuple(elements) => {
            for (index, nested) in elements.iter().enumerate() {
                expression_models(
                    *nested,
                    owner,
                    graph,
                    &format!("{suffix}.{index}"),
                    output,
                    budget,
                    depth + 1,
                )?;
            }
        }
        TypeExpressionKind::Array { element, .. } | TypeExpressionKind::Slice(element) => {
            expression_models(
                element,
                owner,
                graph,
                &format!("{suffix}[]"),
                output,
                budget,
                depth + 1,
            )?;
        }
        TypeExpressionKind::Reference { target, .. }
        | TypeExpressionKind::RawPointer { target, .. } => {
            expression_models(target, owner, graph, suffix, output, budget, depth + 1)?;
        }
        TypeExpressionKind::AssociatedType(value) => {
            expression_models(
                value.self_type(),
                owner,
                graph,
                suffix,
                output,
                budget,
                depth + 1,
            )?;
        }
        TypeExpressionKind::Parameter(_)
        | TypeExpressionKind::ParameterName(_)
        | TypeExpressionKind::FunctionPointer(_)
        | TypeExpressionKind::TraitObject(_)
        | TypeExpressionKind::Opaque(_)
        | TypeExpressionKind::Never => {}
    }
    Ok(())
}

#[cfg(test)]
mod build_budget_tests {
    use std::sync::Arc;

    use qubit_model_derive::Model;
    use qubit_model_metadata::metadata::TypeMetadata;
    use qubit_model_metadata::registry::ModelRegistry;
    use qubit_model_metadata::resolve::StructureResolver;

    use super::BuildBudget;
    use super::expression_models;
    use super::nested_models;

    #[Model(id = "budget.internal.Leaf")]
    struct BudgetLeaf {
        #[text(non_blank)]
        name: String,
    }

    #[Model(id = "budget.internal.Wrapped")]
    struct BudgetWrapped {
        child: Option<Box<BudgetLeaf>>,
    }

    #[Model(id = "budget.internal.Unsupported")]
    struct BudgetUnsupported {
        pair_00: (BudgetLeaf, BudgetLeaf),
        pair_01: (BudgetLeaf, BudgetLeaf),
        pair_02: (BudgetLeaf, BudgetLeaf),
        pair_03: (BudgetLeaf, BudgetLeaf),
        pair_04: (BudgetLeaf, BudgetLeaf),
        pair_05: (BudgetLeaf, BudgetLeaf),
        pair_06: (BudgetLeaf, BudgetLeaf),
        pair_07: (BudgetLeaf, BudgetLeaf),
        pair_08: (BudgetLeaf, BudgetLeaf),
        pair_09: (BudgetLeaf, BudgetLeaf),
        pair_10: (BudgetLeaf, BudgetLeaf),
        pair_11: (BudgetLeaf, BudgetLeaf),
        pair_12: (BudgetLeaf, BudgetLeaf),
        pair_13: (BudgetLeaf, BudgetLeaf),
        pair_14: (BudgetLeaf, BudgetLeaf),
        pair_15: (BudgetLeaf, BudgetLeaf),
        pair_16: (BudgetLeaf, BudgetLeaf),
        pair_17: (BudgetLeaf, BudgetLeaf),
        pair_18: (BudgetLeaf, BudgetLeaf),
        pair_19: (BudgetLeaf, BudgetLeaf),
        pair_20: (BudgetLeaf, BudgetLeaf),
        pair_21: (BudgetLeaf, BudgetLeaf),
        pair_22: (BudgetLeaf, BudgetLeaf),
        pair_23: (BudgetLeaf, BudgetLeaf),
        pair_24: (BudgetLeaf, BudgetLeaf),
        pair_25: (BudgetLeaf, BudgetLeaf),
        pair_26: (BudgetLeaf, BudgetLeaf),
        pair_27: (BudgetLeaf, BudgetLeaf),
        pair_28: (BudgetLeaf, BudgetLeaf),
        pair_29: (BudgetLeaf, BudgetLeaf),
        pair_30: (BudgetLeaf, BudgetLeaf),
        pair_31: (BudgetLeaf, BudgetLeaf),
        pair_32: (BudgetLeaf, BudgetLeaf),
        pair_33: (BudgetLeaf, BudgetLeaf),
        pair_34: (BudgetLeaf, BudgetLeaf),
        pair_35: (BudgetLeaf, BudgetLeaf),
        pair_36: (BudgetLeaf, BudgetLeaf),
        pair_37: (BudgetLeaf, BudgetLeaf),
        pair_38: (BudgetLeaf, BudgetLeaf),
        pair_39: (BudgetLeaf, BudgetLeaf),
        pair_40: (BudgetLeaf, BudgetLeaf),
        pair_41: (BudgetLeaf, BudgetLeaf),
        pair_42: (BudgetLeaf, BudgetLeaf),
        pair_43: (BudgetLeaf, BudgetLeaf),
        pair_44: (BudgetLeaf, BudgetLeaf),
        pair_45: (BudgetLeaf, BudgetLeaf),
        pair_46: (BudgetLeaf, BudgetLeaf),
        pair_47: (BudgetLeaf, BudgetLeaf),
        pair_48: (BudgetLeaf, BudgetLeaf),
        pair_49: (BudgetLeaf, BudgetLeaf),
        pair_50: (BudgetLeaf, BudgetLeaf),
    }

    #[Model(id = "budget.internal.NoWork")]
    struct BudgetNoWork {
        value: u8,
    }

    fn setup() -> (
        &'static TypeMetadata,
        Arc<qubit_model_metadata::resolve::ModelGraph<'static>>,
    ) {
        let root = TypeMetadata::of::<BudgetWrapped>();
        let models = ModelRegistry::try_global().expect("linked model registry");
        let graph = Arc::new(
            StructureResolver::for_static_roots(models, [root])
                .resolve()
                .expect("wrapper graph resolves"),
        );
        (root, graph)
    }

    fn graph_for(
        root: &'static TypeMetadata,
    ) -> Arc<qubit_model_metadata::resolve::ModelGraph<'static>> {
        let models = ModelRegistry::try_global().expect("linked model registry");
        Arc::new(
            StructureResolver::for_static_roots(models, [root])
                .resolve()
                .expect("test graph resolves"),
        )
    }

    #[test]
    fn test_descriptor_wrapper_depth_uses_the_injected_limit() {
        let (root, graph) = setup();
        let context = graph.registry().context().expect("reflection context");
        let field = root.fields().first().expect("wrapper field");
        let descriptor = root
            .field_descriptor_in(field, context)
            .and_then(|field| field.find_concrete_type());
        let expression = field.type_expression_in(context);
        let mut budget = BuildBudget::with_limits(0, 10_000, 10, 10, 10);

        let detail = nested_models(descriptor, expression, root, &graph, &mut budget).unwrap_err();
        assert_eq!(
            detail.kind(),
            crate::validation::ValidationBuildLimitKind::Depth
        );
        assert_eq!(detail.limit(), 0);
    }

    #[test]
    fn test_expression_fallback_depth_uses_the_injected_limit() {
        let (root, graph) = setup();
        let context = graph.registry().context().expect("reflection context");
        let field = root.fields().first().expect("wrapper field");
        let expression = field
            .type_expression_in(context)
            .expect("source expression");
        let mut budget = BuildBudget::with_limits(1, 10_000, 10, 10, 10);
        let mut targets = Vec::new();

        let detail = expression_models(expression, root, &graph, "", &mut targets, &mut budget, 0)
            .unwrap_err();
        assert_eq!(
            detail.kind(),
            crate::validation::ValidationBuildLimitKind::Depth
        );
    }

    #[test]
    fn test_no_work_graph_stops_when_work_budget_is_zero() {
        let root = TypeMetadata::of::<BudgetNoWork>();
        let graph = graph_for(root);
        let budget = BuildBudget::with_limits(64, 0, 10, 10, 10);
        let (occurrences, errors, _) = super::collect_with_budget(root, &graph, budget);

        assert!(occurrences.is_empty());
        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0].kind(),
            crate::validation::ValidationBuildErrorKind::BuildLimitExceeded
        );
        let detail = errors[0].build_limit().expect("work limit detail");
        assert_eq!(
            detail.kind(),
            crate::validation::ValidationBuildLimitKind::Work
        );
        assert_eq!(detail.limit(), 0);
    }

    #[test]
    fn test_unrelated_model_does_not_consume_root_work_budget() {
        let root = TypeMetadata::of::<BudgetNoWork>();
        let models = ModelRegistry::try_global().expect("linked model registry");
        let root_graph = graph_for(root);
        let shared_graph = Arc::new(
            StructureResolver::for_static_roots(
                models,
                [root, TypeMetadata::of::<BudgetWrapped>()],
            )
            .resolve()
            .expect("shared graph resolves"),
        );
        // This covers root discovery and the retained graph-wide fallback
        // name checks, but not a structural scan of the shared graph.
        let budget = || BuildBudget::with_limits(64, 14, 100, 100, 100);
        let (root_occurrences, root_errors, _) =
            super::collect_with_budget(root, &root_graph, budget());
        let (shared_occurrences, shared_errors, _) =
            super::collect_with_budget(root, &shared_graph, budget());

        assert!(
            root_errors.is_empty(),
            "14 work units suffice for the root alone"
        );
        assert!(
            shared_errors.is_empty(),
            "the unrelated model must not exhaust the root's work budget"
        );
        assert!(root_occurrences.is_empty());
        assert_eq!(shared_occurrences.len(), root_occurrences.len());
    }

    #[test]
    fn test_nested_model_callback_preserves_depth_limit_detail() {
        let root = TypeMetadata::of::<BudgetWrapped>();
        let graph = graph_for(root);
        let budget = BuildBudget::with_limits(0, 10_000, 10, 10, 10);
        let (occurrences, errors, _) = super::collect_with_budget(root, &graph, budget);

        assert!(occurrences.is_empty());
        assert_eq!(errors.len(), 1);
        let detail = errors[0].build_limit().expect("depth limit detail");
        assert_eq!(
            detail.kind(),
            crate::validation::ValidationBuildLimitKind::Depth
        );
        assert_eq!(detail.limit(), 0);
    }

    #[test]
    fn test_diagnostic_overflow_keeps_first_error_and_one_terminal_error() {
        let root = TypeMetadata::of::<BudgetUnsupported>();
        let graph = graph_for(root);
        let budget = BuildBudget::with_limits(64, 100_000, 10_000, 10_000, 100);
        let (occurrences, errors, _) = super::collect_with_budget(root, &graph, budget);

        assert!(occurrences.is_empty());
        assert_eq!(errors.len(), 101);
        assert!(errors[..100].iter().all(|error| {
            error.kind() == crate::validation::ValidationBuildErrorKind::UnsupportedExecution
                && error.build_limit().is_none()
        }));
        assert_eq!(
            errors[100].kind(),
            crate::validation::ValidationBuildErrorKind::BuildLimitExceeded
        );
        let detail = errors[100].build_limit().expect("diagnostics limit detail");
        assert_eq!(
            detail.kind(),
            crate::validation::ValidationBuildLimitKind::Diagnostics
        );
        assert_eq!(detail.limit(), 100);
    }

    #[test]
    fn test_occurrence_overflow_stops_before_collecting_the_next_declaration() {
        let root = TypeMetadata::of::<BudgetLeaf>();
        let graph = graph_for(root);
        let budget = BuildBudget::with_limits(64, 10_000, 0, 10, 10);
        let (occurrences, errors, _) = super::collect_with_budget(root, &graph, budget);

        assert!(occurrences.is_empty());
        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0].kind(),
            crate::validation::ValidationBuildErrorKind::BuildLimitExceeded
        );
        let detail = errors[0].build_limit().expect("occurrences limit detail");
        assert_eq!(
            detail.kind(),
            crate::validation::ValidationBuildLimitKind::Occurrences
        );
        assert_eq!(detail.limit(), 0);
    }
}
