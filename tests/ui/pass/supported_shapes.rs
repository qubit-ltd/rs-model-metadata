// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![allow(dead_code)]

use std::collections::HashMap;


#[qubit_model_derive::Model(id = "test.derive.Named", no_clone, no_debug, no_display, no_partial_eq, no_hash, no_serialize, no_deserialize)]
struct Named {
    #[sequence(min_items = 1, max_items = 3, unique_items)]
    values: Option<Vec<String>>,
    #[map(min_entries = 1, max_entries = 2)]
    labels: HashMap<String, String>,
    #[sequence(unique_items)]
    fixed_unique_values: [String; 3],
}

#[qubit_model_derive::Model(id = "test.derive.Unit", no_clone, no_debug, no_display, no_partial_eq, no_hash, no_serialize, no_deserialize)]
struct Unit;

#[qubit_model_derive::Model(id = "test.derive.Newtype", no_clone, no_debug, no_display, no_partial_eq, no_hash, no_serialize, no_deserialize)]
struct Newtype(#[text(max_chars = 8)] String);

#[derive(Debug)]
struct OpaquePayload;

#[qubit_model_derive::Enum(id = "test.derive.Fieldless", no_clone, no_debug, no_display, no_partial_eq, no_hash, no_serialize, no_deserialize)]
enum Fieldless {
    First,
    Second,
}

#[qubit_model_derive::Enum(id = "test.derive.DataEnum", no_clone, no_debug, no_display, no_partial_eq, no_hash, no_serialize, no_deserialize)]
enum DataEnum {
    Unit,
    Tuple(#[text(max_chars = 8)] String),
    Struct {
        #[sequence(max_items = 3)]
        values: Vec<String>,
    },
}

#[qubit_model_derive::Enum(
    id = "test.derive.PayloadAttributes",
    no_clone,
    no_debug,
    no_display,
    no_partial_eq,
    no_hash,
    no_serialize,
    no_deserialize
)]
enum PayloadAttributes {
    Tuple(
        #[text(max_chars = 8)] String,
        #[sequence(max_items = 3)] Vec<String>,
        #[map(max_entries = 2)] HashMap<String, String>,
    ),
    Struct {
        #[time(precision = millisecond)]
        created_at: chrono::DateTime<chrono::Utc>,
        #[decimal(precision = 8, scale = 3)]
        ratio: bigdecimal::BigDecimal,
        #[element(text(allowed_chars = ascii))]
        codes: Vec<String>,
        #[codec = "encrypted"]
        #[generator(name = "token")]
        value: String,
        #[opaque]
        external: OpaquePayload,
        #[keep_serializing]
        #[serde(rename = "optional_value")]
        optional: Option<String>,
        #[redact(level = "secret")]
        secret: String,
    },
}

fn main() {}
