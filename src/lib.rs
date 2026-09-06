// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Static metadata overlays for Rust domain models.

#[doc(hidden)]
pub mod __private;
mod abi_violation;
mod constraint;
mod field_metadata;
mod field_semantics;
mod generic_model_metadata;
mod local_property_set;
mod metadata_registry;
mod metadata_resolver;
mod model_id;
mod model_impl_metadata;
mod model_metadata_error;
mod property;
mod property_build_error;
mod property_build_error_kind;
mod property_build_errors;
mod property_fragment;
mod property_fragment_source;
mod property_resolution_error;
mod reflect_facade;
mod relation;
mod role;
mod type_metadata;
#[cfg(feature = "validation")]
pub mod validation;
#[cfg(feature = "validation")]
pub use crate::validation::ValidationBuildInputs;
#[cfg(feature = "validation")]
pub use crate::validation::ValidationPlan;
#[cfg(feature = "validation")]
pub use crate::validation::{FieldPath, ModelValidationError, ValidationMode, ValidationOptions, ValidationSelection};
pub use qubit_redact::Sensitivity;
pub use qubit_reflect::FieldDefinitionDescriptor;
pub use qubit_reflect::FieldDescriptor;
pub use qubit_reflect::FieldSetFailure;
pub use qubit_reflect::FieldSetRecovery;
pub use qubit_reflect::InvocationOutput;
pub use qubit_reflect::Reflect;
pub use qubit_reflect::ReflectRegistry;
pub use qubit_reflect::ReflectedMut;
pub use qubit_reflect::ReflectedOwned;
pub use qubit_reflect::ReflectedRef;
pub use qubit_reflect::TypeDefinitionDescriptor;
pub use qubit_reflect::TypeDefinitionId;
pub use qubit_reflect::TypeDescriptor;
pub use qubit_reflect::TypeMismatch;
pub use qubit_reflect::VariantDefinitionDescriptor;
pub use qubit_reflect::VariantDescriptor;
pub use qubit_reflect::descriptor::StructKind;
pub use qubit_reflect::descriptor::TypeKind;
pub use qubit_reflect::descriptor::TypeRef;
pub use qubit_reflect::expression::ConstExpression;
pub use qubit_reflect::expression::TypeExpression;
pub use qubit_reflect::identity::FragmentIdentity;
pub use qubit_reflect::identity::Visibility;
pub use qubit_reflect::register_reflected_type;
pub use qubit_validator::NamedValidationArgument;
pub use qubit_validator::ValidationArgument;

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
pub use crate::field_semantics::CodecMetadata;
pub use crate::field_semantics::CodecReference;
pub use crate::field_semantics::CodecSource;
pub use crate::field_semantics::DependencyBindingMetadata;
pub use crate::field_semantics::DeclaredEntityTarget;
pub use crate::field_semantics::DeclaredEntityTargetKind;
pub use crate::field_semantics::FieldAttributeMetadata;
pub use crate::field_semantics::IdentifierAssignment;
pub use crate::field_semantics::IdentifierMetadata;
pub use crate::field_semantics::IndexingReasons;
pub use crate::field_semantics::KeyPartMetadata;
pub use crate::field_semantics::RedactMetadata;
pub use crate::field_semantics::RedactModeMetadata;
pub use crate::field_semantics::RedactPosition;
pub use crate::field_semantics::ReferenceMetadata as FieldReferenceMetadata;
pub use crate::field_semantics::ReferenceSelection;
pub use crate::field_semantics::SelectorMetadata;
pub use crate::field_semantics::SelectorPosition;
pub use crate::field_semantics::SerdeBehaviorSource;
pub use crate::field_semantics::SerdeFieldMetadata;
pub use crate::field_semantics::OnNone;
pub use crate::field_semantics::TargetMode;
pub use crate::field_semantics::ValidationTarget;
pub use crate::field_semantics::UniqueMetadata as FieldUniqueMetadata;
pub use crate::field_semantics::ValidatorMetadata;
pub use crate::generic_model_metadata::GenericModelMetadata;
pub use crate::local_property_set::LocalPropertySet;
pub use crate::metadata_registry::ModelEntry;
pub use crate::metadata_registry::ModelRegistry;
pub use crate::metadata_registry::ModelRegistryError;
pub use crate::metadata_registry::ModelRegistryErrorKind;
pub use crate::metadata_resolver::ModelResolutionCause;
pub use crate::metadata_resolver::ModelResolveError;
pub use crate::metadata_resolver::ModelResolveErrorKind;
pub use crate::metadata_resolver::ModelResolveErrors;
pub use crate::metadata_resolver::ModelResolver;
pub use crate::metadata_resolver::ProjectionExecutionError;
pub use crate::metadata_resolver::QueryField;
pub use crate::metadata_resolver::QueryMetadata;
pub use crate::metadata_resolver::ResolveInputs;
pub use crate::metadata_resolver::ResolvedCodec;
pub use crate::metadata_resolver::ResolvedModelGraph;
pub use crate::metadata_resolver::ResolvedProjectionProducer;
pub use crate::metadata_resolver::ResolvedProjectionSource;
pub use crate::metadata_resolver::ResolvedReference;
pub use crate::metadata_resolver::UniqueQueryKey;
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
#[doc(hidden)]
pub use crate::reflect_facade::ModelImplProvider;
#[doc(hidden)]
pub use crate::reflect_facade::ModelMetadataProvider;
#[doc(hidden)]
pub use crate::reflect_facade::model_impl_key;
#[doc(hidden)]
pub use crate::reflect_facade::model_metadata_key;
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
