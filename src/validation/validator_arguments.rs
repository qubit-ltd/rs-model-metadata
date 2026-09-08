// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Conversion from declaration-side arguments to validator runtime arguments.

use qubit_validator::NamedValidationArgument as RuntimeNamedValidationArgument;
use qubit_validator::ValidationArgument as RuntimeValidationArgument;

use crate::metadata::NamedValidationArgument;
use crate::metadata::ValidationArgument;

/// Converts declaration arguments to the validator runtime vocabulary.
pub(crate) fn validator_arguments<'a>(
    arguments: &[NamedValidationArgument<'a>],
) -> Vec<RuntimeNamedValidationArgument<'a>> {
    arguments
        .iter()
        .map(|argument| {
            let value = match argument.value() {
                ValidationArgument::Bool(value) => RuntimeValidationArgument::Bool(value),
                ValidationArgument::Integer(value) => RuntimeValidationArgument::Integer(value),
                ValidationArgument::Unsigned(value) => RuntimeValidationArgument::Unsigned(value),
                ValidationArgument::String(value) => RuntimeValidationArgument::String(value),
                ValidationArgument::BoolList(value) => RuntimeValidationArgument::BoolList(value),
                ValidationArgument::IntegerList(value) => {
                    RuntimeValidationArgument::IntegerList(value)
                }
                ValidationArgument::UnsignedList(value) => {
                    RuntimeValidationArgument::UnsignedList(value)
                }
                ValidationArgument::StringList(value) => {
                    RuntimeValidationArgument::StringList(value)
                }
            };
            RuntimeNamedValidationArgument::new(argument.name(), value)
        })
        .collect()
}
