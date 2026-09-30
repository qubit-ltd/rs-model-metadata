// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Public declaration location behavior.

use qubit_model_metadata::metadata::DeclarationLocation;
use qubit_model_metadata::metadata::SelectorPosition;

#[test]
fn test_unknown_location_can_retain_a_selector_position() {
    let location = DeclarationLocation::unknown().with_selector(SelectorPosition::Element);

    assert_eq!(location.file, None);
    assert_eq!(location.selector, Some(SelectorPosition::Element));
}
