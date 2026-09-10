// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Stable ID conversion and borrowed-key lookup for dynamic registry inputs.

use std::collections::HashMap;

use qubit_model_metadata::metadata::ModelId;
use qubit_model_metadata::metadata::ModelIdBuf;
use qubit_model_metadata::metadata::ModelIdError;

/// Dynamic and static inputs agree and survive destruction of the input string.
#[test]
fn test_owned_id_conversions_preserve_identity_and_registry_lookup() {
    let input = String::from("directory.Account");
    let from_borrow = ModelIdBuf::try_from(input.as_str()).expect("borrowed input");
    let from_owned = ModelIdBuf::try_from(input).expect("moved input");
    let from_static = ModelIdBuf::from(ModelId::new("directory.Account"));
    assert_eq!(from_borrow, from_owned);
    assert_eq!(from_owned, from_static);
    assert_eq!(from_owned.as_ref(), "directory.Account");
    assert_eq!(from_owned.to_string(), "directory.Account");

    let registry = HashMap::from([(from_owned, 7)]);
    assert_eq!(registry.get("directory.Account"), Some(&7));
    assert_eq!(registry.get("directory.account"), None);
    assert_eq!(registry.get("other.Account"), None);
}

/// Both dynamic conversion entry points retain the precise parse failure.
#[test]
fn test_owned_id_conversions_reject_invalid_dynamic_input() {
    for (input, expected) in [
        ("", ModelIdError::Empty),
        ("directory..Account", ModelIdError::EmptySegment),
        ("directory.1Account", ModelIdError::InvalidSegment),
        ("directory.账户", ModelIdError::InvalidSegment),
    ] {
        assert_eq!(ModelIdBuf::try_from(input), Err(expected), "borrowed: {input}");
        assert_eq!(ModelIdBuf::try_from(input.to_owned()), Err(expected), "owned: {input}");
    }
}
