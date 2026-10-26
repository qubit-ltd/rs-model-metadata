// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Builder used by generated code to assemble type metadata.

use qubit_reflect::descriptor::Reflect;

use crate::metadata::TypeMetadata;

#[doc(hidden)]
#[must_use]
pub struct GeneratedTypeMetadataBuilder {
    /// Metadata accumulated from generated declarations.
    metadata: TypeMetadata,
}

impl GeneratedTypeMetadataBuilder {
    /// Starts building metadata for one reflected type.
    ///
    /// # Type Parameters
    ///
    /// - `T`: Reflected Rust type represented by the generated metadata.
    ///
    /// # Parameters
    ///
    /// - `model_id`: Optional stable model identifier.
    /// - `fields`: Static field metadata for the type.
    /// - `role`: Static role metadata for the type.
    ///
    /// # Returns
    ///
    /// A builder initialized with the reflected type and its core metadata.
    pub fn new<T: Reflect + 'static>(
        model_id: Option<crate::metadata::ModelId>,
        fields: &'static [crate::metadata::FieldMetadata],
        role: &'static crate::metadata::RoleMetadata,
    ) -> Self {
        Self {
            metadata: TypeMetadata::new::<T>(model_id, fields, role),
        }
    }

    /// Adds generated property metadata to the builder.
    ///
    /// # Parameters
    ///
    /// - `properties`: Static property metadata to attach.
    ///
    /// # Returns
    ///
    /// The builder with the supplied property metadata.
    #[inline]
    pub const fn properties(mut self, properties: &'static [crate::metadata::PropertyMetadata]) -> Self {
        self.metadata = self.metadata.with_properties(properties);
        self
    }

    /// Adds generated field property fragments to the builder.
    ///
    /// # Parameters
    ///
    /// - `fragments`: Static source fragments to attach to generated fields.
    ///
    /// # Returns
    ///
    /// The builder with the supplied fragments.
    #[inline]
    pub const fn property_fragments(mut self, fragments: &'static [crate::metadata::PropertyFragment]) -> Self {
        self.metadata = self.metadata.with_property_fragments(fragments);
        self
    }

    /// Adds lazy providers for concrete nested model field types.
    ///
    /// # Parameters
    ///
    /// - `providers`: Static callbacks that resolve nested model metadata.
    ///
    /// # Returns
    ///
    /// The builder with the supplied callbacks.
    #[inline]
    pub const fn nested_metadata_providers(
        mut self,
        providers: &'static [fn() -> Option<&'static crate::metadata::TypeMetadata>],
    ) -> Self {
        self.metadata = self.metadata.with_nested_metadata_providers(providers);
        self
    }

    /// Records the generic definition represented by this metadata.
    ///
    /// # Parameters
    ///
    /// - `definition`: Static generic model definition to associate.
    ///
    /// # Returns
    ///
    /// The builder associated with the generic definition.
    #[inline]
    #[cfg(feature = "generic")]
    pub const fn generic_definition(mut self, definition: &'static crate::generic::GenericModelMetadata) -> Self {
        self.metadata = self.metadata.with_generic_definition(definition);
        self
    }

    /// Validates and finishes generated metadata for `T`.
    ///
    /// # Type Parameters
    ///
    /// - `T`: Reflected Rust type that the metadata must describe.
    ///
    /// # Returns
    ///
    /// The validated metadata for `T`.
    ///
    /// # Panics
    ///
    /// Panics when generated metadata does not match the reflected type.
    #[must_use]
    pub fn finish<T: 'static>(self) -> TypeMetadata {
        self.metadata.assert_valid_for::<T>();
        self.metadata
    }

    /// Finishes generated metadata for later fallible ABI validation.
    ///
    /// # Returns
    ///
    /// The accumulated metadata without checking it against a reflected type.
    #[doc(hidden)]
    #[must_use]
    #[inline]
    pub const fn finish_unchecked(self) -> TypeMetadata {
        self.metadata
    }
}
