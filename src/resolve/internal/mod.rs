// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! State retained only during one structural resolution attempt.

mod owned_property_path;
mod resolution_context;

pub(super) use owned_property_path::OwnedPropertyPath;
pub(super) use resolution_context::ResolutionContext;
