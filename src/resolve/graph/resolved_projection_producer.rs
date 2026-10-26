// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Resolved Entity-to-Projection producer edges and execution.

use qubit_id::Id;
use qubit_reflect::descriptor::FieldAccessError;
use qubit_reflect::error::TypeMismatch;
use qubit_reflect::value::DynamicRef;
use qubit_reflect::value::Local;

use crate::metadata::GetterMetadata;
use crate::metadata::PropertyMetadata;
use crate::metadata::PropertyValue;
use crate::metadata::TypeMetadata;
use crate::resolve::ProjectionExecutionError;

/// One resolved readable property that produces a Projection from an Entity.
///
/// # Examples
///
/// ```
/// use qubit_model_derive::Model;
/// use qubit_model_metadata::metadata::TypeMetadata;
/// use qubit_model_metadata::registry::ModelRegistry;
/// use qubit_model_metadata::resolve::ResolveInputs;
/// use qubit_model_metadata::resolve::ResolvedProjectionProducer;
/// use qubit_model_metadata::resolve::StructureResolver;
///
/// #[Model]
/// struct Draft { title: String }
/// # fn main() {
/// let models = ModelRegistry::try_global().expect("model registry");
/// let root = TypeMetadata::of::<Draft>();
/// let graph = StructureResolver::for_static_roots(&models, [root])
///     .resolve().expect("valid structure");
/// let producers: &[ResolvedProjectionProducer] = graph.projection_producers();
/// assert!(producers.is_empty());
/// # }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct ResolvedProjectionProducer {
    /// Entity declaring the readable property.
    pub(in crate::resolve) source: &'static TypeMetadata,
    /// Projection returned by the property getter.
    pub(in crate::resolve) projection: &'static TypeMetadata,
    /// Merged local property that declares the producer edge.
    pub(in crate::resolve) property: PropertyMetadata,
    /// Executable getter adapter, when automatic projection is available.
    pub(in crate::resolve) projector: Option<&'static GetterMetadata>,
}

impl ResolvedProjectionProducer {
    /// Returns the producing Entity metadata.
    ///
    /// # Returns
    ///
    /// Static metadata for the Entity that declares the readable property.
    #[must_use]
    #[inline]
    pub const fn source(&self) -> &'static TypeMetadata {
        self.source
    }

    /// Returns the produced Projection metadata.
    ///
    /// # Returns
    ///
    /// Static metadata for the Projection returned by the property getter.
    #[must_use]
    #[inline]
    pub const fn projection(&self) -> &'static TypeMetadata {
        self.projection
    }

    /// Returns the property that declares this edge.
    ///
    /// # Returns
    ///
    /// The merged local property that declares the producer edge.
    #[must_use]
    #[inline]
    pub const fn property(&self) -> &PropertyMetadata {
        &self.property
    }

    /// Returns the executable getter used as projector. The current resolver
    /// only creates producer edges for properties with a registered getter.
    ///
    /// # Returns
    ///
    /// `Some` with the registered getter adapter, or `None` when this edge
    /// cannot be executed automatically.
    #[inline]
    pub const fn projector(&self) -> Option<&'static GetterMetadata> {
        self.projector
    }

    /// Executes the projector and verifies identifier preservation.
    ///
    /// # Parameters
    ///
    /// - `source`: The concrete Entity instance passed to the registered
    ///   getter.
    ///
    /// # Returns
    ///
    /// The getter result when it is a borrowed or owned Projection with the
    /// same identifier as the source Entity.
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
    pub fn project<'a>(&self, source: DynamicRef<'a, Local>) -> Result<PropertyValue<'a>, ProjectionExecutionError> {
        let projector = self.projector.ok_or(ProjectionExecutionError::MissingProjector)?;
        let source_identifier = self
            .source
            .as_entity()
            .ok_or(ProjectionExecutionError::InvalidProducer)?
            .read_identifier(source.clone())
            .map_err(identifier_target_mismatch)?;
        let result = projector.get_dynamic(source.clone())?;
        let projection_identifier = match &result {
            PropertyValue::Borrowed(value) => self.projection_identifier(value.clone())?,
            PropertyValue::Owned(value) => self.projection_identifier(value.as_ref())?,
            PropertyValue::OptionalBorrowed(_) | PropertyValue::BorrowedSlice(_) => {
                return Err(ProjectionExecutionError::InvalidProducer);
            }
        };
        if *source_identifier != projection_identifier {
            return Err(ProjectionExecutionError::IdentifierMismatch);
        }
        Ok(result)
    }

    /// Reads and validates the identifier from a projected target.
    ///
    /// # Parameters
    ///
    /// - `target`: The projected instance whose identifier is read.
    ///
    /// # Returns
    ///
    /// The projected target identifier as a copied [`Id`].
    ///
    /// # Errors
    ///
    /// Returns [`ProjectionExecutionError`] when the target is not a valid
    /// Projection or its identifier field cannot be read as `qubit_id::Id`.
    ///
    /// # Panics
    ///
    /// Panics if resolved projection metadata refers to an identifier that is
    /// not a concrete reflected field. The resolver guarantees this invariant.
    fn projection_identifier(&self, target: DynamicRef<'_, Local>) -> Result<Id, ProjectionExecutionError> {
        let projection = self
            .projection
            .as_projection()
            .ok_or(ProjectionExecutionError::InvalidProducer)?;
        let identifier = projection.read_identifier(target).map_err(identifier_target_mismatch)?;
        Ok(*identifier)
    }
}

/// Converts a reflected type mismatch into a field-access failure.
#[inline]
fn identifier_target_mismatch(mismatch: TypeMismatch) -> FieldAccessError {
    FieldAccessError::TargetTypeMismatch(mismatch)
}
