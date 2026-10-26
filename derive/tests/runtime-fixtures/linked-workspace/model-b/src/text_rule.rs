// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Defines the validator registration used to verify cross-crate execution.

use std::sync::Arc;

use qubit_validator::ArgumentReader;
use qubit_validator::BindError;
use qubit_validator::InputType;
use qubit_validator::PreparedValidator;
use qubit_validator::ValidatorDescriptor;
use qubit_validator::ValidatorSignature;

mod internal;

use self::internal::TextRule;

/// The single text signature supported by this fixture's validator.
static SIGNATURES: &[ValidatorSignature] = &[
    ValidatorSignature::new(InputType::Text, &[], prepare_text),
];

/// The immutable descriptor used by the static validator registration.
pub(super) static RULE_DESCRIPTOR: ValidatorDescriptor = ValidatorDescriptor::new(SIGNATURES);

/// Prepares the text validator after the framework supplies its argument reader.
///
/// # Parameters
///
/// - `_reader`: Argument reader supplied by the framework during binding.
///
/// # Returns
///
/// Returns a prepared validator that checks the bound text value.
///
/// # Errors
///
/// The current implementation does not reject binding, but uses the framework's
/// `BindError` result type to satisfy the validator preparation interface.
pub(super) fn prepare_text(
    _reader: &mut ArgumentReader<'_>,
) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(Arc::new(TextRule))
}
