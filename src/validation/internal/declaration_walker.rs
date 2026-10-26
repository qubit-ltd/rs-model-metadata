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

use qubit_reflect::ReflectionContext;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::descriptor::GenericExpressionArgumentKind;
use qubit_reflect::descriptor::TypeExpression;
use qubit_reflect::descriptor::TypeExpressionKind;

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
use crate::structure::children;
use crate::validation::ValidationBuildError;

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
pub(crate) fn collect(
    root: &'static TypeMetadata,
    graph: &ModelGraph<'_>,
) -> (Vec<ValidationOccurrence>, Vec<ValidationBuildError>) {
    let context = graph
        .registry()
        .context()
        .expect("resolved validation graphs retain their reflection context");
    let Some(root) = graph.model(root.type_id()) else {
        return (Vec::new(), vec![ValidationBuildError::missing_root(root)]);
    };
    let work = reachable_work(graph, context);
    let mut occurrences = Vec::new();
    let mut errors = Vec::new();
    let mut ordinal = 0;
    walk(
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
    );
    (occurrences, errors)
}

/// Computes executable-work reachability by monotone propagation, including
/// cycles.
///
/// # Returns
///
/// Returns the set of model types whose reachable declarations require
/// executable work.
fn reachable_work(graph: &ModelGraph<'_>, context: &ReflectionContext) -> HashSet<TypeId> {
    let mut work = HashSet::new();
    let mut predecessors: HashMap<TypeId, Vec<TypeId>> = HashMap::new();
    for model in graph.models() {
        for field in fields(model) {
            if !declarations(field).is_empty() {
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
            ) {
                predecessors.entry(target.type_id()).or_default().push(model.type_id());
            }
        }
    }
    let mut pending: Vec<_> = work.iter().copied().collect();
    while let Some(id) = pending.pop() {
        for predecessor in predecessors.get(&id).into_iter().flatten() {
            if work.insert(*predecessor) {
                pending.push(*predecessor);
            }
        }
    }
    work
}

/// Visits usage paths in source order; only the active path deduplicates types.
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
) {
    active.insert(owner.type_id());
    for field in fields(owner) {
        let enum_variant = enum_variant_for_field(owner, field);
        let mut segments = prefix.to_vec();
        let transparent_unnamed = is_transparent_unnamed_field(owner, field);
        segments.push(
            field
                .name()
                .unwrap_or(if transparent_unnamed { "0" } else { "<unnamed>" }),
        );
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
                &field.name().map_or_else(|| field.index().to_string(), str::to_owned),
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
        );
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
        );
    }
    active.remove(&owner.type_id());
}

/// Descends through a field's executable nested models in structural order.
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
) {
    let field_descriptor = owner
        .field_descriptor_in(field, context)
        .and_then(|field| field.find_concrete_type());
    for (target, suffix, access_segments) in
        nested_models(field_descriptor, field.type_expression_in(context), owner, graph)
    {
        if !work.contains(&target.type_id()) {
            continue;
        }
        let direct = suffix.is_empty()
            && direct_model(field_descriptor, graph).is_some_and(|model| model.type_id() == target.type_id());
        let mut nested_segments = segments.to_vec();
        if let Some(access_segments) = access_segments {
            nested_segments.extend(access_segments);
        }
        let nested_path = (unsupported || owner.as_enum().is_some() || !direct).then(|| {
            let mut path = unsupported_path.map_or_else(
                || {
                    let mut path = diagnostic_prefix.map_or_else(|| prefix.join("."), str::to_owned);
                    if let Some(variant) = enum_variant {
                        append_segment(&mut path, variant.rust_name());
                    }
                    append_segment(
                        &mut path,
                        &field.name().map_or_else(|| field.index().to_string(), str::to_owned),
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
        );
    }
}

/// Records this field's declarations in their original metadata order.
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
) {
    for (selector, declaration) in declarations(field) {
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
            errors.push(ValidationBuildError::unsupported(&occurrence));
        } else {
            occurrences.push(occurrence);
        }
    }
}

/// Resolves the concrete enum variant associated with a payload field.
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
/// # Returns
///
/// Returns whether `field` is the transparent value's sole anonymous field.
#[must_use]
pub(crate) fn is_transparent_unnamed_field(owner: &TypeMetadata, field: &FieldMetadata) -> bool {
    owner
        .as_value()
        .and_then(|value| value.transparent_field())
        .is_some_and(|inner| std::ptr::eq(inner, field) && field.index() == 0 && field.name().is_none())
}

/// Returns struct fields or enum payload fields in declaration order.
///
/// # Returns
///
/// Returns all fields of `model` in declaration order.
fn fields(model: &'static TypeMetadata) -> Vec<&'static FieldMetadata> {
    match model.as_enum() {
        Some(value) => value.variants().iter().flat_map(|variant| variant.fields()).collect(),
        None => model.fields().iter().collect(),
    }
}

/// Enumerates actual executable declarations without mistaking empty selectors
/// for work.
///
/// # Returns
///
/// Returns each executable constraint and validator with its optional selector
/// position, preserving metadata order.
fn declarations(field: &'static FieldMetadata) -> Vec<(Option<SelectorPosition>, ExecutionDeclaration)> {
    let mut result = Vec::new();
    for constraint in field.constraints() {
        if constraint_has_work(constraint) {
            result.push((None, ExecutionDeclaration::Constraint(constraint)));
        }
        for selector in selectors(constraint) {
            for constraint in selector.constraints() {
                if constraint_has_work(constraint) {
                    result.push((Some(selector.position()), ExecutionDeclaration::Constraint(constraint)));
                }
            }
            result.extend(
                selector
                    .validators()
                    .iter()
                    .map(|value| (Some(selector.position()), ExecutionDeclaration::Validator(value))),
            );
        }
    }
    result.extend(
        field
            .validators()
            .iter()
            .map(|value| (None, ExecutionDeclaration::Validator(value))),
    );
    result
}

/// Returns nested selector metadata in key-before-value declaration order.
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
        ConstraintMetadata::Map(value) => value.min_entries().is_some() || value.max_entries().is_some(),
        ConstraintMetadata::Decimal(_) | ConstraintMetadata::Time(_) => true,
    }
}

/// Resolves a model through optional or smart-pointer wrappers only.
///
/// # Returns
///
/// Returns the first model found through the descriptor or its supported
/// wrappers, or `None` when no model is registered along that path.
fn direct_model<'ctx>(
    mut descriptor: Option<TypeDescriptor<'ctx>>,
    graph: &ModelGraph<'_>,
) -> Option<&'static TypeMetadata> {
    while let Some(current) = descriptor {
        if let Some(model) = graph.model(current.type_id()) {
            return Some(model);
        }
        let is_wrapper = matches!(
            current.kind(),
            qubit_reflect::descriptor::TypeKind::Optional(_) | qubit_reflect::descriptor::TypeKind::SmartPointer(_)
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
    }
    None
}

/// Appends one field, variant or tuple index to a dot-separated diagnostic
/// path.
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
            let name = query_name.or(name).map_or_else(|| index.to_string(), str::to_owned);
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

/// Finds every model usage behind structural wrappers without expanding model
/// bodies. Returned suffixes distinguish static uses of the same target type.
/// Only active ancestors suppress cycles; siblings are never deduplicated.
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
) -> Vec<(&'static TypeMetadata, String, Option<Vec<&'static str>>)> {
    // Enter/leave frames keep traversal finite without recursive Rust calls.
    let mut pending: Vec<_> = descriptor
        .into_iter()
        .map(|value| (value, String::new(), Some(Vec::new()), true))
        .collect();
    let mut active = HashSet::new();
    let mut targets = Vec::new();
    while let Some((current, path, access_segments, entering)) = pending.pop() {
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
        pending.push((current, String::new(), None, false));
        let first_child = pending.len();
        children(current, |edge| {
            let suffix = structural_suffix(edge.kind);
            let access_segments = match (access_segments.as_ref(), edge.kind) {
                (Some(segments), StructuralEdgeKind::StructField { name, query_name, .. }) => {
                    query_name.or(name).map(|name| {
                        let mut next = segments.clone();
                        next.push(name);
                        next
                    })
                }
                (Some(segments), StructuralEdgeKind::OptionalElement | StructuralEdgeKind::PointerPointee) => {
                    Some(segments.clone())
                }
                _ => None,
            };
            pending.push((edge.target, format!("{path}{suffix}"), access_segments, true));
        });
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
        expression_models(expression, owner, graph, "", &mut targets);
    }
    targets.sort_by_key(|(metadata, suffix, _)| (metadata.type_id(), suffix.clone()));
    targets.dedup_by(|left, right| left.0.type_id() == right.0.type_id() && left.1 == right.1);
    targets
}

/// Retains model usages found in source expressions when a context has no
/// registered concrete field descriptor for the owning type.
fn expression_models(
    expression: TypeExpression<'_>,
    owner: &'static TypeMetadata,
    graph: &ModelGraph<'_>,
    suffix: &str,
    output: &mut Vec<(&'static TypeMetadata, String, Option<Vec<&'static str>>)>,
) {
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
                return;
            }
            let matches = graph
                .models()
                .iter()
                .filter(|model| model.type_name() == path)
                .collect::<Vec<_>>();
            if let [model] = matches.as_slice() {
                output.push((model, suffix.to_owned(), None));
                return;
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
                        format!("{suffix}{}", if position == 0 { "[key]" } else { "[value]" })
                    }
                    _ => suffix.to_owned(),
                };
                expression_models(nested, owner, graph, &nested_suffix, output);
            }
        }
        TypeExpressionKind::Tuple(elements) => {
            for (index, nested) in elements.iter().enumerate() {
                expression_models(*nested, owner, graph, &format!("{suffix}.{index}"), output);
            }
        }
        TypeExpressionKind::Array { element, .. } | TypeExpressionKind::Slice(element) => {
            expression_models(element, owner, graph, &format!("{suffix}[]"), output);
        }
        TypeExpressionKind::Reference { target, .. } | TypeExpressionKind::RawPointer { target, .. } => {
            expression_models(target, owner, graph, suffix, output);
        }
        TypeExpressionKind::AssociatedType(value) => {
            expression_models(value.self_type(), owner, graph, suffix, output);
        }
        TypeExpressionKind::Parameter(_)
        | TypeExpressionKind::ParameterName(_)
        | TypeExpressionKind::FunctionPointer(_)
        | TypeExpressionKind::TraitObject(_)
        | TypeExpressionKind::Opaque(_)
        | TypeExpressionKind::Never => {}
    }
}
