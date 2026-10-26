// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests for concrete field identity.

use std::any::TypeId;

use crate::relation::FieldLocation;

#[test]
fn test_location_preserves_owner_variant_and_field_index() {
    let location = FieldLocation::new(TypeId::of::<String>(), Some(2), 4);

    assert_eq!(location.owner(), TypeId::of::<String>());
    assert_eq!(location.variant(), Some(2));
    assert_eq!(location.index(), 4);
}
