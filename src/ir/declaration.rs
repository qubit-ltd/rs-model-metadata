// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Normalized declaration-level options shared by all role macros.
#[derive(Clone)]
struct DeclarationOptions {
    /// Optional stable model identifier.
    id: Option<LitStr>,
    /// Fixed projection source Rust type.
    source: Option<Type>,
    /// Fixed projection source identifier.
    source_id: Option<LitStr>,
    /// Whether a projection accepts an open source.
    open: bool,
    /// Whether a value uses transparent representation.
    transparent: bool,
    /// Disables generated `Clone`.
    no_clone: bool,
    /// Disables generated `Debug`.
    no_debug: bool,
    /// Disables generated `Display`.
    no_display: bool,
    /// Disables generated `PartialEq`.
    no_partial_eq: bool,
    /// Disables generated `Eq`.
    no_eq: bool,
    /// Disables generated `Hash`.
    no_hash: bool,
    /// Disables generated serialization.
    no_serialize: bool,
    /// Disables generated deserialization.
    no_deserialize: bool,
    /// Disables generated redaction.
    no_redact: bool,
    /// Disables generated `Copy`.
    no_copy: bool,
    /// Enables generated `Copy`.
    copy: bool,
    /// Enables generated `Default`.
    default: bool,
    /// Enables generated `PartialOrd`.
    partial_ord: bool,
    /// Enables generated `Ord`.
    ord: bool,
    /// Canonical value codec type.
    codec: Option<Type>,
}

/// A single field-level metadata attribute after parsing.
#[derive(Clone)]
enum FieldOccurrence {
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
enum IdentifierAssignmentIr {
    Application,
    Database,
}

/// Represents one supported validation constraint.
#[derive(Clone)]
enum ConstraintIr {
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
struct TextConstraintIr {
    /// Minimum character count.
    min_chars: Option<u32>,
    /// Maximum character count.
    max_chars: Option<u32>,
    /// Minimum UTF-8 byte count.
    min_bytes: Option<u32>,
    /// Maximum UTF-8 byte count.
    max_bytes: Option<u32>,
    /// Allowed character set name.
    allowed_chars: Option<String>,
    /// Whether blank text is rejected.
    non_blank: bool,
    /// Optional named text format.
    format: Option<String>,
}

/// Normalized decimal constraint parameters.
#[derive(Clone)]
struct DecimalConstraintIr {
    /// Optional total precision.
    precision: Option<u16>,
    /// Number of fractional digits.
    scale: u16,
    /// Named rounding mode.
    rounding: String,
    /// Whether the decimal represents money.
    money: bool,
    /// Inclusive or exclusive lower bound literal.
    min: Option<LitStr>,
    /// Inclusive or exclusive upper bound literal.
    max: Option<LitStr>,
    /// Whether the lower bound is inclusive.
    min_inclusive: bool,
    /// Whether the upper bound is inclusive.
    max_inclusive: bool,
}

/// Metadata for a selector applied to a collection element or map component.
#[derive(Clone)]
struct SelectorIr {
    /// Collection position selected by this rule.
    position: SelectorPositionIr,
    /// Nested constraints.
    constraints: Vec<ConstraintIr>,
    /// Nested validators.
    validators: Vec<ValidatorIr>,
    /// Nested value codec.
    codec: Option<CodecIr>,
    /// Nested redaction mode.
    redact: Option<RedactIr>,
}

/// Identifies the collection position targeted by a selector.
#[derive(Clone, Copy, Eq, PartialEq)]
enum SelectorPositionIr {
    /// Collection element.
    Element,
    /// Map key.
    MapKey,
    /// Map value.
    MapValue,
}

/// Normalized uniqueness declaration.
#[derive(Clone)]
struct UniqueIr {
    /// Field paths participating in uniqueness comparisons.
    respect_to: Vec<Vec<String>>,
    /// Whether comparisons ignore case.
    ignore_case: bool,
}

/// Identifies a reference target by Rust type or stable model ID.
#[derive(Clone)]
enum ReferenceTargetIr {
    /// Target Rust type.
    RustType(Box<Type>),
    /// Target stable model identifier.
    ModelId(LitStr),
}

/// Normalized relationship/reference declaration.
#[derive(Clone)]
struct ReferenceIr {
    /// Referenced model target.
    target: ReferenceTargetIr,
    /// Optional referenced property path.
    property: Option<Vec<String>>,
    /// Whether the referenced target must already exist.
    existing: bool,
    /// Optional path that must match the source.
    same_as: Option<Vec<String>>,
}

/// Literal argument accepted by a validator strategy.
#[derive(Clone)]
enum StrategyArgumentIr {
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
struct ValidatorIr {
    /// Stable validator registration identifier.
    id: LitStr,
    /// Named validator strategy parameters.
    params: Vec<(String, StrategyArgumentIr)>,
    /// Field paths the validator reads.
    depends_on: Vec<Vec<String>>,
}

/// Value codec selected by declared ID or Rust type.
#[derive(Clone)]
enum CodecIr {
    /// Codec Rust type.
    RustType(Box<Type>),
    /// Codec registration identifier.
    DeclaredId(LitStr),
}

/// Redaction mode attached to a field or selector.
#[derive(Clone)]
struct RedactIr {
    /// Selected redaction behavior.
    mode: RedactModeIr,
}

/// Supported redaction shapes emitted in metadata.
#[derive(Clone)]
enum RedactModeIr {
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
struct SerdeIr {
    /// Serialized field name override.
    serialize_name: Option<LitStr>,
    /// Deserialized field name override.
    deserialize_name: Option<LitStr>,
    /// Skip this field while serializing.
    skip_serializing: bool,
    /// Skip this field while deserializing.
    skip_deserializing: bool,
    /// Flatten nested serialization.
    flatten: bool,
    /// Custom Serde module path.
    with: Option<LitStr>,
    /// Use the type's default during deserialization.
    default: bool,
    /// Whether skip-serializing-if was explicitly set.
    explicit_skip_serializing_if: bool,
    /// Use the model default source.
    default_from_model: bool,
    /// Omit this field from the model view.
    omit_from_model: bool,
    /// Whether omission was explicitly suppressed.
    omit_suppressed: bool,
}

/// Parsed field metadata and its source type.
#[derive(Clone)]
struct FieldIr {
    /// Zero-based source field index.
    index: usize,
    /// Rust field type.
    ty: Type,
    /// Span of the declared field type for field-local diagnostics.
    span: proc_macro2::Span,
    /// Parsed field-level attributes.
    occurrences: Vec<FieldOccurrence>,
    /// Preserve this field under model serialization.
    keep_serializing: bool,
    /// Whether the source field has a name.
    named: bool,
}

/// Parsed enum variant names and fields.
#[derive(Clone)]
struct VariantIr {
    /// Rust source variant name.
    rust_name: String,
    /// Canonical model variant name.
    canonical_name: String,
    /// Serialized variant name.
    serialized_name: String,
    /// Deserialized variant name.
    deserialized_name: String,
    /// Whether this variant is the default.
    default: bool,
    /// Parsed variant fields.
    fields: Vec<FieldIr>,
}

/// Complete normalized declaration consumed by the expansion stage.
struct DeclarationIr {
    /// Selected macro role.
    kind: MacroKind,
    /// Declaration-level options.
    options: DeclarationOptions,
    /// Struct fields.
    fields: Vec<FieldIr>,
    /// Enum variants.
    variants: Vec<VariantIr>,
}

impl DeclarationIr {
    /// Parses role options and fields, then validates role-specific invariants.
    fn parse(
        kind: MacroKind,
        options: Punctuated<Meta, Token![,]>,
        item: &DeriveInput,
    ) -> Result<Self> {
        let options = DeclarationOptions::parse(options)?;
        if kind == MacroKind::Entity && options.id.is_none() {
            return Err(Error::new_spanned(
                &item.ident,
                "Entity requires `id = \"...\"`",
            ));
        }
        if let Some(id) = &options.id {
            validate_ascii_id(id, "model ID")?;
        }
        if let Some(source_id) = &options.source_id {
            validate_ascii_id(source_id, "Projection source ID")?;
        }

        let (mut fields, mut variants) = match &item.data {
            Data::Struct(data) => (parse_fields(&data.fields)?, Vec::new()),
            Data::Enum(data) => (Vec::new(), parse_variants(data)?),
            Data::Union(_) => {
                return Err(Error::new_spanned(
                    item,
                    "model role macros do not support unions",
                ));
            }
        };
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
