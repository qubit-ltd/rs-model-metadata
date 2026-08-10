// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::AttributeKind;
use super::ElementMetadata;
use super::IndexMetadata;
use super::KeyMetadata;
use super::PrimaryKeyMetadata;
use super::SensitiveMetadata;
use super::StrategyRef;
use super::UniqueMetadata;
use crate::constraint::DecimalConstraint;
use crate::constraint::MapConstraint;
use crate::constraint::SequenceConstraint;
use crate::constraint::TemporalConstraint;
use crate::constraint::TextConstraint;
use crate::relation::LookupRelationMetadata;
use crate::relation::OwnershipMetadata;
use crate::relation::ReferenceMetadata;

/// A strongly typed metadata attribute.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum AttributeMetadata {
    /// Text constraints for a field.
    Text(TextConstraint),
    /// Sequence constraints for a field.
    Sequence(SequenceConstraint),
    /// Map constraints for a field.
    Map(MapConstraint),
    /// Temporal constraints for a field.
    Temporal(TemporalConstraint),
    /// Decimal constraints for a field.
    Decimal(DecimalConstraint),
    /// Constraints applied to each element of a sequence field.
    Element(ElementMetadata),
    /// A model-level primary-key definition.
    PrimaryKey(PrimaryKeyMetadata),
    /// A model-level unique-constraint definition.
    Unique(UniqueMetadata),
    /// A model-level index definition.
    Index(IndexMetadata),
    /// A model-level logical-key definition.
    Key(KeyMetadata),
    /// A direct reference to another model.
    Reference(ReferenceMetadata),
    /// A lookup relation to another model.
    LookupRelation(LookupRelationMetadata),
    /// An ownership relation to another model.
    Ownership(OwnershipMetadata),
    /// A codec strategy name.
    Codec(StrategyRef),
    /// A generator strategy name.
    Generator(StrategyRef),
    /// Sensitive-data handling metadata.
    Sensitive(SensitiveMetadata),
}

impl AttributeMetadata {
    /// Returns the discriminant used by generic attribute queries.
    ///
    /// # Returns
    ///
    /// The [`AttributeKind`] corresponding to this attribute.
    #[must_use]
    pub const fn kind(self) -> AttributeKind {
        match self {
            Self::Text(_) => AttributeKind::Text,
            Self::Sequence(_) => AttributeKind::Sequence,
            Self::Map(_) => AttributeKind::Map,
            Self::Temporal(_) => AttributeKind::Temporal,
            Self::Decimal(_) => AttributeKind::Decimal,
            Self::Element(_) => AttributeKind::Element,
            Self::PrimaryKey(_) => AttributeKind::PrimaryKey,
            Self::Unique(_) => AttributeKind::Unique,
            Self::Index(_) => AttributeKind::Index,
            Self::Key(_) => AttributeKind::Key,
            Self::Reference(_) => AttributeKind::Reference,
            Self::LookupRelation(_) => AttributeKind::LookupRelation,
            Self::Ownership(_) => AttributeKind::Ownership,
            Self::Codec(_) => AttributeKind::Codec,
            Self::Generator(_) => AttributeKind::Generator,
            Self::Sensitive(_) => AttributeKind::Sensitive,
        }
    }
}
