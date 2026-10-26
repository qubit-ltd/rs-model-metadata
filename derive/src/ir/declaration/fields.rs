// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Field and enum-variant IR.

mod field_ir;
mod field_occurrence;
mod identifier_assignment_ir;
mod variant_ir;

pub(crate) use field_ir::FieldIr;
pub(crate) use field_occurrence::FieldOccurrence;
pub(crate) use identifier_assignment_ir::IdentifierAssignmentIr;
pub(crate) use variant_ir::VariantIr;
