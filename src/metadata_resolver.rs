// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// qubit-style: allow multiple-public-types
//! Explicit cross-model resolution and immutable resolved views.

#[cfg(test)]
mod tests;

use std::any::TypeId;
use std::collections::HashMap;
use std::collections::HashSet;

use qubit_codec::ValueCodecDescriptor;
use qubit_codec::ValueCodecRegistration;
use qubit_codec::ValueCodecRegistry;
use qubit_id::Id;
use qubit_reflect::FieldAccessError;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::descriptor::TypeRef;
use qubit_reflect::identity::FragmentIdentity;

use crate::CodecMetadata;
use crate::DeclaredEntityTarget;
use crate::FieldMetadata;
use crate::FieldReferenceMetadata;
use crate::GetterMetadata;
use crate::IndexingReasons;
use crate::LocalPropertySet;
use crate::ModelMetadataError;
use crate::ModelRegistry;
use crate::ModelRole;
use crate::ProjectionMetadata;
use crate::PropertyAccessError;
use crate::PropertyMetadata;
use crate::PropertyPath;
use crate::PropertyResolutionError;
use crate::PropertyValue;
use crate::ReferenceSelection;
use crate::ReflectedRef;
use crate::SelectorPosition;
use crate::TypeMetadata;

/// Inputs used for one complete resolution attempt.
#[derive(Clone, Copy)]
pub struct ResolveInputs<'a> {
    /// Registry containing concrete and generic model registrations.
    pub models: &'a ModelRegistry<'a>,
    /// Registry containing executable value codecs.
    pub codecs: &'a ValueCodecRegistry,
}

/// Resolves declaration-only metadata against explicit registries.
pub struct ModelResolver<'a> {
    /// Explicit registries used by this resolver.
    inputs: ResolveInputs<'a>,
}

impl<'a> ModelResolver<'a> {
    /// Creates a resolver without consulting process globals.
    #[must_use]
    pub const fn new(inputs: ResolveInputs<'a>) -> Self {
        Self { inputs }
    }

    /// Resolves every registration or returns all deterministic errors.
    ///
    /// # Errors
    ///
    /// Returns [`ModelResolveErrors`] when any property, relationship, role,
    /// codec, projection, value-closure, or query invariant cannot be
    /// resolved against the configured registries.
    #[must_use = "handle all model resolution failures"]
    pub fn resolve_all(&self) -> Result<ResolvedModelGraph<'a>, ModelResolveErrors> {
        self.resolve_internal()
    }

    /// Resolves model structure and property capabilities without requiring
    /// validator registrations. Validation declarations remain available from
    /// the metadata and are bound later by [`crate::ValidationPlan`].
    #[cfg(feature = "validation")]
    #[must_use = "handle all model structure resolution failures"]
    pub fn resolve_structure(&self) -> Result<ResolvedModelGraph<'a>, ModelResolveErrors> {
        self.resolve_internal()
    }

    fn resolve_internal(&self) -> Result<ResolvedModelGraph<'a>, ModelResolveErrors> {
        let mut references = HashMap::new();
        let mut projection_sources = HashMap::new();
        let mut codecs = HashMap::new();
        let mut queries = HashMap::new();
        let mut properties = HashMap::new();
        let mut projection_producers = Vec::new();
        let mut errors = Vec::new();

        for (metadata, fragment_source) in self.inputs.models.concrete_entries() {
            match self.inputs.models.properties_for(metadata) {
                Ok(local) => {
                    properties.insert(metadata.type_id(), local);
                }
                Err(PropertyResolutionError::Assembly(build_errors)) => {
                    for error in build_errors.errors() {
                        let segments = [error.property_name()];
                        let path = PropertyPath::new(&segments);
                        errors.push(
                            ModelResolveError::new(
                                ModelResolveErrorKind::InvalidProperties,
                                metadata.model_id().map(|id| id.as_str()),
                                Some(path),
                                None,
                                Some(metadata.role()),
                                Some(fragment_source),
                            )
                            .with_cause(PropertyResolutionError::Assembly(build_errors).into()),
                        );
                    }
                }
                Err(error) => errors.push(ModelResolveError::resolution(metadata, None, fragment_source, error)),
            }
            let variant_fields = metadata
                .as_enum()
                .into_iter()
                .flat_map(|enumeration| enumeration.variants())
                .flat_map(|variant| variant.fields());
            for field in metadata.fields().iter().chain(variant_fields) {
                let field_segments = field.name().map(|name| [name]);
                let field_path = field_segments.as_ref().map(|segments| PropertyPath::new(segments));
                if field.is_opaque() {
                    let hidden = match field.type_ref().as_resolved() {
                        Some(descriptor) => match metadata_for_descriptor(descriptor, self.inputs.models) {
                            Ok(metadata) => metadata,
                            Err(error) => {
                                errors.push(ModelResolveError::resolution(
                                    metadata,
                                    field_path,
                                    fragment_source,
                                    error,
                                ));
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
                            ModelResolveErrorKind::OpaqueModel,
                            metadata,
                            field,
                            Some(hidden.role()),
                            fragment_source,
                        );
                    }
                } else if metadata.role() == ModelRole::Entity
                    && field.reference().is_none()
                    && let Some(descriptor) = field.descriptor()
                {
                    match forbidden_entity_nested_role(descriptor, self.inputs.models) {
                        Ok(Some(role)) => push_field_error(
                            &mut errors,
                            ModelResolveErrorKind::InvalidEntityNesting,
                            metadata,
                            field,
                            Some(role),
                            fragment_source,
                        ),
                        Ok(None) => {}
                        Err(error) => errors.push(ModelResolveError::resolution(
                            metadata,
                            field_path,
                            fragment_source,
                            error,
                        )),
                    }
                }
                resolve_field_codecs(
                    metadata,
                    field,
                    self.inputs,
                    &mut codecs,
                    fragment_source,
                    &mut errors,
                );
                if let Some(reference) = field.reference() {
                    let mut local_reference_valid = true;
                    if let Some(path) = reference.same_as() {
                        match resolve_property_path(metadata, path, self.inputs.models) {
                            Ok(Some(property)) if property.is_readable() => {}
                            Ok(Some(_)) => {
                                errors.push(ModelResolveError::new(
                                    ModelResolveErrorKind::UnreadableProperty,
                                    metadata.model_id().map(|id| id.as_str()),
                                    Some(*path),
                                    None,
                                    Some(metadata.role()),
                                    Some(fragment_source),
                                ));
                                local_reference_valid = false;
                            }
                            Ok(None) => {
                                errors.push(ModelResolveError::new(
                                    ModelResolveErrorKind::MissingProperty,
                                    metadata.model_id().map(|id| id.as_str()),
                                    Some(*path),
                                    None,
                                    Some(metadata.role()),
                                    Some(fragment_source),
                                ));
                                local_reference_valid = false;
                            }

                            Err(error) => {
                                errors.push(ModelResolveError::resolution(
                                    metadata,
                                    Some(*path),
                                    fragment_source,
                                    error,
                                ));
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
                                            errors.push(ModelResolveError::new(
                                                ModelResolveErrorKind::UnreadableProperty,
                                                target.model_id().map(|id| id.as_str()),
                                                Some(*path),
                                                Some(ModelRole::Entity),
                                                Some(target.role()),
                                                Some(fragment_source),
                                            ));
                                            continue;
                                        }
                                        Ok(None) => {
                                            errors.push(ModelResolveError::new(
                                                ModelResolveErrorKind::MissingProperty,
                                                target.model_id().map(|id| id.as_str()),
                                                Some(*path),
                                                Some(ModelRole::Entity),
                                                Some(target.role()),
                                                Some(fragment_source),
                                            ));
                                            continue;
                                        }

                                        Err(error) => {
                                            errors.push(ModelResolveError::resolution(
                                                metadata,
                                                Some(*path),
                                                fragment_source,
                                                error,
                                            ));
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
                                && expected.type_id() != actual.type_id()
                            {
                                errors.push(
                                    ModelResolveError::new(
                                        ModelResolveErrorKind::TypeMismatch,
                                        target.model_id().map(|id| id.as_str()),
                                        reference.selection().property_path().copied(),
                                        Some(ModelRole::Entity),
                                        Some(target.role()),
                                        Some(fragment_source),
                                    )
                                    .with_types(expected.type_id(), actual.type_id()),
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
                        Some(target) => errors.push(ModelResolveError::new(
                            ModelResolveErrorKind::WrongModelRole,
                            target.model_id().map(|id| id.as_str()),
                            None,
                            Some(ModelRole::Entity),
                            Some(target.role()),
                            Some(fragment_source),
                        )),
                        None => errors.push(ModelResolveError::new(
                            ModelResolveErrorKind::MissingModelId,
                            declared_target_id(reference.target()),
                            None,
                            Some(ModelRole::Entity),
                            None,
                            Some(fragment_source),
                        )),
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
                        ) && expected.type_id() != actual.type_id()
                        {
                            errors.push(
                                ModelResolveError::new(
                                    ModelResolveErrorKind::TypeMismatch,
                                    target.model_id().map(|id| id.as_str()),
                                    None,
                                    Some(ModelRole::Entity),
                                    Some(target.role()),
                                    Some(fragment_source),
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
                    Some(target) => errors.push(ModelResolveError::new(
                        ModelResolveErrorKind::WrongModelRole,
                        target.model_id().map(|id| id.as_str()),
                        None,
                        Some(ModelRole::Entity),
                        Some(target.role()),
                        Some(fragment_source),
                    )),
                    None => errors.push(ModelResolveError::new(
                        ModelResolveErrorKind::InvalidProjectionSource,
                        declared_target_id(source),
                        None,
                        Some(ModelRole::Entity),
                        None,
                        Some(fragment_source),
                    )),
                }
            }

            if metadata.role() == ModelRole::Value {
                let mut visited = HashSet::new();
                validate_value_closure(metadata, self.inputs.models, &mut visited, fragment_source, &mut errors);
            }

            if let Some(codec) = metadata.as_value().and_then(crate::ValueMetadata::canonical_codec) {
                resolve_codec(
                    metadata,
                    codec,
                    metadata.type_id(),
                    self.inputs.codecs,
                    &mut codecs,
                    fragment_source,
                    &mut errors,
                );
            }

            if let Some(entity) = metadata.as_entity()
                && let Some(query) = build_query(metadata, self.inputs.models, fragment_source, &mut errors)
            {
                queries.insert(entity as *const crate::EntityMetadata as usize, query);
            }
        }

        for (source, fragment_source) in self.inputs.models.concrete_entries() {
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
                    errors.push(ModelResolveError::new(
                        ModelResolveErrorKind::InvalidProjectionProducer,
                        projection.model_id().map(|id| id.as_str()),
                        None,
                        Some(ModelRole::Entity),
                        Some(source.role()),
                        Some(fragment_source),
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
                    errors.push(ModelResolveError::new(
                        ModelResolveErrorKind::InvalidProjectionProducer,
                        projection.model_id().map(|id| id.as_str()),
                        None,
                        Some(ModelRole::Projection),
                        Some(projection.role()),
                        Some(fragment_source),
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
            Ok(ResolvedModelGraph {
                registry: self.inputs.models,
                references,
                projection_sources,
                codecs,
                queries,
                properties,
                projection_producers,
            })
        } else {
            errors.sort_by(ModelResolveError::compare);
            Err(ModelResolveErrors { errors })
        }
    }

    /// Resolves a declaration-time target through the configured model
    /// registry.
    fn resolve_target(&self, target: &DeclaredEntityTarget) -> Option<&'static TypeMetadata> {
        match target {
            DeclaredEntityTarget::RustType(provider) => Some(provider()),
            DeclaredEntityTarget::ModelId(id) => self.inputs.models.metadata(id.as_str()),
        }
    }
}

/// Records an error anchored to one direct field path.
fn push_field_error(
    errors: &mut Vec<ModelResolveError>,
    kind: ModelResolveErrorKind,
    metadata: &'static TypeMetadata,
    field: &'static FieldMetadata,
    actual_role: Option<ModelRole>,
    source: &FragmentIdentity,
) {
    let path = field.name().map(|name| {
        let segments = [name];
        OwnedPropertyPath::from_segments(&segments)
    });
    let mut error = ModelResolveError::new(
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

/// Resolves field codecs while deliberately leaving validator declarations
/// for the validation-plan binding phase.
#[allow(clippy::too_many_arguments)]
fn resolve_field_codecs<'a>(
    metadata: &'static TypeMetadata,
    field: &'static FieldMetadata,
    inputs: ResolveInputs<'a>,
    codecs: &mut HashMap<usize, ResolvedCodec<'a>>,
    source: &FragmentIdentity,
    errors: &mut Vec<ModelResolveError>,
) {
    let Some(descriptor) = field.descriptor() else {
        return;
    };
    if let Some(codec) = field.codec() {
        resolve_codec(
            metadata,
            codec,
            descriptor
                .as_optional()
                .and_then(|view| runtime_type_id(view.element_type()))
                .unwrap_or_else(|| descriptor.type_id()),
            inputs.codecs,
            codecs,
            source,
            errors,
        );
    }
    let sequence_selector = field
        .sequence_constraint()
        .and_then(|sequence| sequence.element());
    let map = field.map_constraint();
    for (selector, position) in [
        (sequence_selector, SelectorPosition::Element),
        (map.and_then(|map| map.key()), SelectorPosition::MapKey),
        (map.and_then(|map| map.value()), SelectorPosition::MapValue),
    ] {
        let Some(selector) = selector else {
            continue;
        };
        let Some(expected_type) = selector_type_id(descriptor, position) else {
            errors.push(ModelResolveError::new(
                ModelResolveErrorKind::UnresolvedSelectorType,
                metadata.model_id().map(|id| id.as_str()),
                None,
                None,
                Some(metadata.role()),
                Some(source),
            ));
            continue;
        };
        if let Some(codec) = selector.codec() {
            resolve_codec(metadata, codec, expected_type, inputs.codecs, codecs, source, errors);
        }
    }
}

/// Returns the runtime type ID at a nested collection position.
fn selector_type_id(descriptor: &'static TypeDescriptor, position: SelectorPosition) -> Option<TypeId> {
    let descriptor = transparent_descriptor(descriptor)?;
    let type_ref = match position {
        SelectorPosition::Element => descriptor
            .as_sequence()
            .map(|view| view.element_type())
            .or_else(|| descriptor.as_set().map(|view| view.element_type()))
            .or_else(|| descriptor.as_array().map(|view| view.element_type()))
            .or_else(|| descriptor.as_slice().map(|view| view.element_type())),
        SelectorPosition::MapKey => descriptor.as_map().map(|view| view.key_type()),
        SelectorPosition::MapValue => descriptor.as_map().map(|view| view.value_type()),
    }?;
    runtime_type_id(type_ref)
}

/// Returns the innermost descriptor through transparent wrappers.
fn transparent_descriptor(mut descriptor: &'static TypeDescriptor) -> Option<&'static TypeDescriptor> {
    loop {
        let element = descriptor
            .as_optional()
            .map(|view| view.element_type())
            .or_else(|| descriptor.as_smart_pointer().map(|view| view.pointee_type()));
        let Some(element) = element else {
            return Some(descriptor);
        };
        descriptor = element.as_resolved()?;
    }
}

/// Returns the concrete type ID represented by a resolved or opaque reference.
fn runtime_type_id(type_ref: &TypeRef) -> Option<TypeId> {
    type_ref
        .as_resolved()
        .map(TypeDescriptor::type_id)
        .or_else(|| type_ref.as_opaque().map(|descriptor| descriptor.type_id()))
}

/// Resolves one codec occurrence against the executable codec registry.
#[allow(clippy::too_many_arguments)]
fn resolve_codec<'a>(
    metadata: &'static TypeMetadata,
    occurrence: &'static CodecMetadata,
    expected_type: TypeId,
    registry: &'a ValueCodecRegistry,
    codecs: &mut HashMap<usize, ResolvedCodec<'a>>,
    source: &FragmentIdentity,
    errors: &mut Vec<ModelResolveError>,
) {
    let (descriptor, registration) = match occurrence.codec() {
        crate::CodecReference::RustType(descriptor) => (*descriptor, None),
        crate::CodecReference::DeclaredId(id) => {
            let Some(registration) = registry.get(id) else {
                errors.push(ModelResolveError::new(
                    ModelResolveErrorKind::MissingCodec,
                    metadata.model_id().map(|model_id| model_id.as_str()),
                    None,
                    None,
                    Some(metadata.role()),
                    Some(source),
                ));
                return;
            };
            (registration.descriptor(), Some(registration))
        }
    };
    let actual_type = descriptor.value_type_id();
    if actual_type != expected_type {
        errors.push(
            ModelResolveError::new(
                ModelResolveErrorKind::CodecTypeMismatch,
                metadata.model_id().map(|id| id.as_str()),
                None,
                None,
                Some(metadata.role()),
                Some(source),
            )
            .with_types(expected_type, actual_type),
        );
        return;
    }
    codecs.insert(
        occurrence as *const CodecMetadata as usize,
        ResolvedCodec {
            declaration: occurrence,
            descriptor,
            registration,
        },
    );
}

/// Provides resolver-specific access to a selected property path.
trait ReferenceSelectionExt {
    /// Returns the selected property path for property references.
    fn property_path(&self) -> Option<&PropertyPath<'static>>;
}

impl ReferenceSelectionExt for ReferenceSelection {
    /// Returns `None` for an entity-level selection and the path otherwise.
    fn property_path(&self) -> Option<&PropertyPath<'static>> {
        match self {
            Self::Entity => None,
            Self::Property(path) => Some(path),
        }
    }
}

/// Finds model metadata attached to or registered for a descriptor.
fn metadata_for_descriptor(
    descriptor: &'static TypeDescriptor,
    registry: &ModelRegistry,
) -> Result<Option<&'static TypeMetadata>, ModelMetadataError> {
    registry.metadata_for(descriptor)
}

/// Records a metadata failure with its traversal context before returning no
/// metadata.
fn reported_metadata(
    descriptor: &'static TypeDescriptor,
    registry: &ModelRegistry,
    root: &'static TypeMetadata,
    path: Option<PropertyPath<'_>>,
    source: &FragmentIdentity,
    errors: &mut Vec<ModelResolveError>,
) -> Option<&'static TypeMetadata> {
    match metadata_for_descriptor(descriptor, registry) {
        Ok(metadata) => metadata,
        Err(error) => {
            errors.push(ModelResolveError::resolution(root, path, source, error));
            None
        }
    }
}

/// An owned runtime path whose segment names originate in static declarations.
#[derive(Clone, Debug, Eq, PartialEq)]
struct OwnedPropertyPath {
    segments: Box<[&'static str]>,
}

impl OwnedPropertyPath {
    /// Copies one runtime-generated segment sequence.
    fn from_segments(segments: &[&'static str]) -> Self {
        Self {
            segments: segments.into(),
        }
    }

    /// Copies one statically declared path.
    fn from_static(path: PropertyPath<'static>) -> Self {
        Self::from_segments(path.segments())
    }

    /// Borrows this owned path as the public lightweight view.
    fn as_path(&self) -> PropertyPath<'_> {
        PropertyPath::new(&self.segments)
    }
}

/// Builds indexed query metadata for one entity.
fn build_query(
    metadata: &'static TypeMetadata,
    registry: &ModelRegistry,
    source: &FragmentIdentity,
    errors: &mut Vec<ModelResolveError>,
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
                        Ok(Some(_)) => errors.push(ModelResolveError::new(
                            ModelResolveErrorKind::UnreadableProperty,
                            metadata.model_id().map(|id| id.as_str()),
                            Some(*scope),
                            None,
                            Some(metadata.role()),
                            Some(source),
                        )),
                        Ok(None) => errors.push(ModelResolveError::new(
                            ModelResolveErrorKind::MissingProperty,
                            metadata.model_id().map(|id| id.as_str()),
                            Some(*scope),
                            None,
                            Some(metadata.role()),
                            Some(source),
                        )),

                        Err(error) => {
                            errors.push(ModelResolveError::resolution(metadata, Some(*scope), source, error));
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
            errors.push(ModelResolveError::new(
                ModelResolveErrorKind::InvalidValueClosure,
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
    errors: &mut Vec<ModelResolveError>,
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
    errors: &mut Vec<ModelResolveError>,
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
            collect_query_fields(target, path, false, registry, filters, flat_names, root, source, errors)
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
    errors: &mut Vec<ModelResolveError>,
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
        errors.push(ModelResolveError::new(
            ModelResolveErrorKind::QueryNameConflict,
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

/// Resolves a declared target using either its provider or stable model ID.
fn resolve_declared_target(target: &DeclaredEntityTarget, registry: &ModelRegistry) -> Option<&'static TypeMetadata> {
    match target {
        DeclaredEntityTarget::RustType(provider) => Some(provider()),
        DeclaredEntityTarget::ModelId(id) => registry.metadata(id.as_str()),
    }
}

/// Finds an Entity or Projection nested through ordinary container structure.
fn forbidden_entity_nested_role(
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
fn path_from_segments(segments: &[&'static str]) -> OwnedPropertyPath {
    OwnedPropertyPath::from_segments(segments)
}

/// Verifies that a value model contains only closed value types.
fn validate_value_closure(
    metadata: &'static TypeMetadata,
    registry: &ModelRegistry,
    visited: &mut HashSet<TypeId>,
    source: &FragmentIdentity,
    errors: &mut Vec<ModelResolveError>,
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
    errors: &mut Vec<ModelResolveError>,
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
                errors.push(ModelResolveError::new(
                    ModelResolveErrorKind::InvalidValueClosure,
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
    errors: &mut Vec<ModelResolveError>,
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
    errors: &mut Vec<ModelResolveError>,
    path: &[&'static str],
) -> bool {
    let metadata = match metadata_for_descriptor(descriptor, registry) {
        Ok(metadata) => metadata,
        Err(error) => {
            errors.push(ModelResolveError::resolution(
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
fn resolve_property_path(
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

/// A successfully resolved direct reference.
#[derive(Debug)]
pub struct ResolvedReference {
    /// The original field-reference declaration.
    declaration: &'static FieldReferenceMetadata,
    /// The resolved target model metadata.
    target: &'static TypeMetadata,
    /// The selected target property, or `None` for an entity-level reference.
    property: Option<&'static PropertyMetadata>,
}

impl ResolvedReference {
    /// Returns the original field-reference declaration.
    #[must_use]
    pub const fn declaration(&self) -> &'static FieldReferenceMetadata {
        self.declaration
    }

    /// Returns the resolved target model metadata.
    #[must_use]
    #[inline(always)]
    pub const fn target(&self) -> &'static TypeMetadata {
        self.target
    }

    /// Returns the selected target property, or `None` for an entity reference.
    #[must_use]
    #[inline(always)]
    pub const fn property(&self) -> Option<&'static PropertyMetadata> {
        self.property
    }
}

/// A successfully resolved Projection source.
#[derive(Debug)]
pub struct ResolvedProjectionSource {
    /// The resolved entity model supplying the projection.
    target: &'static TypeMetadata,
}

/// One resolved readable property that produces a Projection from an Entity.
#[derive(Clone, Copy, Debug)]
pub struct ResolvedProjectionProducer {
    /// Entity declaring the readable property.
    source: &'static TypeMetadata,
    /// Projection returned by the property getter.
    projection: &'static TypeMetadata,
    /// Merged local property that declares the producer edge.
    property: &'static PropertyMetadata,
    /// Executable getter adapter, when automatic projection is available.
    projector: Option<&'static GetterMetadata>,
}

impl ResolvedProjectionProducer {
    /// Returns the producing Entity metadata.
    #[must_use]
    #[inline(always)]
    pub const fn source(&self) -> &'static TypeMetadata {
        self.source
    }

    /// Returns the produced Projection metadata.
    #[must_use]
    #[inline(always)]
    pub const fn projection(&self) -> &'static TypeMetadata {
        self.projection
    }

    /// Returns the property that declares this edge.
    #[must_use]
    #[inline(always)]
    pub const fn property(&self) -> &'static PropertyMetadata {
        self.property
    }

    /// Returns the executable getter used as projector.
    #[must_use]
    #[inline(always)]
    pub const fn projector(&self) -> Option<&'static GetterMetadata> {
        self.projector
    }

    /// Executes the projector and verifies identifier preservation.
    ///
    /// # Errors
    ///
    /// Returns a structured adapter, field-access, or identifier error.
    #[must_use = "handle projection execution failure"]
    pub fn project<'a>(&self, source: ReflectedRef<'a>) -> Result<PropertyValue<'a>, ProjectionExecutionError> {
        let projector = self.projector.ok_or(ProjectionExecutionError::MissingProjector)?;
        let source_identifier = self
            .source
            .as_entity()
            .ok_or(ProjectionExecutionError::InvalidProducer)?
            .identifier()
            .reflect()
            .expect("resolved entity identifiers are concrete fields")
            .get(source.clone())?
            .downcast::<Id>()
            .map_err(|_| ProjectionExecutionError::InvalidIdentifierType)
            .copied()?;
        let result = projector.get(source)?;
        let projection_identifier = match &result {
            PropertyValue::Borrowed(value) => self.projection_identifier(value.clone())?,
            PropertyValue::Owned(value) => self.projection_identifier(value.as_reflected_ref())?,
            PropertyValue::OptionalBorrowed(_) | PropertyValue::BorrowedSlice(_) => {
                return Err(ProjectionExecutionError::InvalidProducer);
            }
        };
        if source_identifier != projection_identifier {
            return Err(ProjectionExecutionError::IdentifierMismatch);
        }
        Ok(result)
    }

    /// Reads and validates the identifier from a projected target.
    ///
    /// # Errors
    ///
    /// Returns [`ProjectionExecutionError`] when the target is not a valid
    /// Projection or its identifier field cannot be read as `qubit_id::Id`.
    fn projection_identifier(&self, target: ReflectedRef<'_>) -> Result<Id, ProjectionExecutionError> {
        self.projection
            .as_projection()
            .ok_or(ProjectionExecutionError::InvalidProducer)?
            .identifier()
            .reflect()
            .expect("resolved projection identifiers are concrete fields")
            .get(target)?
            .downcast::<Id>()
            .map_err(|_| ProjectionExecutionError::InvalidIdentifierType)
            .copied()
    }
}

/// Failure while executing an automatic Projection producer.
#[must_use]
#[derive(Debug, thiserror::Error)]
pub enum ProjectionExecutionError {
    /// No executable adapter is registered for this producer.
    #[error("projection producer has no executable projector")]
    MissingProjector,
    /// The resolved edge no longer has the required Entity/Projection shape.
    #[error("projection producer metadata is invalid")]
    InvalidProducer,
    /// An identifier field did not contain the exact `qubit_id::Id` type.
    #[error("projection identifier has an invalid Rust type")]
    InvalidIdentifierType,
    /// The produced Projection changed the source Entity identifier.
    #[error("projection identifier differs from its source entity")]
    IdentifierMismatch,
    /// A property getter failed.
    #[error("projection property access failed: {0}")]
    Property(#[from] PropertyAccessError),
    /// A reflected identifier field could not be read.
    #[error("projection identifier field access failed: {0}")]
    Field(#[from] FieldAccessError),
}

impl ResolvedProjectionSource {
    /// Returns the resolved entity model supplying the projection.
    #[must_use]
    #[inline(always)]
    pub const fn target(&self) -> &'static TypeMetadata {
        self.target
    }
}

/// Immutable result of a complete successful resolution pass.
#[must_use]
#[derive(Debug)]
pub struct ResolvedModelGraph<'a> {
    /// The registry used to resolve the graph.
    registry: &'a ModelRegistry<'a>,
    /// Resolved field references keyed by declaration identity.
    references: HashMap<usize, ResolvedReference>,
    /// Resolved projection sources keyed by declaration identity.
    projection_sources: HashMap<usize, ResolvedProjectionSource>,
    /// Resolved codecs keyed by declaration identity.
    codecs: HashMap<usize, ResolvedCodec<'a>>,
    /// Resolved query metadata keyed by entity declaration identity.
    queries: HashMap<usize, QueryMetadata>,
    properties: HashMap<TypeId, &'static LocalPropertySet>,
    projection_producers: Vec<ResolvedProjectionProducer>,
}

impl<'a> ResolvedModelGraph<'a> {
    /// Returns locally merged properties accepted during graph resolution.
    #[must_use]
    pub fn properties(&self, model: &TypeMetadata) -> Option<&'static LocalPropertySet> {
        self.properties.get(&model.type_id()).copied()
    }

    /// Returns all resolved Entity-to-Projection producer edges.
    #[must_use]
    #[inline(always)]
    pub fn projection_producers(&self) -> &[ResolvedProjectionProducer] {
        &self.projection_producers
    }
    /// Returns the registry used for this resolution pass.
    #[must_use]
    #[inline(always)]
    pub const fn registry(&self) -> &'a ModelRegistry<'a> {
        self.registry
    }

    /// Returns a resolved reference for `field`, or `None` when it has none.
    #[must_use]
    pub fn reference(&self, field: &FieldMetadata) -> Option<&ResolvedReference> {
        self.references.get(&pointer_key(field))
    }

    /// Returns a resolved source for `projection`, or `None` when it is open.
    #[must_use]
    pub fn projection_source(&self, projection: &ProjectionMetadata) -> Option<&ResolvedProjectionSource> {
        self.projection_sources
            .get(&(projection as *const ProjectionMetadata as usize))
    }

    /// Returns a resolved codec occurrence, or `None` when not declared.
    #[must_use]
    pub fn codec(&self, occurrence: &CodecMetadata) -> Option<&ResolvedCodec<'a>> {
        self.codecs.get(&(occurrence as *const CodecMetadata as usize))
    }

    /// Returns query metadata for `entity`, or `None` when it is not resolved.
    #[must_use]
    pub fn query(&self, entity: &crate::EntityMetadata) -> Option<&QueryMetadata> {
        self.queries.get(&(entity as *const crate::EntityMetadata as usize))
    }
}

/// A codec occurrence bound to one executable descriptor.
#[derive(Debug)]
pub struct ResolvedCodec<'a> {
    /// The declaration occurrence resolved by this entry.
    declaration: &'static CodecMetadata,
    /// The executable descriptor selected for the declaration.
    descriptor: &'static ValueCodecDescriptor,
    /// The registry entry for a stable-ID declaration, if used.
    registration: Option<&'a ValueCodecRegistration>,
}

impl ResolvedCodec<'_> {
    /// Returns the declaration occurrence.
    #[must_use]
    #[inline(always)]
    pub const fn declaration(&self) -> &'static CodecMetadata {
        self.declaration
    }

    /// Returns the executable codec descriptor.
    #[must_use]
    #[inline(always)]
    pub const fn descriptor(&self) -> &'static ValueCodecDescriptor {
        self.descriptor
    }

    /// Returns the registry entry for stable-ID declarations.
    #[must_use]
    #[inline(always)]
    pub const fn registration(&self) -> Option<&ValueCodecRegistration> {
        self.registration
    }
}

/// Queryable indexed fields derived for one resolved entity.
#[derive(Debug)]
pub struct QueryMetadata {
    /// Indexed field paths that can be used as query filters.
    filters: Box<[QueryField]>,
    /// Identifier and globally unique lookup keys.
    unique_keys: Box<[UniqueQueryKey]>,
}

impl QueryMetadata {
    /// Returns queryable indexed fields in deterministic path order.
    #[must_use]
    #[inline(always)]
    pub fn filters(&self) -> &[QueryField] {
        &self.filters
    }

    /// Returns identifier and globally unique keys in deterministic order.
    #[must_use]
    #[inline(always)]
    pub fn unique_keys(&self) -> &[UniqueQueryKey] {
        &self.unique_keys
    }

    /// Finds a queryable field by its complete property path.
    #[must_use]
    pub fn filter(&self, path: &PropertyPath<'_>) -> Option<&QueryField> {
        self.filters.iter().find(|field| field.path.as_path() == *path)
    }

    /// Finds a queryable field by its flattened external name.
    #[must_use]
    pub fn filter_by_flat_name(&self, name: &str) -> Option<&QueryField> {
        self.filters.iter().find(|field| field.flat_name.as_ref() == name)
    }
}

/// One queryable field path.
#[derive(Clone, Debug)]
pub struct QueryField {
    /// The complete property path represented by this field.
    path: OwnedPropertyPath,
    /// The flattened external name used for queries.
    flat_name: Box<str>,
    /// The resolved descriptor, or `None` for an opaque type.
    descriptor: Option<&'static TypeDescriptor>,
    /// The declaration facts that made the field queryable.
    reasons: IndexingReasons,
}

impl QueryField {
    /// Returns the complete property path.
    #[must_use]
    #[inline(always)]
    pub fn path(&self) -> PropertyPath<'_> {
        self.path.as_path()
    }

    /// Returns the flattened external name used for queries.
    #[must_use]
    #[inline(always)]
    pub fn flat_name(&self) -> &str {
        &self.flat_name
    }

    /// Returns the resolved descriptor, or `None` for an opaque type.
    #[must_use]
    #[inline(always)]
    pub const fn descriptor(&self) -> Option<&'static TypeDescriptor> {
        self.descriptor
    }

    /// Returns the declaration facts that made the field queryable.
    #[must_use]
    #[inline(always)]
    pub const fn reasons(&self) -> IndexingReasons {
        self.reasons
    }
}

/// One identifier or global-unique lookup key.
#[derive(Clone, Debug)]
pub struct UniqueQueryKey {
    /// The property paths that form this lookup key.
    paths: Box<[OwnedPropertyPath]>,
}

impl UniqueQueryKey {
    /// Creates a lookup key from one or more owned property paths.
    fn new(paths: Vec<OwnedPropertyPath>) -> Self {
        Self {
            paths: paths.into_boxed_slice(),
        }
    }

    /// Iterates over property paths in key-component order.
    #[must_use]
    pub fn paths(&self) -> impl ExactSizeIterator<Item = PropertyPath<'_>> + '_ {
        self.paths.iter().map(OwnedPropertyPath::as_path)
    }

    /// Returns the sole path, or `None` when this key is composite.
    #[must_use]
    pub fn path(&self) -> Option<PropertyPath<'_>> {
        (self.paths.len() == 1).then(|| self.paths[0].as_path())
    }
}

/// Machine-readable model resolution error class.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ModelResolveErrorKind {
    /// A reflected model capability or its ABI could not be resolved.
    MetadataResolution,
    /// Property capabilities could not be resolved.
    PropertyResolution,
    /// Field and method fragments could not form a valid local property set.
    InvalidProperties,
    /// An Entity embeds another Entity or Projection without a reference.
    InvalidEntityNesting,
    /// An opaque field attempts to hide a registered model role.
    OpaqueModel,
    /// A readable Entity property violates a Projection source contract.
    InvalidProjectionProducer,
    /// The target model ID could not be resolved.
    MissingModelId,
    /// The resolved model has a role incompatible with the declaration.
    WrongModelRole,
    /// A referenced property does not exist.
    MissingProperty,
    /// A referenced property exists but cannot be read.
    UnreadableProperty,
    /// The expected and actual types differ.
    TypeMismatch,
    /// A declared codec is not registered.
    MissingCodec,
    /// A codec registration has the wrong value type.
    CodecTypeMismatch,
    /// A nested selector type cannot be resolved.
    UnresolvedSelectorType,
    /// A projection source is missing or has the wrong role.
    InvalidProjectionSource,
    /// A value model contains a non-value nested type.
    InvalidValueClosure,
    /// Two query paths flatten to the same external name.
    QueryNameConflict,
}

/// The original failure retained by a contextual model resolution error.
#[derive(Clone, Debug, thiserror::Error)]
pub enum ModelResolutionCause {
    /// Reflected metadata could not be obtained or validated.
    #[error(transparent)]
    Metadata(#[from] ModelMetadataError),
    /// A property set could not be obtained or assembled.
    #[error(transparent)]
    Properties(#[from] PropertyResolutionError),
}

/// One structured deterministic resolution error.
#[must_use]
#[derive(Clone, Debug)]
pub struct ModelResolveError {
    /// The machine-readable resolution failure class.
    kind: ModelResolveErrorKind,
    /// The involved property path, when the failure identifies one.
    path: Option<OwnedPropertyPath>,
    /// The involved stable model ID, when the failure identifies one.
    model_id: Option<&'static str>,
    /// The role expected by the resolution step, when applicable.
    expected_role: Option<ModelRole>,
    /// The role actually observed by the resolution step, when applicable.
    actual_role: Option<ModelRole>,
    /// The type expected by the resolution step, when applicable.
    expected_type: Option<TypeId>,
    /// The type actually observed by the resolution step, when applicable.
    actual_type: Option<TypeId>,
    /// Fragment identities involved in the failure.
    sources: Vec<FragmentIdentity>,
    /// Original structured failure, if this error crosses a fallible boundary.
    cause: Option<ModelResolutionCause>,
}

impl ModelResolveError {
    /// Creates one resolution error with optional contextual details.
    fn new(
        kind: ModelResolveErrorKind,
        model_id: Option<&'static str>,
        path: Option<PropertyPath<'_>>,
        expected_role: Option<ModelRole>,
        actual_role: Option<ModelRole>,
        source: Option<&FragmentIdentity>,
    ) -> Self {
        Self {
            kind,
            path: path.map(|path| OwnedPropertyPath::from_segments(path.segments())),
            model_id,
            expected_role,
            actual_role,
            expected_type: None,
            actual_type: None,
            sources: source.into_iter().cloned().collect(),
            cause: None,
        }
    }

    /// Creates a contextual error without discarding its underlying cause.
    fn resolution(
        root: &'static TypeMetadata,
        path: Option<PropertyPath<'_>>,
        source: &FragmentIdentity,
        cause: impl Into<ModelResolutionCause>,
    ) -> Self {
        let cause = cause.into();
        let kind = match &cause {
            ModelResolutionCause::Metadata(_) => ModelResolveErrorKind::MetadataResolution,
            ModelResolutionCause::Properties(_) => ModelResolveErrorKind::PropertyResolution,
        };
        let mut error = Self::new(
            kind,
            root.model_id().map(|id| id.as_str()),
            path,
            None,
            Some(root.role()),
            Some(source),
        );
        error.cause = Some(cause);
        error
    }

    /// Attaches the original failure to an already classified diagnostic.
    fn with_cause(mut self, cause: ModelResolutionCause) -> Self {
        self.cause = Some(cause);
        self
    }

    /// Returns the original structured failure, when present.
    #[must_use]
    pub const fn cause(&self) -> Option<&ModelResolutionCause> {
        self.cause.as_ref()
    }

    /// Adds the expected and observed type identities to this error.
    fn with_types(mut self, expected: TypeId, actual: TypeId) -> Self {
        self.expected_type = Some(expected);
        self.actual_type = Some(actual);
        self
    }

    /// Orders errors by kind, model, path, and source identity.
    fn compare(left: &Self, right: &Self) -> std::cmp::Ordering {
        left.kind
            .cmp(&right.kind)
            .then_with(|| left.model_id.cmp(&right.model_id))
            .then_with(|| {
                left.path
                    .as_ref()
                    .map(|path| path.as_path().to_string())
                    .cmp(&right.path.as_ref().map(|path| path.as_path().to_string()))
            })
            .then_with(|| left.sources.cmp(&right.sources))
    }

    /// Returns the machine-readable resolution failure class.
    #[must_use]
    pub const fn kind(&self) -> ModelResolveErrorKind {
        self.kind
    }
    /// Returns the involved path, or `None` when the failure is model-wide.
    #[must_use]
    pub fn path(&self) -> Option<PropertyPath<'_>> {
        self.path.as_ref().map(OwnedPropertyPath::as_path)
    }
    /// Returns the involved stable model ID, when present.
    #[must_use]
    pub const fn model_id(&self) -> Option<&str> {
        self.model_id
    }
    /// Returns the expected role, when role matching was required.
    #[must_use]
    pub const fn expected_role(&self) -> Option<ModelRole> {
        self.expected_role
    }
    /// Returns the actual role, when role matching was required.
    #[must_use]
    pub const fn actual_role(&self) -> Option<ModelRole> {
        self.actual_role
    }
    /// Returns the expected type identity, when type matching was required.
    #[must_use]
    pub const fn expected_type(&self) -> Option<TypeId> {
        self.expected_type
    }
    /// Returns the actual type identity, when type matching was required.
    #[must_use]
    pub const fn actual_type(&self) -> Option<TypeId> {
        self.actual_type
    }
    /// Returns the fragment identities involved in this failure.
    #[must_use]
    pub fn sources(&self) -> &[FragmentIdentity] {
        &self.sources
    }
}

impl core::fmt::Display for ModelResolveError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "model resolution failed: {:?}", self.kind)
    }
}

/// All errors from one complete failed resolution pass.
#[must_use]
#[derive(Debug)]
pub struct ModelResolveErrors {
    /// All deterministic failures collected during one resolution pass.
    errors: Vec<ModelResolveError>,
}

impl ModelResolveErrors {
    /// Returns every collected resolution failure.
    #[must_use = "inspect the resolution failures"]
    #[inline(always)]
    pub fn errors(&self) -> &[ModelResolveError] {
        &self.errors
    }

    /// Consumes this collection and returns its failures.
    #[must_use]
    pub fn into_errors(self) -> Vec<ModelResolveError> {
        self.errors
    }
}

impl core::fmt::Display for ModelResolveErrors {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "{} model resolution error(s)", self.errors.len())
    }
}

impl std::error::Error for ModelResolveErrors {}

/// Returns the stable identity key used for a static field declaration.
fn pointer_key(field: &FieldMetadata) -> usize {
    field as *const FieldMetadata as usize
}

/// Returns a stable target ID for textual target declarations.
fn declared_target_id(target: &DeclaredEntityTarget) -> Option<&'static str> {
    target.model_id().map(|id| id.as_str())
}

impl std::error::Error for ModelResolveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.cause.as_ref().map(|cause| cause as &dyn std::error::Error)
    }
}
