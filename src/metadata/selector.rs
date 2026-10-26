// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Nested collection selector declarations.

#[path = "selector_metadata.rs"]
mod selector_metadata;
#[path = "selector_position.rs"]
mod selector_position;

pub use selector_metadata::SelectorMetadata;
pub use selector_position::SelectorPosition;
