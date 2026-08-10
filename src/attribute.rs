// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Strongly typed type-level and field-level metadata attributes.

mod attribute_kind;
mod attribute_metadata;
mod element_metadata;
mod index_metadata;
mod key_metadata;
mod primary_key_field_metadata;
mod primary_key_metadata;
mod sensitive_handling;
mod sensitive_metadata;
mod strategy_ref;
mod unique_comparison;
mod unique_field_metadata;
mod unique_metadata;
mod validation;

pub use self::attribute_kind::AttributeKind;
pub use self::attribute_metadata::AttributeMetadata;
pub use self::element_metadata::ElementMetadata;
pub use self::index_metadata::IndexMetadata;
pub use self::key_metadata::KeyMetadata;
pub use self::primary_key_field_metadata::PrimaryKeyFieldMetadata;
pub use self::primary_key_metadata::PrimaryKeyMetadata;
pub use self::sensitive_handling::SensitiveHandling;
pub use self::sensitive_metadata::SensitiveMetadata;
pub use self::strategy_ref::StrategyRef;
pub use self::unique_comparison::UniqueComparison;
pub use self::unique_field_metadata::UniqueFieldMetadata;
pub use self::unique_metadata::UniqueMetadata;
