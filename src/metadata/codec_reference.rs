// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use crate::metadata::RustTypeReference;

/// A codec declared by Rust type or stable textual ID.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::CodecReference;
///
/// let reference = CodecReference::DeclaredId("example.codec");
/// assert_eq!(reference.declared_id(), Some("example.codec"));
/// ```
#[derive(Clone, Copy, Debug)]
pub enum CodecReference {
    /// A codec identified by its Rust type.
    RustType(RustTypeReference),
    /// A codec identified by its stable registry ID.
    DeclaredId(&'static str),
}

impl CodecReference {
    /// Returns the referenced Rust type, if the declaration uses one.
    ///
    /// # Returns
    ///
    /// The Rust codec type reference, or `None` for a textual ID declaration.
    #[inline]
    pub const fn rust_type(self) -> Option<RustTypeReference> {
        match self {
            Self::RustType(reference) => Some(reference),
            Self::DeclaredId(_) => None,
        }
    }

    /// Returns the declared registry ID, if the declaration uses one.
    ///
    /// # Returns
    ///
    /// The stable codec ID, or `None` when a Rust type identifies the codec.
    #[inline]
    pub const fn declared_id(self) -> Option<&'static str> {
        match self {
            Self::RustType(_) => None,
            Self::DeclaredId(id) => Some(id),
        }
    }
}
