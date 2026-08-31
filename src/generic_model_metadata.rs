// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Metadata for one registered generic model declaration.

use qubit_reflect::expression::GenericDefinitionDescriptor;

use crate::FieldMetadata;
use crate::ModelId;
use crate::ModelRole;

/// Immutable metadata for a generic model template.
#[derive(Clone, Copy, Debug)]
pub struct GenericModelMetadata {
    model_id: ModelId,
    role: ModelRole,
    definition: &'static GenericDefinitionDescriptor,
    fields: &'static [FieldMetadata],
}

impl GenericModelMetadata {
    /// Creates generic model metadata.
    #[must_use]
    pub const fn new(
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

    /// Returns the registered generic model ID.
    pub const fn model_id(&self) -> ModelId {
        self.model_id
    }
    /// Returns the template model role.
    pub const fn role(&self) -> ModelRole {
        self.role
    }
    /// Returns the shared reflection generic definition.
    pub const fn definition(&self) -> &'static GenericDefinitionDescriptor {
        self.definition
    }
    /// Returns symbolic field overlays for the template.
    pub const fn fields(&self) -> &'static [FieldMetadata] {
        self.fields
    }
}
