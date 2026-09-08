// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Verifies that declaration metadata does not expose execution descriptors.

use qubit_model_metadata::metadata::CodecReference;
use qubit_model_metadata::metadata::NamedValidationArgument;
use qubit_model_metadata::metadata::RustTypeReference;
use qubit_model_metadata::metadata::Sensitivity;
use qubit_model_metadata::metadata::ValidationArgument;

struct ExampleCodec;

#[test]
fn test_declaration_vocabulary_is_execution_independent() {
    let reference = RustTypeReference::of::<ExampleCodec>();
    assert_eq!(
        reference.type_name(),
        core::any::type_name::<ExampleCodec>()
    );
    assert_eq!(
        CodecReference::RustType(reference).rust_type(),
        Some(reference)
    );
    let argument = NamedValidationArgument::new(
        "minimum",
        ValidationArgument::Unsigned(1),
    );
    assert_eq!(argument.name(), "minimum");
    assert_eq!(argument.value(), ValidationArgument::Unsigned(1));
    assert_eq!(Sensitivity::Secret.as_str(), "secret");
}
