// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_model_derive::Projection;
use model_runtime::__private::qubit_id::Id;

#[Projection(id = "test.Projection", no_eq)]
struct InvalidProjection {
    #[identifier]
    id: Id,
}

fn main() {}
