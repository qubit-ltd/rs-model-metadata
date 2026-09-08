// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Stable declaration-side model vocabulary.

mod codec;
mod redaction;
mod selector;
mod validation;

pub use codec::*;
pub use redaction::*;
pub use selector::*;
pub use validation::*;

pub use crate::abi_violation::AbiViolation;
pub use crate::constraint::AllowedChars;
pub use crate::constraint::ConstraintMetadata;
pub use crate::constraint::DecimalConstraint;
pub use crate::constraint::DecimalSemantic;
pub use crate::constraint::MapConstraint;
pub use crate::constraint::RoundingMode;
pub use crate::constraint::SequenceConstraint;
pub use crate::constraint::TemporalConstraint;
pub use crate::constraint::TemporalPrecision;
pub use crate::constraint::TextConstraint;
pub use crate::constraint::TextFormat;
pub use crate::constraint::TimeConstraint;
pub use crate::field_metadata::FieldMetadata;
pub use crate::local_property_set::LocalPropertySet;
pub use crate::metadata_vocabulary::DeclaredEntityTarget;
pub use crate::metadata_vocabulary::DeclaredEntityTargetKind;
pub use crate::metadata_vocabulary::FieldAttributeMetadata;
pub use crate::metadata_vocabulary::IdentifierAssignment;
pub use crate::metadata_vocabulary::IdentifierMetadata;
pub use crate::metadata_vocabulary::IndexingReasons;
pub use crate::metadata_vocabulary::KeyPartMetadata;
pub use crate::metadata_vocabulary::ReferenceMetadata as FieldReferenceMetadata;
pub use crate::metadata_vocabulary::ReferenceSelection;
pub use crate::metadata_vocabulary::SerdeBehaviorSource;
pub use crate::metadata_vocabulary::SerdeFieldMetadata;
pub use crate::metadata_vocabulary::UniqueMetadata as FieldUniqueMetadata;
pub use crate::model_id::ModelId;
pub use crate::model_id::ModelIdBuf;
pub use crate::model_id::ModelIdError;
pub use crate::model_impl_metadata::ModelImplMetadata;
pub use crate::model_metadata_error::ModelMetadataError;
pub use crate::property::BorrowedPropertySlice;
pub use crate::property::GetterAdapter;
pub use crate::property::GetterMetadata;
pub use crate::property::GetterOutputKind;
pub use crate::property::PropertyAccessError;
pub use crate::property::PropertyMetadata;
pub use crate::property::PropertySetFailure;
pub use crate::property::PropertyStorageKind;
pub use crate::property::PropertyValue;
pub use crate::property::SetterAdapter;
pub use crate::property::SetterMetadata;
pub use crate::property_build_error::PropertyBuildError;
pub use crate::property_build_error_kind::PropertyBuildErrorKind;
pub use crate::property_build_errors::PropertyBuildErrors;
pub use crate::property_fragment::PropertyFragment;
pub use crate::property_fragment_source::PropertyFragmentSource;
pub use crate::property_resolution_error::PropertyResolutionError;
pub use crate::relation::PropertyPath;
pub use crate::role::EntityMetadata;
pub use crate::role::ModelMetadata;
pub use crate::role::ModelRole;
pub use crate::role::ProjectionMetadata;
pub use crate::role::RoleMetadata;
pub use crate::role::ValueMetadata;
pub use crate::type_metadata::EnumMetadata;
pub use crate::type_metadata::EnumVariantMetadata;
pub use crate::type_metadata::HasTypeMetadata;
pub use crate::type_metadata::TypeMetadata;
