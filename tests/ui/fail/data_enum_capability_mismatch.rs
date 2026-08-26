// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#[qubit_model_derive::Enum(id = "test.derive.DataEnumTextCapability")]
enum DataEnumTextCapability {
    Value(#[text(max_chars = 8)] i64),
}

fn main() {}
