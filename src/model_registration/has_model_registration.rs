// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Explicit access to a model's generated registration.

use crate::model_registration::ModelRegistration;
use crate::type_metadata::HasTypeMetadata;

/// Exposes the static registration generated for one model type.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::HasModelRegistration;
/// use qubit_model_metadata::HasTypeMetadata;
/// use qubit_model_metadata::HasTypeShape;
/// use qubit_model_metadata::ModelId;
/// use qubit_model_metadata::ModelRegistration;
/// use qubit_model_metadata::SourceLocation;
/// use qubit_model_metadata::StructMetadata;
/// use qubit_model_metadata::TypeCapabilities;
/// use qubit_model_metadata::TypeIdentity;
/// use qubit_model_metadata::TypeKind;
/// use qubit_model_metadata::TypeMetadata;
/// use qubit_model_metadata::TypeShape;
/// use qubit_model_metadata::registration_of;
///
/// struct Account;
///
/// static METADATA: TypeMetadata = TypeMetadata::new(
///     ModelId::new("example.Account"),
///     TypeIdentity::of::<Account>(),
///     TypeKind::Struct(StructMetadata::new(&[])),
///     &[],
/// );
///
/// impl HasTypeShape for Account {
///     const TYPE_SHAPE: TypeShape = TypeShape::Opaque;
///     const CAPABILITIES: TypeCapabilities = TypeCapabilities::NONE;
/// }
///
/// impl HasTypeMetadata for Account {
///     fn type_metadata() -> &'static TypeMetadata {
///         &METADATA
///     }
/// }
///
/// static REGISTRATION: ModelRegistration = ModelRegistration::new(
///     ModelId::new("example.Account"),
///     &METADATA,
///     "example::Account",
///     "example",
///     SourceLocation::new("account.rs", 1, 1),
/// );
///
/// impl HasModelRegistration for Account {
///     fn model_registration() -> &'static ModelRegistration {
///         &REGISTRATION
///     }
/// }
///
/// assert_eq!(registration_of::<Account>().id().as_str(), "example.Account");
/// ```
pub trait HasModelRegistration: HasTypeMetadata {
    /// Returns the immutable registration generated for this type.
    ///
    /// # Returns
    ///
    /// The static registration emitted for the implementing type.
    #[must_use]
    fn model_registration() -> &'static ModelRegistration;
}

/// Returns the generated registration for `T`.
///
/// # Type Parameters
///
/// * `T` - The model type whose registration is requested.
///
/// # Returns
///
/// The static registration emitted for `T`.
#[must_use]
#[inline(always)]
pub fn registration_of<T: HasModelRegistration>() -> &'static ModelRegistration {
    T::model_registration()
}
