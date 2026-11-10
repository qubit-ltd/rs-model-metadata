// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Counter categories charged during validation-plan construction.

/// A finite work, occurrence, binding, or diagnostic counter.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Counter {
    /// Model, field, edge, declaration, and binding traversal work.
    Work,
    /// A declaration occurrence discovered before binding.
    Occurrence,
    /// One rule emitted by a declaration binder.
    Binding,
    /// One ordinary error retained for the build result.
    Diagnostic,
}
