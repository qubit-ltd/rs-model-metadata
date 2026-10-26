// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// The filename is part of a Cargo fixture rather than an integration-test
// target.

//! Declares the model whose reference intentionally has no target.

use qubit_model_derive::Model;
use qubit_model_metadata::__private::qubit_id::Id;

/// A fixture model used to verify unresolved-reference diagnostics.
///
/// # Examples
///
/// ```
/// use model_a::MissingTarget;
/// use qubit_model_metadata::metadata::TypeMetadata;
///
/// let model = MissingTarget {
///     target_id: Default::default(),
/// };
/// assert!(TypeMetadata::of::<MissingTarget>().field("target_id").is_some());
/// let _reference = model.target_id;
/// ```
#[Model(id = "test.linked.MissingTarget")]
pub struct MissingTarget {
    /// Identifier expected to reference the intentionally absent model.
    #[reference(entity_id = "test.linked.Absent", property = id)]
    pub target_id: Id,
}
