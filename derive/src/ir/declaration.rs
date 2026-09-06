// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Defines the source-located intermediate representation for declarations.

use syn::LitStr;
use syn::Type;

use super::MacroKind;
use crate::ir::Located;
// qubit-style: allow multiple-public-types
// The declaration IR is one private vocabulary shared by parsing,
// normalization, validation, and expansion.

/// Normalized declaration-level options shared by all role macros.
#[derive(Clone)]
pub(crate) struct DeclarationOptions {
    /// Optional stable model identifier.
    pub(crate) id: Option<LitStr>,
    /// Fixed projection source Rust type.
    pub(crate) source: Option<Type>,
    /// Fixed projection source identifier.
    pub(crate) source_id: Option<LitStr>,
    /// Whether a projection accepts an open source.
    pub(crate) open: bool,
    /// Whether a value uses transparent representation.
    pub(crate) transparent: bool,
    /// Disables generated `Clone`.
    pub(crate) no_clone: bool,
    /// Disables generated `Debug`.
    pub(crate) no_debug: bool,
    /// Disables generated `Display`.
    pub(crate) no_display: bool,
    /// Disables generated `PartialEq`.
    pub(crate) no_partial_eq: bool,
    /// Disables generated `Eq`.
    pub(crate) no_eq: bool,
    /// Disables generated `Hash`.
    pub(crate) no_hash: bool,
    /// Disables generated serialization.
    pub(crate) no_serialize: bool,
    /// Disables generated deserialization.
    pub(crate) no_deserialize: bool,
    /// Disables generated redaction.
    pub(crate) no_redact: bool,
    /// Disables generated `Copy`.
    pub(crate) no_copy: bool,
    /// Enables generated `Copy`.
    pub(crate) copy: bool,
    /// Enables generated `Default`.
    pub(crate) default: bool,
    /// Enables generated `PartialOrd`.
    pub(crate) partial_ord: bool,
    /// Enables generated `Ord`.
    pub(crate) ord: bool,
    /// Canonical value codec type.
    pub(crate) codec: Option<Type>,
}

/// A single field-level metadata attribute after parsing.
#[derive(Clone)]
pub(crate) enum FieldOccurrence {
    /// Identifier assignment metadata.
    Identifier(IdentifierAssignmentIr),
    /// Explicit database index marker.
    Indexed,
    /// Uniqueness metadata.
    Unique(UniqueIr),
    /// Relationship metadata.
    Reference(ReferenceIr),
    /// Composite-key position.
    KeyPart(usize),
    /// Validation constraint.
    Constraint(ConstraintIr),
    /// Collection selector metadata.
    Selector(SelectorIr),
    /// Validator metadata.
    Validator(ValidatorIr),
    /// Codec metadata.
    Codec(CodecIr),
    /// Redaction metadata.
    Redact(RedactIr),
    /// Serde metadata.
    Serde(SerdeIr),
    /// Opaque reflection marker.
    Opaque,
}

/// Selects the owner of an automatically assigned identifier.
#[derive(Clone, Copy)]
pub(crate) enum IdentifierAssignmentIr {
    /// Assigns the identifier in application code.
    Application,
    /// Assigns the identifier in the database.
    Database,
}

/// Represents one supported validation constraint.
#[derive(Clone)]
pub(crate) enum ConstraintIr {
    /// Text constraint.
    Text(TextConstraintIr),
    /// Decimal constraint.
    Decimal(DecimalConstraintIr),
    /// Time constraint format.
    Time(String),
    /// Sequence length and uniqueness constraint.
    Sequence {
        /// Minimum number of items.
        min: Option<usize>,
        /// Maximum number of items.
        max: Option<usize>,
        /// Whether items must be unique.
        unique: bool,
    },
    /// Map entry-count constraint.
    Map {
        /// Minimum number of entries.
        min: Option<usize>,
        /// Maximum number of entries.
        max: Option<usize>,
    },
}

/// Normalized text constraint parameters.
#[derive(Clone, Default)]
pub(crate) struct TextConstraintIr {
    /// Minimum character count.
    pub(crate) min_chars: Option<u32>,
    /// Maximum character count.
    pub(crate) max_chars: Option<u32>,
    /// Minimum UTF-8 byte count.
    pub(crate) min_bytes: Option<u32>,
    /// Maximum UTF-8 byte count.
    pub(crate) max_bytes: Option<u32>,
    /// Allowed character set name.
    pub(crate) allowed_chars: Option<String>,
    /// Whether blank text is rejected.
    pub(crate) non_blank: bool,
    /// Optional named text format.
    pub(crate) format: Option<String>,
}

/// Normalized decimal constraint parameters.
#[derive(Clone)]
pub(crate) struct DecimalConstraintIr {
    /// Optional total precision.
    pub(crate) precision: Option<u16>,
    /// Number of fractional digits.
    pub(crate) scale: u16,
    /// Named rounding mode.
    pub(crate) rounding: String,
    /// Whether the decimal represents money.
    pub(crate) money: bool,
    /// Inclusive or exclusive lower bound literal.
    pub(crate) min: Option<LitStr>,
    /// Inclusive or exclusive upper bound literal.
    pub(crate) max: Option<LitStr>,
    /// Whether the lower bound is inclusive.
    pub(crate) min_inclusive: bool,
    /// Whether the upper bound is inclusive.
    pub(crate) max_inclusive: bool,
}

/// Metadata for a selector applied to a collection element or map component.
#[derive(Clone)]
pub(crate) struct SelectorIr {
    /// Collection position selected by this rule.
    pub(crate) position: SelectorPositionIr,
    /// Nested constraints.
    pub(crate) constraints: Vec<ConstraintIr>,
    /// Nested validators.
    pub(crate) validators: Vec<ValidatorIr>,
    /// Nested value codec.
    pub(crate) codec: Option<CodecIr>,
    /// Nested redaction mode.
    pub(crate) redact: Option<RedactIr>,
}

/// Identifies the collection position targeted by a selector.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum SelectorPositionIr {
    /// Collection element.
    Element,
    /// Map key.
    MapKey,
    /// Map value.
    MapValue,
}

/// Normalized uniqueness declaration.
#[derive(Clone)]
pub(crate) struct UniqueIr {
    /// Field paths participating in uniqueness comparisons.
    pub(crate) respect_to: Vec<Vec<String>>,
    /// Whether comparisons ignore case.
    pub(crate) ignore_case: bool,
}

/// Identifies a reference target by Rust type or stable model ID.
#[derive(Clone)]
pub(crate) enum ReferenceTargetIr {
    /// Target Rust type.
    RustType(Box<Type>),
    /// Target stable model identifier.
    ModelId(LitStr),
}

/// Normalized relationship/reference declaration.
#[derive(Clone)]
pub(crate) struct ReferenceIr {
    /// Referenced model target.
    pub(crate) target: ReferenceTargetIr,
    /// Optional referenced property path.
    pub(crate) property: Option<Vec<String>>,
    /// Whether the referenced target must already exist.
    pub(crate) existing: bool,
    /// Optional path that must match the source.
    pub(crate) same_as: Option<Vec<String>>,
}

/// Literal argument accepted by a validator strategy.
#[derive(Clone)]
pub(crate) enum StrategyArgumentIr {
    /// Boolean literal.
    Bool(bool),
    /// Signed integer literal.
    Integer(i128),
    /// Unsigned integer literal.
    Unsigned(u128),
    /// String literal.
    String(LitStr),
    /// Boolean list.
    BoolList(Vec<bool>),
    /// Signed integer list.
    IntegerList(Vec<i128>),
    /// Unsigned integer list.
    UnsignedList(Vec<u128>),
    /// String list.
    StringList(Vec<LitStr>),
}

/// Normalized validator registration and its arguments.
#[derive(Clone)]
pub(crate) struct ValidatorIr {
    /// Stable validator registration identifier.
    pub(crate) id: LitStr,
    /// Named validator strategy parameters.
    pub(crate) params: Vec<(String, StrategyArgumentIr)>,
    /// Field paths the validator reads.
    pub(crate) depends_on: Vec<Vec<String>>,
    /// Named dependency slots and the field paths supplying them.
    pub(crate) dependency_bindings: Vec<(String, Vec<String>)>,
    /// Whether the validator receives the expanded value or its container.
    pub(crate) target: TargetModeIr,
    /// How an expanded optional value with no value is handled.
    pub(crate) on_none: OnNoneIr,
}

/// Selects the value shape supplied to a validator declaration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TargetModeIr {
    /// Expand supported optional and transparent smart-pointer wrappers.
    Value,
    /// Preserve the declared container type.
    Container,
}

impl Default for TargetModeIr {
    fn default() -> Self {
        Self::Value
    }
}

/// Selects the behavior for an absent expanded optional value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OnNoneIr {
    /// Skip the validator occurrence.
    Skip,
    /// Report the absent value as a required-value violation.
    Reject,
}

impl Default for OnNoneIr {
    fn default() -> Self {
        Self::Skip
    }
}

/// Value codec selected by declared ID or Rust type.
#[derive(Clone)]
pub(crate) enum CodecIr {
    /// Codec Rust type.
    RustType(Box<Type>),
    /// Codec registration identifier.
    DeclaredId(LitStr),
}

/// Redaction mode attached to a field or selector.
#[derive(Clone)]
pub(crate) struct RedactIr {
    /// Selected redaction behavior.
    pub(crate) mode: RedactModeIr,
}

/// Supported redaction shapes emitted in metadata.
#[derive(Clone)]
pub(crate) enum RedactModeIr {
    /// Named redaction level.
    Level(String),
    /// Skip redaction.
    Skip,
    /// Nested redaction.
    Nested,
    /// Map redaction.
    Map,
    /// Redact using another field.
    KeyedBy(String),
    /// JSON redaction.
    Json,
}

/// Normalized Serde behavior for one field.
#[derive(Clone, Default)]
pub(crate) struct SerdeIr {
    /// Serialized field name override.
    pub(crate) serialize_name: Option<LitStr>,
    /// Deserialized field name override.
    pub(crate) deserialize_name: Option<LitStr>,
    /// Skip this field while serializing.
    pub(crate) skip_serializing: bool,
    /// Skip this field while deserializing.
    pub(crate) skip_deserializing: bool,
    /// Flatten nested serialization.
    pub(crate) flatten: bool,
    /// Custom Serde module path.
    pub(crate) with: Option<LitStr>,
    /// Use the type's default during deserialization.
    pub(crate) default: bool,
    /// Whether skip-serializing-if was explicitly set.
    pub(crate) explicit_skip_serializing_if: bool,
    /// Use the model default source.
    pub(crate) default_from_model: bool,
    /// Omit this field from the model view.
    pub(crate) omit_from_model: bool,
    /// Whether omission was explicitly suppressed.
    pub(crate) omit_suppressed: bool,
}

/// Parsed field metadata and its source type.
#[derive(Clone)]
pub(crate) struct FieldIr {
    /// Zero-based source field index and its declaration span.
    pub(crate) index: Located<usize>,
    /// Rust field type.
    pub(crate) ty: Type,
    /// Parsed field-level attributes.
    pub(crate) occurrences: Vec<FieldOccurrence>,
    /// Preserve this field under model serialization.
    pub(crate) keep_serializing: bool,
    /// Whether the source field has a name.
    pub(crate) named: bool,
}

/// Parsed enum variant names and fields.
#[derive(Clone)]
pub(crate) struct VariantIr {
    /// Rust source variant name.
    pub(crate) rust_name: String,
    /// Canonical model variant name.
    pub(crate) canonical_name: String,
    /// Serialized variant name.
    pub(crate) serialized_name: String,
    /// Deserialized variant name.
    pub(crate) deserialized_name: String,
    /// Whether this variant is the default.
    pub(crate) default: bool,
    /// Parsed variant fields.
    pub(crate) fields: Vec<FieldIr>,
}

/// Complete normalized declaration consumed by the expansion stage.
pub(crate) struct DeclarationIr {
    /// Selected macro role.
    pub(crate) kind: MacroKind,
    /// Declaration-level options.
    pub(crate) options: DeclarationOptions,
    /// Struct fields.
    pub(crate) fields: Vec<FieldIr>,
    /// Enum variants.
    pub(crate) variants: Vec<VariantIr>,
}
