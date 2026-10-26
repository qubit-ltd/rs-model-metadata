// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// The filename is part of a Cargo fixture rather than an integration-test
// target.

//! Declares the duplicate-registration fixture model.

use qubit_model_derive::Model;

/// A marker model used to exercise duplicate registration diagnostics.
///
/// # Examples
///
/// ```
/// use model_a::Duplicate;
/// use qubit_model_metadata::metadata::TypeMetadata;
///
/// let _model = Duplicate;
/// assert_eq!(
///     TypeMetadata::of::<Duplicate>().model_id().unwrap().as_str(),
///     "test.linked.Duplicate",
/// );
/// ```
#[Model(id = "test.linked.Duplicate")]
pub struct Duplicate;
