// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_model_derive::Model;

#[Model(id = "test.linked.Source")]
pub struct Source {
    #[reference(entity = "test.linked.Target", property = id)]
    pub target_id: i64,
}

#[cfg(feature = "duplicate-fixture")]
#[Model(id = "test.linked.Duplicate")]
pub struct Duplicate;

#[Model(id = "test.linked.MissingTarget")]
pub struct MissingTarget {
    #[reference(entity = "test.linked.Absent", property = id)]
    pub target_id: i64,
}
