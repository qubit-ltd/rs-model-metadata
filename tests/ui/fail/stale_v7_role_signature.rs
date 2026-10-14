// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_model_metadata::__private::v7;
use qubit_model_metadata::metadata::FieldMetadata;
use qubit_model_metadata::metadata::RoleMetadata;

const _: fn(&'static FieldMetadata) -> RoleMetadata = v7::entity_role;

fn main() {}
