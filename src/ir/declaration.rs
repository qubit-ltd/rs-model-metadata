// =============================================================================

use syn::Data;
use syn::DeriveInput;
use syn::Error;
use syn::LitStr;
use syn::Meta;
use syn::Result;
use syn::Token;
use syn::Type;
use syn::punctuated::Punctuated;

use super::MacroKind;
use super::declaration_normalize::normalize_selector_containers;
use super::declaration_normalize::validate_declaration_ir;
use super::declaration_parse::parse_fields;
use super::declaration_parse::parse_variants;
use super::declaration_parse::validate_ascii_id;
use super::declaration_validate::combine;
use crate::ir::Located;
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// qubit-style: allow multiple-public-types
// The declaration IR is one private vocabulary shared by parsing,
// normalization, validation, and expansion.

/// Normalized declaration-level options shared by all role macros.
#[derive(Clone)]
pub(super) struct DeclarationOptions {
    /// Optional stable model identifier.
    pub(super) id: Option<LitStr>,
    /// Fixed projection source Rust type.
    pub(super) source: Option<Type>,
    /// Fixed projection source identifier.
    pub(super) source_id: Option<LitStr>,
    /// Whether a projection accepts an open source.
    pub(super) open: bool,
    /// Whether a value uses transparent representation.
    pub(super) transparent: bool,
    /// Disables generated `Clone`.
    pub(super) no_clone: bool,
    /// Disables generated `Debug`.
    pub(super) no_debug: bool,
    /// Disables generated `Display`.
    pub(super) no_display: bool,
    /// Disables generated `PartialEq`.
    pub(super) no_partial_eq: bool,
    /// Disables generated `Eq`.
    pub(super) no_eq: bool,
    /// Disables generated `Hash`.
    pub(super) no_hash: bool,
    /// Disables generated serialization.
    pub(super) no_serialize: bool,
    /// Disables generated deserialization.
    pub(super) no_deserialize: bool,
    /// Disables generated redaction.
    pub(super) no_redact: bool,
    /// Disables generated `Copy`.
    pub(super) no_copy: bool,
    /// Enables generated `Copy`.
    pub(super) copy: bool,
    /// Enables generated `Default`.
    pub(super) default: bool,
    /// Enables generated `PartialOrd`.
    pub(super) partial_ord: bool,
    /// Enables generated `Ord`.
    pub(super) ord: bool,
    /// Canonical value codec type.
    pub(super) codec: Option<Type>,
}

/// A single field-level metadata attribute after parsing.
#[derive(Clone)]
pub(super) enum FieldOccurrence {
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
pub(super) enum IdentifierAssignmentIr {
    Application,
    Database,
}

/// Represents one supported validation constraint.
#[derive(Clone)]
pub(super) enum ConstraintIr {
    /// Text constraint.
    Text(TextConstraintIr),
    /// Decimal constraint.
    Decimal(DecimalConstraintIr),
    /// Time constraint format.
    Time(String),
    Sequence {
        /// Minimum number of items.
        min: Option<usize>,
        /// Maximum number of items.
        max: Option<usize>,
        /// Whether items must be unique.
        unique: bool,
    },
    Map {
        /// Minimum number of entries.
        min: Option<usize>,
        /// Maximum number of entries.
        max: Option<usize>,
    },
}

/// Normalized text constraint parameters.
#[derive(Clone, Default)]
pub(super) struct TextConstraintIr {
    /// Minimum character count.
    pub(super) min_chars: Option<u32>,
    /// Maximum character count.
    pub(super) max_chars: Option<u32>,
    /// Minimum UTF-8 byte count.
    pub(super) min_bytes: Option<u32>,
    /// Maximum UTF-8 byte count.
    pub(super) max_bytes: Option<u32>,
    /// Allowed character set name.
    pub(super) allowed_chars: Option<String>,
    /// Whether blank text is rejected.
    pub(super) non_blank: bool,
    /// Optional named text format.
    pub(super) format: Option<String>,
}

/// Normalized decimal constraint parameters.
#[derive(Clone)]
pub(super) struct DecimalConstraintIr {
    /// Optional total precision.
    pub(super) precision: Option<u16>,
    /// Number of fractional digits.
    pub(super) scale: u16,
    /// Named rounding mode.
    pub(super) rounding: String,
    /// Whether the decimal represents money.
    pub(super) money: bool,
    /// Inclusive or exclusive lower bound literal.
    pub(super) min: Option<LitStr>,
    /// Inclusive or exclusive upper bound literal.
    pub(super) max: Option<LitStr>,
    /// Whether the lower bound is inclusive.
    pub(super) min_inclusive: bool,
    /// Whether the upper bound is inclusive.
    pub(super) max_inclusive: bool,
}

/// Metadata for a selector applied to a collection element or map component.
#[derive(Clone)]
pub(super) struct SelectorIr {
    /// Collection position selected by this rule.
    pub(super) position: SelectorPositionIr,
    /// Nested constraints.
    pub(super) constraints: Vec<ConstraintIr>,
    /// Nested validators.
    pub(super) validators: Vec<ValidatorIr>,
    /// Nested value codec.
    pub(super) codec: Option<CodecIr>,
    /// Nested redaction mode.
    pub(super) redact: Option<RedactIr>,
}

/// Identifies the collection position targeted by a selector.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum SelectorPositionIr {
    /// Collection element.
    Element,
    /// Map key.
    MapKey,
    /// Map value.
    MapValue,
}

/// Normalized uniqueness declaration.
#[derive(Clone)]
pub(super) struct UniqueIr {
    /// Field paths participating in uniqueness comparisons.
    pub(super) respect_to: Vec<Vec<String>>,
    /// Whether comparisons ignore case.
    pub(super) ignore_case: bool,
}

/// Identifies a reference target by Rust type or stable model ID.
#[derive(Clone)]
pub(super) enum ReferenceTargetIr {
    /// Target Rust type.
    RustType(Box<Type>),
    /// Target stable model identifier.
    ModelId(LitStr),
}

/// Normalized relationship/reference declaration.
#[derive(Clone)]
pub(super) struct ReferenceIr {
    /// Referenced model target.
    pub(super) target: ReferenceTargetIr,
    /// Optional referenced property path.
    pub(super) property: Option<Vec<String>>,
    /// Whether the referenced target must already exist.
    pub(super) existing: bool,
    /// Optional path that must match the source.
    pub(super) same_as: Option<Vec<String>>,
}

/// Literal argument accepted by a validator strategy.
#[derive(Clone)]
pub(super) enum StrategyArgumentIr {
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
pub(super) struct ValidatorIr {
    /// Stable validator registration identifier.
    pub(super) id: LitStr,
    /// Named validator strategy parameters.
    pub(super) params: Vec<(String, StrategyArgumentIr)>,
    /// Field paths the validator reads.
    pub(super) depends_on: Vec<Vec<String>>,
}

/// Value codec selected by declared ID or Rust type.
#[derive(Clone)]
pub(super) enum CodecIr {
    /// Codec Rust type.
    RustType(Box<Type>),
    /// Codec registration identifier.
    DeclaredId(LitStr),
}

/// Redaction mode attached to a field or selector.
#[derive(Clone)]
pub(super) struct RedactIr {
    /// Selected redaction behavior.
    pub(super) mode: RedactModeIr,
}

/// Supported redaction shapes emitted in metadata.
#[derive(Clone)]
pub(super) enum RedactModeIr {
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
pub(super) struct SerdeIr {
    /// Serialized field name override.
    pub(super) serialize_name: Option<LitStr>,
    /// Deserialized field name override.
    pub(super) deserialize_name: Option<LitStr>,
    /// Skip this field while serializing.
    pub(super) skip_serializing: bool,
    /// Skip this field while deserializing.
    pub(super) skip_deserializing: bool,
    /// Flatten nested serialization.
    pub(super) flatten: bool,
    /// Custom Serde module path.
    pub(super) with: Option<LitStr>,
    /// Use the type's default during deserialization.
    pub(super) default: bool,
    /// Whether skip-serializing-if was explicitly set.
    pub(super) explicit_skip_serializing_if: bool,
    /// Use the model default source.
    pub(super) default_from_model: bool,
    /// Omit this field from the model view.
    pub(super) omit_from_model: bool,
    /// Whether omission was explicitly suppressed.
    pub(super) omit_suppressed: bool,
}

/// Parsed field metadata and its source type.
#[derive(Clone)]
pub(super) struct FieldIr {
    /// Zero-based source field index and its declaration span.
    pub(super) index: Located<usize>,
    /// Rust field type.
    pub(super) ty: Type,
    /// Parsed field-level attributes.
    pub(super) occurrences: Vec<FieldOccurrence>,
    /// Preserve this field under model serialization.
    pub(super) keep_serializing: bool,
    /// Whether the source field has a name.
    pub(super) named: bool,
}

/// Parsed enum variant names and fields.
#[derive(Clone)]
pub(super) struct VariantIr {
    /// Rust source variant name.
    pub(super) rust_name: String,
    /// Canonical model variant name.
    pub(super) canonical_name: String,
    /// Serialized variant name.
    pub(super) serialized_name: String,
    /// Deserialized variant name.
    pub(super) deserialized_name: String,
    /// Whether this variant is the default.
    pub(super) default: bool,
    /// Parsed variant fields.
    pub(super) fields: Vec<FieldIr>,
}

/// Complete normalized declaration consumed by the expansion stage.
pub(super) struct DeclarationIr {
    /// Selected macro role.
    pub(super) kind: MacroKind,
    /// Declaration-level options.
    pub(super) options: DeclarationOptions,
    /// Struct fields.
    pub(super) fields: Vec<FieldIr>,
    /// Enum variants.
    pub(super) variants: Vec<VariantIr>,
}

impl DeclarationIr {
    /// Parses role options and fields, then validates role-specific invariants.
    pub(super) fn parse(kind: MacroKind, options: Punctuated<Meta, Token![,]>, item: &DeriveInput) -> Result<Self> {
        let mut errors = None;
        let options = match DeclarationOptions::parse(options) {
            Ok(options) => Some(options),
            Err(error) => {
                combine(&mut errors, error);
                None
            }
        };
        let (fields, variants) = match &item.data {
            Data::Struct(data) => match parse_fields(&data.fields) {
                Ok(fields) => (Some(fields), Some(Vec::new())),
                Err(error) => {
                    combine(&mut errors, error);
                    (None, None)
                }
            },
            Data::Enum(data) => match parse_variants(data) {
                Ok(variants) => (Some(Vec::new()), Some(variants)),
                Err(error) => {
                    combine(&mut errors, error);
                    (None, None)
                }
            },
            Data::Union(_) => {
                combine(
                    &mut errors,
                    Error::new_spanned(item, "model role macros do not support unions"),
                );
                (None, None)
            }
        };
        if let Some(error) = errors {
            return Err(error);
        }
        let options = options.expect("errors returned when declaration options are unavailable");
        let mut fields = fields.expect("errors returned when fields are unavailable");
        let mut variants = variants.expect("errors returned when variants are unavailable");
        if kind == MacroKind::Entity && options.id.is_none() {
            return Err(Error::new_spanned(&item.ident, "Entity requires `id = \"...\"`"));
        }
        if let Some(id) = &options.id {
            validate_ascii_id(id, "model ID")?;
        }
        if let Some(source_id) = &options.source_id {
            validate_ascii_id(source_id, "Projection source ID")?;
        }

        if matches!(kind, MacroKind::Entity | MacroKind::Projection)
            && fields
                .iter()
                .filter(|field| {
                    field
                        .occurrences
                        .iter()
                        .any(|value| matches!(value, FieldOccurrence::Identifier(_)))
                })
                .count()
                != 1
        {
            return Err(Error::new_spanned(
                &item.ident,
                "Entity and Projection require exactly one `#[identifier]` field",
            ));
        }
        for field in fields
            .iter_mut()
            .chain(variants.iter_mut().flat_map(|variant| &mut variant.fields))
        {
            normalize_selector_containers(field);
        }
        let result = Self {
            kind,
            options,
            fields,
            variants,
        };
        validate_declaration_ir(&result, item)?;
        Ok(result)
    }
}
