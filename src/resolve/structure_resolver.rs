// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Explicit cross-model resolution and immutable resolved views.

use std::any::TypeId;
use std::collections::HashMap;

use super::error::ResolveError;
use super::error::ResolveErrorKind;
use super::error::ResolveErrors;
use super::graph::ModelGraph;
use super::graph::ResolvedProjectionProducer;
use super::graph::ResolvedProjectionSource;
use super::graph::ResolvedReference;
use super::internal::ResolutionContext;
use super::internal::ResolveScope;
use super::internal::discover_nodes;
use super::queries::build_query;
use super::reference_selection_ext::ReferenceSelectionExt;
use super::relations::declared_target_id;
use super::relations::forbidden_entity_nested_role;
use super::relations::push_field_error;
use super::relations::reported_metadata;
use super::relations::resolve_property_path;
use super::relations::validate_value_closure;
use super::resolve_inputs::ResolveInputs;
use crate::metadata::DeclaredEntityTarget;
use crate::metadata::ModelRole;
use crate::metadata::ProjectionMetadata;
use crate::metadata::PropertyMetadata;
use crate::metadata::PropertyPath;
use crate::metadata::PropertyResolutionError;
use crate::metadata::ReferenceSelection;
use crate::metadata::TypeMetadata;
use crate::registry::ModelRegistry;
use crate::registry::ModelRegistryOrigin;

/// Resolves declaration-only metadata against explicit registries.
///
/// The registry supplies the snapshot's model capabilities. Explicit roots
/// include anonymous models without inventing stable IDs, and discovered
/// children participate in the same structural checks. Resolution borrows the
/// registry; it does not initialize or consult a process-global registry.
/// Use [`StructureResolver::new`] to audit every concrete registration, or
/// [`StructureResolver::for_roots`] to resolve only the subgraph reachable
/// from explicit roots. Use [`StructureResolver::for_static_roots`] when the
/// resulting graph must outlive a caller-local root array.
///
/// # Examples
///
/// Include `qubit-model-derive` to declare models. Structural resolution works
/// without optional features; execution of validation declarations separately
/// requires the `validation` feature and a validation plan.
///
/// ```
/// use std::error::Error;
///
/// use qubit_model_derive::Model;
/// use qubit_model_derive::Value;
/// use qubit_model_metadata::metadata::TypeMetadata;
/// use qubit_model_metadata::registry::ModelRegistry;
/// use qubit_model_metadata::resolve::ResolveErrorKind;
/// use qubit_model_metadata::resolve::ResolveInputs;
/// use qubit_model_metadata::resolve::StructureResolver;
/// use qubit_reflect::ReflectionContext;
///
/// #[Model(eq, hash)]
/// struct Note { title: String }
///
/// #[Value]
/// struct InvalidValue { note: Note }
///
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let reflection = ReflectionContext::builder().include_linked().build()?;
/// let models = ModelRegistry::from_context(&reflection)?;
/// let root = TypeMetadata::of::<Note>();
/// let roots = [root];
/// let graph = StructureResolver::new(ResolveInputs { models: &models, roots: &roots })
///     .resolve()?;
/// assert!(graph.properties(root).is_some());
/// assert!(root.model_id().is_none());
///
/// // A Value cannot contain a plain Model, even when reached from a root.
/// let invalid_roots = [TypeMetadata::of::<InvalidValue>()];
/// match StructureResolver::new(ResolveInputs { models: &models, roots: &invalid_roots })
///     .resolve()
/// {
///     Ok(_) => panic!("invalid value closure must be rejected"),
///     Err(errors) => assert!(errors.errors().iter()
///         .any(|error| error.kind() == ResolveErrorKind::InvalidValueClosure)),
/// }
/// # Ok(())
/// # }
/// ```
pub struct StructureResolver<'a> {
    /// Explicit registries used by this resolver.
    inputs: ResolveInputs<'a>,
    /// Roots owned by this resolver when their input slice must not constrain
    /// the resulting graph's registry lifetime.
    owned_roots: Option<Box<[&'static TypeMetadata]>>,
    /// Initial node set used by the resolution attempt.
    scope: ResolveScope,
}

impl<'a> StructureResolver<'a> {
    /// Creates a resolver that audits all concrete registrations and roots.
    ///
    /// # Parameters
    ///
    /// - `inputs`: the borrowed registry and additional concrete roots.
    ///
    /// # Returns
    ///
    /// A declaration resolver borrowing `inputs` for the resulting graph's
    /// lifetime.
    #[must_use]
    #[inline]
    pub const fn new(inputs: ResolveInputs<'a>) -> Self {
        Self {
            inputs,
            owned_roots: None,
            scope: ResolveScope::AllRegistered,
        }
    }

    /// Creates a resolver that starts from explicit roots and their reachable
    /// models.
    ///
    /// The registry remains available for resolving reachable references and
    /// supplying the canonical metadata and properties for each model.
    ///
    /// # Parameters
    ///
    /// - `inputs`: the borrowed registry and additional concrete roots.
    ///
    /// # Returns
    ///
    /// A declaration resolver borrowing `inputs` for the resulting graph's
    /// lifetime and limited to the roots' reachable model subgraph.
    #[must_use]
    #[inline]
    pub const fn for_roots(inputs: ResolveInputs<'a>) -> Self {
        Self {
            inputs,
            owned_roots: None,
            scope: ResolveScope::ReachableFromRoots,
        }
    }

    /// Creates a root-scoped resolver that owns its root slice.
    ///
    /// The resulting graph borrows only `models`; the root array is used
    /// during resolution and may be local to the caller.
    ///
    /// # Parameters
    ///
    /// - `models`: Registry supplying model capabilities and retained by the
    ///   graph.
    /// - `roots`: Static metadata roots copied into the resolver.
    ///
    /// # Returns
    ///
    /// A resolver whose graph lifetime follows the registry, independent of
    /// the caller's root array.
    #[must_use]
    pub fn for_static_roots<const N: usize>(models: &'a ModelRegistry<'a>, roots: [&'static TypeMetadata; N]) -> Self {
        let owned_roots: Box<[&'static TypeMetadata]> = Box::new(roots);
        Self {
            inputs: ResolveInputs { models, roots: &[] },
            owned_roots: Some(owned_roots),
            scope: ResolveScope::ReachableFromRoots,
        }
    }

    /// Resolves model structure and property capabilities.
    ///
    /// # Returns
    ///
    /// An immutable graph borrowing the configured registry and retaining
    /// properties and relationships for all discovered concrete models.
    ///
    /// # Errors
    ///
    /// Returns [`ResolveErrors`] when any property, relationship, role,
    /// projection, value-closure, or query invariant cannot be
    /// resolved against the configured registries.
    /// Validation declarations remain available from the metadata and are
    /// bound later by `ValidationPlan` when the `validation` feature is
    /// enabled.
    pub fn resolve(self) -> Result<ModelGraph<'a>, ResolveErrors> {
        self.resolve_internal()
    }

    /// Returns the explicit roots, whether borrowed or owned by this resolver.
    #[inline]
    fn roots(&self) -> &[&'static TypeMetadata] {
        self.owned_roots.as_deref().unwrap_or(self.inputs.roots)
    }

    /// Resolves registered models, explicit roots, and discovered children.
    ///
    /// # Returns
    ///
    /// An immutable graph when every structural invariant holds.
    ///
    /// # Errors
    ///
    /// Accumulates property assembly, capability, relationship, role, closure,
    /// projection, and query failures in deterministic diagnostic order.
    fn resolve_internal(&self) -> Result<ModelGraph<'a>, ResolveErrors> {
        let mut context = ResolutionContext::new(self.inputs.models, self.roots());
        let mut references = HashMap::new();
        let mut dependencies = Vec::new();
        let mut projection_sources = HashMap::new();
        let mut queries = HashMap::new();
        let mut properties = HashMap::new();
        let mut projection_producers = Vec::new();
        let mut errors = Vec::new();

        let (nodes, mut discovery_errors) = discover_nodes(self.inputs.models, self.roots(), self.scope, &mut context);
        errors.append(&mut discovery_errors);
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
                            .with_cause(PropertyResolutionError::Assembly(build_errors.clone()).into()),
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
                resolve_field_dependencies(
                    metadata,
                    field,
                    &context,
                    fragment_source,
                    &mut errors,
                    &mut dependencies,
                );

                let field_segments = field.name().map(|name| [name]);
                let field_path = field_segments.as_ref().map(|segments| PropertyPath::new(segments));
                if field.is_opaque() {
                    let hidden = field
                        .opaque_type_id()
                        .and_then(|type_id| context.reflection().find_type_by_id(type_id).ok().flatten())
                        .and_then(|descriptor| context.metadata_for(descriptor).ok().flatten())
                        .or_else(|| {
                            metadata
                                .field_descriptor_in(field, context.reflection())
                                .and_then(|field| field.find_concrete_type())
                                .and_then(|descriptor| context.metadata_for(descriptor).ok().flatten())
                        });
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
                    && let Some(descriptor) = metadata
                        .field_descriptor_in(field, context.reflection())
                        .and_then(|field| field.find_concrete_type())
                {
                    match forbidden_entity_nested_role(descriptor, &context) {
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
                super::relations::validate_unique_scope(metadata, field, &context, fragment_source, &mut errors);
                if let Some(reference) = field.reference() {
                    let mut local_reference_valid = true;
                    if let Some(navigation) = reference.path() {
                        match super::relations::resolve_object_binding(metadata, navigation, &context) {
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
                                    match resolve_property_path(target, path, &context) {
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
                                Some(property) => property.descriptor_in(context.reflection()).ok().flatten(),
                                None => Some(target.descriptor_in(context.reflection())),
                            };
                            if let (Some(expected), Some(actual)) = (
                                expected,
                                metadata
                                    .field_descriptor_in(field, context.reflection())
                                    .and_then(|field| field.find_concrete_type()),
                            ) && !super::relations::reference_value_matches(expected, actual)
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
                                    field.location().expect("validated concrete field identity"),
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
                            target
                                .as_entity()
                                .and_then(|entity| {
                                    target.field_descriptor_in(entity.identifier(), context.reflection())
                                })
                                .and_then(|field| field.find_concrete_type()),
                            metadata
                                .field_descriptor_in(projection.identifier(), context.reflection())
                                .and_then(|field| field.find_concrete_type()),
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
                        projection_sources.insert(metadata.type_id(), ResolvedProjectionSource { target });
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
                validate_value_closure(metadata, &context, &mut visited, fragment_source, &mut errors);
            }

            for error in &mut errors[first_error..] {
                error.attach_owner(metadata);
            }
            if metadata.as_entity().is_some() {
                let query = build_query(metadata);
                queries.insert(metadata.type_id(), query);
            }
        }

        self.resolve_projection_producers(&nodes, &properties, &context, &mut errors, &mut projection_producers);

        if errors.is_empty() {
            Ok(ModelGraph {
                registry: self.inputs.models,
                dependencies,
                models: nodes.iter().map(|(metadata, _)| *metadata).collect(),
                model_index: nodes
                    .iter()
                    .map(|(metadata, _)| (metadata.type_id(), *metadata))
                    .collect(),
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

    /// Resolves entity properties that produce projections.
    ///
    /// Metadata lookup failures and inconsistent source or identifier types
    /// are appended to `errors`; valid producers are appended in node and
    /// property order.
    fn resolve_projection_producers<'b>(
        &self,
        nodes: &[(&'static TypeMetadata, Option<&'b ModelRegistryOrigin>)],
        properties: &HashMap<TypeId, &'static [PropertyMetadata]>,
        context: &ResolutionContext<'a>,
        errors: &mut Vec<ResolveError>,
        projection_producers: &mut Vec<ResolvedProjectionProducer>,
    ) {
        for &(source, fragment_source) in nodes {
            if source.role() != ModelRole::Entity {
                continue;
            }
            let Some(local_properties) = properties.get(&source.type_id()) else {
                continue;
            };
            for property in local_properties.properties() {
                let Some(getter) = property.getter() else {
                    continue;
                };
                let Some(projection) = property
                    .descriptor_in(context.reflection())
                    .ok()
                    .flatten()
                    .and_then(|descriptor| {
                        reported_metadata(
                            descriptor,
                            context,
                            source,
                            Some(PropertyPath::new(&[property.name()])),
                            fragment_source,
                            errors,
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
                    let mut error = ResolveError::new(
                        ResolveErrorKind::InvalidProjectionProducer,
                        projection.model_id().map(|id| id.as_str()),
                        None,
                        Some(ModelRole::Entity),
                        Some(source.role()),
                        fragment_source,
                    );
                    error.attach_owner(source);
                    errors.push(error);
                    continue;
                }
                let source_id = source
                    .as_entity()
                    .and_then(|entity| source.field_descriptor_in(entity.identifier(), context.reflection()))
                    .and_then(|field| field.find_concrete_type());
                let projection_id = projection
                    .as_projection()
                    .and_then(|role| projection.field_descriptor_in(role.identifier(), context.reflection()))
                    .and_then(|field| field.find_concrete_type());
                if source_id
                    .zip(projection_id)
                    .is_some_and(|(source, projection)| source.type_id() != projection.type_id())
                {
                    let mut error = ResolveError::new(
                        ResolveErrorKind::InvalidProjectionProducer,
                        projection.model_id().map(|id| id.as_str()),
                        None,
                        Some(ModelRole::Projection),
                        Some(projection.role()),
                        fragment_source,
                    );
                    error.attach_owner(source);
                    errors.push(error);
                    continue;
                }
                projection_producers.push(ResolvedProjectionProducer {
                    source,
                    projection,
                    property: *property,
                    projector: Some(getter),
                });
            }
        }
    }

    /// Resolves a declaration-time target through the configured model
    /// registry.
    ///
    /// # Parameters
    ///
    /// - `target`: the stable model ID or concrete type provider declared by
    ///   the reference.
    ///
    /// # Returns
    ///
    /// Returns the target metadata when it is available from the configured
    /// registry; otherwise returns `None`.
    #[inline]
    fn resolve_target(&self, target: &DeclaredEntityTarget) -> Option<&'static TypeMetadata> {
        super::relations::resolve_declared_target(target, self.inputs.models)
    }
}

/// Validates and records dependency bindings declared by one field.
///
/// Bindings are visited in validator and selector order; validation errors
/// are appended before each binding is retained in the resolved graph.
fn resolve_field_dependencies<'a>(
    metadata: &'static TypeMetadata,
    field: &PropertyMetadata,
    context: &ResolutionContext<'_>,
    fragment_source: Option<&'a ModelRegistryOrigin>,
    errors: &mut Vec<ResolveError>,
    dependencies: &mut Vec<super::graph::ResolvedDependency>,
) {
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
            let requirement = if declaration.object_path().requires_parent() {
                super::graph::ContextRequirement::ParentObject
            } else {
                super::graph::ContextRequirement::None
            };
            super::relations::validate_dependency(metadata, declaration, context, fragment_source, errors);
            dependencies.push(super::graph::ResolvedDependency {
                declaration,
                context: requirement,
            });
        }
    }
}
