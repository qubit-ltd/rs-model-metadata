// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! The compiler's declaration vocabulary, organized by semantic concern.

mod codec_ir;
mod constraints;
mod declaration_ir;
mod fields;
mod options;
mod redact;
mod reference_target_ir;
mod references;
mod serde;
mod strategy_argument_ir;
mod unique_ir;
mod validation;

pub(crate) use codec_ir::CodecIr;
pub(crate) use constraints::ConstraintIr;
pub(crate) use constraints::DecimalConstraintIr;
pub(crate) use constraints::TextConstraintIr;
pub(crate) use declaration_ir::DeclarationIr;
pub(crate) use fields::FieldIr;
pub(crate) use fields::FieldOccurrence;
pub(crate) use fields::IdentifierAssignmentIr;
pub(crate) use fields::VariantIr;
pub(crate) use options::DeclarationOptions;
pub(crate) use redact::RedactIr;
pub(crate) use redact::RedactModeIr;
pub(crate) use reference_target_ir::ReferenceTargetIr;
pub(crate) use references::ReferenceIr;
pub(crate) use serde::SerdeIr;
pub(crate) use strategy_argument_ir::StrategyArgumentIr;
pub(crate) use unique_ir::UniqueIr;
pub(crate) use validation::SelectorIr;
pub(crate) use validation::SelectorPositionIr;
pub(crate) use validation::ValidatorIr;
