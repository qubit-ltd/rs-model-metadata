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
    source: &FragmentIdentity,
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
        Some(source),
    );
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
    source: &FragmentIdentity,
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
    source: &FragmentIdentity,
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
    source: &FragmentIdentity,
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
                    Some(source),
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
    source: &FragmentIdentity,
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
    source: &FragmentIdentity,
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
        if index + 1 < path.segments().len() {
            let Some(descriptor) = property.descriptor() else {
                return Ok(None);
            };
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
