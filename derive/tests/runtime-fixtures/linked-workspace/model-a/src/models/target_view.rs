// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// The filename is part of a Cargo fixture rather than an integration-test
// target.

//! Declares the projection of the linked workspace target.

use qubit_model_derive::Projection;
use qubit_model_metadata::__private::qubit_id::Id;

/// A view of the linked target that exposes its identifier.
///
/// # Examples
///
/// ```
/// use model_a::TargetView;
/// use qubit_model_metadata::metadata::TypeMetadata;
///
/// let view = TargetView {
///     id: Default::default(),
/// };
/// assert!(TypeMetadata::of::<TargetView>().field("id").is_some());
/// let _identifier = view.id;
/// ```
#[Projection(id = "test.linked.TargetView", source_id = "test.linked.Target")]
pub struct TargetView {
    /// Identifier projected from the source target.
    #[identifier]
    pub id: Id,
}
