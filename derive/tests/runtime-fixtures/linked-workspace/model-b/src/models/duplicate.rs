// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Declares the feature-gated duplicate model used by the fixture.

use qubit_model_derive::Model;

/// A model that intentionally duplicates a registration ID from another crate.
///
/// # Examples
///
/// ```
/// use core::any::TypeId;
/// use model_b::Duplicate;
/// use qubit_model_metadata::TypeMetadata;
///
/// let _duplicate = Duplicate;
/// assert_eq!(TypeMetadata::of::<Duplicate>().type_id(), TypeId::of::<Duplicate>());
/// ```
#[Model(id = "test.linked.Duplicate")]
pub struct Duplicate;
