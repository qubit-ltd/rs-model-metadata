// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_model_metadata::metadata::FieldAttributeMetadata;
use qubit_model_metadata::metadata::FieldMetadata;

fn obsolete_nested_flag(field: &FieldMetadata) -> bool {
    let _ = FieldAttributeMetadata::ValidateNested;
    field.validate_nested()
}

fn main() {}
