// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Selects the value shape supplied to a validator occurrence.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::{OnNone, TargetMode, ValidatorMetadata};
///
/// let validator = ValidatorMetadata::new_bound(
///     "example.container",
///     &[],
///     &[],
///     &[],
///     TargetMode::Container,
///     OnNone::Skip,
/// );
/// assert_eq!(validator.target(), TargetMode::Container);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TargetMode {
    /// Expand supported optional and transparent smart-pointer wrappers.
    Value,
    /// Preserve the declared container type.
    Container,
}

/// Compatibility name for [`TargetMode`].
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::{OnNone, ValidationTarget, ValidatorMetadata};
///
/// let target: ValidationTarget = ValidationTarget::Container;
/// let validator = ValidatorMetadata::new_bound("example.container", &[], &[], &[], target, OnNone::Skip);
/// assert_eq!(validator.target(), ValidationTarget::Container);
/// ```
pub type ValidationTarget = TargetMode;
