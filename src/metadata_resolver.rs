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

use qubit_reflect::TypeDescriptor;
use qubit_reflect::descriptor::TypeRef;
use qubit_reflect::identity::FragmentIdentity;

use crate::CodecMetadata;
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
use crate::TypeMetadata;
use crate::ValidatorMetadata;

/// Inputs used for one complete resolution attempt.
#[derive(Clone, Copy)]
pub struct ResolveInputs<'a> {
    pub models: &'a ModelRegistry,
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
        let mut queries = HashMap::new();
        let mut errors = Vec::new();

        for registration in self.inputs.models.registrations() {
            let Some(metadata) = registration.metadata() else {
                continue;
            };
            for field in metadata.fields() {
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

trait ReferenceSelectionExt {
    fn property_path(&self) -> Option<&PropertyPath>;
}

impl ReferenceSelectionExt for ReferenceSelection {
    fn property_path(&self) -> Option<&PropertyPath> {
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

fn build_query(
    metadata: &'static TypeMetadata,
    registry: &ModelRegistry,
    source: &'static FragmentIdentity,
    errors: &mut Vec<ModelResolveError>,
) -> Option<QueryMetadata> {
    let initial_error_count = errors.len();
    let mut filters = Vec::new();
    let mut unique_keys = Vec::new();
    let mut flat_names = HashMap::<&'static str, PropertyPath>::new();

    for field in metadata.fields() {
        let Some(name) = field.name() else { continue };
        let root_path = path_from_segments(&[name]);
        if field.is_identifier() {
            unique_keys.push(UniqueQueryKey::new(Box::leak(vec![root_path].into_boxed_slice())));
            continue;
        }
        if let Some(unique) = field.unique() {
            if unique.is_scoped() {
                let mut paths = vec![root_path];
                for scope in unique.respect_to() {
                    match resolve_property_path(metadata, scope, registry) {
                        Some(property) if property.is_readable() => paths.push(*scope),
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
                unique_keys.push(UniqueQueryKey::new(Box::leak(paths.into_boxed_slice())));
            } else {
                unique_keys.push(UniqueQueryKey::new(Box::leak(vec![root_path].into_boxed_slice())));
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
                Some(root_path),
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
    flat_names: &mut HashMap<&'static str, PropertyPath>,
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
    flat_names: &mut HashMap<&'static str, PropertyPath>,
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
    flat_names: &mut HashMap<&'static str, PropertyPath>,
    path: PropertyPath,
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
    let flat_name = Box::leak(path.segments().join("_").into_boxed_str());
    if flat_names.get(flat_name).is_some_and(|existing| *existing != path) {
        errors.push(ModelResolveError::new(
            ModelResolveErrorKind::QueryNameConflict,
            root.model_id().map(|id| id.as_str()),
            Some(path),
            None,
            None,
            Some(source),
        ));
        return;
    }
    flat_names.insert(flat_name, path);
    filters.push(QueryField {
        path,
        flat_name,
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

fn path_from_segments(segments: &[&'static str]) -> PropertyPath {
    PropertyPath::new(Box::leak(segments.to_vec().into_boxed_slice()))
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
            errors.push(ModelResolveError::new(
                ModelResolveErrorKind::InvalidValueClosure,
                metadata.model_id().map(|id| id.as_str()),
                Some(path_from_segments(&[name])),
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
    path: &PropertyPath,
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
    pub fn validator(&self, _occurrence: &ValidatorMetadata) -> Option<&ResolvedValidator> {
        None
    }
    pub fn codec(&self, _occurrence: &CodecMetadata) -> Option<&ResolvedCodec> {
        None
    }
    pub fn query(&self, entity: &crate::EntityMetadata) -> Option<&QueryMetadata> {
        self.queries.get(&(entity as *const crate::EntityMetadata as usize))
    }
}

/// Reserved resolved validator view; runtime validator resolution is deferred.
pub struct ResolvedValidator;
/// Reserved resolved codec view until qubit-codec exposes a registry protocol.
pub struct ResolvedCodec;

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
    pub fn filter(&self, path: &PropertyPath) -> Option<&QueryField> {
        self.filters.iter().find(|field| field.path == *path)
    }
    pub fn filter_by_flat_name(&self, name: &str) -> Option<&QueryField> {
        self.filters.iter().find(|field| field.flat_name == name)
    }
}

/// One queryable field path.
#[derive(Clone, Copy, Debug)]
pub struct QueryField {
    path: PropertyPath,
    flat_name: &'static str,
    descriptor: Option<&'static TypeDescriptor>,
    reasons: IndexingReasons,
}

impl QueryField {
    pub const fn path(&self) -> &PropertyPath {
        &self.path
    }
    pub const fn flat_name(&self) -> &'static str {
        self.flat_name
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
    paths: Box<[PropertyPath]>,
}

impl UniqueQueryKey {
    fn new(paths: &'static [PropertyPath]) -> Self {
        Self { paths: paths.into() }
    }
    pub fn paths(&self) -> &[PropertyPath] {
        &self.paths
    }
    pub fn path(&self) -> Option<&PropertyPath> {
        (self.paths.len() == 1).then(|| &self.paths[0])
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
    InvalidProjectionSource,
    InvalidValueClosure,
    QueryNameConflict,
    InvalidReferenceGraph,
}

/// One structured deterministic resolution error.
#[derive(Clone, Debug)]
pub struct ModelResolveError {
    kind: ModelResolveErrorKind,
    path: Option<PropertyPath>,
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
        path: Option<PropertyPath>,
        expected_role: Option<ModelRole>,
        actual_role: Option<ModelRole>,
        source: Option<&FragmentIdentity>,
    ) -> Self {
        Self {
            kind,
            path,
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
                    .map(|path| path.to_string())
                    .cmp(&right.path.map(|path| path.to_string()))
            })
            .then_with(|| left.sources.cmp(&right.sources))
    }

    pub const fn kind(&self) -> ModelResolveErrorKind {
        self.kind
    }
    pub const fn path(&self) -> Option<&PropertyPath> {
        self.path.as_ref()
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
