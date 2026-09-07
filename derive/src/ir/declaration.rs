// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
//    You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
//    Unless required by applicable law or agreed to in writing, software
//    distributed under the License is distributed on an "AS IS" BASIS,
//    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//    See the License for the specific language governing permissions and
//    limitations under the License.
// =============================================================================

//! The compiler's declaration vocabulary, organized by semantic concern.

// qubit-style: allow public-type-layout
// qubit-style: allow multiple-public-types

mod codec;
mod constraints;
mod fields;
mod options;
mod redact;
mod references;
mod serde;
mod validation;

pub(crate) use codec::CodecIr;
pub(crate) use constraints::ConstraintIr;
pub(crate) use constraints::DecimalConstraintIr;
pub(crate) use constraints::TextConstraintIr;
pub(crate) use fields::FieldIr;
pub(crate) use fields::FieldOccurrence;
pub(crate) use fields::IdentifierAssignmentIr;
pub(crate) use fields::VariantIr;
pub(crate) use options::DeclarationOptions;
pub(crate) use redact::RedactIr;
pub(crate) use redact::RedactModeIr;
pub(crate) use references::ReferenceIr;
pub(crate) use references::ReferenceTargetIr;
pub(crate) use references::UniqueIr;
pub(crate) use serde::SerdeIr;
pub(crate) use validation::OnNoneIr;
pub(crate) use validation::SelectorIr;
pub(crate) use validation::SelectorPositionIr;
pub(crate) use validation::StrategyArgumentIr;
pub(crate) use validation::TargetModeIr;
pub(crate) use validation::ValidatorIr;

use super::MacroKind;

/// Complete normalized declaration consumed by the expansion stage.
pub(crate) struct DeclarationIr {
    pub(crate) kind: MacroKind,
    pub(crate) options: DeclarationOptions,
    pub(crate) fields: Vec<FieldIr>,
    pub(crate) variants: Vec<VariantIr>,
}
