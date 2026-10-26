// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Normalized selector declaration IR.

use super::super::CodecIr;
use super::super::ConstraintIr;
use super::super::RedactIr;
use super::super::ValidatorIr;
use super::SelectorPositionIr;

/// Normalized selector declaration and metadata applied to selected values.
#[derive(Clone)]
pub(crate) struct SelectorIr {
    /// Selected sequence or map position.
    pub(crate) position: SelectorPositionIr,
    /// Constraints declared for the selected values.
    pub(crate) constraints: Vec<ConstraintIr>,
    /// Validators declared for the selected values.
    pub(crate) validators: Vec<ValidatorIr>,
    /// Codec declared for the selected values.
    pub(crate) codec: Option<CodecIr>,
    /// Redaction declared for the selected values.
    pub(crate) redact: Option<RedactIr>,
}
