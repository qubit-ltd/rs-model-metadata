// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Compile-time constraints and constructors shared by generated code.

mod borrowed_property_output;
mod decimal_constraint_target;
mod generated_helpers;
mod generated_type_metadata_builder;
mod identifier_type;
mod identifier_type_seal;
mod map_constraint_target;
mod nested_type_metadata_probe;
mod optional_borrowed_property_output;
mod property_output_compatible;
mod sequence_constraint_target;
mod temporal_constraint_target;
mod text_constraint_target;
mod unique_items_constraint_target;
mod variable_length_sequence_target;

pub use borrowed_property_output::BorrowedPropertyOutput;
pub use decimal_constraint_target::DecimalConstraintTarget;
pub use generated_helpers::local_property_set;
pub use generated_helpers::model_impl_metadata;
pub use generated_helpers::property_fragment;
pub use generated_type_metadata_builder::GeneratedTypeMetadataBuilder;
pub use identifier_type::IdentifierType;
pub use map_constraint_target::MapConstraintTarget;
pub use nested_type_metadata_probe::NestedTypeMetadataProbe;
pub use optional_borrowed_property_output::OptionalBorrowedPropertyOutput;
pub use property_output_compatible::PropertyOutputCompatible;
pub use sequence_constraint_target::SequenceConstraintTarget;
pub use temporal_constraint_target::TemporalConstraintTarget;
pub use text_constraint_target::TextConstraintTarget;
pub use unique_items_constraint_target::UniqueItemsConstraintTarget;
pub use variable_length_sequence_target::VariableLengthSequenceTarget;
