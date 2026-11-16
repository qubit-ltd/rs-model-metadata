// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Resolved direct field references.

use super::context_requirement::ContextRequirement;
use crate::metadata::FieldReferenceMetadata;
use crate::metadata::PropertyMetadata;
use crate::metadata::TypeMetadata;

/// A successfully resolved direct reference.
///
/// # Examples
///
/// ```
/// use qubit_id::Id;
/// use qubit_model_derive::Entity;
/// use qubit_model_derive::Model;
/// use qubit_model_metadata::metadata::TypeMetadata;
/// use qubit_model_metadata::registry::ModelRegistry;
/// use qubit_model_metadata::resolve::ResolvedReference;
/// use qubit_model_metadata::resolve::StructureResolver;
///
/// #[Entity(id = "references.Target")]
/// struct Target {
///     #[identifier]
///     id: Id,
/// }
///
/// #[Model(id = "references.Root")]
/// struct Root {
///     #[reference(entity = Target, property = id)]
///     target: Id,
/// }
/// # fn main() {
/// let models = ModelRegistry::try_global().expect("model registry");
/// let root = TypeMetadata::of::<Root>();
/// let graph = StructureResolver::for_static_roots(&models, [root])
///     .resolve().expect("valid structure");
/// let location = root.fields()[0].location().expect("reflected field");
/// let reference: &ResolvedReference = graph.reference(location).expect("resolved reference");
/// assert_eq!(reference.target().type_id(), TypeMetadata::of::<Target>().type_id());
/// assert_eq!(reference.property().expect("selected property").name(), "id");
/// # }
/// ```
#[derive(Debug)]
pub struct ResolvedReference {
    /// The original field-reference declaration.
    pub(in crate::resolve) declaration: &'static FieldReferenceMetadata,
    /// The resolved target model metadata.
    pub(in crate::resolve) target: &'static TypeMetadata,
    /// The selected target property, or `None` for an entity-level reference.
    pub(in crate::resolve) property: Option<PropertyMetadata>,
}

impl ResolvedReference {
    /// Reports whether the reference binding path needs a containing object.
    ///
    /// # Returns
    ///
    /// [`ContextRequirement::ParentObject`] when the binding path needs a
    /// caller-provided parent, or [`ContextRequirement::None`] otherwise.
    #[must_use]
    #[inline]
    pub fn context_requirement(&self) -> ContextRequirement {
        if self.declaration.path().is_some_and(|path| path.requires_parent()) {
            ContextRequirement::ParentObject
        } else {
            ContextRequirement::None
        }
    }

    /// Returns the original field-reference declaration.
    ///
    /// # Returns
    ///
    /// The static field-reference declaration used to resolve this edge.
    #[must_use]
    #[inline]
    pub const fn declaration(&self) -> &'static FieldReferenceMetadata {
        self.declaration
    }

    /// Returns the resolved target model metadata.
    ///
    /// # Returns
    ///
    /// The static metadata for the resolved target model.
    #[must_use]
    #[inline]
    pub const fn target(&self) -> &'static TypeMetadata {
        self.target
    }

    /// Returns the selected target property, or `None` for an entity reference.
    ///
    /// # Returns
    ///
    /// `Some` with the selected property for a property reference, or `None`
    /// when the reference targets the entity as a whole.
    #[must_use]
    #[inline]
    pub const fn property(&self) -> Option<&PropertyMetadata> {
        self.property.as_ref()
    }
}
