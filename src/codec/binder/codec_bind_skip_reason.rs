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
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::codec::CodecBindSkipReason;
///
/// fn describe(reason: CodecBindSkipReason) -> &'static str {
///     match reason {
///         CodecBindSkipReason::FieldDescriptorUnavailable => "field type unavailable",
///         CodecBindSkipReason::SelectorTypeUnavailable => "selector type unavailable",
///         _ => "other type-resolution boundary",
///     }
/// }
///
/// assert_eq!(describe(CodecBindSkipReason::FieldDescriptorUnavailable), "field type unavailable");
/// ```
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodecBindSkipReason {
    /// The field's concrete type descriptor could not be resolved.
    FieldDescriptorUnavailable,
    /// A selector's target type could not be resolved.
    SelectorTypeUnavailable,
}
