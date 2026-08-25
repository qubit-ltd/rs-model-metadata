// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Distributed declarations for statically linked model metadata.

use std::fmt;

use linkme::distributed_slice;

use crate::model_id::ModelId;
use crate::type_metadata::TypeMetadata;

mod has_model_registration;
mod source_location;

pub use self::has_model_registration::HasModelRegistration;
pub use self::has_model_registration::registration_of;
pub use self::source_location::SourceLocation;

/// A statically linked model metadata declaration.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::ModelId;
/// use qubit_model_metadata::ModelRegistration;
/// use qubit_model_metadata::SourceLocation;
/// use qubit_model_metadata::StructMetadata;
/// use qubit_model_metadata::TypeIdentity;
/// use qubit_model_metadata::TypeKind;
/// use qubit_model_metadata::TypeMetadata;
///
/// static METADATA: TypeMetadata = TypeMetadata::new(
///     ModelId::new("example.Account"),
///     TypeIdentity::of::<u8>(),
///     TypeKind::Struct(StructMetadata::new(&[])),
///     &[],
/// );
/// let registration = ModelRegistration::new(
///     ModelId::new("example.Account"),
///     &METADATA,
///     "example::Account",
///     "example",
///     SourceLocation::new("account.rs", 1, 1),
/// );
/// assert_eq!(registration.id().as_str(), "example.Account");
/// ```
#[derive(Debug)]
pub struct ModelRegistration {
    /// The stable model identifier used as the registry key.
    id: ModelId,
    /// The metadata exposed by the registered Rust type.
    metadata: &'static TypeMetadata,
    /// The fully qualified Rust type name supplied by the derive macro.
    rust_type_name: &'static str,
    /// The Rust module path supplied by the derive macro.
    rust_module_path: &'static str,
    /// The registration declaration's source location.
    source: SourceLocation,
}

impl ModelRegistration {
    /// Creates a static model registration.
    ///
    /// # Parameters
    ///
    /// * `id` - The stable identifier used for registry lookups.
    /// * `metadata` - The immutable metadata registered for this model.
    /// * `rust_type_name` - The fully qualified Rust type name.
    /// * `rust_module_path` - The Rust module path containing the declaration.
    /// * `source` - The declaration's source location.
    ///
    /// # Returns
    ///
    /// A registration that can be placed in [`MODEL_REGISTRATIONS`].
    #[must_use]
    #[inline(always)]
    pub const fn new(
        id: ModelId,
        metadata: &'static TypeMetadata,
        rust_type_name: &'static str,
        rust_module_path: &'static str,
        source: SourceLocation,
    ) -> Self {
        Self {
            id,
            metadata,
            rust_type_name,
            rust_module_path,
            source,
        }
    }

    /// Returns the stable identifier used to register this model.
    ///
    /// # Returns
    ///
    /// The stable model identifier stored in this registration.
    #[inline(always)]
    pub const fn id(&self) -> ModelId {
        self.id
    }

    /// Returns the immutable metadata registered for this model.
    ///
    /// # Returns
    ///
    /// The static metadata linked to this registration.
    #[inline(always)]
    pub const fn metadata(&self) -> &'static TypeMetadata {
        self.metadata
    }

    /// Returns the fully qualified Rust type name supplied by the registration.
    #[must_use]
    #[inline(always)]
    pub const fn rust_type_name(&self) -> &'static str {
        self.rust_type_name
    }

    /// Returns the Rust module path supplied by the registration.
    #[must_use]
    #[inline(always)]
    pub const fn rust_module_path(&self) -> &'static str {
        self.rust_module_path
    }

    /// Returns the source location where this registration was declared.
    #[must_use]
    #[inline(always)]
    pub const fn source(&self) -> SourceLocation {
        self.source
    }
}

impl fmt::Display for ModelRegistration {
    /// Formats the Rust type, module path, and declaration source location.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} in {} at {}",
            self.rust_type_name, self.rust_module_path, self.source,
        )
    }
}

/// Collects registrations emitted by linked model crates.
#[distributed_slice]
pub static MODEL_REGISTRATIONS: [ModelRegistration];
