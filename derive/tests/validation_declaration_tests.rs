// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use model_runtime::metadata::OnNone;
use model_runtime::metadata::TargetMode;
use model_runtime::metadata::TypeMetadata;
use qubit_model_derive::Model;

#[Model]
struct NamedDependencyExample {
    kind: u8,
    #[validator(
        id = "example.rule",
        depends_on(kind(property = kind))
    )]
    number: Option<String>,
}

#[Model]
struct LegacyDependencyExample {
    other: String,
    #[validator(id = "example.legacy", depends_on(other))]
    value: String,
}

#[test]
fn named_dependency_metadata_preserves_slot_and_path() {
    let field = TypeMetadata::of::<NamedDependencyExample>()
        .field("number")
        .expect("number field");
    let validator = &field.validators()[0];
    let dependency = &validator.dependency_bindings()[0];

    assert_eq!(validator.declared_id(), "example.rule");
    assert_eq!(dependency.name(), "kind");
    assert_eq!(dependency.path().segments(), &["kind"]);
    assert_eq!(validator.target(), TargetMode::Value);
    assert_eq!(validator.on_none(), OnNone::Skip);
}

#[test]
fn bare_dependency_metadata_remains_available_for_legacy_declarations() {
    let field = TypeMetadata::of::<LegacyDependencyExample>()
        .field("value")
        .expect("value field");
    let validator = &field.validators()[0];

    assert_eq!(validator.dependency_bindings()[0].property().segments(), &["other"]);
    assert_eq!(validator.target(), TargetMode::Value);
    assert_eq!(validator.on_none(), OnNone::Skip);
}
// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================
