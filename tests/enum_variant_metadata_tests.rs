// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_model_metadata::EnumVariantMetadata;

#[test]
fn test_enum_variant_metadata_exposes_its_declaration_details() {
    let metadata = EnumVariantMetadata::new(2, "active");

    assert_eq!(metadata.ordinal(), 2);
    assert_eq!(metadata.name(), "active");
}
