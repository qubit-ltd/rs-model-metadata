// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Immutable structure graph and resolved structural views.
// qubit-style: allow multiple-public-types

use std::any::TypeId;
use std::collections::HashMap;

use qubit_id::Id;
use qubit_reflect::FieldAccessError;
use qubit_reflect::ReflectedRef;

use super::owned_property_path::OwnedPropertyPath;
use crate::metadata::DependencyBindingMetadata;
use crate::metadata::FieldLocation;
use crate::metadata::FieldMetadata;
use crate::metadata::FieldReferenceMetadata;
use crate::metadata::GetterMetadata;
use crate::metadata::IndexingReasons;
use crate::metadata::LocalPropertySet;
use crate::metadata::PropertyAccessError;
use crate::metadata::PropertyMetadata;
use crate::metadata::PropertyPath;
use crate::metadata::PropertyValue;
use crate::metadata::TypeMetadata;
use crate::registry::ModelRegistry;

/// Instance context that cannot be obtained from a structural registry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextRequirement {
    /// All object navigation is local to the declared root.
    None,
    /// Navigation needs a containing object supplied by the consumer.
    ParentObject,
}

/// A dependency occurrence with its independently retained context requirement.
#[derive(Debug)]
pub struct ResolvedDependency {
    /// The exact dependency declaration retained from its validator occurrence.
    pub(super) declaration: &'static DependencyBindingMetadata,
    /// Whether resolving the object path requires a caller-supplied parent.
    pub(super) context: ContextRequirement,
}

impl ResolvedDependency {
    /// Returns the original occurrence, including source and separate paths.
    #[must_use]
    #[inline(always)]
    pub const fn declaration(&self) -> &'static DependencyBindingMetadata {
        self.declaration
    }

    /// Reports whether the consumer must supply a containing object.
    #[must_use]
    #[inline(always)]
    pub const fn context_requirement(&self) -> ContextRequirement {
        self.context
    }
}

/// A successfully resolved direct reference.
#[derive(Debug)]
pub struct ResolvedReference {
    /// The original field-reference declaration.
    pub(super) declaration: &'static FieldReferenceMetadata,
    /// The resolved target model metadata.
    pub(super) target: &'static TypeMetadata,
    /// The selected target property, or `None` for an entity-level reference.
    pub(super) property: Option<&'static PropertyMetadata>,
}

impl ResolvedReference {
    /// Reports whether the reference binding path needs a containing object.
    #[must_use]
    pub fn context_requirement(&self) -> ContextRequirement {
        if self.declaration.path().is_some_and(|path| path.requires_parent()) {
            ContextRequirement::ParentObject
        } else {
            ContextRequirement::None
        }
    }

    /// Returns the original field-reference declaration.
    #[must_use]
    #[inline(always)]
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
    pub(super) target: &'static TypeMetadata,
}

/// One resolved readable property that produces a Projection from an Entity.
#[derive(Clone, Copy, Debug)]
pub struct ResolvedProjectionProducer {
    /// Entity declaring the readable property.
    pub(super) source: &'static TypeMetadata,
    /// Projection returned by the property getter.
    pub(super) projection: &'static TypeMetadata,
    /// Merged local property that declares the producer edge.
    pub(super) property: &'static PropertyMetadata,
    /// Executable getter adapter, when automatic projection is available.
    pub(super) projector: Option<&'static GetterMetadata>,
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

    /// Returns the executable getter used as projector. The current resolver
    /// only creates producer edges for properties with a registered getter.
    #[must_use]
    #[inline(always)]
    pub const fn projector(&self) -> Option<&'static GetterMetadata> {
        self.projector
    }

    /// Executes the projector and verifies identifier preservation.
    ///
    /// # Errors
    ///
    /// Returns [`ProjectionExecutionError::Field`] when an identifier cannot
    /// be read, including a source of the wrong concrete type; propagates
    /// getter failures as [`ProjectionExecutionError::Property`]. Returns
    /// [`ProjectionExecutionError::IdentifierMismatch`] if the produced
    /// projection changes the source identifier. Invalid producer metadata or
    /// output shapes are reported by the corresponding structural variants.
    ///
    /// # Panics
    ///
    /// Panics if resolved producer metadata refers to an identifier that is
    /// not a concrete reflected field. The resolver guarantees this invariant.
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
///
/// The graph borrows its registry and freezes the metadata and properties
/// selected during resolution. Query concrete models by [`TypeId`] and
/// references by field location; neither lookup uses a copied field's address.
/// Anonymous models must be included among the explicit roots when they are
/// not discoverable through the supplied registry.
///
/// # Examples
///
/// ```
/// use qubit_model_derive::Model;
/// use qubit_model_metadata::metadata::TypeMetadata;
/// use qubit_model_metadata::registry::ModelRegistry;
/// use qubit_model_metadata::resolve::ResolveInputs;
/// use qubit_model_metadata::resolve::StructureResolver;
///
/// #[Model]
/// struct Draft { title: String }
/// # fn main() {
/// let models = ModelRegistry::from_metadata(&[]).expect("isolated registry");
/// let root = TypeMetadata::of::<Draft>();
/// let roots = [root];
/// let graph = StructureResolver::new(ResolveInputs { models: &models, roots: &roots })
///     .resolve().expect("valid structure");
/// assert_eq!(graph.models().len(), 1);
/// assert_eq!(graph.model(root.type_id()).expect("explicit root").type_id(), root.type_id());
/// # }
/// ```
#[must_use]
#[derive(Debug)]
pub struct ModelGraph<'a> {
    /// Concrete model nodes accepted by this graph.
    pub(super) models: Vec<&'static TypeMetadata>,
    /// Concrete nodes indexed independently of registration names.
    pub(super) model_index: HashMap<TypeId, &'static TypeMetadata>,
    /// Dependency occurrences in model and source declaration order.
    pub(super) dependencies: Vec<ResolvedDependency>,
    /// The registry used to resolve the graph.
    pub(super) registry: &'a ModelRegistry<'a>,
    /// Resolved field references keyed by declaration identity.
    pub(super) references: HashMap<FieldLocation, ResolvedReference>,
    /// Resolved projection sources keyed by declaration identity.
    pub(super) projection_sources: HashMap<TypeId, ResolvedProjectionSource>,
    /// Resolved query metadata keyed by entity declaration identity.
    pub(super) queries: HashMap<TypeId, QueryMetadata>,
    /// Locally assembled properties keyed by concrete type identity.
    pub(super) properties: HashMap<TypeId, &'static LocalPropertySet>,
    /// Automatic Entity-to-Projection producer edges.
    pub(super) projection_producers: Vec<ResolvedProjectionProducer>,
}

impl<'a> ModelGraph<'a> {
    /// Returns every dependency occurrence, including deferred parent paths.
    #[must_use]
    #[inline(always)]
    pub fn dependencies(&self) -> &[ResolvedDependency] {
        &self.dependencies
    }

    /// Returns registered and explicitly reachable concrete nodes.
    #[must_use]
    #[inline(always)]
    pub fn models(&self) -> &[&'static TypeMetadata] {
        &self.models
    }

    /// Returns locally merged properties accepted during graph resolution,
    /// or `None` when the concrete type was not included in this graph.
    ///
    /// The lookup uses the supplied metadata's type identity; it does not
    /// reassemble its properties or consult the global registry.
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

    /// Returns a concrete node included in this graph, including anonymous
    /// roots, or `None` for a type outside this graph. This lookup does not
    /// register types or consult a global registry.
    #[must_use]
    pub fn model(&self, id: TypeId) -> Option<&'static TypeMetadata> {
        self.model_index.get(&id).copied()
    }

    /// Returns the reference at this declaration, or `None` if absent.
    #[must_use]
    pub fn reference(&self, location: FieldLocation) -> Option<&ResolvedReference> {
        self.references.get(&location)
    }

    /// Returns the source of this projection type, or `None` if absent.
    #[must_use]
    pub fn projection_source(&self, projection: TypeId) -> Option<&ResolvedProjectionSource> {
        self.projection_sources.get(&projection)
    }

    /// Returns query metadata for this entity type, or `None` if absent.
    #[must_use]
    pub fn query(&self, entity: TypeId) -> Option<&QueryMetadata> {
        self.queries.get(&entity)
    }
}

/// Direct indexed declarations available to downstream query generators.
#[derive(Debug)]
pub struct QueryMetadata {
    /// Direct declarations in source field order.
    pub(super) declarations: Box<[QueryDeclaration]>,
}

impl QueryMetadata {
    /// Returns declarations without choosing filter operators or external
    /// names.
    #[must_use]
    #[inline(always)]
    pub fn declarations(&self) -> &[QueryDeclaration] {
        &self.declarations
    }
}

/// One declared indexed member and the facts making it indexed.
#[derive(Clone, Debug)]
pub struct QueryDeclaration {
    /// Original field carrying type, uniqueness and reference metadata.
    pub(super) field: &'static FieldMetadata,
    /// Direct member path relative to the declaring Entity.
    pub(super) path: OwnedPropertyPath,
}

impl QueryDeclaration {
    /// Returns the original declaration for further metadata navigation.
    #[must_use]
    #[inline(always)]
    pub const fn field(&self) -> &'static FieldMetadata {
        self.field
    }

    /// Returns the direct property path in declaration order.
    #[must_use]
    pub fn path(&self) -> PropertyPath<'_> {
        self.path.as_path()
    }

    /// Returns all explicit and implicit indexing reasons.
    #[must_use]
    pub fn reasons(&self) -> IndexingReasons {
        self.field.indexing_reasons()
    }
}
