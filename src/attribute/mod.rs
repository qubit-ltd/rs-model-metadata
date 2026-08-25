// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Parsing for `#[model(...)]` attributes.

mod decimal_attribute;
mod element_attribute;
mod element_constraint_attribute;
mod field_attribute;
mod field_name;
mod field_unique_attribute;
mod identifier_attribute;
mod lookup_relation_attribute;
mod map_attribute;
mod model_attribute;
mod model_attributes;
mod named_fields_attribute;
mod ownership_attribute;
mod parse;
mod primary_key_attribute;
mod reference_attribute;
mod rounding_mode;
mod sequence_attribute;
mod spanned_value;
mod strategy_attribute;
mod temporal_attribute;
mod temporal_precision;
mod text_attribute;
mod text_format;
mod text_repertoire;

pub(crate) use decimal_attribute::DecimalAttribute;
pub(crate) use element_attribute::ElementAttribute;
pub(crate) use element_constraint_attribute::ElementConstraintAttribute;
pub(crate) use field_attribute::FieldAttribute;
pub(crate) use field_name::FieldName;
pub(crate) use lookup_relation_attribute::LookupRelationAttribute;
pub(crate) use map_attribute::MapAttribute;
pub(crate) use model_attribute::ModelAttribute;
pub(crate) use named_fields_attribute::NamedFieldsAttribute;
pub(crate) use parse::parse_field_attributes;
pub(crate) use parse::parse_model_attributes;
pub(crate) use reference_attribute::ReferenceAttribute;
pub(crate) use rounding_mode::RoundingMode;
pub(crate) use sequence_attribute::SequenceAttribute;
pub(crate) use spanned_value::SpannedValue;
pub(crate) use strategy_attribute::StrategyAttribute;
pub(crate) use temporal_attribute::TemporalAttribute;
pub(crate) use temporal_precision::TemporalPrecision;
pub(crate) use text_attribute::TextAttribute;
pub(crate) use text_format::TextFormat;
pub(crate) use text_repertoire::TextRepertoire;
