// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Reasons why a codec binding traversal could not inspect a declaration.

/// Describes a declaration skipped because its runtime type was unavailable.
///
/// These cases indicate an inspection boundary, not a missing codec. A
/// declaration without a resolvable type may or may not require a codec.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodecBindSkipReason {
    /// The field's concrete type descriptor could not be resolved.
    FieldDescriptorUnavailable,
    /// A selector's target type could not be resolved.
    SelectorTypeUnavailable,
}
