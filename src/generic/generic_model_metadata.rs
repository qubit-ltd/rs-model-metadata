// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Metadata for one registered generic model declaration.

use qubit_reflect::TypeDefinitionDescriptor;

use crate::metadata::EnumVariantMetadata;
use crate::metadata::FieldMetadata;
use crate::metadata::ModelId;
use crate::metadata::ModelRole;

/// Immutable metadata for a generic model template.
///
/// Available with the `generic` feature. Model macros emit this symbolic
/// definition once; [`TypeMetadata::of`](crate::metadata::TypeMetadata::of)
/// exposes concrete specializations that share it. The template's fields and
/// variants describe type parameters rather than concrete runtime instances.
///
/// # Examples
///
/// Enable `qubit-model-metadata`'s `generic` feature and include
/// `qubit-model-derive` to declare the model. This example builds an isolated
/// registry and supplies the concrete specialization as an explicit root.
///
/// ```
/// use qubit_model_derive::Model;
/// use qubit_model_metadata::generic::GenericModelMetadata;
/// use qubit_model_metadata::metadata::TypeMetadata;
/// use qubit_model_metadata::registry::ModelRegistry;
/// use qubit_model_metadata::resolve::ResolveInputs;
/// use qubit_model_metadata::resolve::StructureResolver;
/// use qubit_reflect::identity::FragmentIdentity;
/// use std::error::Error;
///
/// #[Model(id = "example.GenericEnvelope")]
/// struct Envelope<T> { value: T }
///
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let root = TypeMetadata::of::<Envelope<String>>();
/// let template: &GenericModelMetadata = root.generic_definition()
///     .expect("the Model macro emits a generic definition");
/// let source = FragmentIdentity::new("example", "rustdoc", 1, 1, "generic-model", 0);
/// let models = ModelRegistry::from_static_metadata_with_generics(
///     &[], &[(template, &source)],
/// )?;
/// assert_eq!(models.generic("example.GenericEnvelope").map(|item| item.model_id()),
///     Some(template.model_id()));
/// assert!(models.generic("example.Missing").is_none());
/// let roots = [root];
/// let graph = StructureResolver::new(ResolveInputs { models: &models, roots: &roots })
///     .resolve()?;
/// assert!(graph.properties(root).is_some());
/// assert_eq!(template.fields().len(), 1);
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct GenericModelMetadata {
    /// The optional stable identifier; anonymous definitions remain queryable.
    model_id: Option<ModelId>,
    /// The semantic role assigned to the model definition.
    role: ModelRole,
    /// The reflection descriptor for the generic definition.
    definition: &'static TypeDefinitionDescriptor,
    /// Symbolic field overlays declared by the generic model.
    fields: &'static [FieldMetadata],
    /// Symbolic enum-variant overlays declared by the generic model.
    variants: &'static [EnumVariantMetadata],
}

impl GenericModelMetadata {
    /// Retains a generated symbolic template without allocating or validating
    /// it.
    ///
    /// # Parameters
    ///
    /// - `model_id`: optional stable ID; `None` retains an anonymous
    ///   definition.
    /// - `role`: the semantic role assigned by the model macro.
    /// - `definition`: shared reflection descriptor for the generic definition.
    /// - `fields`: symbolic overlays for struct fields.
    /// - `variants`: symbolic overlays for enum variants.
    ///
    /// # Returns
    ///
    /// Immutable metadata borrowing the supplied process-lifetime descriptors.
    #[must_use]
    #[inline]
    pub(crate) const fn new(
        model_id: Option<ModelId>,
        role: ModelRole,
        definition: &'static TypeDefinitionDescriptor,
        fields: &'static [FieldMetadata],
        variants: &'static [EnumVariantMetadata],
    ) -> Self {
        Self {
            model_id,
            role,
            definition,
            fields,
            variants,
        }
    }

    /// Returns the stable identifier registered for this generic model.
    ///
    /// # Returns
    ///
    /// `Some` identifies a named definition; `None` denotes an anonymous
    /// definition that remains queryable by its reflection definition identity.
    #[must_use = "the stable model ID identifies the registered generic definition"]
    #[inline]
    pub const fn model_id(&self) -> Option<ModelId> {
        self.model_id
    }

    /// Returns the semantic role assigned to this generic model.
    ///
    /// # Returns
    ///
    /// The role shared by all concrete specializations of this template.
    #[must_use]
    #[inline]
    pub const fn role(&self) -> ModelRole {
        self.role
    }

    /// Returns the shared reflection generic definition.
    ///
    /// # Returns
    ///
    /// The process-lifetime descriptor of the unspecialized type definition.
    #[must_use]
    #[inline]
    pub const fn definition(&self) -> &'static TypeDefinitionDescriptor {
        self.definition
    }

    /// Returns symbolic field overlays for the template.
    ///
    /// # Returns
    ///
    /// Borrowed symbolic struct fields in declaration order; empty for enums
    /// and declarations with no fields.
    #[must_use]
    #[inline]
    pub const fn fields(&self) -> &'static [FieldMetadata] {
        self.fields
    }

    /// Returns symbolic enum-variant overlays for the template.
    ///
    /// # Returns
    ///
    /// Borrowed symbolic variants in declaration order; empty for struct
    /// models.
    #[must_use]
    #[inline]
    pub const fn variants(&self) -> &'static [EnumVariantMetadata] {
        self.variants
    }
}
