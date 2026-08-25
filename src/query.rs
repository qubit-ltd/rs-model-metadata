// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Read-only attribute and field-path query operations.

mod attribute_query;
mod field_path_resolve_error;

pub use self::attribute_query::AttributeQuery;
pub use self::field_path_resolve_error::FieldPathResolveError;
