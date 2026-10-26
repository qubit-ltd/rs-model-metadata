// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Collects the models reachable from the configured resolution roots.

use std::collections::HashSet;

use super::ResolutionContext;
use super::ResolveScope;
use crate::metadata::ModelMetadataError;
use crate::metadata::TypeMetadata;
use crate::registry::ModelRegistry;
use crate::registry::ModelRegistryOrigin;
use crate::resolve::error::ResolveError;
use crate::resolve::relations::resolve_declared_target;
use crate::structure::children;

/// Builds the ordered node set and records errors found while discovering it.
///
/// Root validation, descriptor lookup, and child traversal retain the order of
/// the resolution attempt. Discovered metadata is cached in `context`, and
/// discovery failures are returned for the caller to append in that order.
///
/// # Parameters
///
/// - `registry`: Registry supplying registered nodes and declared targets.
/// - `roots`: Explicit concrete roots to validate and traverse.
/// - `scope`: Initial registry-wide or root-reachable node selection.
/// - `context`: Attempt-local descriptor and metadata cache.
///
/// # Returns
///
/// The ordered nodes with their registry origins and any discovery errors.
#[must_use]
pub(super) fn discover_nodes<'a>(
    registry: &'a ModelRegistry<'a>,
    roots: &[&'static TypeMetadata],
    scope: ResolveScope,
    context: &mut ResolutionContext<'a>,
) -> (
    Vec<(&'static TypeMetadata, Option<&'a ModelRegistryOrigin>)>,
    Vec<ResolveError>,
) {
    let mut errors = Vec::new();
    let mut nodes: Vec<_> = match scope {
        ResolveScope::AllRegistered => registry
            .concrete_entries()
            .map(|(metadata, source)| (metadata, Some(source)))
            .collect(),
        ResolveScope::ReachableFromRoots => Vec::new(),
    };
    let mut seen: HashSet<_> = nodes.iter().map(|(metadata, _)| metadata.type_id()).collect();
    for &root in roots {
        let root_descriptor = root.descriptor_in(context.reflection());
        if let Err(source) = root.validate_descriptor(&root_descriptor) {
            errors.push(ResolveError::resolution(
                root,
                None,
                None,
                ModelMetadataError::Abi {
                    type_id: root.type_id(),
                    type_name: root.type_name(),
                    source,
                },
            ));
            continue;
        }
        let metadata = match context.metadata_for(root_descriptor) {
            Ok(Some(metadata)) => metadata,
            Ok(None) => root,
            Err(error) => {
                errors.push(ResolveError::resolution(root, None, None, error));
                continue;
            }
        };
        if seen.insert(metadata.type_id()) {
            let source = registry
                .concrete_entries()
                .find(|(registered, _)| registered.type_id() == metadata.type_id())
                .map(|(_, source)| source);
            nodes.push((metadata, source));
        }
    }
    let mut cursor = 0;
    let mut descriptors = HashSet::new();
    while cursor < nodes.len() {
        let (metadata, source) = nodes[cursor];
        cursor += 1;
        let variants = metadata
            .as_enum()
            .into_iter()
            .flat_map(|value| value.variants())
            .flat_map(|variant| variant.fields());
        let mut pending = Vec::new();
        for field in metadata
            .fields()
            .iter()
            .chain(variants)
            .filter(|field| !field.is_opaque())
        {
            if field.reference().is_none()
                && let Some(descriptor) = metadata
                    .field_descriptor_in(field, context.reflection())
                    .and_then(|field| field.find_concrete_type())
            {
                pending.push((descriptor, field));
            }
            if let Some(reference) = field.reference()
                && let Some(target) = resolve_declared_target(reference.target(), registry)
                && seen.insert(target.type_id())
            {
                context.remember(target);
                nodes.push((target, None));
            }
        }
        pending.reverse();
        while let Some((descriptor, field)) = pending.pop() {
            if !descriptors.insert(descriptor.type_id()) {
                continue;
            }
            if let Some(nested) = metadata
                .nested_metadata()
                .find(|nested| nested.type_id() == descriptor.type_id())
            {
                context.remember(nested);
            }
            match context.metadata_for(descriptor) {
                Ok(Some(nested)) => {
                    context.remember(nested);
                    if seen.insert(nested.type_id()) {
                        nodes.push((nested, None));
                    }
                    continue;
                }
                Ok(None) => {}
                Err(error) => {
                    errors.push(
                        ResolveError::resolution(metadata, None, source, error).with_declaration(*field.declaration()),
                    );
                    continue;
                }
            }
            let first_child = pending.len();
            children(descriptor, |edge| {
                pending.push((edge.target, field));
            });
            pending[first_child..].reverse();
        }
    }
    (nodes, errors)
}
