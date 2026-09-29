// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Public contracts for checked relative object navigation.

use qubit_model_metadata::metadata::NavigationStep;
use qubit_model_metadata::metadata::ObjectPath;
use qubit_model_metadata::metadata::ObjectPathError;

#[test]
fn test_object_paths_validate_steps_and_track_parent_context() {
    let current = ObjectPath::current();
    assert!(current.steps().is_empty());
    assert!(!current.requires_parent());

    let nested = ObjectPath::new(&[NavigationStep::Property("parent"), NavigationStep::Parent])
        .expect("balanced parent traversal");
    assert_eq!(
        nested.steps(),
        &[NavigationStep::Property("parent"), NavigationStep::Parent]
    );
    assert!(!nested.requires_parent());
    assert_eq!(nested.to_string(), "parent/..");

    let external = ObjectPath::new(&[NavigationStep::Parent]).expect("leading parent traversal");
    assert!(external.requires_parent());
    assert_eq!(external.to_string(), "..");
    assert_eq!(
        ObjectPath::new(&[NavigationStep::Property("invalid/name")]),
        Err(ObjectPathError {
            index: 0,
            name: "invalid/name",
        }),
    );
}
