// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

// qubit-style: allow multiple-public-types
//! Explicit cross-model resolution and immutable resolved views.
#![allow(
    missing_docs,
    reason = "the resolved graph vocabulary is documented as one cohesive contract in the module guide"
)]

use std::any::TypeId;
use std::collections::HashMap;
use std::collections::HashSet;

use qubit_codec::ValueCodecDescriptor;
use qubit_codec::ValueCodecRegistration;
use qubit_codec::ValueCodecRegistry;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::descriptor::TypeRef;
use qubit_reflect::identity::FragmentIdentity;
use qubit_validator::ValidatorRegistration;
use qubit_validator::ValidatorRegistry;

use crate::CodecMetadata;
use crate::ConstraintMetadata;
use crate::DeclaredEntityTarget;
use crate::FieldMetadata;
use crate::FieldReferenceMetadata;
use crate::IndexingReasons;
use crate::ModelDescriptorExt;
use crate::ModelRegistry;
use crate::ModelRole;
use crate::ProjectionMetadata;
use crate::PropertyMetadata;
use crate::PropertyPath;
use crate::ReferenceSelection;
use crate::SelectorMetadata;
use crate::SelectorPosition;
use crate::TypeMetadata;
use crate::ValidatorMetadata;

/// Inputs used for one complete resolution attempt.
#[derive(Clone, Copy)]
pub struct ResolveInputs<'a> {
    pub models: &'a ModelRegistry,
    pub validators: &'a ValidatorRegistry,
    pub codecs: &'a ValueCodecRegistry,
}

/// Resolves declaration-only metadata against explicit registries.
pub struct ModelResolver<'a> {
    inputs: ResolveInputs<'a>,
}

impl<'a> ModelResolver<'a> {
    /// Creates a resolver without consulting process globals.
    #[must_use]
    pub const fn new(inputs: ResolveInputs<'a>) -> Self {
        Self { inputs }
    }

    /// Resolves every registration or returns all deterministic errors.
    pub fn resolve_all(&self) -> Result<ResolvedModelGraph<'a>, ModelResolveErrors> {
        let mut references = HashMap::new();
        let mut projection_sources = HashMap::new();
        let mut validators = HashMap::new();
        let mut codecs = HashMap::new();
        let mut queries = HashMap::new();
        let mut errors = Vec::new();

        for registration in self.inputs.models.registrations() {
            let Some(metadata) = registration.metadata() else {
                continue;
            };
            for field in metadata.fields() {
                resolve_field_strategies(
                    metadata,
                    field,
                    self.inputs,
                    &mut validators,
                    &mut codecs,
                    registration.source(),
                    &mut errors,
                );
                if let Some(reference) = field.reference() {
                    let mut local_reference_valid = true;
                    if let Some(path) = reference.same_as() {
                        match resolve_property_path(metadata, path, self.inputs.models) {
                            Some(property) if property.is_readable() => {}
                            Some(_) => {
                                errors.push(ModelResolveError::new(
                                    ModelResolveErrorKind::UnreadableProperty,
                                    metadata.model_id().map(|id| id.as_str()),
                                    Some(*path),
                                    None,
                                    Some(metadata.role()),
                                    Some(registration.source()),
                                ));
                                local_reference_valid = false;
                            }
                            None => {
                                errors.push(ModelResolveError::new(
                                    ModelResolveErrorKind::MissingProperty,
                                    metadata.model_id().map(|id| id.as_str()),
                                    Some(*path),
                                    None,
                                    Some(metadata.role()),
                                    Some(registration.source()),
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
                                        Some(property) if property.is_readable() => Some(property),
                                        Some(_) => {
                                            errors.push(ModelResolveError::new(
                                                ModelResolveErrorKind::UnreadableProperty,
                                                target.model_id().map(|id| id.as_str()),
                                                Some(*path),
                                                Some(ModelRole::Entity),
                                                Some(target.role()),
                                                Some(registration.source()),
                                            ));
                                            continue;
                                        }
                                        None => {
                                            errors.push(ModelResolveError::new(
                                                ModelResolveErrorKind::MissingProperty,
                                                target.model_id().map(|id| id.as_str()),
                                                Some(*path),
                                                Some(ModelRole::Entity),
                                                Some(target.role()),
                                                Some(registration.source()),
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
                                        Some(registration.source()),
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
                            Some(registration.source()),
                        )),
                        None => errors.push(ModelResolveError::new(
                            ModelResolveErrorKind::MissingModelId,
                            declared_target_id(reference.target()),
                            None,
                            Some(ModelRole::Entity),
                            None,
                            Some(registration.source()),
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
                                    Some(registration.source()),
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
                        Some(registration.source()),
                    )),
                    None => errors.push(ModelResolveError::new(
                        ModelResolveErrorKind::InvalidProjectionSource,
                        declared_target_id(source),
                        None,
                        Some(ModelRole::Entity),
                        None,
                        Some(registration.source()),
                    )),
                }
            }

            if metadata.role() == ModelRole::Value {
                let mut visited = HashSet::new();
                validate_value_closure(
                    metadata,
                    self.inputs.models,
                    &mut visited,
                    registration.source(),
                    &mut errors,
                );
            }

            if let Some(codec) = metadata.as_value().and_then(crate::ValueMetadata::canonical_codec) {
                resolve_codec(
                    metadata,
                    codec,
                    metadata.type_id(),
                    self.inputs.codecs,
                    &mut codecs,
                    registration.source(),
                    &mut errors,
                );
            }

            if let Some(entity) = metadata.as_entity()
                && let Some(query) = build_query(metadata, self.inputs.models, registration.source(), &mut errors)
            {
                queries.insert(entity as *const crate::EntityMetadata as usize, query);
            }
        }

        if errors.is_empty() {
            Ok(ResolvedModelGraph {
                registry: self.inputs.models,
                references,
                projection_sources,
                validators,
                codecs,
                queries,
            })
        } else {
            errors.sort_by(ModelResolveError::compare);
            Err(ModelResolveErrors { errors })
        }
    }

    fn resolve_target(&self, target: &DeclaredEntityTarget) -> Option<&'static TypeMetadata> {
        match target {
            DeclaredEntityTarget::RustType(provider) => Some(provider()),
            DeclaredEntityTarget::ModelId(id) => self.inputs.models.metadata(id.as_str()),
        }
    }
}

/// Resolves executable strategies declared directly on one field.
#[allow(clippy::too_many_arguments)]
fn resolve_field_strategies<'a>(
    metadata: &'static TypeMetadata,
    field: &'static FieldMetadata,
    inputs: ResolveInputs<'a>,
    validators: &mut HashMap<usize, ResolvedValidator<'a>>,
    codecs: &mut HashMap<usize, ResolvedCodec<'a>>,
    source: &'static FragmentIdentity,
    errors: &mut Vec<ModelResolveError>,
) {
    let Some(descriptor) = field.descriptor() else {
        return;
    };
    for occurrence in field.validators() {
        resolve_validator(
            metadata,
            occurrence,
            descriptor.type_id(),
            inputs,
            validators,
            source,
            errors,
        );
    }
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
    for constraint in field.constraints() {
        match constraint {
            ConstraintMetadata::Sequence(sequence) => {
                if let Some(selector) = sequence.element() {
                    resolve_selector_strategies(
                        metadata,
                        selector,
                        selector_type_id(descriptor, SelectorPosition::Element),
                        inputs,
                        validators,
                        codecs,
                        source,
                        errors,
                    );
                }
            }
            ConstraintMetadata::Map(map) => {
                for (selector, position) in [
                    (map.key(), SelectorPosition::MapKey),
                    (map.value(), SelectorPosition::MapValue),
                ] {
                    if let Some(selector) = selector {
                        resolve_selector_strategies(
                            metadata,
                            selector,
                            selector_type_id(descriptor, position),
                            inputs,
                            validators,
                            codecs,
                            source,
                            errors,
                        );
                    }
                }
            }
            _ => {}
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn resolve_selector_strategies<'a>(
    metadata: &'static TypeMetadata,
    selector: &'static SelectorMetadata,
    expected_type: Option<TypeId>,
    inputs: ResolveInputs<'a>,
    validators: &mut HashMap<usize, ResolvedValidator<'a>>,
    codecs: &mut HashMap<usize, ResolvedCodec<'a>>,
    source: &'static FragmentIdentity,
    errors: &mut Vec<ModelResolveError>,
) {
    let Some(expected_type) = expected_type else {
        errors.push(ModelResolveError::new(
            ModelResolveErrorKind::UnresolvedSelectorType,
            metadata.model_id().map(|id| id.as_str()),
            None,
            None,
            Some(metadata.role()),
            Some(source),
        ));
        return;
    };
    for validator in selector.validators() {
        resolve_validator(metadata, validator, expected_type, inputs, validators, source, errors);
    }
    if let Some(codec) = selector.codec() {
        resolve_codec(metadata, codec, expected_type, inputs.codecs, codecs, source, errors);
    }
}

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

fn runtime_type_id(type_ref: &TypeRef) -> Option<TypeId> {
    type_ref
        .as_resolved()
        .map(TypeDescriptor::type_id)
        .or_else(|| type_ref.as_opaque().map(|descriptor| descriptor.type_id()))
}

/// Resolves one validator registration and its readable dependencies.
#[allow(clippy::too_many_arguments)]
fn resolve_validator<'a>(
    metadata: &'static TypeMetadata,
    occurrence: &'static ValidatorMetadata,
    expected_type: TypeId,
    inputs: ResolveInputs<'a>,
    validators: &mut HashMap<usize, ResolvedValidator<'a>>,
    source: &'static FragmentIdentity,
    errors: &mut Vec<ModelResolveError>,
) {
    let Some(registration) = inputs.validators.get(occurrence.declared_id()) else {
        errors.push(ModelResolveError::new(
            ModelResolveErrorKind::MissingValidator,
            metadata.model_id().map(|id| id.as_str()),
            None,
            None,
            Some(metadata.role()),
            Some(source),
        ));
        return;
    };
    let actual_type = registration.descriptor().value_type_id();
    if actual_type != expected_type {
        errors.push(
            ModelResolveError::new(
                ModelResolveErrorKind::ValidatorTypeMismatch,
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
    let mut dependencies = Vec::with_capacity(occurrence.depends_on().len());
    let initial_error_count = errors.len();
    for path in occurrence.depends_on() {
        match resolve_property_path(metadata, path, inputs.models) {
            Some(property) if property.is_readable() => dependencies.push(property),
            Some(_) => errors.push(ModelResolveError::new(
                ModelResolveErrorKind::UnreadableProperty,
                metadata.model_id().map(|id| id.as_str()),
                Some(*path),
                None,
                Some(metadata.role()),
                Some(source),
            )),
            None => errors.push(ModelResolveError::new(
                ModelResolveErrorKind::MissingProperty,
                metadata.model_id().map(|id| id.as_str()),
                Some(*path),
                None,
                Some(metadata.role()),
                Some(source),
            )),
        }
    }
    if errors.len() == initial_error_count {
        validators.insert(
            occurrence as *const ValidatorMetadata as usize,
            ResolvedValidator {
                declaration: occurrence,
                registration,
                dependencies: dependencies.into_boxed_slice(),
            },
        );
    }
}

/// Resolves one statically typed or stable-ID codec declaration.
#[allow(clippy::too_many_arguments)]
fn resolve_codec<'a>(
    metadata: &'static TypeMetadata,
    occurrence: &'static CodecMetadata,
    expected_type: TypeId,
    registry: &'a ValueCodecRegistry,
    codecs: &mut HashMap<usize, ResolvedCodec<'a>>,
    source: &'static FragmentIdentity,
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

trait ReferenceSelectionExt {
    fn property_path(&self) -> Option<&PropertyPath<'static>>;
}

impl ReferenceSelectionExt for ReferenceSelection {
    fn property_path(&self) -> Option<&PropertyPath<'static>> {
        match self {
            Self::Entity => None,
            Self::Property(path) => Some(path),
        }
    }
}

fn metadata_for_descriptor(
    descriptor: &'static TypeDescriptor,
    registry: &ModelRegistry,
) -> Option<&'static TypeMetadata> {
    descriptor
        .model_metadata()
        .or_else(|| registry.by_type_id(descriptor.type_id()))
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

fn build_query(
    metadata: &'static TypeMetadata,
    registry: &ModelRegistry,
    source: &'static FragmentIdentity,
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
                        Some(property) if property.is_readable() => {
                            paths.push(OwnedPropertyPath::from_static(*scope));
                        }
                        Some(_) => errors.push(ModelResolveError::new(
                            ModelResolveErrorKind::UnreadableProperty,
                            metadata.model_id().map(|id| id.as_str()),
                            Some(*scope),
                            None,
                            Some(metadata.role()),
                            Some(source),
                        )),
                        None => errors.push(ModelResolveError::new(
                            ModelResolveErrorKind::MissingProperty,
                            metadata.model_id().map(|id| id.as_str()),
                            Some(*scope),
                            None,
                            Some(metadata.role()),
                            Some(source),
                        )),
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
        if !added {
            errors.push(ModelResolveError::new(
                ModelResolveErrorKind::InvalidValueClosure,
                metadata.model_id().map(|id| id.as_str()),
                Some(root_path.as_path()),
                Some(ModelRole::Value),
                field
                    .descriptor()
                    .and_then(|descriptor| metadata_for_descriptor(descriptor, registry))
                    .map(TypeMetadata::role),
                Some(source),
            ));
        }
    }

    (errors.len() == initial_error_count).then_some(QueryMetadata {
        filters: filters.into_boxed_slice(),
        unique_keys: unique_keys.into_boxed_slice(),
    })
}

#[allow(clippy::too_many_arguments)]
fn collect_query_fields(
    metadata: &'static TypeMetadata,
    prefix: &[&'static str],
    allow_references: bool,
    registry: &ModelRegistry,
    filters: &mut Vec<QueryField>,
    flat_names: &mut HashMap<String, OwnedPropertyPath>,
    root: &'static TypeMetadata,
    source: &'static FragmentIdentity,
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

#[allow(clippy::too_many_arguments)]
fn collect_indexed_field(
    field: &'static FieldMetadata,
    path: &[&'static str],
    allow_references: bool,
    registry: &ModelRegistry,
    filters: &mut Vec<QueryField>,
    flat_names: &mut HashMap<String, OwnedPropertyPath>,
    root: &'static TypeMetadata,
    source: &'static FragmentIdentity,
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
    if let Some(descriptor) = field.descriptor()
        && let Some(nested) = metadata_for_descriptor(descriptor, registry)
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

#[allow(clippy::too_many_arguments)]
fn push_query_field(
    filters: &mut Vec<QueryField>,
    flat_names: &mut HashMap<String, OwnedPropertyPath>,
    path: OwnedPropertyPath,
    descriptor: Option<&'static TypeDescriptor>,
    reasons: IndexingReasons,
    root: &'static TypeMetadata,
    source: &'static FragmentIdentity,
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

fn resolve_declared_target(target: &DeclaredEntityTarget, registry: &ModelRegistry) -> Option<&'static TypeMetadata> {
    match target {
        DeclaredEntityTarget::RustType(provider) => Some(provider()),
        DeclaredEntityTarget::ModelId(id) => registry.metadata(id.as_str()),
    }
}

fn path_from_segments(segments: &[&'static str]) -> OwnedPropertyPath {
    OwnedPropertyPath::from_segments(segments)
}

fn validate_value_closure(
    metadata: &'static TypeMetadata,
    registry: &ModelRegistry,
    visited: &mut HashSet<TypeId>,
    source: &'static FragmentIdentity,
    errors: &mut Vec<ModelResolveError>,
) {
    if !visited.insert(metadata.type_id()) {
        return;
    }
    for field in metadata.fields() {
        if field.is_opaque() {
            continue;
        }
        let Some(name) = field.name() else { continue };
        if !field.type_ref().as_resolved().is_some_and(|descriptor| {
            value_descriptor_is_closed(descriptor, registry, visited, source, errors, &[name])
        }) {
            let path = path_from_segments(&[name]);
            errors.push(ModelResolveError::new(
                ModelResolveErrorKind::InvalidValueClosure,
                metadata.model_id().map(|id| id.as_str()),
                Some(path.as_path()),
                Some(ModelRole::Value),
                field
                    .descriptor()
                    .and_then(|descriptor| metadata_for_descriptor(descriptor, registry))
                    .map(TypeMetadata::role),
                Some(source),
            ));
        }
    }
}

fn value_type_ref_is_closed(
    type_ref: &'static TypeRef,
    registry: &ModelRegistry,
    visited: &mut HashSet<TypeId>,
    source: &'static FragmentIdentity,
    errors: &mut Vec<ModelResolveError>,
    path: &[&'static str],
) -> bool {
    type_ref
        .as_resolved()
        .is_some_and(|descriptor| value_descriptor_is_closed(descriptor, registry, visited, source, errors, path))
}

fn value_descriptor_is_closed(
    descriptor: &'static TypeDescriptor,
    registry: &ModelRegistry,
    visited: &mut HashSet<TypeId>,
    source: &'static FragmentIdentity,
    errors: &mut Vec<ModelResolveError>,
    path: &[&'static str],
) -> bool {
    if descriptor.as_primitive().is_some() || descriptor.as_text().is_some() {
        return true;
    }
    if let Some(metadata) = metadata_for_descriptor(descriptor, registry) {
        return match metadata.role() {
            ModelRole::Value => {
                validate_value_closure(metadata, registry, visited, source, errors);
                true
            }
            ModelRole::Enum => metadata.as_enum().is_some_and(|enumeration| {
                enumeration.variants().iter().all(|variant| {
                    variant.fields().iter().all(|field| {
                        field.is_opaque()
                            || value_type_ref_is_closed(field.type_ref(), registry, visited, source, errors, path)
                    })
                })
            }),
            ModelRole::Entity | ModelRole::Projection | ModelRole::Model => false,
        };
    }
    if let Some(optional) = descriptor.as_optional() {
        return value_type_ref_is_closed(optional.element_type(), registry, visited, source, errors, path);
    }
    if let Some(sequence) = descriptor.as_sequence() {
        return value_type_ref_is_closed(sequence.element_type(), registry, visited, source, errors, path);
    }
    if let Some(set) = descriptor.as_set() {
        return value_type_ref_is_closed(set.element_type(), registry, visited, source, errors, path);
    }
    if let Some(array) = descriptor.as_array() {
        return value_type_ref_is_closed(array.element_type(), registry, visited, source, errors, path);
    }
    if let Some(map) = descriptor.as_map() {
        return value_type_ref_is_closed(map.key_type(), registry, visited, source, errors, path)
            && value_type_ref_is_closed(map.value_type(), registry, visited, source, errors, path);
    }
    if let Some(tuple) = descriptor.as_tuple() {
        return tuple
            .elements()
            .iter()
            .all(|element| value_type_ref_is_closed(element, registry, visited, source, errors, path));
    }
    if let Some(pointer) = descriptor.as_smart_pointer() {
        return value_type_ref_is_closed(pointer.pointee_type(), registry, visited, source, errors, path);
    }
    false
}

fn resolve_property_path(
    target: &'static TypeMetadata,
    path: &PropertyPath<'_>,
    registry: &ModelRegistry,
) -> Option<&'static PropertyMetadata> {
    let mut current = target;
    let mut result = None;
    for (index, segment) in path.segments().iter().enumerate() {
        let property = current.property(segment)?;
        result = Some(property);
        if index + 1 < path.segments().len() {
            current = metadata_for_descriptor(property.descriptor()?, registry)?;
        }
    }
    result
}

/// A successfully resolved direct reference.
#[derive(Debug)]
pub struct ResolvedReference {
    declaration: &'static FieldReferenceMetadata,
    target: &'static TypeMetadata,
    property: Option<&'static PropertyMetadata>,
}

impl ResolvedReference {
    pub const fn declaration(&self) -> &'static FieldReferenceMetadata {
        self.declaration
    }
    pub const fn target(&self) -> &'static TypeMetadata {
        self.target
    }
    pub const fn property(&self) -> Option<&'static PropertyMetadata> {
        self.property
    }
}

/// A successfully resolved Projection source.
#[derive(Debug)]
pub struct ResolvedProjectionSource {
    target: &'static TypeMetadata,
}

impl ResolvedProjectionSource {
    pub const fn target(&self) -> &'static TypeMetadata {
        self.target
    }
}

/// Immutable result of a complete successful resolution pass.
#[derive(Debug)]
pub struct ResolvedModelGraph<'a> {
    registry: &'a ModelRegistry,
    references: HashMap<usize, ResolvedReference>,
    projection_sources: HashMap<usize, ResolvedProjectionSource>,
    validators: HashMap<usize, ResolvedValidator<'a>>,
    codecs: HashMap<usize, ResolvedCodec<'a>>,
    queries: HashMap<usize, QueryMetadata>,
}

impl<'a> ResolvedModelGraph<'a> {
    pub const fn registry(&self) -> &'a ModelRegistry {
        self.registry
    }
    pub fn reference(&self, field: &FieldMetadata) -> Option<&ResolvedReference> {
        self.references.get(&pointer_key(field))
    }
    pub fn projection_source(&self, projection: &ProjectionMetadata) -> Option<&ResolvedProjectionSource> {
        self.projection_sources
            .get(&(projection as *const ProjectionMetadata as usize))
    }
    pub fn validator(&self, occurrence: &ValidatorMetadata) -> Option<&ResolvedValidator<'a>> {
        self.validators.get(&(occurrence as *const ValidatorMetadata as usize))
    }
    pub fn codec(&self, occurrence: &CodecMetadata) -> Option<&ResolvedCodec<'a>> {
        self.codecs.get(&(occurrence as *const CodecMetadata as usize))
    }
    pub fn query(&self, entity: &crate::EntityMetadata) -> Option<&QueryMetadata> {
        self.queries.get(&(entity as *const crate::EntityMetadata as usize))
    }
}

/// A validator occurrence bound to one executable registration.
#[derive(Debug)]
pub struct ResolvedValidator<'a> {
    declaration: &'static ValidatorMetadata,
    registration: &'a ValidatorRegistration,
    dependencies: Box<[&'static PropertyMetadata]>,
}

impl ResolvedValidator<'_> {
    /// Returns the declaration occurrence.
    pub const fn declaration(&self) -> &'static ValidatorMetadata {
        self.declaration
    }

    /// Returns the executable validator registration.
    pub const fn registration(&self) -> &ValidatorRegistration {
        self.registration
    }

    /// Returns resolved readable dependency properties.
    pub fn dependencies(&self) -> &[&'static PropertyMetadata] {
        &self.dependencies
    }
}

/// A codec occurrence bound to one executable descriptor.
#[derive(Debug)]
pub struct ResolvedCodec<'a> {
    declaration: &'static CodecMetadata,
    descriptor: &'static ValueCodecDescriptor,
    registration: Option<&'a ValueCodecRegistration>,
}

impl ResolvedCodec<'_> {
    /// Returns the declaration occurrence.
    pub const fn declaration(&self) -> &'static CodecMetadata {
        self.declaration
    }

    /// Returns the executable codec descriptor.
    pub const fn descriptor(&self) -> &'static ValueCodecDescriptor {
        self.descriptor
    }

    /// Returns the registry entry for stable-ID declarations.
    pub const fn registration(&self) -> Option<&ValueCodecRegistration> {
        self.registration
    }
}

/// Queryable indexed fields derived for one resolved entity.
#[derive(Debug)]
pub struct QueryMetadata {
    filters: Box<[QueryField]>,
    unique_keys: Box<[UniqueQueryKey]>,
}

impl QueryMetadata {
    pub fn filters(&self) -> &[QueryField] {
        &self.filters
    }
    pub fn unique_keys(&self) -> &[UniqueQueryKey] {
        &self.unique_keys
    }
    pub fn filter(&self, path: &PropertyPath<'_>) -> Option<&QueryField> {
        self.filters.iter().find(|field| field.path.as_path() == *path)
    }
    pub fn filter_by_flat_name(&self, name: &str) -> Option<&QueryField> {
        self.filters.iter().find(|field| field.flat_name.as_ref() == name)
    }
}

/// One queryable field path.
#[derive(Clone, Debug)]
pub struct QueryField {
    path: OwnedPropertyPath,
    flat_name: Box<str>,
    descriptor: Option<&'static TypeDescriptor>,
    reasons: IndexingReasons,
}

impl QueryField {
    pub fn path(&self) -> PropertyPath<'_> {
        self.path.as_path()
    }
    pub fn flat_name(&self) -> &str {
        &self.flat_name
    }
    pub const fn descriptor(&self) -> Option<&'static TypeDescriptor> {
        self.descriptor
    }
    pub const fn reasons(&self) -> IndexingReasons {
        self.reasons
    }
}

/// One identifier or global-unique lookup key.
#[derive(Clone, Debug)]
pub struct UniqueQueryKey {
    paths: Box<[OwnedPropertyPath]>,
}

impl UniqueQueryKey {
    fn new(paths: Vec<OwnedPropertyPath>) -> Self {
        Self {
            paths: paths.into_boxed_slice(),
        }
    }
    pub fn paths(&self) -> impl ExactSizeIterator<Item = PropertyPath<'_>> + '_ {
        self.paths.iter().map(OwnedPropertyPath::as_path)
    }
    pub fn path(&self) -> Option<PropertyPath<'_>> {
        (self.paths.len() == 1).then(|| self.paths[0].as_path())
    }
}

/// Machine-readable model resolution error class.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ModelResolveErrorKind {
    MissingModelId,
    WrongModelRole,
    MissingProperty,
    UnreadableProperty,
    TypeMismatch,
    MissingValidator,
    ValidatorTypeMismatch,
    MissingCodec,
    CodecTypeMismatch,
    UnresolvedSelectorType,
    InvalidProjectionSource,
    InvalidValueClosure,
    QueryNameConflict,
}

/// One structured deterministic resolution error.
#[derive(Clone, Debug)]
pub struct ModelResolveError {
    kind: ModelResolveErrorKind,
    path: Option<OwnedPropertyPath>,
    model_id: Option<&'static str>,
    expected_role: Option<ModelRole>,
    actual_role: Option<ModelRole>,
    expected_type: Option<TypeId>,
    actual_type: Option<TypeId>,
    sources: Vec<FragmentIdentity>,
}

impl ModelResolveError {
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
        }
    }

    fn with_types(mut self, expected: TypeId, actual: TypeId) -> Self {
        self.expected_type = Some(expected);
        self.actual_type = Some(actual);
        self
    }

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

    pub const fn kind(&self) -> ModelResolveErrorKind {
        self.kind
    }
    pub fn path(&self) -> Option<PropertyPath<'_>> {
        self.path.as_ref().map(OwnedPropertyPath::as_path)
    }
    pub const fn model_id(&self) -> Option<&str> {
        self.model_id
    }
    pub const fn expected_role(&self) -> Option<ModelRole> {
        self.expected_role
    }
    pub const fn actual_role(&self) -> Option<ModelRole> {
        self.actual_role
    }
    pub const fn expected_type(&self) -> Option<TypeId> {
        self.expected_type
    }
    pub const fn actual_type(&self) -> Option<TypeId> {
        self.actual_type
    }
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
#[derive(Debug)]
pub struct ModelResolveErrors {
    errors: Vec<ModelResolveError>,
}

impl ModelResolveErrors {
    pub fn errors(&self) -> &[ModelResolveError] {
        &self.errors
    }
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

fn pointer_key(field: &FieldMetadata) -> usize {
    field as *const FieldMetadata as usize
}

fn declared_target_id(target: &DeclaredEntityTarget) -> Option<&'static str> {
    target.model_id().map(|id| id.as_str())
}
