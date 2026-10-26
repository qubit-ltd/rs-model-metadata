// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Original causes retained by model-registry errors.

use std::fmt::Display;

use qubit_reflect::capability::CapabilityError;
use qubit_reflect::error::ContextError;
#[cfg(feature = "generic")]
use qubit_reflect::error::LookupError;

use crate::metadata::AbiViolation;

/// A concrete cause variant preserved for source-chain reporting.
#[derive(Clone, Debug)]
pub(crate) enum Cause {
    /// A failure while querying reflection context.
    Context(ContextError),
    /// A failure while resolving model capabilities.
    Capability(CapabilityError),
    /// A failed generic definition lookup.
    #[cfg(feature = "generic")]
    Lookup(LookupError),
    /// Invalid ABI metadata supplied for a model.
    Abi(AbiViolation),
}

impl Display for Cause {
    /// Formats the retained underlying error without changing its message.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Context(error) => Display::fmt(error, formatter),
            Self::Capability(error) => Display::fmt(error, formatter),
            #[cfg(feature = "generic")]
            Self::Lookup(error) => Display::fmt(error, formatter),
            Self::Abi(error) => Display::fmt(error, formatter),
        }
    }
}
