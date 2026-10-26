// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Selects the behavior for an absent expanded optional value.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::{OnNone, TargetMode, ValidatorMetadata};
///
/// let validator = ValidatorMetadata::new_bound("example.required", &[], &[], &[], TargetMode::Value, OnNone::Reject);
/// assert_eq!(validator.on_none(), OnNone::Reject);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OnNone {
    /// Skip the validator occurrence.
    Skip,
    /// Reject an absent value before invoking the validator.
    Reject,
}
