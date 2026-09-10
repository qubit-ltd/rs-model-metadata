// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! User-facing diagnostics distinguish missing IDs and malformed segments.

use std::error::Error;

use qubit_model_metadata::metadata::ModelIdBuf;
use qubit_model_metadata::metadata::ModelIdError;

/// Actual input failures retain a useful standalone diagnostic and no false
/// cause.
#[test]
fn test_model_id_error_diagnostics() {
    for (input, expected, message) in [
        ("", ModelIdError::Empty, "model ID cannot be empty"),
        (
            "directory.",
            ModelIdError::EmptySegment,
            "model ID cannot contain empty segments",
        ),
        (
            "directory.Account-2",
            ModelIdError::InvalidSegment,
            "model ID has an invalid segment",
        ),
    ] {
        let error = ModelIdBuf::parse(input).expect_err("invalid user-provided ID");
        assert_eq!(error, expected);
        assert_eq!(error.to_string(), message);
        assert!(error.source().is_none());
    }
}
