// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Field and enum-variant IR.

#[path = "field_ir.rs"]
mod field_ir;
#[path = "field_occurrence.rs"]
mod field_occurrence;
#[path = "identifier_assignment_ir.rs"]
mod identifier_assignment_ir;
#[path = "variant_ir.rs"]
mod variant_ir;

pub(crate) use field_ir::FieldIr;
pub(crate) use field_occurrence::FieldOccurrence;
pub(crate) use identifier_assignment_ir::IdentifierAssignmentIr;
pub(crate) use variant_ir::VariantIr;
