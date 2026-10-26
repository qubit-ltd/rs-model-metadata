// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Collection position selected by a selector declaration.

/// Kind of collection position addressed by a selector declaration.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum SelectorPositionIr {
    /// Sequence element position.
    Element,
    /// Map key position.
    MapKey,
    /// Map value position.
    MapValue,
}
