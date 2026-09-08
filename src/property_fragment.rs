// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! One unmerged field, getter, or setter property declaration.

use qubit_reflect::descriptor::TypeRef;

use crate::PropertyFragmentSource;

/// Preserves one local source fact before property compatibility is checked.
#[derive(Clone, Copy, Debug)]
pub struct PropertyFragment {
    /// Canonical public property name.
    name: &'static str,
    /// Exact type declared by this fragment.
    type_ref: &'static TypeRef,
    /// Field/getter/setter declaration source.
    source: PropertyFragmentSource,
}

impl PropertyFragment {
    /// Creates one generated property source fact.
    #[must_use]
    pub(crate) const fn new(name: &'static str, type_ref: &'static TypeRef, source: PropertyFragmentSource) -> Self {
        Self { name, type_ref, source }
    }

    /// Returns the canonical public property name.
    #[must_use]
    #[inline(always)]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Returns the exact type declared by this source fragment.
    #[must_use]
    #[inline(always)]
    pub const fn type_ref(&self) -> &'static TypeRef {
        self.type_ref
    }

    /// Returns the field, getter, or setter that declared this fragment.
    #[must_use]
    #[inline(always)]
    pub const fn source(&self) -> PropertyFragmentSource {
        self.source
    }
}
