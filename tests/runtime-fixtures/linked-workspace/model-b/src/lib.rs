// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow test-file-name
// The filename is part of a Cargo or trybuild fixture protocol.
//! Re-exports model declarations used by linked-workspace fixtures.

mod models;

pub use models::Target;
#[cfg(feature = "duplicate-fixture")]
pub use models::Duplicate;
