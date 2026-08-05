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

pub use crate::attribute::{
    AttributeKind,
    AttributeMetadata,
    ElementMetadata,
    IndexMetadata,
    KeyMetadata,
    PrimaryKeyFieldMetadata,
    PrimaryKeyMetadata,
    SensitiveHandling,
    SensitiveMetadata,
    StrategyRef,
    UniqueComparison,
    UniqueFieldMetadata,
    UniqueMetadata,
};
pub use crate::constraint::{
    DecimalConstraint,
    DecimalSemantic,
    MapConstraint,
    RoundingMode,
    SequenceConstraint,
    TemporalConstraint,
    TemporalNormalization,
    TemporalPrecision,
    TextConstraint,
    TextFormat,
    TextRepertoire,
};
pub use crate::field_metadata::FieldMetadata;
pub use crate::metadata_registry::MetadataRegistry;
pub use crate::metadata_resolver::MetadataResolver;
pub use crate::query::{
    AttributeQuery,
    FieldPathResolveError,
};
pub use crate::relation::{
    FieldPath,
    LookupRelationMetadata,
    OwnershipMetadata,
    ReferenceMetadata,
};
pub use crate::type_metadata::{
    EnumMetadata,
    EnumVariantMetadata,
    HasTypeMetadata,
    NamedTypeRef,
    NewtypeMetadata,
    StructMetadata,
    TypeIdentity,
    TypeKind,
    TypeMetadata,
    metadata_of,
};
pub use crate::type_shape::{
    HasTypeShape,
    ScalarType,
    TypeCapabilities,
    TypeRef,
    TypeShape,
};
