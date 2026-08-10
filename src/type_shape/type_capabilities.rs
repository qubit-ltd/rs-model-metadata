// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Capabilities that determine the metadata attributes a type can accept.

use bitflags::bitflags;

bitflags! {
    /// Capabilities that determine which metadata attributes a type can accept.
    #[must_use]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct TypeCapabilities: u8 {
        /// The type accepts no metadata constraints.
        const NONE = 0;
        /// The type accepts text constraints.
        const TEXT = 1 << 0;
        /// The type accepts sequence constraints.
        const SEQUENCE = 1 << 1;
        /// The type is a set.
        const SET = 1 << 2;
        /// The type accepts map constraints.
        const MAP = 1 << 3;
        /// The type accepts temporal constraints.
        const TEMPORAL = 1 << 4;
        /// The type accepts decimal constraints.
        const DECIMAL = 1 << 5;
        /// The type is a fixed-length array.
        const ARRAY = 1 << 6;
    }
}
