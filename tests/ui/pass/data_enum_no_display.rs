// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![allow(dead_code)]

struct NotDebug;

#[qubit_model_derive::Enum(
    id = "test.derive.NoDisplayData",
    no_clone,
    no_debug,
    no_display,
    no_eq,
    no_partial_eq,
    no_partial_ord,
    no_ord,
    no_hash,
    no_serialize,
    no_deserialize
)]
enum NoDisplayData {
    Value(#[opaque] NotDebug),
}

fn main() {}
