// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Immutable graph produced by a complete structural resolution pass.

use std::any::TypeId;
use std::collections::HashMap;

use crate::metadata::FieldLocation;
use crate::metadata::ResolvedProperties;
use crate::metadata::TypeMetadata;
use crate::registry::ModelRegistry;
use crate::resolve::QueryMetadata;
use crate::resolve::ResolvedDependency;
use crate::resolve::ResolvedProjectionProducer;
use crate::resolve::ResolvedProjectionSource;
use crate::resolve::ResolvedReference;

/// Immutable result of a complete successful resolution pass.
///
/// The graph borrows its registry and freezes the metadata and properties
/// selected during resolution. Query concrete models by [`TypeId`] and
/// references by field location; neither lookup uses a copied field's address.
/// Anonymous models must be included among the explicit roots when they are
/// not discoverable through the supplied registry.
///
/// # Type Parameters
///
/// - `'a`: Lifetime of the registry borrow retained by this graph.
///
/// # Examples
///
/// ```
/// use qubit_model_derive::Model;
/// use qubit_model_metadata::metadata::TypeMetadata;
/// use qubit_model_metadata::registry::ModelRegistry;
/// use qubit_model_metadata::resolve::StructureResolver;
///
/// #[Model]
/// struct Draft { title: String }
/// # fn main() {
/// let models = ModelRegistry::try_global().expect("model registry");
/// let root = TypeMetadata::of::<Draft>();
/// let graph = StructureResolver::for_static_roots(&models, [root])
///     .resolve().expect("valid structure");
/// assert_eq!(graph.models().len(), 1);
/// assert_eq!(graph.model(root.type_id()).expect("explicit root").type_id(), root.type_id());
/// # }
/// ```
#[must_use]
#[derive(Debug)]
pub struct ModelGraph<'a> {
    /// Concrete model nodes accepted by this graph.
    pub(in crate::resolve) models: Vec<&'static TypeMetadata>,
    /// Concrete nodes indexed independently of registration names.
    pub(in crate::resolve) model_index: HashMap<TypeId, &'static TypeMetadata>,
    /// Dependency occurrences in model and source declaration order.
    pub(in crate::resolve) dependencies: Vec<ResolvedDependency>,
    /// The registry used to resolve the graph.
    pub(in crate::resolve) registry: &'a ModelRegistry<'a>,
    /// Resolved field references keyed by declaration identity.
    pub(in crate::resolve) references: HashMap<FieldLocation, ResolvedReference>,
    /// Resolved projection sources keyed by declaration identity.
    pub(in crate::resolve) projection_sources: HashMap<TypeId, ResolvedProjectionSource>,
    /// Resolved query metadata keyed by entity declaration identity.
    pub(in crate::resolve) queries: HashMap<TypeId, QueryMetadata>,
    /// Locally assembled properties keyed by concrete type identity.
    pub(in crate::resolve) properties: HashMap<TypeId, ResolvedProperties>,
    /// Automatic Entity-to-Projection producer edges.
    pub(in crate::resolve) projection_producers: Vec<ResolvedProjectionProducer>,
}

impl<'a> ModelGraph<'a> {
    /// Returns every dependency occurrence, including deferred parent paths.
    ///
    /// # Returns
    ///
    /// Dependency occurrences in model and source declaration order.
    #[must_use]
    #[inline]
    pub fn dependencies(&self) -> &[ResolvedDependency] {
        &self.dependencies
    }

    /// Returns registered and explicitly reachable concrete nodes.
    ///
    /// # Returns
    ///
    /// Static metadata for each concrete node accepted by this graph.
    #[must_use]
    #[inline]
    pub fn models(&self) -> &[&'static TypeMetadata] {
        &self.models
    }

    /// Returns locally merged properties accepted during graph resolution,
    /// or `None` when the concrete type was not included in this graph.
    ///
    /// The lookup uses the supplied metadata's type identity; it does not
    /// reassemble its properties or consult the global registry.
    ///
    /// # Parameters
    ///
    /// - `model`: Metadata whose concrete type identity selects the properties.
    ///
    /// # Returns
    ///
    /// `Some` with locally merged properties when the model is in this graph,
    /// or `None` when its concrete type was not included.
    #[inline]
    pub fn properties(&self, model: &TypeMetadata) -> Option<&ResolvedProperties> {
        self.properties.get(&model.type_id())
    }

    /// Returns all resolved Entity-to-Projection producer edges.
    ///
    /// # Returns
    ///
    /// Producer edges in resolver order.
    #[must_use]
    #[inline]
    pub fn projection_producers(&self) -> &[ResolvedProjectionProducer] {
        &self.projection_producers
    }

    /// Returns the registry used for this resolution pass.
    ///
    /// # Returns
    ///
    /// The registry borrowed by this graph during resolution.
    #[must_use]
    #[inline]
    pub const fn registry(&self) -> &'a ModelRegistry<'a> {
        self.registry
    }

    /// Returns a concrete node included in this graph, including anonymous
    /// roots, or `None` for a type outside this graph. This lookup does not
    /// register types or consult a global registry.
    ///
    /// # Parameters
    ///
    /// - `id`: Concrete Rust type identity to look up.
    ///
    /// # Returns
    ///
    /// `Some` with the graph node for an included type, or `None` when the
    /// identity is outside this graph.
    #[inline]
    pub fn model(&self, id: TypeId) -> Option<&'static TypeMetadata> {
        self.model_index.get(&id).copied()
    }

    /// Returns the reference at this declaration, or `None` if absent.
    ///
    /// # Parameters
    ///
    /// - `location`: Source field declaration identity for the reference.
    ///
    /// # Returns
    ///
    /// `Some` with the resolved edge at this location, or `None` when no
    /// reference was resolved for it.
    #[inline]
    pub fn reference(&self, location: FieldLocation) -> Option<&ResolvedReference> {
        self.references.get(&location)
    }

    /// Returns the source of this projection type, or `None` if absent.
    ///
    /// # Parameters
    ///
    /// - `projection`: Concrete type identity of the Projection to look up.
    ///
    /// # Returns
    ///
    /// `Some` with the source Entity metadata, or `None` when the graph has no
    /// source edge for this type.
    #[inline]
    pub fn projection_source(&self, projection: TypeId) -> Option<&ResolvedProjectionSource> {
        self.projection_sources.get(&projection)
    }

    /// Returns query metadata for this entity type, or `None` if absent.
    ///
    /// # Parameters
    ///
    /// - `entity`: Concrete type identity of the Entity to look up.
    ///
    /// # Returns
    ///
    /// `Some` with the indexed declarations for this Entity, or `None` when
    /// it has no query metadata in this graph.
    #[inline]
    pub fn query(&self, entity: TypeId) -> Option<&QueryMetadata> {
        self.queries.get(&entity)
    }
}
