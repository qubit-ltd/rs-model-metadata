// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_model_derive::Model;

#[Model(
    id = "test.attribute.DisabledDefaults",
    no_clone,
    no_debug,
    no_display,
    no_partial_eq,
    no_hash,
    no_serialize,
    no_deserialize
)]
struct DisabledDefaults(f64);

#[test]
fn test_model_options_disable_default_capabilities() {
    let DisabledDefaults(value) = DisabledDefaults(1.0);
    assert_eq!(value, 1.0);
}
