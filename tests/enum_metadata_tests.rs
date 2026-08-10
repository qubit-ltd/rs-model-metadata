// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_model_metadata::EnumMetadata;
use qubit_model_metadata::EnumVariantMetadata;

static VARIANTS: [EnumVariantMetadata; 1] =
    [EnumVariantMetadata::new(0, "active")];

#[test]
fn test_enum_metadata_finds_variants_by_name() {
    let metadata = EnumMetadata::new(&VARIANTS);

    assert_eq!(
        metadata.variant("active").map(EnumVariantMetadata::ordinal),
        Some(0)
    );
}
