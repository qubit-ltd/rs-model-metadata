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
use crate::metadata::DeclaredEntityTarget;
use crate::metadata::FieldMetadata;

/// Projection-specific metadata.
/// Obtain it through [`crate::metadata::TypeMetadata::as_projection`] for a
/// generated projection type.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::ProjectionMetadata;
/// use qubit_model_metadata::metadata::TypeMetadata;
///
/// # mod example {
/// use qubit_id::Id;
/// use qubit_model_derive::Entity;
/// use qubit_model_derive::Projection;
///
/// #[Entity(id = "example.Source")]
/// pub struct Source {
///     #[identifier]
///     pub id: Id,
/// }
///
/// #[Projection(id = "example.View", source = Source)]
/// pub struct View {
///     #[identifier]
///     pub id: Id,
/// }
/// # }
///
/// let projection: &ProjectionMetadata = TypeMetadata::of::<example::View>().as_projection().expect("projection metadata");
/// assert!(projection.is_fixed());
/// ```
#[derive(Clone, Copy, Debug)]
pub struct ProjectionMetadata {
    /// The field that supplies the projection identifier.
    identifier: &'static FieldMetadata,
    /// The optional entity target from which projection fields originate.
    source: Option<&'static DeclaredEntityTarget>,
    /// The generated function that reads the typed identifier value.
    identifier_accessor: IdentifierAccessor,
}

impl ProjectionMetadata {
    /// Creates projection metadata.
    ///
    /// # Parameters
    ///
    /// - `identifier`: the reflected field that supplies the projection
    ///   identity.
    /// - `source`: the optional declared entity source for the projection.
    /// - `identifier_accessor`: the generated function that reads the typed
    ///   identifier value from a reflected projection.
    ///
    /// # Returns
    ///
    /// Projection metadata retaining its identifier and source declaration.
    #[must_use]
    #[inline]
    pub(crate) const fn new(
        identifier: &'static FieldMetadata,
        source: Option<&'static DeclaredEntityTarget>,
        identifier_accessor: IdentifierAccessor,
    ) -> Self {
        Self {
            identifier,
            source,
            identifier_accessor,
        }
    }

    /// Returns the projection identifier field.
    ///
    /// # Returns
    ///
    /// The process-lifetime field metadata used as this projection's
    /// identifier.
    #[must_use]
    #[inline]
    pub const fn identifier(&self) -> &'static FieldMetadata {
        self.identifier
    }

    /// Returns the optional declared source without consulting a registry.
    ///
    /// # Returns
    ///
    /// The declared entity target when one was configured, or `None` for an
    /// open projection.
    #[inline]
    pub const fn source(&self) -> Option<&'static DeclaredEntityTarget> {
        self.source
    }

    /// Returns whether undeclared source fields are accepted.
    ///
    /// # Returns
    ///
    /// `true` when this projection has no declared source.
    #[must_use]
    #[inline]
    pub const fn is_open(&self) -> bool {
        self.source.is_none()
    }

    /// Returns whether the projection field set is fixed.
    ///
    /// # Returns
    ///
    /// `true` when this projection declares a source entity.
    #[must_use]
    #[inline]
    pub const fn is_fixed(&self) -> bool {
        self.source.is_some()
    }

    /// Reads this projection's identifier through its generated typed accessor.
    ///
    /// # Parameters
    ///
    /// - `value`: a reflected borrow of the projection value to inspect.
    ///
    /// # Returns
    ///
    /// The identifier borrowed from `value` when it has the generated
    /// projection type.
    ///
    /// # Errors
    ///
    /// Returns a type mismatch when `value` is not an instance of this
    /// projection.
    #[inline]
    pub(crate) fn read_identifier<'a>(&self, value: DynamicRef<'a, Local>) -> Result<&'a Id, TypeMismatch> {
        (self.identifier_accessor)(value)
    }
}
