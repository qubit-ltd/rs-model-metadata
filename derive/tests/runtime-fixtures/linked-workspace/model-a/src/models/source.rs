// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// The filename is part of a Cargo fixture rather than an integration-test
// target.

//! Declares the source model used by linked registration and resolution.

use qubit_model_derive::Model;
use qubit_model_metadata::__private::qubit_id::Id;

/// A source model whose reference resolves to the linked workspace target.
///
/// # Examples
///
/// ```
/// use model_a::Source;
/// use qubit_model_metadata::metadata::TypeMetadata;
///
/// let source = Source {
///     target_id: Default::default(),
///     text: String::from("example"),
/// };
/// assert!(TypeMetadata::of::<Source>().field("target_id").is_some());
/// assert_eq!(source.text, "example");
/// ```
#[Model(id = "test.linked.Source")]
pub struct Source {
    /// Identifies the target model referenced by this source.
    #[reference(entity_id = "test.linked.Target", property = id)]
    pub target_id: Id,
    /// Text validated and encoded through linked registrations.
    #[validator(id = "test.linked.text")]
    #[codec(id = "test.linked.text")]
    pub text: String,
}
