// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// The filename is part of a Cargo fixture rather than an integration-test
// target.

//! Re-exports the model types and registrations used by linked-workspace fixtures.

use qubit_codec::ValueCodecId;
use qubit_codec::ValueCodecRegistrationSource;
use qubit_codec::ValueStringCodecRegistration;
use qubit_validator::RegistrationSource;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorRegistration;

#[cfg(feature = "duplicate-fixture")]
mod duplicate;
mod target;
mod text_codec;
mod text_rule;

#[cfg(feature = "duplicate-fixture")]
pub use duplicate::Duplicate;
pub use target::Target;
pub use text_codec::TextCodec;

// Preserve the source location recorded before the codec was split into its own module.
/// The text codec registration consumed by the linked-workspace fixture.
pub static CODEC: ValueStringCodecRegistration = ValueStringCodecRegistration::new(
    ValueCodecId::new("test.linked.text"),
    &text_codec::CODEC_DESCRIPTOR,
    ValueCodecRegistrationSource::new("model-b", "models", file!(), 77),
);

// Preserve the source location recorded before the validator was split into its own module.
/// The validator registration consumed by the linked-workspace fixture.
pub static RULE: ValidatorRegistration = ValidatorRegistration::new(
    ValidatorId::new("test.linked.text"),
    &text_rule::RULE_DESCRIPTOR,
    RegistrationSource::new("model-b", "models", file!(), 104),
);
