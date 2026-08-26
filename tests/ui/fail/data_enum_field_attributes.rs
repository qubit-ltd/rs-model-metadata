// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#[qubit_model_derive::Enum(id = "test.derive.UniqueData", no_clone, no_debug, no_display, no_partial_eq, no_hash, no_serialize, no_deserialize)]
enum UniqueData {
    Value(#[unique] String),
}

#[qubit_model_derive::Enum(id = "test.derive.IndexedData", no_clone, no_debug, no_display, no_partial_eq, no_hash, no_serialize, no_deserialize)]
enum IndexedData {
    Value(#[indexed] i64),
}

#[qubit_model_derive::Enum(id = "test.derive.ReferenceData", no_clone, no_debug, no_display, no_partial_eq, no_hash, no_serialize, no_deserialize)]
enum ReferenceData {
    Value(#[reference(entity = "test.derive.Target")] i64),
}

#[qubit_model_derive::Enum(id = "test.derive.LookupData", no_clone, no_debug, no_display, no_partial_eq, no_hash, no_serialize, no_deserialize)]
enum LookupData {
    Value(#[lookup_relation(target = Target, target_field = id)] i64),
}

fn main() {}
