// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow test-file-name
// The filename is part of a Cargo fixture rather than an integration-test target.

//! Defines source-side models for linked registration and resolution fixtures.

use qubit_model_derive::Model;
use qubit_model_metadata::__private::qubit_id::Id;

#[Model(id = "test.linked.Source")]
pub struct Source {
    #[reference(entity_id = "test.linked.Target", property = id)]
    pub target_id: Id,
}

#[cfg(feature = "duplicate-fixture")]
#[Model(id = "test.linked.Duplicate")]
pub struct Duplicate;

#[cfg(feature = "missing-fixture")]
#[Model(id = "test.linked.MissingTarget")]
pub struct MissingTarget {
    #[reference(entity_id = "test.linked.Absent", property = id)]
    pub target_id: Id,
}
