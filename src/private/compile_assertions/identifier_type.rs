// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Exact identifier type accepted by generated entity and projection roles.

use qubit_id::Id;

use super::identifier_type_seal::IdentifierTypeSeal;

/// Marks the exact identifier type accepted by Entity and Projection.
///
/// Type aliases of [`qubit_id::Id`] satisfy this bound because aliases do
/// not create a distinct Rust type. Wrappers and containers do not.
#[doc(hidden)]
pub trait IdentifierType: IdentifierTypeSeal {}

impl IdentifierType for Id {}
