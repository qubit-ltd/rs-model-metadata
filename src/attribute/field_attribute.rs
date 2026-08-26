// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use proc_macro2::Span;

use super::decimal_attribute::DecimalAttribute;
use super::element_attribute::ElementAttribute;
use super::field_unique_attribute::FieldUniqueAttribute;
use super::identifier_attribute::IdentifierAttribute;
use super::lookup_relation_attribute::LookupRelationAttribute;
use super::map_attribute::MapAttribute;
use super::reference_attribute::ReferenceAttribute;
use super::sequence_attribute::SequenceAttribute;
use super::strategy_attribute::StrategyAttribute;
use super::temporal_attribute::TemporalAttribute;
use super::text_attribute::TextAttribute;

/// Parsed field-level attribute syntax.
pub(crate) enum FieldAttribute {
    /// A single-field primary-key shorthand.
    Identifier(
        /// Parsed `identifier` syntax, including optional `generated`.
        IdentifierAttribute,
    ),
    /// A single-field unique-constraint shorthand.
    Unique(
        /// Parsed unique-constraint syntax for this field or model.
        FieldUniqueAttribute,
    ),
    /// A single-field index shorthand.
    Index(
        /// Source span of the field-level `indexed` marker.
        Span,
    ),
    /// Text constraints.
    Text(
        /// Parsed text-constraint values.
        TextAttribute,
    ),
    /// Ordered-sequence constraints.
    Sequence(
        /// Parsed ordered-sequence constraint values.
        SequenceAttribute,
    ),
    /// Map constraints.
    Map(
        /// Parsed map-constraint values.
        MapAttribute,
    ),
    /// Temporal constraints.
    Temporal(
        /// Parsed temporal-constraint values.
        TemporalAttribute,
    ),
    /// Ordinary decimal constraints.
    Decimal(
        /// Parsed decimal constraint values, or normalized decimal IR.
        DecimalAttribute,
    ),
    /// Monetary decimal constraints.
    Money(
        /// Parsed monetary decimal constraint values.
        DecimalAttribute,
    ),
    /// Constraints applied to each sequence element.
    Element(
        /// Parsed or normalized constraints applied to sequence elements.
        ElementAttribute,
    ),
    /// A direct model reference.
    Reference(
        /// Parsed direct-reference values.
        ReferenceAttribute,
    ),
    /// A relation resolved by looking up another model.
    LookupRelation(
        /// Parsed lookup-relation values.
        LookupRelationAttribute,
    ),
    /// A codec strategy.
    Codec(
        /// Parsed codec strategy name.
        StrategyAttribute,
    ),
    /// A generator strategy.
    Generator(
        /// Parsed generator strategy name.
        StrategyAttribute,
    ),
    /// An explicit opaque type marker.
    Opaque(
        /// Source span of the `opaque` marker.
        Span,
    ),
    /// Retains the normal serialized representation for an otherwise skipped
    /// field.
    KeepSerializing,
}
