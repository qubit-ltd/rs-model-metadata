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
use super::internal::OwnedPropertyPath;
use super::internal::ResolutionContext;
use crate::metadata::DeclaredEntityTarget;
use crate::metadata::DependencyBindingMetadata;
use crate::metadata::FieldMetadata;
use crate::metadata::ModelMetadataError;
use crate::metadata::ModelRole;
use crate::metadata::NavigationStep;
use crate::metadata::ObjectPath;
use crate::metadata::PropertyMetadata;
use crate::metadata::PropertyPath;
use crate::metadata::TypeMetadata;
use crate::registry::ModelRegistry;
use crate::structure::children;

/// Records an error anchored to one direct field path.
///
/// # Parameters
///
/// - `errors`: Collection that receives the constructed diagnostic.
/// - `kind`: Resolution failure category to attach to the diagnostic.
/// - `metadata`: Model that owns the field declaration.
/// - `field`: Field whose declaration and optional name anchor the error.
/// - `actual_role`: Observed role, when metadata resolution found one.
/// - `source`: Originating metadata fragment, when available.
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
///
/// # Parameters
///
/// - `descriptor`: Static runtime descriptor to resolve.
/// - `context`: Resolution-local metadata and registry lookup context.
///
/// # Returns
///
/// `Ok(Some(metadata))` when metadata is known, `Ok(None)` when neither source
/// knows the model, or an error from metadata lookup/descriptor validation.
///
/// # Errors
///
/// Propagates the model metadata lookup or descriptor compatibility failure.
pub(super) fn metadata_for_descriptor(
    descriptor: &'static TypeDescriptor,
    context: &ResolutionContext,
) -> Result<Option<&'static TypeMetadata>, ModelMetadataError> {
    context.metadata_for(descriptor)
}

/// Records a metadata failure with its traversal context before returning no
/// metadata.
///
/// # Parameters
///
/// - `descriptor`: Runtime descriptor whose metadata is requested.
/// - `context`: Resolution-local and registry lookup context.
/// - `root`: Original model owning the traversal.
/// - `path`: Traversal property path, when available.
/// - `source`: Originating metadata fragment, when available.
/// - `errors`: Collection receiving a resolution diagnostic on lookup failure.
///
/// # Returns
///
/// `Some(metadata)` when lookup succeeds with metadata, or `None` when the
/// descriptor is unknown or lookup failed. Lookup failures are also appended
/// to `errors`.
pub(super) fn reported_metadata(
    descriptor: &'static TypeDescriptor,
    context: &ResolutionContext,
    root: &'static TypeMetadata,
    path: Option<PropertyPath<'_>>,
    source: Option<&FragmentIdentity>,
    errors: &mut Vec<ResolveError>,
) -> Option<&'static TypeMetadata> {
    match metadata_for_descriptor(descriptor, context) {
        Ok(metadata) => metadata,
        Err(error) => {
            errors.push(ResolveError::resolution(root, path, source, error));
            None
        }
    }
}

/// Resolves a declared target using either its provider or stable model ID.
///
/// # Parameters
///
/// - `target`: Declared Rust-type provider or stable model ID.
/// - `registry`: Registry used to resolve stable model IDs.
///
/// # Returns
///
/// `Some(metadata)` when the provider or registry supplies the target, or
/// `None` when the stable model ID is not registered.
pub(super) fn resolve_declared_target(
    target: &DeclaredEntityTarget,
    registry: &ModelRegistry,
) -> Option<&'static TypeMetadata> {
    match target {
        DeclaredEntityTarget::RustType(provider) => Some(provider()),
        DeclaredEntityTarget::ModelId(id) => registry.metadata(id.as_str()),
    }
}

/// Finds an Entity or Projection through visible non-reference structure.
///
/// Uses an explicit traversal stack and current ancestors to terminate cycles
/// without suppressing sibling uses. Model fields retain their opaque and
/// reference boundaries; raw descriptors use shared structural children.
/// Returns the first forbidden role in declaration order, or `None` when no
/// visible child has that role. Metadata lookup failures preserve their cause.
///
/// # Parameters
///
/// - `descriptor`: Root runtime descriptor whose visible children are walked.
/// - `context`: Resolution context for metadata attached to discovered types.
///
/// # Returns
///
/// `Ok(Some(role))` for the first nested Entity or Projection, `Ok(None)` when
/// no forbidden role is visible, or an error when metadata lookup fails.
///
/// # Errors
///
/// Propagates metadata lookup or descriptor validation failures.
pub(super) fn forbidden_entity_nested_role(
    descriptor: &'static TypeDescriptor,
    context: &ResolutionContext,
) -> Result<Option<ModelRole>, ModelMetadataError> {
    let mut ancestors = HashSet::new();
    let mut pending = vec![(descriptor, false)];
    while let Some((current, exiting)) = pending.pop() {
        if exiting {
            ancestors.remove(&current.type_id());
            continue;
        }
        if !ancestors.insert(current.type_id()) {
            continue;
        }
        pending.push((current, true));
        let first_child = pending.len();
        if let Some(metadata) = metadata_for_descriptor(current, context)? {
            if matches!(metadata.role(), ModelRole::Entity | ModelRole::Projection) {
                return Ok(Some(metadata.role()));
            }
            let variants = metadata
                .as_enum()
                .into_iter()
                .flat_map(|enumeration| enumeration.variants())
                .flat_map(|variant| variant.fields());
            for field in metadata.fields().iter().chain(variants) {
                if !field.is_opaque()
                    && field.reference().is_none()
                    && let Some(child) = field.descriptor()
                {
                    pending.push((child, false));
                }
            }
        } else {
            children(current, |edge| {
                if let Some(child) = edge.target.as_resolved() {
                    pending.push((child, false));
                }
            });
        }
        pending[first_child..].reverse();
    }
    Ok(None)
}

/// Copies static segments into an owned runtime path.
///
/// # Parameters
///
/// - `segments`: Static field names in traversal order.
///
/// # Returns
///
/// An owned path retaining the supplied segment sequence.
#[must_use]
pub(super) fn path_from_segments(segments: &[&'static str]) -> OwnedPropertyPath {
    OwnedPropertyPath::from_segments(segments)
}

/// Verifies that a value model contains only closed value types.
///
/// # Parameters
///
/// - `metadata`: Value model whose nested field types are checked.
/// - `context`: Resolution context for nested descriptors.
/// - `visited`: Active type identities used to terminate recursive cycles.
/// - `source`: Originating metadata fragment, when available.
/// - `errors`: Collection receiving diagnostics for violations or lookup
///   failures.
pub(super) fn validate_value_closure(
    metadata: &'static TypeMetadata,
    context: &ResolutionContext,
    visited: &mut HashSet<TypeId>,
    source: Option<&FragmentIdentity>,
    errors: &mut Vec<ResolveError>,
) {
    // Root callers consume the diagnostic collection; recursive callers use the
    // closure result.
    let _ = validate_nested_value(metadata, metadata, &[], context, visited, source, errors);
}

/// Validates a value subtree while retaining the originating model and full
/// path.
///
/// # Parameters
///
/// - `metadata`: Current value model being traversed.
/// - `root`: Original value model owning the closure check.
/// - `prefix`: Property path accumulated before this subtree.
/// - `context`: Resolution context for nested descriptors.
/// - `visited`: Active type identities used to terminate recursive cycles.
/// - `source`: Originating metadata fragment, when available.
/// - `errors`: Collection receiving diagnostics for invalid fields.
///
/// # Returns
///
/// `true` when every visible field in this subtree is closed, or `false` when
/// at least one field is invalid or cannot be resolved.
#[allow(clippy::too_many_arguments)]
#[must_use]
fn validate_nested_value(
    metadata: &'static TypeMetadata,
    root: &'static TypeMetadata,
    prefix: &[&'static str],
    context: &ResolutionContext,
    visited: &mut HashSet<TypeId>,
    source: Option<&FragmentIdentity>,
    errors: &mut Vec<ResolveError>,
) -> bool {
    if !visited.insert(metadata.type_id()) {
        return true;
    }
    let initial = errors.len();
    let mut closed = true;
    for field in metadata.fields() {
        if field.is_opaque() {
            continue;
        }
        let mut path = prefix.to_vec();
        if let Some(name) = field.name() {
            path.push(name);
        }
        closed &= validate_value_field(field, root, &path, context, visited, source, errors);
    }
    visited.remove(&metadata.type_id());
    closed && errors.len() == initial
}

/// Checks a field's closure and anchors new failures to its actual declaration.
///
/// `path` contains only available property names and may be empty for unnamed
/// fields. Existing nested declarations take precedence over the outer field;
/// `root` retains the original closure owner's concrete identity. Returns
/// whether the type is closed and preserves metadata lookup failures in errors.
///
/// # Parameters
///
/// - `field`: Field whose type closure and declaration are checked.
/// - `root`: Original value model owning the closure check.
/// - `path`: Property names from the root to this field.
/// - `context`: Resolution context for nested descriptors.
/// - `visited`: Active type identities used to terminate recursive cycles.
/// - `source`: Originating metadata fragment, when available.
/// - `errors`: Collection receiving path-anchored diagnostics.
///
/// # Returns
///
/// `true` when the field type is closed, or `false` when it violates the
/// value-role closure or a nested type cannot be resolved.
#[allow(clippy::too_many_arguments)]
#[must_use]
fn validate_value_field(
    field: &'static FieldMetadata,
    root: &'static TypeMetadata,
    path: &[&'static str],
    context: &ResolutionContext,
    visited: &mut HashSet<TypeId>,
    source: Option<&FragmentIdentity>,
    errors: &mut Vec<ResolveError>,
) -> bool {
    let before = errors.len();
    let closed = value_type_ref_is_closed(
        field.type_ref(),
        context,
        visited,
        root,
        source,
        errors,
        path,
    );
    let property_path = (!path.is_empty()).then(|| PropertyPath::new(path));
    if !closed && errors.len() == before {
        let actual_role = field
            .descriptor()
            .and_then(|descriptor| {
                reported_metadata(descriptor, context, root, property_path, source, errors)
            })
            .map(TypeMetadata::role);
        if errors.len() == before {
            errors.push(ResolveError::new(
                ResolveErrorKind::InvalidValueClosure,
                root.model_id().map(|id| id.as_str()),
                property_path,
                Some(ModelRole::Value),
                actual_role,
                source,
            ));
        }
    }
    for error in &mut errors[before..] {
        error.attach_owner(root);
        if error.declaration().is_none() {
            error.attach_declaration(*field.declaration());
        }
    }
    closed
}

/// Checks one nested reference while preserving diagnostic context.
///
/// # Parameters
///
/// - `type_ref`: Reflected type reference to check.
/// - `context`: Resolution context for nested descriptors.
/// - `visited`: Active type identities used to terminate recursive cycles.
/// - `root`: Original value model owning the closure check.
/// - `source`: Originating metadata fragment, when available.
/// - `errors`: Collection receiving diagnostics for invalid nested types.
/// - `path`: Property path to the nested reference.
///
/// # Returns
///
/// `true` when the resolved nested type is closed, or `false` when it is
/// unresolved, invalid, or introduces a forbidden model role.
#[allow(clippy::too_many_arguments)]
#[must_use]
fn value_type_ref_is_closed(
    type_ref: &'static TypeRef,
    context: &ResolutionContext,
    visited: &mut HashSet<TypeId>,
    root: &'static TypeMetadata,
    source: Option<&FragmentIdentity>,
    errors: &mut Vec<ResolveError>,
    path: &[&'static str],
) -> bool {
    type_ref.as_resolved().is_some_and(|descriptor| {
        value_descriptor_is_closed(descriptor, context, visited, root, source, errors, path)
    })
}

/// Checks a descriptor without turning lookup failures into role violations.
///
/// # Parameters
///
/// - `descriptor`: Runtime type descriptor whose value closure is checked.
/// - `context`: Resolution context for metadata lookup.
/// - `visited`: Active type identities used to terminate recursive cycles.
/// - `root`: Original value model owning the closure check.
/// - `source`: Originating metadata fragment, when available.
/// - `errors`: Collection receiving metadata and closure diagnostics.
/// - `path`: Property path to this descriptor.
///
/// # Returns
///
/// `true` when the descriptor and all visible nested types are closed, or
/// `false` when a role violation or metadata lookup failure is found.
#[allow(clippy::too_many_arguments)]
#[must_use]
fn value_descriptor_is_closed(
    descriptor: &'static TypeDescriptor,
    context: &ResolutionContext,
    visited: &mut HashSet<TypeId>,
    root: &'static TypeMetadata,
    source: Option<&FragmentIdentity>,
    errors: &mut Vec<ResolveError>,
    path: &[&'static str],
) -> bool {
    let metadata = match metadata_for_descriptor(descriptor, context) {
        Ok(metadata) => metadata,
        Err(error) => {
            errors.push(ResolveError::resolution(
                root,
                (!path.is_empty()).then(|| PropertyPath::new(path)),
                source,
                error,
            ));
            return false;
        }
    };
    if let Some(metadata) = metadata {
        return match metadata.role() {
            ModelRole::Value => {
                validate_nested_value(metadata, root, path, context, visited, source, errors)
            }
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
                        closed &= validate_value_field(
                            field, root, &nested, context, visited, source, errors,
                        );
                    }
                }
                visited.remove(&metadata.type_id());
                closed
            }
            ModelRole::Entity | ModelRole::Projection | ModelRole::Model => {
                let mut error = ResolveError::new(
                    ResolveErrorKind::InvalidValueClosure,
                    root.model_id().map(|id| id.as_str()),
                    (!path.is_empty()).then(|| PropertyPath::new(path)),
                    Some(ModelRole::Value),
                    Some(metadata.role()),
                    source,
                );
                error.attach_owner(root);
                errors.push(error);
                false
            }
        };
    }
    if descriptor.as_primitive().is_some() || descriptor.as_text().is_some() {
        return true;
    }
    if descriptor.as_optional().is_none()
        && descriptor.as_sequence().is_none()
        && descriptor.as_set().is_none()
        && descriptor.as_array().is_none()
        && descriptor.as_smart_pointer().is_none()
        && descriptor.as_map().is_none()
        && descriptor.as_tuple().is_none()
    {
        return false;
    }
    if !visited.insert(descriptor.type_id()) {
        return true;
    }
    let mut closed = true;
    children(descriptor, |edge| {
        closed &=
            value_type_ref_is_closed(edge.target, context, visited, root, source, errors, path);
    });
    visited.remove(&descriptor.type_id());
    closed
}

/// Resolves a nested property path against a registered target model.
///
/// # Parameters
///
/// - `target`: Model metadata at which traversal begins.
/// - `path`: Ordered property names to resolve.
/// - `context`: Registry and metadata context used while traversing.
///
/// # Returns
///
/// `Ok(Some(property))` for the last property reached, `Ok(None)` when a path
/// segment or nested model cannot be resolved, or an error on metadata access.
///
/// # Errors
///
/// Returns the model resolution cause raised while reading registered
/// properties or resolving nested metadata.
pub(super) fn resolve_property_path(
    target: &'static TypeMetadata,
    path: &PropertyPath<'_>,
    context: &ResolutionContext,
) -> Result<Option<PropertyMetadata>, ModelResolutionCause> {
    let mut current = target;
    let mut result = None;
    for (index, segment) in path.segments().iter().enumerate() {
        let properties = context.registry().properties_for(current)?;
        let Some(property) = properties.property(segment) else {
            return Ok(None);
        };
        result = Some(*property);
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
                    .or_else(|| {
                        descriptor
                            .as_smart_pointer()
                            .map(|value| value.pointee_type())
                    });
                let Some(inner) = inner else {
                    break;
                };
                let Some(resolved) = inner.as_resolved() else {
                    return Ok(None);
                };
                descriptor = resolved;
            }
            let Some(nested) = metadata_for_descriptor(descriptor, context)? else {
                return Ok(None);
            };
            current = nested;
        }
    }
    Ok(result)
}
/// Returns a stable target ID for textual target declarations.
///
/// # Parameters
///
/// - `target`: Declared target whose optional model ID is requested.
///
/// # Returns
///
/// `Some(id)` for a textual model-ID target, or `None` for a Rust-type target.
#[inline]
pub(super) fn declared_target_id(target: &DeclaredEntityTarget) -> Option<&'static str> {
    target.model_id().map(|id| id.as_str())
}

/// Checks scoped uniqueness structurally without prescribing query products.
///
/// # Parameters
///
/// - `metadata`: Model declaring the unique field.
/// - `field`: Field whose configured uniqueness scopes are checked.
/// - `context`: Registry and resolution context used for property lookups.
/// - `source`: Originating metadata fragment, when available.
/// - `errors`: Collection receiving diagnostics for invalid scopes.
pub(super) fn validate_unique_scope(
    metadata: &'static TypeMetadata,
    field: &'static FieldMetadata,
    context: &ResolutionContext,
    source: Option<&FragmentIdentity>,
    errors: &mut Vec<ResolveError>,
) {
    let Some(unique) = field.unique() else {
        return;
    };
    for scope in unique.respect_to() {
        let kind = match resolve_property_path(metadata, scope, context) {
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
///
/// # Parameters
///
/// - `expected`: Descriptor of the referenced target type.
/// - `actual`: Descriptor of the field value, possibly wrapped in allowed
///   optional, collection, array, or smart-pointer containers.
///
/// # Returns
///
/// `true` when unwrapping allowed containers reaches the expected type, or
/// `false` for mismatches, maps, unresolved wrappers, or cycles.
#[must_use]
pub(super) fn reference_value_matches(
    expected: &TypeDescriptor,
    mut actual: &'static TypeDescriptor,
) -> bool {
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
///
/// # Parameters
///
/// - `root`: Model metadata where object-path traversal starts.
/// - `path`: Declared sequence of parent and property navigation steps.
/// - `context`: Registry and resolution-local metadata context.
///
/// # Returns
///
/// `Ok((Some(model), false))` for a resolved object, `(None, false)` for an
/// unresolved path, or `(None, true)` when traversal ascends above its root.
///
/// # Errors
///
/// Returns the cause from registry property lookup or nested metadata
/// resolution.
pub(super) fn resolve_object_binding(
    root: &'static TypeMetadata,
    path: &ObjectPath,
    context: &ResolutionContext,
) -> Result<(Option<&'static TypeMetadata>, bool), ModelResolutionCause> {
    resolve_object_path(root, path, context, true)
}

/// Selects stored objects for validators or full Entity bindings for
/// references.
///
/// # Parameters
///
/// - `root`: Model metadata where traversal starts.
/// - `path`: Ordered parent/property navigation steps.
/// - `context`: Registry and resolution-local metadata context.
/// - `follow_entity_bindings`: Whether reference steps select their Entity
///   target.
///
/// # Returns
///
/// `Ok((Some(model), false))` for a resolved object, `(None, false)` for an
/// unresolved path, or `(None, true)` when traversal ascends above its root.
///
/// # Errors
///
/// Returns failures from registry property lookup or descriptor resolution.
fn resolve_object_path(
    root: &'static TypeMetadata,
    path: &ObjectPath,
    context: &ResolutionContext,
    follow_entity_bindings: bool,
) -> Result<(Option<&'static TypeMetadata>, bool), ModelResolutionCause> {
    let mut current = root;
    let mut parents = Vec::new();
    for step in path.steps() {
        match step {
            NavigationStep::Parent => {
                let Some(parent) = parents.pop() else {
                    return Ok((None, true));
                };
                current = parent;
            }
            NavigationStep::Property(name) => {
                let properties = context.registry().properties_for(current)?;
                let Some(property) = properties
                    .property(name)
                    .filter(|property| property.is_readable())
                else {
                    return Ok((None, false));
                };
                let next = if follow_entity_bindings
                    && let Some(reference) = property.field().and_then(FieldMetadata::reference)
                {
                    resolve_declared_target(reference.target(), context.registry())
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
                        Some(descriptor) => context.metadata_for(descriptor)?,
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
///
/// # Parameters
///
/// - `owner`: Model whose dependency declaration is being checked.
/// - `dependency`: Declared object path and property dependency.
/// - `context`: Registry and resolution-local metadata context.
/// - `source`: Originating metadata fragment, when available.
/// - `errors`: Collection receiving a diagnostic for an unresolved target.
pub(super) fn validate_dependency(
    owner: &'static TypeMetadata,
    dependency: &DependencyBindingMetadata,
    context: &ResolutionContext,
    source: Option<&FragmentIdentity>,
    errors: &mut Vec<ResolveError>,
) {
    let resolved = resolve_object_path(owner, &dependency.object_path(), context, false);
    let property = dependency.property();
    let failure = match resolved {
        Ok((_, true)) => return,
        Ok((Some(target), false)) => match resolve_property_path(target, &property, context) {
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
