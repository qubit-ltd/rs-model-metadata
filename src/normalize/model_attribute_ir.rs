// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::named_fields_ir::NamedFieldsIr;
use super::ownership_ir::OwnershipIr;
use super::primary_key_ir::PrimaryKeyIr;
use super::unique_ir::UniqueIr;

/// A canonical model-level attribute.
pub(crate) enum ModelAttributeIr {
    /// The model's primary-key definition.
    PrimaryKey(
        /// Canonical primary-key fields and generated markers.
        PrimaryKeyIr,
    ),
    /// A unique-constraint definition.
    Unique(
        /// Canonical unique fields, names, and ignore-case markers.
        UniqueIr,
    ),
    /// An index definition.
    Index(
        /// Canonical index name and ordered field list.
        NamedFieldsIr,
    ),
    /// A logical-key definition.
    Key(
        /// Canonical logical-key name and ordered field list.
        NamedFieldsIr,
    ),
    /// An ownership relation.
    Ownership(
        /// Canonical owning-model type path.
        OwnershipIr,
    ),
}
