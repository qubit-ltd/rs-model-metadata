// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Helpers for safe validation binding diagnostics.

use qubit_validator::BindError;
use qubit_validator::BindErrorKind;
/// Creates a validator binding error for a path that cannot be read.
pub(crate) fn path_error(kind: BindErrorKind) -> BindError {
    BindError::new(kind)
}
// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================
