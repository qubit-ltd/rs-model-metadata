// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_id::Id;
use qubit_reflect::error::TypeMismatch;
use qubit_reflect::value::DynamicRef;
use qubit_reflect::value::Local;

use crate::__private::IdentifierAccessor;
use crate::metadata::FieldMetadata;

/// Entity-specific metadata.
/// Obtain it through [`crate::metadata::TypeMetadata::as_entity`] for a
/// generated entity type.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::EntityMetadata;
/// use qubit_model_metadata::metadata::TypeMetadata;
///
/// # mod example {
/// use qubit_id::Id;
/// use qubit_model_derive::Entity;
///
/// #[Entity(id = "example.Customer")]
/// pub struct Customer {
///     #[identifier]
///     pub id: Id,
/// }
/// # }
///
/// let entity: &EntityMetadata = TypeMetadata::of::<example::Customer>().as_entity().expect("entity metadata");
/// assert_eq!(entity.identifier().name(), Some("id"));
/// ```
#[derive(Clone, Copy, Debug)]
pub struct EntityMetadata {
    /// The field that supplies the entity identifier.
    identifier: &'static FieldMetadata,
    /// The generated function that reads the typed identifier value.
    identifier_accessor: IdentifierAccessor,
}

impl EntityMetadata {
    /// Creates entity metadata.
    ///
    /// # Parameters
    ///
    /// - `identifier`: the reflected field that supplies the entity identity.
    /// - `identifier_accessor`: the generated function that reads the typed
    ///   identifier value from a reflected entity.
    ///
    /// # Returns
    ///
    /// Entity metadata retaining the process-lifetime identifier field.
    #[must_use]
    #[inline]
    pub(crate) const fn new(identifier: &'static FieldMetadata, identifier_accessor: IdentifierAccessor) -> Self {
        Self {
            identifier,
            identifier_accessor,
        }
    }

    /// Returns the entity identifier field.
    ///
    /// # Returns
    ///
    /// The process-lifetime field metadata used as this entity's identifier.
    #[must_use]
    #[inline]
    pub const fn identifier(&self) -> &'static FieldMetadata {
        self.identifier
    }

    /// Reads this entity's identifier through its generated typed accessor.
    ///
    /// # Returns
    ///
    /// The identifier borrowed from `value` when it has the generated entity
    /// type.
    ///
    /// # Errors
    ///
    /// Returns a type mismatch when `value` is not an instance of this entity.
    #[inline]
    pub(crate) fn read_identifier<'a>(&self, value: DynamicRef<'a, Local>) -> Result<&'a Id, TypeMismatch> {
        (self.identifier_accessor)(value)
    }
}
