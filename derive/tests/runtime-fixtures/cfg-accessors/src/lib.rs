// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// Cargo's fixture entry point intentionally uses lib.rs.

//! Consumer feature matrix for property and reflection presence.

mod configured;
pub use configured::Configured;

mod generic_configured;

mod missing_impl;
