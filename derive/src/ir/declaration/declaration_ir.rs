// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Complete declaration IR.

use super::DeclarationOptions;
use super::FieldIr;
use super::VariantIr;
use crate::ir::MacroKind;

/// Complete normalized declaration consumed by the expansion stage.
pub(crate) struct DeclarationIr {
    /// Macro role being compiled.
    pub(crate) kind: MacroKind,
    /// Normalized declaration-level options.
    pub(crate) options: DeclarationOptions,
    /// Normalized struct fields.
    pub(crate) fields: Vec<FieldIr>,
    /// Normalized enum variants.
    pub(crate) variants: Vec<VariantIr>,
}
