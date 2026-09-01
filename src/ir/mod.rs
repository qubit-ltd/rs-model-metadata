// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Semantic values shared by the macro compiler stages.

/// Identifies the public macro being compiled by the shared pipeline.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MacroKind {
    /// Persistent identity-bearing model.
    Entity,
    /// Entity view declaration.
    Projection,
    /// Ordinary structured model.
    Model,
    /// Domain enum declaration.
    Enum,
    /// Domain value declaration.
    Value,
    /// Getter/setter property declaration.
    ModelProperties,
}
