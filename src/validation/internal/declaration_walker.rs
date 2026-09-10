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

use qubit_reflect::TypeDescriptor;

use super::execution_declaration::ExecutionDeclaration;
use super::validation_occurrence::ValidationOccurrence;
use crate::metadata::AllowedChars;
use crate::metadata::ConstraintMetadata;
use crate::metadata::FieldMetadata;
use crate::metadata::SelectorMetadata;
use crate::metadata::SelectorPosition;
use crate::metadata::TypeMetadata;
use crate::resolve::ModelGraph;
use crate::validation::ValidationBuildError;

/// Collects supported occurrences and all structural execution failures.
pub(crate) fn collect(
    root: &'static TypeMetadata,
    graph: &ModelGraph<'_>,
) -> (Vec<ValidationOccurrence>, Vec<ValidationBuildError>) {
    let Some(root) = graph.model(root.type_id()) else {
        return (Vec::new(), vec![ValidationBuildError::missing_root(root)]);
    };
    let work = reachable_work(graph);
    let mut occurrences = Vec::new();
    let mut errors = Vec::new();
    let mut ordinal = 0;
    walk(
        root,
        root,
        &[],
        None,
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
fn reachable_work(graph: &ModelGraph<'_>) -> HashSet<TypeId> {
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
            for (target, _) in nested_models(field.descriptor(), graph) {
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
        let mut segments = prefix.to_vec();
        segments.push(field.name().unwrap_or("<unnamed>"));
        let unsupported = inherited_unsupported || owner.as_enum().is_some() || field.name().is_none();
        let unsupported_path = unsupported.then(|| {
            let mut path = diagnostic_prefix.map_or_else(|| prefix.join("."), str::to_owned);
            if let Some(variant) = field
                .location()
                .and_then(|location| location.variant())
                .and_then(|index| {
                    owner
                        .as_enum()?
                        .variants()
                        .iter()
                        .find(|variant| variant.index() == index)
                })
            {
                append_segment(&mut path, variant.rust_name());
            }
            append_segment(
                &mut path,
                &field.name().map_or_else(|| field.index().to_string(), str::to_owned),
            );
            path
        });
        for (selector, declaration) in declarations(field) {
            let occurrence = ValidationOccurrence {
                root,
                owner,
                field,
                segments: segments.clone(),
                unsupported_path: unsupported_path.clone(),
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
        if field.is_opaque() || field.reference().is_some() {
            continue;
        }
        for (target, suffix) in nested_models(field.descriptor(), graph) {
            if !work.contains(&target.type_id()) {
                continue;
            }
            // A structural suffix identifies a container, tuple or raw field use,
            // never an executable property access promised by the current backend.
            let direct = suffix.is_empty()
                && direct_model(field.descriptor(), graph).is_some_and(|model| model.type_id() == target.type_id());
            let nested_path = (unsupported || !direct).then(|| {
                let mut path = unsupported_path.clone().unwrap_or_else(|| segments.join("."));
                path.push_str(&suffix);
                path
            });
            if active.contains(&target.type_id()) {
                let occurrence = ValidationOccurrence {
                    root,
                    owner,
                    field,
                    segments: segments.clone(),
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
                &segments,
                nested_path.as_deref(),
                graph,
                work,
                active,
                unsupported || !direct,
                ordinal,
                occurrences,
                errors,
            );
        }
    }
    active.remove(&owner.type_id());
}

/// Returns struct fields or enum payload fields in declaration order.
fn fields(model: &'static TypeMetadata) -> Vec<&'static FieldMetadata> {
    match model.as_enum() {
        Some(value) => value.variants().iter().flat_map(|variant| variant.fields()).collect(),
        None => model.fields().iter().collect(),
    }
}

/// Enumerates actual executable declarations without mistaking empty selectors
/// for work.
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
fn selectors(constraint: &'static ConstraintMetadata) -> Vec<&'static SelectorMetadata> {
    match constraint {
        ConstraintMetadata::Sequence(value) => value.element().into_iter().collect(),
        ConstraintMetadata::Map(value) => value.key().into_iter().chain(value.value()).collect(),
        _ => Vec::new(),
    }
}

/// Reports whether the outer constraint requires an execution operation.
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
fn direct_model(
    mut descriptor: Option<&'static TypeDescriptor>,
    graph: &ModelGraph<'_>,
) -> Option<&'static TypeMetadata> {
    while let Some(current) = descriptor {
        if let Some(model) = graph.model(current.type_id()) {
            return Some(model);
        }
        descriptor = current
            .as_optional()
            .map(|value| value.element_type())
            .or_else(|| current.as_smart_pointer().map(|value| value.pointee_type()))
            .and_then(|value| value.as_resolved());
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

/// Finds every model usage behind structural wrappers without expanding model
/// bodies. Returned suffixes distinguish static uses of the same target type.
/// Only active ancestors suppress cycles; siblings are never deduplicated.
fn nested_models(
    descriptor: Option<&'static TypeDescriptor>,
    graph: &ModelGraph<'_>,
) -> Vec<(&'static TypeMetadata, String)> {
    // Enter/leave frames keep traversal finite without recursive Rust calls.
    let mut pending: Vec<_> = descriptor
        .into_iter()
        .map(|value| (value, String::new(), true))
        .collect();
    let mut active = HashSet::new();
    let mut targets = Vec::new();
    while let Some((current, path, entering)) = pending.pop() {
        if !entering {
            active.remove(&current.type_id());
            continue;
        }
        if !active.insert(current.type_id()) {
            continue;
        }
        if let Some(model) = graph.model(current.type_id()) {
            targets.push((model, path));
            active.remove(&current.type_id());
            continue;
        }
        pending.push((current, String::new(), false));
        let element = current
            .as_optional()
            .map(|value| (value.element_type(), ""))
            .or_else(|| current.as_smart_pointer().map(|value| (value.pointee_type(), "")))
            .or_else(|| current.as_sequence().map(|value| (value.element_type(), "[]")))
            .or_else(|| current.as_set().map(|value| (value.element_type(), "[]")))
            .or_else(|| current.as_array().map(|value| (value.element_type(), "[]")))
            .or_else(|| current.as_slice().map(|value| (value.element_type(), "[]")));
        if let Some((element, suffix)) = element
            && let Some(element) = element.as_resolved()
        {
            pending.push((element, format!("{path}{suffix}"), true));
        }
        if let Some(map) = current.as_map() {
            if let Some(value) = map.value_type().as_resolved() {
                pending.push((value, format!("{path}[value]"), true));
            }
            if let Some(key) = map.key_type().as_resolved() {
                pending.push((key, format!("{path}[key]"), true));
            }
        }
        if let Some(tuple) = current.as_tuple() {
            for (index, element) in tuple.elements().iter().enumerate().rev() {
                if let Some(element) = element.as_resolved() {
                    pending.push((element, format!("{path}.{index}"), true));
                }
            }
        }
        for field in current.fields().iter().rev() {
            if let Some(descriptor) = field.field_type().as_resolved() {
                let name = field
                    .query_name()
                    .map_or_else(|| field.index().to_string(), str::to_owned);
                pending.push((descriptor, format!("{path}.{name}"), true));
            }
        }
        for variant in current.variants().iter().rev() {
            for field in variant.fields().iter().rev() {
                if let Some(descriptor) = field.field_type().as_resolved() {
                    let name = field
                        .query_name()
                        .map_or_else(|| field.index().to_string(), str::to_owned);
                    pending.push((descriptor, format!("{path}.{}.{name}", variant.rust_name()), true));
                }
            }
        }
    }
    targets
}
