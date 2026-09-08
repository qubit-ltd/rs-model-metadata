// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Query metadata synthesis from resolved model declarations.

use std::collections::HashMap;

use qubit_reflect::TypeDescriptor;
use qubit_reflect::identity::FragmentIdentity;

use super::error::ResolveError;
use super::error::ResolveErrorKind;
use super::graph::QueryField;
use super::graph::QueryMetadata;
use super::graph::UniqueQueryKey;
use super::owned_property_path::OwnedPropertyPath;
use super::relations::path_from_segments;
use super::relations::reported_metadata;
use super::relations::resolve_declared_target;
use super::relations::resolve_property_path;
use crate::metadata::FieldMetadata;
use crate::metadata::IndexingReasons;
use crate::metadata::ModelRole;
use crate::metadata::PropertyPath;
use crate::metadata::ReferenceSelection;
use crate::metadata::TypeMetadata;
use crate::registry::ModelRegistry;

/// Builds indexed query metadata for one entity.
pub(super) fn build_query(
    metadata: &'static TypeMetadata,
    registry: &ModelRegistry,
    source: &FragmentIdentity,
    errors: &mut Vec<ResolveError>,
) -> Option<QueryMetadata> {
    let initial_error_count = errors.len();
    let mut filters = Vec::new();
    let mut unique_keys = Vec::new();
    let mut flat_names = HashMap::<String, OwnedPropertyPath>::new();

    for field in metadata.fields() {
        let Some(name) = field.name() else { continue };
        let root_path = path_from_segments(&[name]);
        if field.is_identifier() {
            unique_keys.push(UniqueQueryKey::new(vec![root_path]));
            continue;
        }
        if let Some(unique) = field.unique() {
            if unique.is_scoped() {
                let mut paths = vec![root_path.clone()];
                for scope in unique.respect_to() {
                    match resolve_property_path(metadata, scope, registry) {
                        Ok(Some(property)) if property.is_readable() => {
                            paths.push(OwnedPropertyPath::from_static(*scope));
                        }
                        Ok(Some(_)) => errors.push(ResolveError::new(
                            ResolveErrorKind::UnreadableProperty,
                            metadata.model_id().map(|id| id.as_str()),
                            Some(*scope),
                            None,
                            Some(metadata.role()),
                            Some(source),
                        )),
                        Ok(None) => errors.push(ResolveError::new(
                            ResolveErrorKind::MissingProperty,
                            metadata.model_id().map(|id| id.as_str()),
                            Some(*scope),
                            None,
                            Some(metadata.role()),
                            Some(source),
                        )),

                        Err(error) => {
                            errors.push(ResolveError::resolution(
                                metadata,
                                Some(*scope),
                                source,
                                error,
                            ));
                        }
                    }
                }
                unique_keys.push(UniqueQueryKey::new(paths));
            } else {
                unique_keys.push(UniqueQueryKey::new(vec![root_path]));
                continue;
            }
        }
        if !field.is_indexed() {
            continue;
        }
        if let Some(reference) = field.reference() {
            match reference.selection() {
                ReferenceSelection::Property(_) => push_query_field(
                    &mut filters,
                    &mut flat_names,
                    root_path,
                    field.descriptor(),
                    field.indexing_reasons(),
                    metadata,
                    source,
                    errors,
                ),
                ReferenceSelection::Entity => {
                    if let Some(target) = resolve_declared_target(reference.target(), registry) {
                        collect_query_fields(
                            target,
                            &[name],
                            false,
                            registry,
                            &mut filters,
                            &mut flat_names,
                            metadata,
                            source,
                            errors,
                        );
                    }
                }
            }
            continue;
        }
        let field_error_count = errors.len();
        let added = collect_indexed_field(
            field,
            &[name],
            true,
            registry,
            &mut filters,
            &mut flat_names,
            metadata,
            source,
            errors,
        );
        if !added && errors.len() == field_error_count {
            let actual_role = field
                .descriptor()
                .and_then(|descriptor| {
                    reported_metadata(
                        descriptor,
                        registry,
                        metadata,
                        Some(PropertyPath::new(&[name])),
                        source,
                        errors,
                    )
                })
                .map(TypeMetadata::role);
            if errors.len() != field_error_count {
                continue;
            }
            errors.push(ResolveError::new(
                ResolveErrorKind::InvalidValueClosure,
                metadata.model_id().map(|id| id.as_str()),
                Some(root_path.as_path()),
                Some(ModelRole::Value),
                actual_role,
                Some(source),
            ));
        }
    }

    (errors.len() == initial_error_count).then_some(QueryMetadata {
        filters: filters.into_boxed_slice(),
        unique_keys: unique_keys.into_boxed_slice(),
    })
}

/// Recursively collects indexed fields from a model subtree.
#[allow(clippy::too_many_arguments)]
fn collect_query_fields(
    metadata: &'static TypeMetadata,
    prefix: &[&'static str],
    allow_references: bool,
    registry: &ModelRegistry,
    filters: &mut Vec<QueryField>,
    flat_names: &mut HashMap<String, OwnedPropertyPath>,
    root: &'static TypeMetadata,
    source: &FragmentIdentity,
    errors: &mut Vec<ResolveError>,
) -> bool {
    let mut added = false;
    for field in metadata.fields() {
        let Some(name) = field.name() else { continue };
        if !field.is_indexed() || (!allow_references && field.reference().is_some()) {
            continue;
        }
        let mut path = prefix.to_vec();
        path.push(name);
        added |= collect_indexed_field(
            field,
            &path,
            allow_references,
            registry,
            filters,
            flat_names,
            root,
            source,
            errors,
        );
    }
    added
}

/// Collects one indexed field and any nested value fields.
#[allow(clippy::too_many_arguments)]
fn collect_indexed_field(
    field: &'static FieldMetadata,
    path: &[&'static str],
    allow_references: bool,
    registry: &ModelRegistry,
    filters: &mut Vec<QueryField>,
    flat_names: &mut HashMap<String, OwnedPropertyPath>,
    root: &'static TypeMetadata,
    source: &FragmentIdentity,
    errors: &mut Vec<ResolveError>,
) -> bool {
    if let Some(reference) = field.reference() {
        if !allow_references {
            return false;
        }
        if matches!(reference.selection(), ReferenceSelection::Property(_)) {
            push_query_field(
                filters,
                flat_names,
                path_from_segments(path),
                field.descriptor(),
                field.indexing_reasons(),
                root,
                source,
                errors,
            );
            return true;
        }
        return resolve_declared_target(reference.target(), registry).is_some_and(|target| {
            collect_query_fields(
                target, path, false, registry, filters, flat_names, root, source, errors,
            )
        });
    }
    let initial_error_count = errors.len();
    if let Some(descriptor) = field.descriptor()
        && let Some(nested) = reported_metadata(
            descriptor,
            registry,
            root,
            Some(PropertyPath::new(path)),
            source,
            errors,
        )
        && matches!(nested.role(), ModelRole::Value | ModelRole::Model)
    {
        return collect_query_fields(
            nested,
            path,
            allow_references,
            registry,
            filters,
            flat_names,
            root,
            source,
            errors,
        );
    }
    if errors.len() != initial_error_count {
        return false;
    }
    push_query_field(
        filters,
        flat_names,
        path_from_segments(path),
        field.descriptor(),
        field.indexing_reasons(),
        root,
        source,
        errors,
    );
    true
}

/// Adds one query field while checking flattened-name collisions.
#[allow(clippy::too_many_arguments)]
fn push_query_field(
    filters: &mut Vec<QueryField>,
    flat_names: &mut HashMap<String, OwnedPropertyPath>,
    path: OwnedPropertyPath,
    descriptor: Option<&'static TypeDescriptor>,
    reasons: IndexingReasons,
    root: &'static TypeMetadata,
    source: &FragmentIdentity,
    errors: &mut Vec<ResolveError>,
) {
    if let Some(existing) = filters.iter_mut().find(|field| field.path == path) {
        existing.reasons |= reasons;
        return;
    }
    let flat_name = path.as_path().segments().join("_");
    if flat_names
        .get(flat_name.as_str())
        .is_some_and(|existing| *existing != path)
    {
        errors.push(ResolveError::new(
            ResolveErrorKind::QueryNameConflict,
            root.model_id().map(|id| id.as_str()),
            Some(path.as_path()),
            None,
            None,
            Some(source),
        ));
        return;
    }
    flat_names.insert(flat_name.clone(), path.clone());
    filters.push(QueryField {
        path,
        flat_name: flat_name.into_boxed_str(),
        descriptor,
        reasons,
    });
}
