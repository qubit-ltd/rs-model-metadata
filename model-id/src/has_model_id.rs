// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Stable identity associated with a concrete Rust model type.

use crate::ModelId;

/// Provides the stable model identity declared for one concrete Rust type.
///
/// Implement this trait when a type has one stable model ID. Generic model
/// definitions do not implement it because a generic declaration does not
/// identify one concrete model type.
///
/// # Examples
///
/// ```
/// use qubit_model_id::{HasModelId, ModelId};
///
/// struct Account;
///
/// impl HasModelId for Account {
///     const MODEL_ID: ModelId = ModelId::new("example.Account");
/// }
///
/// assert_eq!(Account::MODEL_ID.as_str(), "example.Account");
/// ```
pub trait HasModelId {
    /// The stable identity associated with this type.
    const MODEL_ID: ModelId;
}
