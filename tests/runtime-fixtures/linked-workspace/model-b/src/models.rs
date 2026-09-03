// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow test-file-name
// The filename is part of a Cargo fixture rather than an integration-test target.

//! Defines target-side models for linked registration and resolution fixtures.

use qubit_model_derive::Entity;
use qubit_model_derive::Model;
use qubit_model_metadata::__private::qubit_id::Id;

#[Entity(id = "test.linked.Target")]
pub struct Target {
    #[identifier]
    pub id: Id,
}

#[Model(id = "test.linked.Duplicate")]
#[cfg(feature = "duplicate-fixture")]
pub struct Duplicate;
