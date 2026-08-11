// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Errors returned while resolving field paths.

/// A typed reason why a field path cannot be resolved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum FieldPathResolveError {
    /// The supplied path contains no segments.
    EmptyPath,
    /// A field segment is not present in the current struct metadata.
    FieldNotFound {
        /// The missing field segment.
        segment: &'static str,
    },
    /// A non-final path segment does not refer to a named struct.
    IntermediateNotStruct {
        /// The segment whose field cannot be traversed.
        segment: &'static str,
    },
    /// A named intermediate field has no metadata resolver.
    NamedMetadataUnavailable {
        /// The segment whose named type could not be resolved.
        segment: &'static str,
    },
}
