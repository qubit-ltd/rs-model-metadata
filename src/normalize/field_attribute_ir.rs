// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::decimal_ir::DecimalIr;
use super::element_ir::ElementIr;
use crate::attribute::LookupRelationAttribute;
use crate::attribute::MapAttribute;
use crate::attribute::ReferenceAttribute;
use crate::attribute::SequenceAttribute;
use crate::attribute::StrategyAttribute;
use crate::attribute::TemporalAttribute;
use crate::attribute::TextAttribute;

/// A canonical field-level attribute emitted into `FieldMetadata`.
pub(crate) enum FieldAttributeIr {
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
    /// Decimal constraints with normalized domain semantics.
    Decimal(
        /// Parsed decimal constraint values, or normalized decimal IR.
        DecimalIr,
    ),
    /// Constraints applied to sequence elements.
    Element(
        /// Parsed or normalized constraints applied to sequence elements.
        ElementIr,
    ),
    /// A direct model reference.
    Reference(
        /// Parsed direct-reference values.
        ReferenceAttribute,
    ),
    /// A lookup relation to another model.
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
}
