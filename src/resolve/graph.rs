//! Immutable structure graph and resolved structural views.
// qubit-style: allow multiple-public-types

use std::any::TypeId;
use std::collections::HashMap;

use qubit_id::Id;
use qubit_reflect::FieldAccessError;
use qubit_reflect::ReflectedRef;
use qubit_reflect::TypeDescriptor;

use super::owned_property_path::OwnedPropertyPath;
use crate::metadata::FieldMetadata;
use crate::metadata::FieldReferenceMetadata;
use crate::metadata::GetterMetadata;
use crate::metadata::IndexingReasons;
use crate::metadata::LocalPropertySet;
use crate::metadata::ProjectionMetadata;
use crate::metadata::PropertyAccessError;
use crate::metadata::PropertyMetadata;
use crate::metadata::PropertyPath;
use crate::metadata::PropertyValue;
use crate::metadata::TypeMetadata;
use crate::registry::ModelRegistry;
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
pub struct ModelGraph<'a> {
    /// The registry used to resolve the graph.
    pub(super) registry: &'a ModelRegistry<'a>,
    /// Resolved field references keyed by declaration identity.
    pub(super) references: HashMap<usize, ResolvedReference>,
    /// Resolved projection sources keyed by declaration identity.
    pub(super) projection_sources: HashMap<usize, ResolvedProjectionSource>,
    /// Resolved query metadata keyed by entity declaration identity.
    pub(super) queries: HashMap<usize, QueryMetadata>,
    pub(super) properties: HashMap<TypeId, &'static LocalPropertySet>,
    pub(super) projection_producers: Vec<ResolvedProjectionProducer>,
}

impl<'a> ModelGraph<'a> {
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

    /// Returns query metadata for `entity`, or `None` when it is not resolved.
    #[must_use]
    pub fn query(&self, entity: &crate::metadata::EntityMetadata) -> Option<&QueryMetadata> {
        self.queries
            .get(&(entity as *const crate::metadata::EntityMetadata as usize))
    }
}

/// Queryable indexed fields derived for one resolved entity.
#[derive(Debug)]
pub struct QueryMetadata {
    /// Indexed field paths that can be used as query filters.
    pub(super) filters: Box<[QueryField]>,
    /// Identifier and globally unique lookup keys.
    pub(super) unique_keys: Box<[UniqueQueryKey]>,
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
    pub(super) path: OwnedPropertyPath,
    /// The flattened external name used for queries.
    pub(super) flat_name: Box<str>,
    /// The resolved descriptor, or `None` for an opaque type.
    pub(super) descriptor: Option<&'static TypeDescriptor>,
    /// The declaration facts that made the field queryable.
    pub(super) reasons: IndexingReasons,
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
    pub(super) fn new(paths: Vec<OwnedPropertyPath>) -> Self {
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
/// Returns the stable identity key used for a static field declaration.
pub(super) fn pointer_key(field: &FieldMetadata) -> usize {
    field as *const FieldMetadata as usize
}
