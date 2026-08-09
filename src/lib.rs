// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Static metadata for Rust domain models.

mod attribute;
mod constraint;
mod field_metadata;
mod metadata_registry;
mod metadata_resolver;
mod query;
mod relation;
mod type_metadata;
mod type_shape;

pub use crate::attribute::AttributeKind;
pub use crate::attribute::AttributeMetadata;
pub use crate::attribute::ElementMetadata;
pub use crate::attribute::IndexMetadata;
pub use crate::attribute::KeyMetadata;
pub use crate::attribute::PrimaryKeyFieldMetadata;
pub use crate::attribute::PrimaryKeyMetadata;
pub use crate::attribute::SensitiveHandling;
pub use crate::attribute::SensitiveMetadata;
pub use crate::attribute::StrategyRef;
pub use crate::attribute::UniqueComparison;
pub use crate::attribute::UniqueFieldMetadata;
pub use crate::attribute::UniqueMetadata;
pub use crate::constraint::DecimalConstraint;
pub use crate::constraint::DecimalSemantic;
pub use crate::constraint::MapConstraint;
pub use crate::constraint::RoundingMode;
pub use crate::constraint::SequenceConstraint;
pub use crate::constraint::TemporalConstraint;
pub use crate::constraint::TemporalNormalization;
pub use crate::constraint::TemporalPrecision;
pub use crate::constraint::TextConstraint;
pub use crate::constraint::TextFormat;
pub use crate::constraint::TextRepertoire;
pub use crate::field_metadata::FieldMetadata;
pub use crate::metadata_registry::MetadataRegistry;
pub use crate::metadata_resolver::MetadataResolver;
pub use crate::query::AttributeQuery;
pub use crate::query::FieldPathResolveError;
pub use crate::relation::FieldPath;
pub use crate::relation::LookupRelationMetadata;
pub use crate::relation::OwnershipMetadata;
pub use crate::relation::ReferenceMetadata;
pub use crate::type_metadata::EnumMetadata;
pub use crate::type_metadata::EnumVariantMetadata;
pub use crate::type_metadata::HasTypeMetadata;
pub use crate::type_metadata::NamedTypeRef;
pub use crate::type_metadata::NewtypeMetadata;
pub use crate::type_metadata::StructMetadata;
pub use crate::type_metadata::TypeIdentity;
pub use crate::type_metadata::TypeKind;
pub use crate::type_metadata::TypeMetadata;
pub use crate::type_metadata::metadata_of;
pub use crate::type_shape::HasTypeShape;
pub use crate::type_shape::ScalarType;
pub use crate::type_shape::TypeCapabilities;
pub use crate::type_shape::TypeRef;
pub use crate::type_shape::TypeShape;
