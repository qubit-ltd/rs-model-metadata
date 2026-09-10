// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Concrete and generic registration alternatives retained by ModelEntry.

#[cfg(feature = "generic")]
use crate::generic::GenericModelMetadata;
use crate::metadata::TypeMetadata;

/// Preserves whether a registry entry names a concrete type or a definition.
#[derive(Clone, Copy, Debug)]
pub(in crate::registry) enum ModelEntryTarget {
    /// A concrete model whose Rust type identity is available.
    Concrete(
        /// Immutable metadata for the concrete registered type.
        &'static TypeMetadata,
    ),
    /// A generic definition whose instances have separate concrete identities.
    #[cfg(feature = "generic")]
    Generic(
        /// Immutable metadata for the registered generic template.
        &'static GenericModelMetadata,
    ),
}
