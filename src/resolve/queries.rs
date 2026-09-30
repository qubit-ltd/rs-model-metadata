// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Read-only indexed declarations; downstream crates choose filter behavior.

use super::graph::QueryDeclaration;
use super::graph::QueryMetadata;
use super::relations::path_from_segments;
use crate::metadata::TypeMetadata;

/// Collects direct indexed members without expanding nested query policies.
///
/// # Parameters
///
/// - `metadata`: Model metadata whose directly declared indexed fields are
///   collected.
///
/// # Returns
///
/// Query metadata containing the indexed declarations in source field order.
#[must_use]
pub(super) fn build_query(metadata: &'static TypeMetadata) -> QueryMetadata {
    let declarations = metadata
        .fields()
        .iter()
        .filter(|field| field.is_indexed())
        .filter_map(|field| {
            field.name().map(|name| QueryDeclaration {
                field,
                path: path_from_segments(&[name]),
            })
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();
    QueryMetadata { declarations }
}
