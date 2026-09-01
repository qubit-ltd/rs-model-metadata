// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Metadata for one registered generic model declaration.

use qubit_reflect::expression::GenericDefinitionDescriptor;

use crate::FieldMetadata;
use crate::ModelId;
use crate::ModelRole;

/// Immutable metadata for a generic model template.
#[derive(Clone, Copy, Debug)]
pub struct GenericModelMetadata {
    /// The stable identifier registered for the generic model definition.
    model_id: ModelId,
    /// The semantic role assigned to the model definition.
    role: ModelRole,
    /// The reflection descriptor for the generic definition.
    definition: &'static GenericDefinitionDescriptor,
    /// Symbolic field overlays declared by the generic model.
    fields: &'static [FieldMetadata],
}

impl GenericModelMetadata {
    /// Creates generic model metadata.
    #[must_use]
    pub(crate) const fn new(
        model_id: ModelId,
        role: ModelRole,
        definition: &'static GenericDefinitionDescriptor,
        fields: &'static [FieldMetadata],
    ) -> Self {
        Self {
            model_id,
            role,
            definition,
            fields,
        }
    }

    /// Returns the stable identifier registered for this generic model.
    #[must_use = "the stable model ID identifies the registered generic definition"]
    pub const fn model_id(&self) -> ModelId {
        self.model_id
    }
    /// Returns the semantic role assigned to this generic model.
    #[must_use]
    pub const fn role(&self) -> ModelRole {
        self.role
    }
    /// Returns the shared reflection generic definition.
    #[must_use]
    pub const fn definition(&self) -> &'static GenericDefinitionDescriptor {
        self.definition
    }
    /// Returns symbolic field overlays for the template.
    #[must_use]
    pub const fn fields(&self) -> &'static [FieldMetadata] {
        self.fields
    }
}
