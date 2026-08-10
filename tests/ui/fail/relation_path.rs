// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#[derive(qubit_model_derive::ModelMetadata)]
#[model(id = "test.derive.Organization")]
struct Organization {
    id: i64,
}

#[derive(qubit_model_derive::ModelMetadata)]
#[model(id = "test.derive.Invalid")]
struct Invalid {
    nested: String,
    #[model(reference(target = "test.derive.Organization", target_field = id, same_as = "missing.id"))]
    organization_id: i64,
}

fn main() {}
