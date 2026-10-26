// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Private sealing trait for exact identifier types.

use qubit_id::Id;

/// Prevents downstream crates from extending exact model type roles.
pub trait IdentifierTypeSeal {}

impl IdentifierTypeSeal for Id {}
