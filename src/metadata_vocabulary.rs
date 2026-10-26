// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Domain semantics attached to reflected model fields.

mod declared_entity_target;
mod declared_entity_target_kind;
mod dependency_binding_metadata;
mod field_attribute_metadata;
mod identifier_assignment;
mod identifier_metadata;
mod indexing_reasons;
mod key_part_metadata;
mod named_validation_argument;
mod on_none;
mod redact_metadata;
mod redact_mode_metadata;
mod redact_position;
mod reference_metadata;
mod reference_selection;
mod selector_metadata;
mod selector_position;
mod sensitivity;
mod serde_behavior_source;
mod serde_field_metadata;
mod target_mode;
mod unique_metadata;
mod validation_argument;
mod validator_metadata;

pub use declared_entity_target::DeclaredEntityTarget;
pub use declared_entity_target_kind::DeclaredEntityTargetKind;
pub use dependency_binding_metadata::DependencyBindingMetadata;
pub use field_attribute_metadata::FieldAttributeMetadata;
pub use identifier_assignment::IdentifierAssignment;
pub use identifier_metadata::IdentifierMetadata;
pub use indexing_reasons::IndexingReasons;
pub use key_part_metadata::KeyPartMetadata;
pub use named_validation_argument::NamedValidationArgument;
pub use on_none::OnNone;
pub use redact_metadata::RedactMetadata;
pub use redact_mode_metadata::RedactModeMetadata;
pub use redact_position::RedactPosition;
pub use reference_metadata::ReferenceMetadata;
pub use reference_selection::ReferenceSelection;
pub use selector_metadata::SelectorMetadata;
pub use selector_position::SelectorPosition;
pub use sensitivity::Sensitivity;
pub use serde_behavior_source::SerdeBehaviorSource;
pub use serde_field_metadata::SerdeFieldMetadata;
pub use target_mode::TargetMode;
pub use target_mode::ValidationTarget;
pub use unique_metadata::UniqueMetadata;
pub use validation_argument::ValidationArgument;
pub use validator_metadata::ValidatorMetadata;
