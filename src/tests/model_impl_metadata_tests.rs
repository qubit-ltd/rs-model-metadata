// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Crate-private construction contract for generated implementation metadata.

use crate::metadata::LocalPropertySet;
use crate::metadata::ModelImplMetadata;

#[test]
fn test_generated_impl_metadata_retains_empty_local_view() {
    let generated =
        ModelImplMetadata::new(&[], Ok(Box::leak(Box::new(LocalPropertySet::new(&[])))));
    assert!(generated.fragments().is_empty());
    assert!(
        generated
            .try_properties()
            .expect("valid empty generated view")
            .properties()
            .is_empty()
    );
}
