// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use core::fmt;

use super::declared_entity_target_kind::DeclaredEntityTargetKind;
use crate::metadata::ModelId;
use crate::type_metadata::TypeMetadata;

/// A declaration-time entity target that does not consult the registry.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::{DeclaredEntityTarget, DeclaredEntityTargetKind, ModelId};
///
/// let target = DeclaredEntityTarget::ModelId(ModelId::new("example.Customer"));
/// assert_eq!(target.kind(), DeclaredEntityTargetKind::ModelId);
/// assert_eq!(target.model_id().unwrap().as_str(), "example.Customer");
/// ```
#[derive(Clone, Copy)]
pub enum DeclaredEntityTarget {
    /// A Rust type metadata provider.
    RustType(fn() -> &'static TypeMetadata),
    /// A stable model ID string.
    ModelId(ModelId),
}

impl DeclaredEntityTarget {
    /// Returns the target representation kind.
    ///
    /// # Returns
    ///
    /// Whether this declaration names a Rust type provider or stable model ID.
    #[must_use]
    #[inline]
    pub const fn kind(&self) -> DeclaredEntityTargetKind {
        match self {
            Self::RustType(_) => DeclaredEntityTargetKind::RustType,
            Self::ModelId(_) => DeclaredEntityTargetKind::ModelId,
        }
    }

    /// Calls the Rust-type provider and returns its metadata. Returns None for
    /// a stable-ID target; this method never resolves that ID in a registry.
    /// The provider can initialize caches or perform its own side effects.
    ///
    /// # Returns
    ///
    /// Provider metadata for Rust-type targets, or `None` for model-ID targets.
    ///
    /// # Panics
    ///
    /// Propagates any panic raised by the supplied provider.
    pub fn metadata(&self) -> Option<&'static TypeMetadata> {
        match self {
            Self::RustType(provider) => Some(provider()),
            Self::ModelId(_) => None,
        }
    }

    /// Returns the declared stable model ID, if any.
    ///
    /// # Returns
    ///
    /// The stable model ID for that target kind, or `None` for Rust types.
    #[inline]
    pub const fn model_id(&self) -> Option<ModelId> {
        match self {
            Self::RustType(_) => None,
            Self::ModelId(id) => Some(*id),
        }
    }
}

impl fmt::Debug for DeclaredEntityTarget {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("DeclaredEntityTarget")
            .field(&self.kind())
            .finish()
    }
}
