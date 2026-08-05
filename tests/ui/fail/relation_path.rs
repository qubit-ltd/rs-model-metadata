// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#[derive(qubit_model_derive::ModelMetadata)]
struct Organization {
    id: i64,
}

#[derive(qubit_model_derive::ModelMetadata)]
struct Invalid {
    nested: String,
    #[model(reference(target = Organization, target_field = id, same_as = "nested.missing"))]
    organization_id: i64,
}

fn main() {}
