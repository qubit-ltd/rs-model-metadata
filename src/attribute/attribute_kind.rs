// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// A discriminant for generic attribute queries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum AttributeKind {
    /// Text constraints.
    Text,
    /// Sequence constraints.
    Sequence,
    /// Map constraints.
    Map,
    /// Temporal constraints.
    Temporal,
    /// Decimal constraints.
    Decimal,
    /// Sequence element constraints.
    Element,
    /// Primary-key definitions.
    PrimaryKey,
    /// Unique-constraint definitions.
    Unique,
    /// Index definitions.
    Index,
    /// Logical-key definitions.
    Key,
    /// Direct references.
    Reference,
    /// Lookup relations.
    LookupRelation,
    /// Ownership relations.
    Ownership,
    /// Codec strategies.
    Codec,
    /// Generator strategies.
    Generator,
    /// Sensitive-data metadata.
    Sensitive,
}
