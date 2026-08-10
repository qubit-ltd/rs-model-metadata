// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests for [`OwnershipMetadata`].

use qubit_model_metadata::NamedTypeRef;
use qubit_model_metadata::OwnershipMetadata;
use qubit_model_metadata::TypeIdentity;

struct Target;

#[test]
fn test_ownership_metadata_preserves_owner() {
    let owner = NamedTypeRef::unresolved(TypeIdentity::of::<Target>());
    let ownership = OwnershipMetadata::new(owner);

    assert_eq!(ownership.owner().identity(), owner.identity());
}
