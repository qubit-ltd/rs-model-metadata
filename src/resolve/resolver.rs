// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// qubit-style: allow multiple-public-types
//! Explicit cross-model resolution and immutable resolved views.

use std::collections::HashMap;
use std::collections::HashSet;

use super::error::ResolveError;
use super::error::ResolveErrorKind;
use super::error::ResolveErrors;
use super::graph::ModelGraph;
use super::graph::ResolvedProjectionProducer;
use super::graph::ResolvedProjectionSource;
use super::graph::ResolvedReference;
use super::graph::pointer_key;
use super::queries::build_query;
use super::reference_selection_ext::ReferenceSelectionExt;
use super::relations::declared_target_id;
use super::relations::forbidden_entity_nested_role;
use super::relations::metadata_for_descriptor;
use super::relations::push_field_error;
use super::relations::reported_metadata;
use super::relations::resolve_property_path;
use super::relations::validate_value_closure;
use crate::metadata::DeclaredEntityTarget;
use crate::metadata::ModelRole;
use crate::metadata::ProjectionMetadata;
use crate::metadata::PropertyPath;
use crate::metadata::PropertyResolutionError;
use crate::metadata::ReferenceSelection;
use crate::metadata::TypeMetadata;
use crate::registry::ModelRegistry;

/// Inputs used for one complete resolution attempt.
#[derive(Clone, Copy)]
pub struct ResolveInputs<'a> {
    /// Registry containing concrete and generic model registrations.
    pub models: &'a ModelRegistry<'a>,
    /// Additional concrete roots, including anonymous and generic models.
    pub roots: &'a [&'static TypeMetadata],
}

/// Resolves declaration-only metadata against explicit registries.
pub struct StructureResolver<'a> {
    /// Explicit registries used by this resolver.
    inputs: ResolveInputs<'a>,
}

impl<'a> StructureResolver<'a> {
    /// Creates a resolver without consulting process globals.
    #[must_use]
    pub const fn new(inputs: ResolveInputs<'a>) -> Self {
        Self { inputs }
    }

    /// Resolves model structure and property capabilities.
    ///
    /// # Errors
    ///
    /// Returns [`ResolveErrors`] when any property, relationship, role,
    /// projection, value-closure, or query invariant cannot be
    /// resolved against the configured registries.
    /// Validation declarations remain available from the metadata and are
    /// bound later by `ValidationPlan` when the `validation` feature is
    /// enabled.
    #[must_use = "handle all model structure resolution failures"]
    pub fn resolve(self) -> Result<ModelGraph<'a>, ResolveErrors> {
        self.resolve_internal()
    }

    /// Resolves all registered models and accumulates deterministic failures.
    fn resolve_internal(&self) -> Result<ModelGraph<'a>, ResolveErrors> {
        let mut references = HashMap::new();
        let mut dependencies = Vec::new();
        let mut projection_sources = HashMap::new();
        let mut queries = HashMap::new();
        let mut properties = HashMap::new();
        let mut projection_producers = Vec::new();
        let mut errors = Vec::new();

        let mut nodes: Vec<_> = self
            .inputs
            .models
            .concrete_entries()
            .map(|(metadata, source)| (metadata, Some(source)))
            .collect();
        let mut seen: HashSet<_> = nodes.iter().map(|(metadata, _)| metadata.type_id()).collect();
        for &root in self.inputs.roots {
            if seen.insert(root.type_id()) {
                nodes.push((root, None));
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
                if let Some(descriptor) = field.descriptor() {
                    pending.push(descriptor);
                }
                if let Some(reference) = field.reference()
                    && let Some(target) = self.resolve_target(reference.target())
                    && seen.insert(target.type_id())
                {
                    nodes.push((target, None));
                }
            }
            while let Some(descriptor) = pending.pop() {
                if !descriptors.insert(descriptor.type_id()) {
                    continue;
                }
                match self.inputs.models.metadata_for(descriptor) {
                    Ok(Some(nested)) => {
                        if seen.insert(nested.type_id()) {
                            nodes.push((nested, None));
                        }
                    }
                    Ok(None) => {}
                    Err(error) => errors.push(ResolveError::resolution(metadata, None, source, error)),
                }
                let element = descriptor
                    .as_optional()
                    .map(|value| value.element_type())
                    .or_else(|| descriptor.as_sequence().map(|value| value.element_type()))
                    .or_else(|| descriptor.as_set().map(|value| value.element_type()))
                    .or_else(|| descriptor.as_array().map(|value| value.element_type()))
                    .or_else(|| descriptor.as_smart_pointer().map(|value| value.pointee_type()));
                if let Some(element) = element.and_then(|value| value.as_resolved()) {
                    pending.push(element);
                }
                if let Some(map) = descriptor.as_map() {
                    pending.extend(
                        [map.key_type(), map.value_type()]
                            .into_iter()
                            .filter_map(|value| value.as_resolved()),
                    );
                }
                if let Some(tuple) = descriptor.as_tuple() {
                    pending.extend(tuple.elements().iter().filter_map(|value| value.as_resolved()));
                }
            }
        }
        for &(metadata, fragment_source) in &nodes {
            let first_error = errors.len();
            match self.inputs.models.properties_for(metadata) {
                Ok(local) => {
                    properties.insert(metadata.type_id(), local);
                }
                Err(PropertyResolutionError::Assembly(build_errors)) => {
                    for error in build_errors.errors() {
                        let segments = [error.property_name()];
                        let path = PropertyPath::new(&segments);
                        errors.push(
                            ResolveError::new(
                                ResolveErrorKind::InvalidProperties,
                                metadata.model_id().map(|id| id.as_str()),
                                Some(path),
                                None,
                                Some(metadata.role()),
                                fragment_source,
                            )
                            .with_cause(PropertyResolutionError::Assembly(build_errors).into()),
                        );
                    }
                }
                Err(error) => errors.push(ResolveError::resolution(metadata, None, fragment_source, error)),
            }
            let variant_fields = metadata
                .as_enum()
                .into_iter()
                .flat_map(|enumeration| enumeration.variants())
                .flat_map(|variant| variant.fields());
            for field in metadata.fields().iter().chain(variant_fields) {
                let selectors = field
                    .sequence_constraint()
                    .and_then(|value| value.element())
                    .into_iter()
                    .chain(field.map_constraint().and_then(|value| value.key()))
                    .chain(field.map_constraint().and_then(|value| value.value()));
                for validator in field
                    .validators()
                    .iter()
                    .chain(selectors.flat_map(|selector| selector.validators()))
                {
                    for declaration in validator.dependency_bindings() {
                        let context = if declaration.object_path().requires_parent() {
                            super::graph::ContextRequirement::ParentObject
                        } else {
                            super::graph::ContextRequirement::None
                        };
                        super::relations::validate_dependency(
                            metadata,
                            declaration,
                            self.inputs.models,
                            fragment_source,
                            &mut errors,
                        );
                        dependencies.push(super::graph::ResolvedDependency { declaration, context });
                    }
                }

                let field_segments = field.name().map(|name| [name]);
                let field_path = field_segments.as_ref().map(|segments| PropertyPath::new(segments));
                if field.is_opaque() {
                    let hidden = match field.type_ref().as_resolved() {
                        Some(descriptor) => match metadata_for_descriptor(descriptor, self.inputs.models) {
                            Ok(metadata) => metadata,
                            Err(error) => {
                                errors.push(
                                    (ResolveError::resolution(metadata, field_path, fragment_source, error))
                                        .with_declaration(*field.declaration()),
                                );
                                None
                            }
                        },
                        None => field
                            .type_ref()
                            .as_opaque()
                            .and_then(|opaque| self.inputs.models.by_type_id(opaque.type_id())),
                    };
                    if let Some(hidden) = hidden.filter(|hidden| {
                        matches!(
                            hidden.role(),
                            ModelRole::Entity | ModelRole::Projection | ModelRole::Model
                        )
                    }) {
                        push_field_error(
                            &mut errors,
                            ResolveErrorKind::OpaqueModel,
                            metadata,
                            field,
                            Some(hidden.role()),
                            fragment_source,
                        );
                    }
                } else if matches!(metadata.role(), ModelRole::Entity | ModelRole::Enum)
                    && field.reference().is_none()
                    && let Some(descriptor) = field.descriptor()
                {
                    match forbidden_entity_nested_role(descriptor, self.inputs.models) {
                        Ok(Some(role)) => push_field_error(
                            &mut errors,
                            ResolveErrorKind::InvalidEntityNesting,
                            metadata,
                            field,
                            Some(role),
                            fragment_source,
                        ),
                        Ok(None) => {}
                        Err(error) => errors.push(
                            (ResolveError::resolution(metadata, field_path, fragment_source, error))
                                .with_declaration(*field.declaration()),
                        ),
                    }
                }
                super::relations::validate_unique_scope(
                    metadata,
                    field,
                    self.inputs.models,
                    fragment_source,
                    &mut errors,
                );
                if let Some(reference) = field.reference() {
                    let mut local_reference_valid = true;
                    if let Some(navigation) = reference.path() {
                        match super::relations::resolve_object_binding(metadata, navigation, self.inputs.models) {
                            Ok((_, true)) => {}
                            Ok((Some(binding), false)) => {
                                if let Some(target) = self.resolve_target(reference.target())
                                    && binding.type_id() != target.type_id()
                                {
                                    errors.push(
                                        (ResolveError::new(
                                            ResolveErrorKind::TypeMismatch,
                                            metadata.model_id().map(|id| id.as_str()),
                                            field_path,
                                            Some(ModelRole::Entity),
                                            Some(binding.role()),
                                            fragment_source,
                                        )
                                        .with_types(target.type_id(), binding.type_id())
                                        .with_object_path(navigation))
                                        .with_declaration(*field.declaration()),
                                    );
                                    local_reference_valid = false;
                                }
                            }
                            Ok((None, false)) => {
                                errors.push(
                                    (ResolveError::new(
                                        ResolveErrorKind::MissingProperty,
                                        metadata.model_id().map(|id| id.as_str()),
                                        field_path,
                                        None,
                                        Some(metadata.role()),
                                        fragment_source,
                                    )
                                    .with_object_path(navigation))
                                    .with_declaration(*field.declaration()),
                                );
                                local_reference_valid = false;
                            }
                            Err(cause) => {
                                errors.push(
                                    (ResolveError::resolution(metadata, field_path, fragment_source, cause)
                                        .with_object_path(navigation))
                                    .with_declaration(*field.declaration()),
                                );
                                local_reference_valid = false;
                            }
                        }
                    }
                    match self.resolve_target(reference.target()) {
                        Some(target) if target.role() == ModelRole::Entity => {
                            let property = match reference.selection() {
                                ReferenceSelection::Entity => None,
                                ReferenceSelection::Property(path) => {
                                    match resolve_property_path(target, path, self.inputs.models) {
                                        Ok(Some(property)) if property.is_readable() => Some(property),
                                        Ok(Some(_)) => {
                                            errors.push(
                                                (ResolveError::new(
                                                    ResolveErrorKind::UnreadableProperty,
                                                    target.model_id().map(|id| id.as_str()),
                                                    Some(*path),
                                                    Some(ModelRole::Entity),
                                                    Some(target.role()),
                                                    fragment_source,
                                                ))
                                                .with_declaration(*field.declaration()),
                                            );
                                            continue;
                                        }
                                        Ok(None) => {
                                            errors.push(
                                                (ResolveError::new(
                                                    ResolveErrorKind::MissingProperty,
                                                    target.model_id().map(|id| id.as_str()),
                                                    Some(*path),
                                                    Some(ModelRole::Entity),
                                                    Some(target.role()),
                                                    fragment_source,
                                                ))
                                                .with_declaration(*field.declaration()),
                                            );
                                            continue;
                                        }

                                        Err(error) => {
                                            errors.push(
                                                (ResolveError::resolution(
                                                    metadata,
                                                    Some(*path),
                                                    fragment_source,
                                                    error,
                                                ))
                                                .with_declaration(*field.declaration()),
                                            );
                                            continue;
                                        }
                                    }
                                }
                            };
                            let expected = match property {
                                Some(property) => property.descriptor(),
                                None => Some(target.descriptor()),
                            };
                            if let (Some(expected), Some(actual)) = (expected, field.descriptor())
                                && !super::relations::reference_value_matches(expected, actual)
                            {
                                errors.push(
                                    (ResolveError::new(
                                        ResolveErrorKind::TypeMismatch,
                                        target.model_id().map(|id| id.as_str()),
                                        reference.selection().property_path().copied(),
                                        Some(ModelRole::Entity),
                                        Some(target.role()),
                                        fragment_source,
                                    )
                                    .with_types(expected.type_id(), actual.type_id()))
                                    .with_declaration(*field.declaration()),
                                );
                                continue;
                            }
                            if local_reference_valid {
                                references.insert(
                                    pointer_key(field),
                                    ResolvedReference {
                                        declaration: reference,
                                        target,
                                        property,
                                    },
                                );
                            }
                        }
                        Some(target) => errors.push(
                            (ResolveError::new(
                                ResolveErrorKind::WrongModelRole,
                                target.model_id().map(|id| id.as_str()),
                                None,
                                Some(ModelRole::Entity),
                                Some(target.role()),
                                fragment_source,
                            ))
                            .with_declaration(*field.declaration()),
                        ),
                        None => errors.push(
                            (ResolveError::new(
                                ResolveErrorKind::MissingModelId,
                                declared_target_id(reference.target()),
                                None,
                                Some(ModelRole::Entity),
                                None,
                                fragment_source,
                            ))
                            .with_declaration(*field.declaration()),
                        ),
                    }
                }
            }

            if let Some(projection) = metadata.as_projection()
                && let Some(source) = projection.source()
            {
                match self.resolve_target(source) {
                    Some(target) if target.role() == ModelRole::Entity => {
                        if let (Some(expected), Some(actual)) = (
                            target.as_entity().and_then(|entity| entity.identifier().descriptor()),
                            projection.identifier().descriptor(),
                        ) && !super::relations::reference_value_matches(expected, actual)
                        {
                            errors.push(
                                ResolveError::new(
                                    ResolveErrorKind::TypeMismatch,
                                    target.model_id().map(|id| id.as_str()),
                                    None,
                                    Some(ModelRole::Entity),
                                    Some(target.role()),
                                    fragment_source,
                                )
                                .with_types(expected.type_id(), actual.type_id()),
                            );
                            continue;
                        }
                        projection_sources.insert(
                            projection as *const ProjectionMetadata as usize,
                            ResolvedProjectionSource { target },
                        );
                    }
                    Some(target) => errors.push(ResolveError::new(
                        ResolveErrorKind::WrongModelRole,
                        target.model_id().map(|id| id.as_str()),
                        None,
                        Some(ModelRole::Entity),
                        Some(target.role()),
                        fragment_source,
                    )),
                    None => errors.push(ResolveError::new(
                        ResolveErrorKind::InvalidProjectionSource,
                        declared_target_id(source),
                        None,
                        Some(ModelRole::Entity),
                        None,
                        fragment_source,
                    )),
                }
            }

            if metadata.role() == ModelRole::Value {
                let mut visited = HashSet::new();
                validate_value_closure(metadata, self.inputs.models, &mut visited, fragment_source, &mut errors);
            }

            for error in &mut errors[first_error..] {
                error.attach_owner(metadata);
            }
            if let Some(entity) = metadata.as_entity() {
                let query = build_query(metadata);
                queries.insert(entity as *const crate::metadata::EntityMetadata as usize, query);
            }
        }

        for &(source, fragment_source) in &nodes {
            if source.role() != ModelRole::Entity {
                continue;
            }
            let Some(local_properties) = properties.get(&source.type_id()).copied() else {
                continue;
            };
            for property in local_properties.properties() {
                let Some(getter) = property.getter() else {
                    continue;
                };
                let Some(projection) = property
                    .descriptor()
                    .and_then(|descriptor| {
                        reported_metadata(
                            descriptor,
                            self.inputs.models,
                            source,
                            Some(PropertyPath::new(&[property.name()])),
                            fragment_source,
                            &mut errors,
                        )
                    })
                    .filter(|metadata| metadata.role() == ModelRole::Projection)
                else {
                    continue;
                };
                let fixed_source = projection
                    .as_projection()
                    .and_then(ProjectionMetadata::source)
                    .and_then(|target| self.resolve_target(target));
                if fixed_source.is_some_and(|fixed| fixed.type_id() != source.type_id()) {
                    errors.push(ResolveError::new(
                        ResolveErrorKind::InvalidProjectionProducer,
                        projection.model_id().map(|id| id.as_str()),
                        None,
                        Some(ModelRole::Entity),
                        Some(source.role()),
                        fragment_source,
                    ));
                    continue;
                }
                let source_id = source.as_entity().and_then(|entity| entity.identifier().descriptor());
                let projection_id = projection
                    .as_projection()
                    .and_then(|projection| projection.identifier().descriptor());
                if source_id
                    .zip(projection_id)
                    .is_some_and(|(source, projection)| source.type_id() != projection.type_id())
                {
                    errors.push(ResolveError::new(
                        ResolveErrorKind::InvalidProjectionProducer,
                        projection.model_id().map(|id| id.as_str()),
                        None,
                        Some(ModelRole::Projection),
                        Some(projection.role()),
                        fragment_source,
                    ));
                    continue;
                }
                projection_producers.push(ResolvedProjectionProducer {
                    source,
                    projection,
                    property,
                    projector: Some(getter),
                });
            }
        }

        if errors.is_empty() {
            Ok(ModelGraph {
                registry: self.inputs.models,
                dependencies,
                models: nodes.iter().map(|(metadata, _)| *metadata).collect(),
                references,
                projection_sources,
                queries,
                properties,
                projection_producers,
            })
        } else {
            errors.sort_by(ResolveError::compare);
            Err(ResolveErrors { errors })
        }
    }

    /// Resolves a declaration-time target through the configured model
    /// registry.
    fn resolve_target(&self, target: &DeclaredEntityTarget) -> Option<&'static TypeMetadata> {
        super::relations::resolve_declared_target(target, self.inputs.models)
    }
}
