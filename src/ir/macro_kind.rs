// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Defines the macro roles accepted by the shared compiler pipeline.

/// Identifies the public macro being compiled by the shared pipeline.
///
/// Each variant selects the role-specific validation and metadata expansion
/// rules applied to the annotated declaration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MacroKind {
    /// Compiles a persistent identity-bearing model.
    Entity,
    /// Compiles an entity view declaration.
    Projection,
    /// Compiles an ordinary structured model.
    Model,
    /// Compiles a domain enum declaration.
    Enum,
    /// Compiles a domain value declaration.
    Value,
    /// Compiles getter/setter property metadata.
    ModelProperties,
}
