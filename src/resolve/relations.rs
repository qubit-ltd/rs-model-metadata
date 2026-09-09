// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Relationship, property-path, and value-closure resolution helpers.

use std::any::TypeId;
use std::collections::HashSet;

use qubit_reflect::TypeDescriptor;
use qubit_reflect::descriptor::TypeRef;
use qubit_reflect::identity::FragmentIdentity;

use super::error::ModelResolutionCause;
use super::error::ResolveError;
use super::error::ResolveErrorKind;
use super::owned_property_path::OwnedPropertyPath;
use crate::metadata::DeclaredEntityTarget;
use crate::metadata::FieldMetadata;
use crate::metadata::ModelMetadataError;
use crate::metadata::ModelRole;
use crate::metadata::PropertyMetadata;
use crate::metadata::PropertyPath;
use crate::metadata::TypeMetadata;
use crate::registry::ModelRegistry;
/// Records an error anchored to one direct field path.
pub(super) fn push_field_error(
    errors: &mut Vec<ResolveError>,
    kind: ResolveErrorKind,
    metadata: &'static TypeMetadata,
    field: &'static FieldMetadata,
    actual_role: Option<ModelRole>,
    source: Option<&FragmentIdentity>,
) {
    let path = field.name().map(|name| {
        let segments = [name];
        OwnedPropertyPath::from_segments(&segments)
    });
    let mut error = ResolveError::new(
        kind,
        metadata.model_id().map(|id| id.as_str()),
        None,
        None,
        actual_role,
        source,
    );
    error.attach_owner(metadata);
    error.attach_declaration(*field.declaration());
    error.path = path;
    errors.push(error);
}

/// Finds model metadata attached to or registered for a descriptor.
pub(super) fn metadata_for_descriptor(
    descriptor: &'static TypeDescriptor,
    registry: &ModelRegistry,
) -> Result<Option<&'static TypeMetadata>, ModelMetadataError> {
    registry.metadata_for(descriptor)
}

/// Records a metadata failure with its traversal context before returning no
/// metadata.
pub(super) fn reported_metadata(
    descriptor: &'static TypeDescriptor,
    registry: &ModelRegistry,
    root: &'static TypeMetadata,
    path: Option<PropertyPath<'_>>,
    source: Option<&FragmentIdentity>,
    errors: &mut Vec<ResolveError>,
) -> Option<&'static TypeMetadata> {
    match metadata_for_descriptor(descriptor, registry) {
        Ok(metadata) => metadata,
        Err(error) => {
            errors.push(ResolveError::resolution(root, path, source, error));
            None
        }
    }
}

/// Resolves a declared target using either its provider or stable model ID.
pub(super) fn resolve_declared_target(
    target: &DeclaredEntityTarget,
    registry: &ModelRegistry,
) -> Option<&'static TypeMetadata> {
    match target {
        DeclaredEntityTarget::RustType(provider) => Some(provider()),
        DeclaredEntityTarget::ModelId(id) => registry.metadata(id.as_str()),
    }
}

/// Finds an Entity or Projection nested through ordinary container structure.
pub(super) fn forbidden_entity_nested_role(
    descriptor: &'static TypeDescriptor,
    registry: &ModelRegistry,
) -> Result<Option<ModelRole>, ModelMetadataError> {
    if let Some(metadata) = metadata_for_descriptor(descriptor, registry)?
        && matches!(metadata.role(), ModelRole::Entity | ModelRole::Projection)
    {
        return Ok(Some(metadata.role()));
    }
    let nested = if let Some(optional) = descriptor.as_optional() {
        Some(optional.element_type())
    } else if let Some(sequence) = descriptor.as_sequence() {
        Some(sequence.element_type())
    } else if let Some(set) = descriptor.as_set() {
        Some(set.element_type())
    } else if let Some(array) = descriptor.as_array() {
        Some(array.element_type())
    } else {
        descriptor.as_smart_pointer().map(|pointer| pointer.pointee_type())
    };
    if let Some(nested) = nested.and_then(TypeRef::as_resolved) {
        return forbidden_entity_nested_role(nested, registry);
    }
    if let Some(map) = descriptor.as_map() {
        for nested in [map.key_type(), map.value_type()]
            .into_iter()
            .filter_map(TypeRef::as_resolved)
        {
            if let Some(role) = forbidden_entity_nested_role(nested, registry)? {
                return Ok(Some(role));
            }
        }
    }
    Ok(None)
}

/// Copies static segments into an owned runtime path.
pub(super) fn path_from_segments(segments: &[&'static str]) -> OwnedPropertyPath {
    OwnedPropertyPath::from_segments(segments)
}

/// Verifies that a value model contains only closed value types.
pub(super) fn validate_value_closure(
    metadata: &'static TypeMetadata,
    registry: &ModelRegistry,
    visited: &mut HashSet<TypeId>,
    source: Option<&FragmentIdentity>,
    errors: &mut Vec<ResolveError>,
) {
    validate_nested_value(metadata, metadata, &[], registry, visited, source, errors);
}

/// Validates a value subtree while retaining the originating model and full
/// path.
#[allow(clippy::too_many_arguments)]
fn validate_nested_value(
    metadata: &'static TypeMetadata,
    root: &'static TypeMetadata,
    prefix: &[&'static str],
    registry: &ModelRegistry,
    visited: &mut HashSet<TypeId>,
    source: Option<&FragmentIdentity>,
    errors: &mut Vec<ResolveError>,
) -> bool {
    if !visited.insert(metadata.type_id()) {
        return true;
    }
    let initial = errors.len();
    for field in metadata.fields() {
        if field.is_opaque() {
            continue;
        }
        let Some(name) = field.name() else { continue };
        let mut path = prefix.to_vec();
        path.push(name);
        let before = errors.len();
        let closed = value_type_ref_is_closed(field.type_ref(), registry, visited, root, source, errors, &path);
        if !closed && errors.len() == before {
            let actual_role = field
                .descriptor()
                .and_then(|descriptor| {
                    reported_metadata(
                        descriptor,
                        registry,
                        root,
                        Some(PropertyPath::new(&path)),
                        source,
                        errors,
                    )
                })
                .map(TypeMetadata::role);
            if errors.len() == before {
                errors.push(ResolveError::new(
                    ResolveErrorKind::InvalidValueClosure,
                    root.model_id().map(|id| id.as_str()),
                    Some(PropertyPath::new(&path)),
                    Some(ModelRole::Value),
                    actual_role,
                    source,
                ));
            }
        }
    }
    visited.remove(&metadata.type_id());
    errors.len() == initial
}

/// Checks one nested reference while preserving diagnostic context.
#[allow(clippy::too_many_arguments)]
fn value_type_ref_is_closed(
    type_ref: &'static TypeRef,
    registry: &ModelRegistry,
    visited: &mut HashSet<TypeId>,
    root: &'static TypeMetadata,
    source: Option<&FragmentIdentity>,
    errors: &mut Vec<ResolveError>,
    path: &[&'static str],
) -> bool {
    type_ref
        .as_resolved()
        .is_some_and(|descriptor| value_descriptor_is_closed(descriptor, registry, visited, root, source, errors, path))
}

/// Checks a descriptor without turning lookup failures into role violations.
#[allow(clippy::too_many_arguments)]
fn value_descriptor_is_closed(
    descriptor: &'static TypeDescriptor,
    registry: &ModelRegistry,
    visited: &mut HashSet<TypeId>,
    root: &'static TypeMetadata,
    source: Option<&FragmentIdentity>,
    errors: &mut Vec<ResolveError>,
    path: &[&'static str],
) -> bool {
    let metadata = match metadata_for_descriptor(descriptor, registry) {
        Ok(metadata) => metadata,
        Err(error) => {
            errors.push(ResolveError::resolution(
                root,
                Some(PropertyPath::new(path)),
                source,
                error,
            ));
            return false;
        }
    };
    if let Some(metadata) = metadata {
        return match metadata.role() {
            ModelRole::Value => validate_nested_value(metadata, root, path, registry, visited, source, errors),
            ModelRole::Enum => {
                let Some(enumeration) = metadata.as_enum() else {
                    return false;
                };
                if !visited.insert(metadata.type_id()) {
                    return true;
                }
                let mut closed = true;
                for variant in enumeration.variants() {
                    for field in variant.fields() {
                        if field.is_opaque() {
                            continue;
                        }
                        let mut nested = path.to_vec();
                        nested.push(variant.canonical_name());
                        if let Some(name) = field.name() {
                            nested.push(name);
                        }
                        if field.reference().is_some() {
                            let mut error = ResolveError::new(
                                ResolveErrorKind::InvalidValueClosure,
                                root.model_id().map(|id| id.as_str()),
                                Some(PropertyPath::new(&nested)),
                                Some(ModelRole::Value),
                                Some(ModelRole::Enum),
                                source,
                            );
                            error.attach_owner(root);
                            error.attach_declaration(*field.declaration());
                            errors.push(error);
                            closed = false;
                            continue;
                        }
                        closed &= value_type_ref_is_closed(
                            field.type_ref(),
                            registry,
                            visited,
                            root,
                            source,
                            errors,
                            &nested,
                        );
                    }
                }
                visited.remove(&metadata.type_id());
                closed
            }
            ModelRole::Entity | ModelRole::Projection | ModelRole::Model => false,
        };
    }
    if descriptor.as_primitive().is_some() || descriptor.as_text().is_some() {
        return true;
    }
    let nested = if let Some(optional) = descriptor.as_optional() {
        Some(optional.element_type())
    } else if let Some(sequence) = descriptor.as_sequence() {
        Some(sequence.element_type())
    } else if let Some(set) = descriptor.as_set() {
        Some(set.element_type())
    } else if let Some(array) = descriptor.as_array() {
        Some(array.element_type())
    } else {
        descriptor.as_smart_pointer().map(|pointer| pointer.pointee_type())
    };
    if let Some(nested) = nested {
        return value_type_ref_is_closed(nested, registry, visited, root, source, errors, path);
    }
    if let Some(map) = descriptor.as_map() {
        let key = value_type_ref_is_closed(map.key_type(), registry, visited, root, source, errors, path);
        let value = value_type_ref_is_closed(map.value_type(), registry, visited, root, source, errors, path);
        return key && value;
    }
    if let Some(tuple) = descriptor.as_tuple() {
        let mut closed = true;
        for element in tuple.elements() {
            closed &= value_type_ref_is_closed(element, registry, visited, root, source, errors, path);
        }
        return closed;
    }
    false
}

/// Resolves a nested property path against a registered target model.
pub(super) fn resolve_property_path(
    target: &'static TypeMetadata,
    path: &PropertyPath<'_>,
    registry: &ModelRegistry,
) -> Result<Option<&'static PropertyMetadata>, ModelResolutionCause> {
    let mut current = target;
    let mut result = None;
    for (index, segment) in path.segments().iter().enumerate() {
        let Some(property) = registry.properties_for(current)?.property(segment) else {
            return Ok(None);
        };
        result = Some(property);
        if !property.is_readable() {
            return Ok(result);
        }
        if index + 1 < path.segments().len() {
            let Some(mut descriptor) = property.descriptor() else {
                return Ok(None);
            };
            let mut visited = HashSet::new();
            while visited.insert(descriptor.type_id()) {
                let inner = descriptor
                    .as_optional()
                    .map(|value| value.element_type())
                    .or_else(|| descriptor.as_smart_pointer().map(|value| value.pointee_type()));
                let Some(inner) = inner else {
                    break;
                };
                let Some(resolved) = inner.as_resolved() else {
                    return Ok(None);
                };
                descriptor = resolved;
            }
            let Some(nested) = metadata_for_descriptor(descriptor, registry)? else {
                return Ok(None);
            };
            current = nested;
        }
    }
    Ok(result)
}
/// Returns a stable target ID for textual target declarations.
pub(super) fn declared_target_id(target: &DeclaredEntityTarget) -> Option<&'static str> {
    target.model_id().map(|id| id.as_str())
}

/// Checks scoped uniqueness structurally without prescribing query products.
pub(super) fn validate_unique_scope(
    metadata: &'static TypeMetadata,
    field: &'static FieldMetadata,
    registry: &ModelRegistry,
    source: Option<&FragmentIdentity>,
    errors: &mut Vec<ResolveError>,
) {
    let Some(unique) = field.unique() else {
        return;
    };
    for scope in unique.respect_to() {
        let kind = match resolve_property_path(metadata, scope, registry) {
            Ok(Some(property)) if property.is_readable() => continue,
            Ok(Some(_)) => ResolveErrorKind::UnreadableProperty,
            Ok(None) => ResolveErrorKind::MissingProperty,
            Err(error) => {
                errors.push(
                    ResolveError::resolution(metadata, Some(*scope), source, error)
                        .with_declaration(*field.declaration()),
                );
                continue;
            }
        };
        errors.push(
            ResolveError::new(
                kind,
                metadata.model_id().map(|id| id.as_str()),
                Some(*scope),
                None,
                Some(metadata.role()),
                source,
            )
            .with_declaration(*field.declaration()),
        );
    }
}

/// Matches the selected value through allowed reference-container shapes.
pub(super) fn reference_value_matches(expected: &TypeDescriptor, mut actual: &'static TypeDescriptor) -> bool {
    let mut visited = HashSet::new();
    while visited.insert(actual.type_id()) {
        if actual.as_map().is_some() {
            return false;
        }
        if expected.type_id() == actual.type_id() {
            return true;
        }
        let inner = actual
            .as_optional()
            .map(|value| value.element_type())
            .or_else(|| actual.as_sequence().map(|value| value.element_type()))
            .or_else(|| actual.as_set().map(|value| value.element_type()))
            .or_else(|| actual.as_array().map(|value| value.element_type()))
            .or_else(|| actual.as_smart_pointer().map(|value| value.pointee_type()));
        let Some(inner) = inner.and_then(|value| value.as_resolved()) else {
            return false;
        };
        actual = inner;
    }
    false
}

/// Resolves object bindings rather than the IDs or projections stored in
/// fields.
pub(super) fn resolve_object_binding(
    root: &'static TypeMetadata,
    path: &crate::metadata::ObjectPath,
    registry: &ModelRegistry,
) -> Result<(Option<&'static TypeMetadata>, bool), super::error::ModelResolutionCause> {
    resolve_object_path(root, path, registry, true)
}

/// Selects stored objects for validators or full Entity bindings for
/// references.
fn resolve_object_path(
    root: &'static TypeMetadata,
    path: &crate::metadata::ObjectPath,
    registry: &ModelRegistry,
    follow_entity_bindings: bool,
) -> Result<(Option<&'static TypeMetadata>, bool), super::error::ModelResolutionCause> {
    let mut current = root;
    let mut parents = Vec::new();
    for step in path.steps() {
        match step {
            crate::metadata::NavigationStep::Parent => {
                let Some(parent) = parents.pop() else {
                    return Ok((None, true));
                };
                current = parent;
            }
            crate::metadata::NavigationStep::Property(name) => {
                let properties = registry.properties_for(current)?;
                let Some(property) = properties.property(name).filter(|property| property.is_readable()) else {
                    return Ok((None, false));
                };
                let next = if follow_entity_bindings
                    && let Some(reference) = property.field().and_then(FieldMetadata::reference)
                {
                    resolve_declared_target(reference.target(), registry)
                } else {
                    let mut descriptor = property.descriptor();
                    while let Some(value) = descriptor {
                        let inner = value
                            .as_optional()
                            .map(|value| value.element_type())
                            .or_else(|| value.as_smart_pointer().map(|value| value.pointee_type()));
                        if let Some(inner) = inner {
                            descriptor = inner.as_resolved();
                        } else {
                            break;
                        }
                    }
                    match descriptor {
                        Some(descriptor) => registry.metadata_for(descriptor)?,
                        None => None,
                    }
                };
                let Some(next) = next else {
                    return Ok((None, false));
                };
                parents.push(current);
                current = next;
            }
        }
    }
    Ok((Some(current), false))
}

/// Checks statically known dependency targets independently from validator
/// binding.
pub(super) fn validate_dependency(
    owner: &'static TypeMetadata,
    dependency: &crate::metadata::DependencyBindingMetadata,
    registry: &ModelRegistry,
    source: Option<&FragmentIdentity>,
    errors: &mut Vec<ResolveError>,
) {
    let resolved = resolve_object_path(owner, &dependency.object_path(), registry, false);
    let property = dependency.property();
    let failure = match resolved {
        Ok((_, true)) => return,
        Ok((Some(target), false)) => match resolve_property_path(target, &property, registry) {
            Ok(Some(value)) if value.is_readable() => return,
            Ok(Some(_)) => Ok(ResolveErrorKind::UnreadableProperty),
            Ok(None) => Ok(ResolveErrorKind::MissingProperty),
            Err(cause) => Err(cause),
        },
        Ok((None, false)) => Ok(ResolveErrorKind::MissingProperty),
        Err(cause) => Err(cause),
    };
    let mut error = match failure {
        Ok(kind) => ResolveError::new(
            kind,
            owner.model_id().map(|id| id.as_str()),
            Some(property),
            None,
            None,
            source,
        ),
        Err(cause) => ResolveError::resolution(owner, Some(property), source, cause),
    }
    .with_object_path(&dependency.object_path());
    error.attach_owner(owner);
    error.attach_declaration(*dependency.declaration());
    errors.push(error);
}
