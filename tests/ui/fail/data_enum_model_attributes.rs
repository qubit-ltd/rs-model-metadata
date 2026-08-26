// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#[qubit_model_derive::Enum(id = "test.derive.PrimaryKeyData", primary_key(fields(value)), no_clone, no_debug, no_display, no_partial_eq, no_hash, no_serialize, no_deserialize)]
enum PrimaryKeyData {
    Value(i64),
}

#[qubit_model_derive::Enum(id = "test.derive.IndexData", index(fields(value)), no_clone, no_debug, no_display, no_partial_eq, no_hash, no_serialize, no_deserialize)]
enum IndexData {
    Value(i64),
}

#[qubit_model_derive::Enum(id = "test.derive.KeyData", key(fields(value)), no_clone, no_debug, no_display, no_partial_eq, no_hash, no_serialize, no_deserialize)]
enum KeyData {
    Value(i64),
}

struct Owner;

#[qubit_model_derive::Enum(id = "test.derive.OwnershipData", ownership(owner = Owner), no_clone, no_debug, no_display, no_partial_eq, no_hash, no_serialize, no_deserialize)]
enum OwnershipData {
    Value(i64),
}

fn main() {}
