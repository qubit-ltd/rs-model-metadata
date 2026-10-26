// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Resolved property data for one step of a compiled access path.

use crate::metadata::OptionalOps;
use crate::metadata::PropertyMetadata;
use crate::metadata::TypeMetadata;

/// Resolved metadata needed to traverse or operate on one path segment.
///
/// The owner identifies the declaring model for runtime receiver checks.
/// `property` describes the named access, while `optional_ops` carries the
/// generated projection needed when that property is a field-backed
/// `Option<T>`. Copying the step keeps path execution independent of further
/// metadata lookup after compilation.
#[derive(Clone, Copy)]
pub(super) struct PropertyStep {
    /// Declaring metadata used to validate the runtime receiver.
    pub(super) owner: &'static TypeMetadata,
    /// Resolved property to read or write.
    pub(super) property: PropertyMetadata,
    /// Generated projection for a field-backed `Option<T>`.
    pub(super) optional_ops: Option<OptionalOps>,
}
