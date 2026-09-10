// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Private compilation and execution machinery.

pub(super) mod field_rule_binding;
pub(super) mod selector_binding;

pub(super) mod declaration_walker;
pub(super) mod execution_declaration;
pub(super) mod occurrence_binder;
pub(super) mod validation_occurrence;

pub(super) mod execution_budget;
pub(super) mod execution_failure;
pub(super) mod path_reader;
pub(super) mod report_accumulator;
