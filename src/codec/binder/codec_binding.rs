// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::ValueStringCodecDescriptor;
use qubit_codec::ValueStringCodecRegistration;

use super::codec_occurrence_id::CodecOccurrenceId;
use crate::metadata::CodecMetadata;

/// One codec declaration bound to an executable descriptor.
///
/// A successful call to [`bind_codecs`](crate::codec::bind_codecs) provides
/// this value through the returned [`CodecBindings`](super::CodecBindings).
///
/// # Type Parameters
///
/// * `'a` - Lifetime of the codec registry entry referenced by the binding.
///
/// # Examples
///
/// ```
/// #![cfg(feature = "codec")]
/// use qubit_model_metadata::codec::CodecBinding;
///
/// // `binding` is obtained from `CodecBindings::bindings()` after
/// // `bind_codecs` succeeds.
/// fn inspect_binding(binding: &CodecBinding<'_>) {
///     let _ = binding.occurrence();
///     let _ = binding.declaration();
///     let _ = binding.descriptor();
///     let _ = binding.registration();
/// }
/// # let _ = inspect_binding;
/// ```
#[derive(Debug)]
pub struct CodecBinding<'a> {
    /// Stable identity used to locate this binding in the resolved graph.
    pub(super) occurrence: CodecOccurrenceId,
    /// Model declaration that selected the codec.
    pub(super) declaration: &'static CodecMetadata,
    /// Executable codec descriptor used for dispatch.
    pub(super) descriptor: &'static ValueStringCodecDescriptor,
    /// Registry entry selected after uniqueness and type checks.
    pub(super) registration: &'a ValueStringCodecRegistration,
}

impl<'a> CodecBinding<'a> {
    /// Returns the occurrence identity.
    ///
    /// # Returns
    ///
    /// The stable identity of the model declaration represented by this
    /// binding.
    #[must_use]
    #[inline]
    pub const fn occurrence(&self) -> &CodecOccurrenceId {
        &self.occurrence
    }

    /// Returns the declaration metadata.
    ///
    /// # Returns
    ///
    /// Static metadata for the codec declaration that selected this
    /// registration.
    #[must_use]
    #[inline]
    pub const fn declaration(&self) -> &'static CodecMetadata {
        self.declaration
    }

    /// Returns the executable descriptor.
    ///
    /// # Returns
    ///
    /// The descriptor used to dispatch values to the selected codec.
    #[must_use]
    #[inline]
    pub const fn descriptor(&self) -> &'static ValueStringCodecDescriptor {
        self.descriptor
    }

    /// Returns the selected registration.
    ///
    /// # Returns
    ///
    /// The registry entry selected after uniqueness and value-type checks.
    #[must_use]
    #[inline]
    pub const fn registration(&self) -> &'a ValueStringCodecRegistration {
        self.registration
    }
}
