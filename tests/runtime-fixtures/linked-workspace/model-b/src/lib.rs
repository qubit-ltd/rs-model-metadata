// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_model_derive::Entity;
use qubit_model_metadata::__private::qubit_id::Id;

#[Entity(id = "test.linked.Target")]
pub struct Target {
    #[identifier]
    pub id: Id,
}

#[qubit_model_derive::Model(id = "test.linked.Duplicate")]
#[cfg(feature = "duplicate-fixture")]
pub struct Duplicate;
