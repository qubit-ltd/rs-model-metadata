// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use crate::metadata::CodecMetadata;
use crate::metadata::FieldMetadata;

/// Value-specific metadata.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::RoleMetadata;
/// use qubit_model_metadata::metadata::TypeMetadata;
/// use qubit_model_metadata::metadata::ValueMetadata;
///
/// # mod example {
/// use qubit_model_derive::Value;
///
/// #[Value(transparent)]
/// pub struct UserId(pub u64);
/// # }
///
/// let RoleMetadata::Value(value) = TypeMetadata::of::<example::UserId>().role_metadata() else {
///     panic!("derived value metadata");
/// };
/// let value: &ValueMetadata = value;
/// assert!(value.is_transparent());
/// ```
#[derive(Clone, Copy, Debug)]
pub struct ValueMetadata {
    /// The sole wrapped field when the value is transparent.
    transparent_field: Option<&'static FieldMetadata>,
    /// The codec that represents the complete value, when configured.
    canonical_codec: Option<&'static CodecMetadata>,
}

impl ValueMetadata {
    /// Creates value metadata.
    ///
    /// # Parameters
    ///
    /// - `transparent_field`: the sole wrapped field when the value is
    ///   transparent.
    /// - `canonical_codec`: the codec that represents the complete value, if
    ///   configured.
    ///
    /// # Returns
    ///
    /// Value metadata retaining the configured transparent-field and codec
    /// semantics.
    #[must_use]
    #[inline]
    pub(crate) const fn new(
        transparent_field: Option<&'static FieldMetadata>,
        canonical_codec: Option<&'static CodecMetadata>,
    ) -> Self {
        Self {
            transparent_field,
            canonical_codec,
        }
    }

    /// Returns whether this value transparently wraps one field.
    ///
    /// # Returns
    ///
    /// `true` when a transparent field is configured.
    #[must_use]
    #[inline]
    pub const fn is_transparent(&self) -> bool {
        self.transparent_field.is_some()
    }

    /// Returns the transparent field, if configured.
    ///
    /// # Returns
    ///
    /// The sole wrapped field for a transparent value, or `None` otherwise.
    #[inline]
    pub const fn transparent_field(&self) -> Option<&'static FieldMetadata> {
        self.transparent_field
    }

    /// Returns the canonical value codec, if configured.
    ///
    /// # Returns
    ///
    /// The codec representing the whole value, or `None` when no canonical
    /// codec was configured.
    #[inline]
    pub const fn canonical_codec(&self) -> Option<&'static CodecMetadata> {
        self.canonical_codec
    }
}
