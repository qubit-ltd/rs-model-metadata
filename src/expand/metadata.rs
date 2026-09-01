// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Compiles non-property model declarations into reflection and metadata
//! tokens.

use heck::ToShoutySnakeCase;
use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::Attribute;
use syn::Data;
use syn::DeriveInput;
use syn::Error;
use syn::Expr;
use syn::ExprLit;
use syn::Fields;
use syn::GenericParam;
use syn::ItemImpl;
use syn::Lit;
use syn::LitStr;
use syn::Meta;
use syn::Result;
use syn::Token;
use syn::Type;
use syn::parse::Parser;
use syn::parse_quote;
use syn::parse2;
use syn::punctuated::Punctuated;

use crate::expand::properties::expand_properties as expand_property_impl;
use crate::expand::properties::validate_property_impl as validate_property_impl_shape;
use crate::ir::MacroKind;
use crate::runtime_path::runtime_path;

/// Expands one declaration and converts all failures to compiler diagnostics.
pub(crate) fn expand(kind: MacroKind, args: TokenStream, input: TokenStream) -> TokenStream {
    expand_result(kind, args, input).unwrap_or_else(Error::into_compile_error)
}

fn expand_result(kind: MacroKind, args: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let raw_options = Punctuated::<Meta, Token![,]>::parse_terminated.parse2(args)?;
    if kind == MacroKind::ModelProperties {
        let item: ItemImpl = parse2(input)?;
        validate_property_impl_shape(&item)?;
        let runtime = runtime_path()?;
        return expand_property_impl(item, &runtime);
    }

    let mut item: DeriveInput = parse2(input)?;
    let runtime = runtime_path();
    if let Err(mut validation) = validate_declaration(kind, &item) {
        if let Err(runtime_error) = runtime {
            validation.combine(runtime_error);
        }
        return Err(validation);
    }
    let runtime = runtime?;
    reject_duplicate_reflect(&item.attrs)?;
    let mut declaration = DeclarationIr::parse(kind, raw_options, &item)?;
    apply_default_derives(&declaration, &mut item, &runtime)?;
    apply_serde_defaults(&mut declaration, &mut item, &runtime);
    rewrite_field_helpers(&mut item.data, &declaration);
    item.attrs.push(parse_quote!(#[derive(#runtime::Reflect)]));
    item.attrs.push(parse_quote!(#[reflect(crate = #runtime)]));
    item.attrs
        .push(parse_quote!(#[reflect(capabilities(#runtime::__private::v1::model_capability))]));
    let display = expand_display(&declaration, &item, &runtime);
    let metadata = expand_metadata(&declaration, &item, &runtime);
    Ok(quote!(#item #display #metadata))
}

#[derive(Clone)]
struct DeclarationOptions {
    id: Option<LitStr>,
    source: Option<Type>,
    source_id: Option<LitStr>,
    open: bool,
    transparent: bool,
    no_clone: bool,
    no_debug: bool,
    no_display: bool,
    no_partial_eq: bool,
    no_eq: bool,
    no_hash: bool,
    no_serialize: bool,
    no_deserialize: bool,
    no_redact: bool,
    no_copy: bool,
    copy: bool,
    default: bool,
    partial_ord: bool,
    ord: bool,
    codec: Option<Type>,
}

#[derive(Clone)]
enum FieldOccurrence {
    Identifier(IdentifierAssignmentIr),
    Indexed,
    Unique(UniqueIr),
    Reference(ReferenceIr),
    KeyPart(usize),
    Constraint(ConstraintIr),
    Selector(SelectorIr),
    Validator(ValidatorIr),
    Codec(CodecIr),
    Redact(RedactIr),
    Serde(SerdeIr),
    Opaque,
}

#[derive(Clone, Copy)]
enum IdentifierAssignmentIr {
    Application,
    Database,
}

#[derive(Clone)]
enum ConstraintIr {
    Text(TextConstraintIr),
    Decimal(DecimalConstraintIr),
    Time(String),
    Sequence {
        min: Option<usize>,
        max: Option<usize>,
        unique: bool,
    },
    Map {
        min: Option<usize>,
        max: Option<usize>,
    },
}

#[derive(Clone, Default)]
struct TextConstraintIr {
    min_chars: Option<u32>,
    max_chars: Option<u32>,
    min_bytes: Option<u32>,
    max_bytes: Option<u32>,
    allowed_chars: Option<String>,
    non_blank: bool,
    format: Option<String>,
}

#[derive(Clone)]
struct DecimalConstraintIr {
    precision: Option<u16>,
    scale: u16,
    rounding: String,
    money: bool,
    min: Option<LitStr>,
    max: Option<LitStr>,
    min_inclusive: bool,
    max_inclusive: bool,
}

#[derive(Clone)]
struct SelectorIr {
    position: SelectorPositionIr,
    constraints: Vec<ConstraintIr>,
    validators: Vec<ValidatorIr>,
    codec: Option<CodecIr>,
    redact: Option<RedactIr>,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum SelectorPositionIr {
    Element,
    MapKey,
    MapValue,
}

#[derive(Clone)]
struct UniqueIr {
    respect_to: Vec<Vec<String>>,
    ignore_case: bool,
}

#[derive(Clone)]
enum ReferenceTargetIr {
    RustType(Box<Type>),
    ModelId(LitStr),
}

#[derive(Clone)]
struct ReferenceIr {
    target: ReferenceTargetIr,
    property: Option<Vec<String>>,
    existing: bool,
    same_as: Option<Vec<String>>,
}

#[derive(Clone)]
enum StrategyArgumentIr {
    Bool(bool),
    Integer(i128),
    Unsigned(u128),
    String(LitStr),
    BoolList(Vec<bool>),
    IntegerList(Vec<i128>),
    UnsignedList(Vec<u128>),
    StringList(Vec<LitStr>),
}

#[derive(Clone)]
struct ValidatorIr {
    id: LitStr,
    params: Vec<(String, StrategyArgumentIr)>,
    depends_on: Vec<Vec<String>>,
}

#[derive(Clone)]
enum CodecIr {
    RustType(Box<Type>),
    DeclaredId(LitStr),
}

#[derive(Clone)]
struct RedactIr {
    mode: RedactModeIr,
}

#[derive(Clone)]
enum RedactModeIr {
    Level(String),
    Skip,
    Nested,
    Map,
    KeyedBy(String),
    Json,
}

#[derive(Clone, Default)]
struct SerdeIr {
    serialize_name: Option<LitStr>,
    deserialize_name: Option<LitStr>,
    skip_serializing: bool,
    skip_deserializing: bool,
    flatten: bool,
    with: Option<LitStr>,
    default: bool,
    explicit_skip_serializing_if: bool,
    default_from_model: bool,
    omit_from_model: bool,
    omit_suppressed: bool,
}

#[derive(Clone)]
struct FieldIr {
    index: usize,
    ty: Type,
    occurrences: Vec<FieldOccurrence>,
    keep_serializing: bool,
    named: bool,
}

#[derive(Clone)]
struct VariantIr {
    rust_name: String,
    canonical_name: String,
    serialized_name: String,
    deserialized_name: String,
    default: bool,
    fields: Vec<FieldIr>,
}

struct DeclarationIr {
    kind: MacroKind,
    options: DeclarationOptions,
    fields: Vec<FieldIr>,
    variants: Vec<VariantIr>,
}

impl DeclarationIr {
    fn parse(kind: MacroKind, options: Punctuated<Meta, Token![,]>, item: &DeriveInput) -> Result<Self> {
        let options = DeclarationOptions::parse(options)?;
        if kind == MacroKind::Entity && options.id.is_none() {
            return Err(Error::new_spanned(&item.ident, "Entity requires `id = \"...\"`"));
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
            Data::Union(_) => unreachable!("unions are rejected before IR construction"),
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

fn parse_fields(fields: &Fields) -> Result<Vec<FieldIr>> {
    let mut parsed = Vec::new();
    let mut errors = None;
    for (index, field) in fields.iter().enumerate() {
        match FieldIr::parse(index, &field.ty, &field.attrs, field.ident.is_some()) {
            Ok(field) => parsed.push(field),
            Err(error) => combine(&mut errors, error),
        }
    }
    match errors {
        Some(error) => Err(error),
        None => Ok(parsed),
    }
}

fn parse_variants(data: &syn::DataEnum) -> Result<Vec<VariantIr>> {
    let mut parsed = Vec::new();
    let mut errors = None;
    for variant in &data.variants {
        let canonical_name = variant.ident.to_string().to_shouty_snake_case();
        let names = parse_variant_serde_names(&variant.attrs, &canonical_name);
        let fields = parse_fields(&variant.fields);
        match (names, fields) {
            (Ok((serialized_name, deserialized_name)), Ok(fields)) => parsed.push(VariantIr {
                rust_name: variant.ident.to_string(),
                canonical_name,
                serialized_name,
                deserialized_name,
                default: variant
                    .attrs
                    .iter()
                    .any(|attribute| attribute.path().is_ident("default")),
                fields,
            }),
            (names, fields) => {
                if let Err(error) = names {
                    combine(&mut errors, error);
                }
                if let Err(error) = fields {
                    combine(&mut errors, error);
                }
            }
        }
    }
    match errors {
        Some(error) => Err(error),
        None => Ok(parsed),
    }
}

fn validate_declaration_ir(declaration: &DeclarationIr, item: &DeriveInput) -> Result<()> {
    let mut errors = None;
    let options = &declaration.options;
    if options.source_id.is_some() && declaration.kind != MacroKind::Projection {
        combine(
            &mut errors,
            Error::new_spanned(&item.ident, "source_id is only valid for Projection"),
        );
    }
    if options.source.is_some() && declaration.kind != MacroKind::Projection {
        combine(
            &mut errors,
            Error::new_spanned(&item.ident, "source is only valid for Projection"),
        );
    }
    if options.source.is_some() && options.source_id.is_some() {
        combine(
            &mut errors,
            Error::new_spanned(&item.ident, "Projection accepts only one of `source` or `source_id`"),
        );
    }
    if options.open && (options.source.is_some() || options.source_id.is_some()) {
        combine(
            &mut errors,
            Error::new_spanned(&item.ident, "open Projection cannot declare a fixed source"),
        );
    }
    if options.open && declaration.kind != MacroKind::Projection {
        combine(
            &mut errors,
            Error::new_spanned(&item.ident, "open is only valid for Projection"),
        );
    }
    if options.transparent && declaration.kind != MacroKind::Value {
        combine(
            &mut errors,
            Error::new_spanned(&item.ident, "transparent is only valid for Value"),
        );
    }
    if options.codec.is_some() && declaration.kind != MacroKind::Value {
        combine(
            &mut errors,
            Error::new_spanned(&item.ident, "canonical codec is only valid for Value"),
        );
    }
    if options.transparent && declaration.fields.len() != 1 {
        combine(
            &mut errors,
            Error::new_spanned(&item.ident, "transparent Value requires exactly one field"),
        );
    }
    if options.no_copy && declaration.kind != MacroKind::Enum {
        combine(
            &mut errors,
            Error::new_spanned(&item.ident, "no_copy is only valid for Enum"),
        );
    }
    if options.no_copy && options.copy {
        combine(
            &mut errors,
            Error::new_spanned(&item.ident, "copy and no_copy cannot be combined"),
        );
    }
    let all_fields: Vec<_> = declaration
        .fields
        .iter()
        .chain(declaration.variants.iter().flat_map(|variant| &variant.fields))
        .collect();
    if options.no_redact
        && all_fields.iter().any(|field| {
            field.occurrences.iter().any(|value| {
                matches!(
                    value,
                    FieldOccurrence::Redact(_) | FieldOccurrence::Selector(SelectorIr { redact: Some(_), .. })
                )
            })
        })
    {
        combine(
            &mut errors,
            Error::new_spanned(&item.ident, "no_redact cannot be combined with field redaction rules"),
        );
    }
    for field in &all_fields {
        let has_identifier = field
            .occurrences
            .iter()
            .any(|value| matches!(value, FieldOccurrence::Identifier(_)));
        let has_reference = field
            .occurrences
            .iter()
            .any(|value| matches!(value, FieldOccurrence::Reference(_)));
        if has_identifier && !matches!(declaration.kind, MacroKind::Entity | MacroKind::Projection) {
            combine(
                &mut errors,
                Error::new_spanned(&item.ident, "identifier is only valid for Entity and Projection"),
            );
        }
        if has_reference && matches!(declaration.kind, MacroKind::Enum | MacroKind::Value) {
            combine(
                &mut errors,
                Error::new_spanned(&item.ident, "Enum and Value fields cannot declare references"),
            );
        }
        let has_implicit_index = field.occurrences.iter().any(|value| {
            matches!(
                value,
                FieldOccurrence::Identifier(_) | FieldOccurrence::Unique(_) | FieldOccurrence::Reference(_)
            )
        });
        if has_implicit_index
            && field
                .occurrences
                .iter()
                .any(|value| matches!(value, FieldOccurrence::Indexed))
        {
            combine(
                &mut errors,
                Error::new_spanned(
                    &item.ident,
                    "explicit indexed is redundant with identifier, unique, or reference",
                ),
            );
        }
        if field.keep_serializing && (!field.named || omission_kind(&field.ty).is_none()) {
            combine(
                &mut errors,
                Error::new_spanned(
                    &item.ident,
                    "keep_serializing requires a named Option or standard collection field",
                ),
            );
        }
        let predicates: [fn(&FieldOccurrence) -> bool; 6] = [
            |value: &FieldOccurrence| matches!(value, FieldOccurrence::Unique(_)),
            |value: &FieldOccurrence| matches!(value, FieldOccurrence::Reference(_)),
            |value: &FieldOccurrence| matches!(value, FieldOccurrence::KeyPart(_)),
            |value: &FieldOccurrence| matches!(value, FieldOccurrence::Codec(_)),
            |value: &FieldOccurrence| matches!(value, FieldOccurrence::Redact(_)),
            |value: &FieldOccurrence| matches!(value, FieldOccurrence::Serde(_)),
        ];
        for predicate in predicates {
            if field.occurrences.iter().filter(|value| predicate(value)).count() > 1 {
                combine(
                    &mut errors,
                    Error::new_spanned(&item.ident, "duplicate singleton field declaration"),
                );
            }
        }
        let opaque = field
            .occurrences
            .iter()
            .any(|value| matches!(value, FieldOccurrence::Opaque));
        if opaque
            && field
                .occurrences
                .iter()
                .any(|value| matches!(value, FieldOccurrence::Identifier(_) | FieldOccurrence::Reference(_)))
        {
            combine(
                &mut errors,
                Error::new_spanned(&item.ident, "opaque cannot be combined with identifier or reference"),
            );
        }
        for position in [
            SelectorPositionIr::Element,
            SelectorPositionIr::MapKey,
            SelectorPositionIr::MapValue,
        ] {
            if field
                .occurrences
                .iter()
                .filter(|value| matches!(value, FieldOccurrence::Selector(selector) if selector.position == position))
                .count()
                > 1
            {
                combine(
                    &mut errors,
                    Error::new_spanned(&item.ident, "duplicate selector position"),
                );
            }
        }
        let field_redact = field
            .occurrences
            .iter()
            .any(|value| matches!(value, FieldOccurrence::Redact(_)));
        let selector_redact = field
            .occurrences
            .iter()
            .any(|value| matches!(value, FieldOccurrence::Selector(SelectorIr { redact: Some(_), .. })));
        if field_redact && selector_redact {
            combine(
                &mut errors,
                Error::new_spanned(&item.ident, "field and selector redaction cannot overlap"),
            );
        }
        for selector in field.occurrences.iter().filter_map(|value| match value {
            FieldOccurrence::Selector(selector) => Some(selector),
            _ => None,
        }) {
            if let Some(redact) = &selector.redact
                && !matches!(redact.mode, RedactModeIr::Level(_))
            {
                combine(
                    &mut errors,
                    Error::new_spanned(
                        &item.ident,
                        "selector redaction currently supports only `redact(level = \"...\")`; `skip` is forbidden",
                    ),
                );
            }
        }
        validate_field_constraints(field, &item.ident, &mut errors);
    }
    let mut orders: Vec<_> = declaration
        .fields
        .iter()
        .filter_map(|field| {
            field.occurrences.iter().find_map(|value| match value {
                FieldOccurrence::KeyPart(order) => Some(*order),
                _ => None,
            })
        })
        .collect();
    orders.sort_unstable();
    if orders
        .iter()
        .copied()
        .enumerate()
        .any(|(expected, actual)| expected != actual)
    {
        combine(
            &mut errors,
            Error::new_spanned(&item.ident, "key_part orders must be unique and contiguous from zero"),
        );
    }
    if let Some(error) = errors { Err(error) } else { Ok(()) }
}

fn normalize_selector_containers(field: &mut FieldIr) {
    let has_element = field.occurrences.iter().any(|value| {
        matches!(
            value,
            FieldOccurrence::Selector(SelectorIr {
                position: SelectorPositionIr::Element,
                ..
            })
        )
    });
    let has_map_selector = field.occurrences.iter().any(|value| {
        matches!(
            value,
            FieldOccurrence::Selector(SelectorIr {
                position: SelectorPositionIr::MapKey | SelectorPositionIr::MapValue,
                ..
            })
        )
    });
    if has_element
        && !field
            .occurrences
            .iter()
            .any(|value| matches!(value, FieldOccurrence::Constraint(ConstraintIr::Sequence { .. })))
    {
        field
            .occurrences
            .push(FieldOccurrence::Constraint(ConstraintIr::Sequence {
                min: None,
                max: None,
                unique: false,
            }));
    }
    if has_map_selector
        && !field
            .occurrences
            .iter()
            .any(|value| matches!(value, FieldOccurrence::Constraint(ConstraintIr::Map { .. })))
    {
        field
            .occurrences
            .push(FieldOccurrence::Constraint(ConstraintIr::Map { min: None, max: None }));
    }
}

fn validate_field_constraints(field: &FieldIr, span: &syn::Ident, errors: &mut Option<Error>) {
    let mut kinds = std::collections::HashSet::new();
    for constraint in field.occurrences.iter().filter_map(|value| match value {
        FieldOccurrence::Constraint(value) => Some(value),
        _ => None,
    }) {
        let kind = match constraint {
            ConstraintIr::Text(_) => "text",
            ConstraintIr::Decimal(value) if value.money => "decimal-or-money",
            ConstraintIr::Decimal(_) => "decimal-or-money",
            ConstraintIr::Time(_) => "time",
            ConstraintIr::Sequence { .. } => "sequence",
            ConstraintIr::Map { .. } => "map",
        };
        if !kinds.insert(kind) {
            combine(errors, Error::new_spanned(span, format!("duplicate {kind} constraint")));
        }
    }
    for selector in field.occurrences.iter().filter_map(|value| match value {
        FieldOccurrence::Selector(value) => Some(value),
        _ => None,
    }) {
        let mut selector_kinds = std::collections::HashSet::new();
        for constraint in &selector.constraints {
            let kind = match constraint {
                ConstraintIr::Text(_) => "text",
                ConstraintIr::Decimal(_) => "decimal-or-money",
                ConstraintIr::Time(_) => "time",
                ConstraintIr::Sequence { .. } | ConstraintIr::Map { .. } => "container",
            };
            if !selector_kinds.insert(kind) {
                combine(
                    errors,
                    Error::new_spanned(span, format!("duplicate selector {kind} constraint")),
                );
            }
        }
    }

    let base = transparent_type_name(&field.ty);
    let has_element = field.occurrences.iter().any(|value| {
        matches!(
            value,
            FieldOccurrence::Selector(SelectorIr {
                position: SelectorPositionIr::Element,
                ..
            })
        )
    });
    let has_map = field.occurrences.iter().any(|value| {
        matches!(
            value,
            FieldOccurrence::Selector(SelectorIr {
                position: SelectorPositionIr::MapKey | SelectorPositionIr::MapValue,
                ..
            })
        )
    });
    if has_element && !is_sequence_type(&field.ty) {
        combine(
            errors,
            Error::new_spanned(span, "element selector requires a sequence, set, or array field"),
        );
    }
    if has_map && !matches!(base.as_deref(), Some("HashMap" | "BTreeMap")) {
        combine(
            errors,
            Error::new_spanned(span, "map_key and map_value selectors require a map field"),
        );
    }
    for constraint in field.occurrences.iter().filter_map(|value| match value {
        FieldOccurrence::Constraint(value) => Some(value),
        _ => None,
    }) {
        match constraint {
            ConstraintIr::Text(_) if base.as_deref().is_some_and(is_known_non_text_type) => {
                combine(
                    errors,
                    Error::new_spanned(span, "text constraint requires a text-capable field"),
                );
            }
            ConstraintIr::Decimal(_) if matches!(base.as_deref(), Some("f32" | "f64")) => {
                combine(
                    errors,
                    Error::new_spanned(span, "decimal constraints reject floating-point fields"),
                );
            }
            ConstraintIr::Sequence { min, max, unique } => {
                if !is_sequence_type(&field.ty) {
                    combine(
                        errors,
                        Error::new_spanned(span, "sequence constraint requires a sequence, set, or array field"),
                    );
                }
                if min.zip(*max).is_some_and(|(min, max)| min > max) {
                    combine(
                        errors,
                        Error::new_spanned(span, "sequence min_items cannot exceed max_items"),
                    );
                }
                if matches!(base.as_deref(), Some("HashSet" | "BTreeSet")) && *unique {
                    combine(
                        errors,
                        Error::new_spanned(span, "unique_items is redundant for set fields"),
                    );
                }
                if matches!(field.ty, Type::Array(_)) && (min.is_some() || max.is_some()) {
                    combine(
                        errors,
                        Error::new_spanned(span, "fixed arrays cannot declare min_items or max_items"),
                    );
                }
            }
            ConstraintIr::Map { min, max } => {
                if !matches!(base.as_deref(), Some("HashMap" | "BTreeMap")) {
                    combine(errors, Error::new_spanned(span, "map constraint requires a map field"));
                }
                if min.zip(*max).is_some_and(|(min, max)| min > max) {
                    combine(
                        errors,
                        Error::new_spanned(span, "map min_entries cannot exceed max_entries"),
                    );
                }
            }
            _ => {}
        }
    }
}

fn transparent_type_name(ty: &Type) -> Option<String> {
    let Type::Path(path) = ty else { return None };
    let segment = path.path.segments.last()?;
    if matches!(segment.ident.to_string().as_str(), "Option" | "Box" | "Rc" | "Arc") {
        let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
            return None;
        };
        return arguments.args.iter().find_map(|argument| match argument {
            syn::GenericArgument::Type(ty) => transparent_type_name(ty),
            _ => None,
        });
    }
    Some(segment.ident.to_string())
}

fn is_sequence_type(ty: &Type) -> bool {
    matches!(ty, Type::Array(_) | Type::Slice(_))
        || matches!(
            transparent_type_name(ty).as_deref(),
            Some("Vec" | "VecDeque" | "LinkedList" | "BinaryHeap" | "HashSet" | "BTreeSet")
        )
}

fn is_known_non_text_type(name: &str) -> bool {
    matches!(
        name,
        "bool"
            | "char"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "isize"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
            | "f32"
            | "f64"
            | "Vec"
            | "VecDeque"
            | "LinkedList"
            | "BinaryHeap"
            | "HashSet"
            | "BTreeSet"
            | "HashMap"
            | "BTreeMap"
    )
}

fn parse_variant_serde_names(attributes: &[Attribute], canonical: &str) -> Result<(String, String)> {
    let mut serialize = canonical.to_owned();
    let mut deserialize = canonical.to_owned();
    for attribute in attributes.iter().filter(|attribute| attribute.path().is_ident("serde")) {
        let value = parse_serde(attribute)?;
        if let Some(name) = value.serialize_name {
            serialize = name.value();
        }
        if let Some(name) = value.deserialize_name {
            deserialize = name.value();
        }
    }
    Ok((serialize, deserialize))
}

impl DeclarationOptions {
    fn parse(options: Punctuated<Meta, Token![,]>) -> Result<Self> {
        let mut result = Self {
            id: None,
            source: None,
            source_id: None,
            open: false,
            transparent: false,
            no_clone: false,
            no_debug: false,
            no_display: false,
            no_partial_eq: false,
            no_eq: false,
            no_hash: false,
            no_serialize: false,
            no_deserialize: false,
            no_redact: false,
            no_copy: false,
            copy: false,
            default: false,
            partial_ord: false,
            ord: false,
            codec: None,
        };
        for option in options {
            match option {
                Meta::NameValue(value) if value.path.is_ident("id") => {
                    set_lit_str(&mut result.id, value.value, "id")?;
                }
                Meta::NameValue(value) if value.path.is_ident("source_id") => {
                    set_lit_str(&mut result.source_id, value.value, "source_id")?;
                }
                Meta::NameValue(value) if value.path.is_ident("source") => {
                    if result.source.is_some() {
                        return Err(Error::new_spanned(value, "duplicate `source` option"));
                    }
                    let expression = value.value;
                    result.source = Some(syn::parse2(quote!(#expression))?);
                }
                Meta::NameValue(value) if value.path.is_ident("codec") => {
                    if result.codec.is_some() {
                        return Err(Error::new_spanned(value, "duplicate `codec` option"));
                    }
                    let expression = value.value;
                    result.codec = Some(syn::parse2(quote!(#expression))?);
                }
                Meta::Path(path) if path.is_ident("open") => result.open = true,
                Meta::Path(path) if path.is_ident("transparent") => result.transparent = true,
                Meta::Path(path) if path.is_ident("no_clone") => result.no_clone = true,
                Meta::Path(path) if path.is_ident("no_debug") => result.no_debug = true,
                Meta::Path(path) if path.is_ident("no_display") => result.no_display = true,
                Meta::Path(path) if path.is_ident("no_partial_eq") => result.no_partial_eq = true,
                Meta::Path(path) if path.is_ident("no_eq") => result.no_eq = true,
                Meta::Path(path) if path.is_ident("no_hash") => result.no_hash = true,
                Meta::Path(path) if path.is_ident("no_serialize") => result.no_serialize = true,
                Meta::Path(path) if path.is_ident("no_deserialize") => result.no_deserialize = true,
                Meta::Path(path) if path.is_ident("no_redact") => result.no_redact = true,
                Meta::Path(path) if path.is_ident("no_copy") => result.no_copy = true,
                Meta::Path(path) if path.is_ident("copy") => result.copy = true,
                Meta::Path(path) if path.is_ident("default") => result.default = true,
                Meta::Path(path) if path.is_ident("partial_ord") => result.partial_ord = true,
                Meta::Path(path) if path.is_ident("ord") => result.ord = true,
                Meta::Path(path) if path.is_ident("redact") => {}
                other => return Err(Error::new_spanned(other, "unsupported model option")),
            }
        }
        Ok(result)
    }
}

impl FieldIr {
    fn parse(index: usize, ty: &Type, attributes: &[Attribute], named: bool) -> Result<Self> {
        let mut occurrences = Vec::new();
        let mut keep_serializing = false;
        for attribute in attributes {
            if attribute.path().is_ident("identifier") {
                occurrences.push(FieldOccurrence::Identifier(parse_identifier(attribute)?));
            } else if attribute.path().is_ident("indexed") {
                occurrences.push(FieldOccurrence::Indexed);
            } else if attribute.path().is_ident("unique") {
                occurrences.push(FieldOccurrence::Unique(parse_unique(attribute)?));
            } else if attribute.path().is_ident("reference") {
                occurrences.push(FieldOccurrence::Reference(parse_reference(attribute)?));
            } else if attribute.path().is_ident("key_part") {
                occurrences.push(FieldOccurrence::KeyPart(parse_key_part(attribute)?));
            } else if is_constraint_attribute(attribute) {
                occurrences.push(FieldOccurrence::Constraint(parse_constraint(attribute)?));
            } else if attribute.path().is_ident("element") {
                occurrences.push(FieldOccurrence::Selector(parse_selector(
                    attribute,
                    SelectorPositionIr::Element,
                )?));
            } else if attribute.path().is_ident("map_key") {
                occurrences.push(FieldOccurrence::Selector(parse_selector(
                    attribute,
                    SelectorPositionIr::MapKey,
                )?));
            } else if attribute.path().is_ident("map_value") {
                occurrences.push(FieldOccurrence::Selector(parse_selector(
                    attribute,
                    SelectorPositionIr::MapValue,
                )?));
            } else if attribute.path().is_ident("validator") {
                occurrences.push(FieldOccurrence::Validator(parse_validator(attribute)?));
            } else if attribute.path().is_ident("codec") {
                occurrences.push(FieldOccurrence::Codec(parse_codec(attribute)?));
            } else if attribute.path().is_ident("redact") {
                occurrences.push(FieldOccurrence::Redact(parse_redact(attribute)?));
            } else if attribute.path().is_ident("serde") {
                occurrences.push(FieldOccurrence::Serde(parse_serde(attribute)?));
            } else if attribute.path().is_ident("opaque") {
                occurrences.push(FieldOccurrence::Opaque);
            } else if attribute.path().is_ident("keep_serializing") {
                if !matches!(attribute.meta, Meta::Path(_)) {
                    return Err(Error::new_spanned(
                        attribute,
                        "keep_serializing is a marker without arguments",
                    ));
                }
                if keep_serializing {
                    return Err(Error::new_spanned(attribute, "duplicate keep_serializing marker"));
                }
                keep_serializing = true;
            }
        }
        Ok(Self {
            index,
            ty: ty.clone(),
            occurrences,
            keep_serializing,
            named,
        })
    }
}

fn parse_identifier(attribute: &Attribute) -> Result<IdentifierAssignmentIr> {
    if matches!(attribute.meta, Meta::Path(_)) {
        return Ok(IdentifierAssignmentIr::Application);
    }
    let mut assignment = None;
    attribute.parse_nested_meta(|meta| {
        if !meta.path.is_ident("assigned_by") {
            return Err(meta.error("unsupported identifier option"));
        }
        let value = parse_ident_value(meta.value()?.parse()?)?;
        assignment = Some(match value.as_str() {
            "application" => IdentifierAssignmentIr::Application,
            "database" => IdentifierAssignmentIr::Database,
            _ => return Err(meta.error("assigned_by must be application or database")),
        });
        Ok(())
    })?;
    assignment.ok_or_else(|| Error::new_spanned(attribute, "identifier requires assigned_by"))
}

fn set_lit_str(slot: &mut Option<LitStr>, value: Expr, name: &str) -> Result<()> {
    if slot.is_some() {
        return Err(Error::new_spanned(value, format!("duplicate `{name}` option")));
    }
    let Expr::Lit(ExprLit {
        lit: Lit::Str(value), ..
    }) = value
    else {
        return Err(Error::new_spanned(value, format!("`{name}` requires a string literal")));
    };
    *slot = Some(value);
    Ok(())
}

fn parse_unique(attribute: &Attribute) -> Result<UniqueIr> {
    let mut value = UniqueIr {
        respect_to: Vec::new(),
        ignore_case: true,
    };
    if matches!(attribute.meta, Meta::Path(_)) {
        return Ok(value);
    }
    attribute.parse_nested_meta(|meta| {
        if meta.path.is_ident("respect_to") {
            meta.parse_nested_meta(|path| {
                value.respect_to.push(path_from_syn(&path.path));
                Ok(())
            })
        } else if meta.path.is_ident("ignore_case") {
            value.ignore_case = meta.value()?.parse::<syn::LitBool>()?.value;
            Ok(())
        } else {
            Err(meta.error("unsupported unique option"))
        }
    })?;
    Ok(value)
}

fn parse_reference(attribute: &Attribute) -> Result<ReferenceIr> {
    let mut target = None;
    let mut property = None;
    let mut existing = true;
    let mut same_as = None;
    attribute.parse_nested_meta(|meta| {
        if meta.path.is_ident("entity") {
            let ty: Type = meta.value()?.parse()?;
            if target.replace(ReferenceTargetIr::RustType(Box::new(ty))).is_some() {
                return Err(meta.error("reference requires exactly one entity target"));
            }
            Ok(())
        } else if meta.path.is_ident("entity_id") {
            let id: LitStr = meta.value()?.parse()?;
            validate_ascii_id(&id, "reference entity ID")?;
            if target.replace(ReferenceTargetIr::ModelId(id)).is_some() {
                return Err(meta.error("reference requires exactly one entity target"));
            }
            Ok(())
        } else if meta.path.is_ident("property") {
            property = Some(parse_path_value(meta.value()?.parse()?)?);
            Ok(())
        } else if meta.path.is_ident("path") || meta.path.is_ident("same_as") {
            same_as = Some(parse_path_value(meta.value()?.parse()?)?);
            Ok(())
        } else if meta.path.is_ident("existing") {
            existing = meta.value()?.parse::<syn::LitBool>()?.value;
            Ok(())
        } else {
            Err(meta.error("unsupported reference option"))
        }
    })?;
    let target = target.ok_or_else(|| Error::new_spanned(attribute, "reference requires `entity` or `entity_id`"))?;
    Ok(ReferenceIr {
        target,
        property,
        existing,
        same_as,
    })
}

fn parse_key_part(attribute: &Attribute) -> Result<usize> {
    let mut order = None;
    attribute.parse_nested_meta(|meta| {
        if meta.path.is_ident("order") {
            let value = meta.value()?.parse::<syn::LitInt>()?.base10_parse()?;
            if order.replace(value).is_some() {
                return Err(meta.error("duplicate key_part order"));
            }
            Ok(())
        } else {
            Err(meta.error("unsupported key_part option"))
        }
    })?;
    order.ok_or_else(|| Error::new_spanned(attribute, "key_part requires `order = n`"))
}

fn is_constraint_attribute(attribute: &Attribute) -> bool {
    attribute.path().is_ident("text")
        || attribute.path().is_ident("decimal")
        || attribute.path().is_ident("money")
        || attribute.path().is_ident("time")
        || attribute.path().is_ident("sequence")
        || attribute.path().is_ident("map")
}

fn parse_constraint(attribute: &Attribute) -> Result<ConstraintIr> {
    if attribute.path().is_ident("text") {
        return parse_text_constraint(attribute).map(ConstraintIr::Text);
    }
    if attribute.path().is_ident("decimal") {
        return parse_decimal_constraint(attribute, false).map(ConstraintIr::Decimal);
    }
    if attribute.path().is_ident("money") {
        return parse_decimal_constraint(attribute, true).map(ConstraintIr::Decimal);
    }
    if attribute.path().is_ident("time") {
        let mut precision = None;
        attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("precision") {
                precision = Some(parse_ident_value(meta.value()?.parse()?)?);
                Ok(())
            } else {
                Err(meta.error("unsupported time option"))
            }
        })?;
        return Ok(ConstraintIr::Time(
            precision.ok_or_else(|| Error::new_spanned(attribute, "time requires precision"))?,
        ));
    }
    if attribute.path().is_ident("sequence") {
        let (mut min, mut max, mut unique) = (None, None, false);
        let mut any = false;
        attribute.parse_nested_meta(|meta| {
            any = true;
            if meta.path.is_ident("min_items") {
                min = Some(meta.value()?.parse::<syn::LitInt>()?.base10_parse()?);
                Ok(())
            } else if meta.path.is_ident("max_items") {
                max = Some(meta.value()?.parse::<syn::LitInt>()?.base10_parse()?);
                Ok(())
            } else if meta.path.is_ident("unique_items") {
                unique = true;
                Ok(())
            } else {
                Err(meta.error("unsupported sequence option"))
            }
        })?;
        if !any {
            return Err(Error::new_spanned(attribute, "sequence requires at least one option"));
        }
        return Ok(ConstraintIr::Sequence { min, max, unique });
    }
    let (mut min, mut max) = (None, None);
    let mut any = false;
    attribute.parse_nested_meta(|meta| {
        any = true;
        if meta.path.is_ident("min_entries") {
            min = Some(meta.value()?.parse::<syn::LitInt>()?.base10_parse()?);
            Ok(())
        } else if meta.path.is_ident("max_entries") {
            max = Some(meta.value()?.parse::<syn::LitInt>()?.base10_parse()?);
            Ok(())
        } else {
            Err(meta.error("unsupported map option"))
        }
    })?;
    if !any {
        return Err(Error::new_spanned(attribute, "map requires min_entries or max_entries"));
    }
    Ok(ConstraintIr::Map { min, max })
}

fn parse_text_constraint(attribute: &Attribute) -> Result<TextConstraintIr> {
    let mut value = TextConstraintIr::default();
    let mut any = false;
    attribute.parse_nested_meta(|meta| {
        any = true;
        if meta.path.is_ident("min_chars") {
            value.min_chars = Some(meta.value()?.parse::<syn::LitInt>()?.base10_parse()?);
            Ok(())
        } else if meta.path.is_ident("max_chars") {
            value.max_chars = Some(meta.value()?.parse::<syn::LitInt>()?.base10_parse()?);
            Ok(())
        } else if meta.path.is_ident("min_bytes") {
            value.min_bytes = Some(meta.value()?.parse::<syn::LitInt>()?.base10_parse()?);
            Ok(())
        } else if meta.path.is_ident("max_bytes") {
            value.max_bytes = Some(meta.value()?.parse::<syn::LitInt>()?.base10_parse()?);
            Ok(())
        } else if meta.path.is_ident("non_blank") {
            value.non_blank = true;
            Ok(())
        } else if meta.path.is_ident("allowed_chars") {
            value.allowed_chars = Some(parse_ident_value(meta.value()?.parse()?)?);
            Ok(())
        } else if meta.path.is_ident("format") {
            value.format = Some(parse_ident_value(meta.value()?.parse()?)?);
            Ok(())
        } else {
            Err(meta.error("unsupported text option"))
        }
    })?;
    if !any {
        return Err(Error::new_spanned(attribute, "text requires at least one option"));
    }
    Ok(value)
}

fn parse_decimal_constraint(attribute: &Attribute, money: bool) -> Result<DecimalConstraintIr> {
    let mut precision = None;
    let mut scale = None;
    let mut rounding = None;
    let mut min: Option<LitStr> = None;
    let mut max: Option<LitStr> = None;
    let mut min_inclusive = true;
    let mut max_inclusive = true;
    let mut any = false;
    attribute.parse_nested_meta(|meta| {
        any = true;
        if meta.path.is_ident("precision") {
            precision = Some(meta.value()?.parse::<syn::LitInt>()?.base10_parse()?);
            Ok(())
        } else if meta.path.is_ident("scale") {
            scale = Some(meta.value()?.parse::<syn::LitInt>()?.base10_parse()?);
            Ok(())
        } else if meta.path.is_ident("rounding") {
            rounding = Some(parse_ident_value(meta.value()?.parse()?)?);
            Ok(())
        } else if meta.path.is_ident("min") {
            min = Some(meta.value()?.parse()?);
            Ok(())
        } else if meta.path.is_ident("max") {
            max = Some(meta.value()?.parse()?);
            Ok(())
        } else if meta.path.is_ident("min_inclusive") {
            min_inclusive = meta.value()?.parse::<syn::LitBool>()?.value;
            Ok(())
        } else if meta.path.is_ident("max_inclusive") {
            max_inclusive = meta.value()?.parse::<syn::LitBool>()?.value;
            Ok(())
        } else {
            Err(meta.error("unsupported decimal option"))
        }
    })?;
    if !any {
        return Err(Error::new_spanned(
            attribute,
            "decimal and money require at least one option",
        ));
    }
    if money && scale.is_none() {
        return Err(Error::new_spanned(attribute, "money requires scale"));
    }
    if precision.is_some_and(|precision| scale.is_some_and(|scale| scale > precision)) {
        return Err(Error::new_spanned(attribute, "decimal scale cannot exceed precision"));
    }
    if let (Some(minimum), Some(maximum)) = (&min, &max) {
        match compare_decimal_literals(&minimum.value(), &maximum.value()) {
            Some(core::cmp::Ordering::Greater) => {
                return Err(Error::new_spanned(attribute, "decimal min cannot exceed max"));
            }
            Some(core::cmp::Ordering::Equal) if !min_inclusive && !max_inclusive => {
                return Err(Error::new_spanned(
                    attribute,
                    "equal decimal bounds cannot both be exclusive",
                ));
            }
            Some(_) => {}
            None => {
                return Err(Error::new_spanned(
                    attribute,
                    "decimal bounds require canonical decimal strings",
                ));
            }
        }
    } else {
        for bound in min.iter().chain(max.iter()) {
            if parse_decimal_literal(&bound.value()).is_none() {
                return Err(Error::new_spanned(
                    bound,
                    "decimal bounds require canonical decimal strings",
                ));
            }
        }
    }
    Ok(DecimalConstraintIr {
        precision,
        scale: scale.unwrap_or(0),
        rounding: rounding.unwrap_or_else(|| {
            if money {
                "unnecessary".into()
            } else {
                "half_even".into()
            }
        }),
        money,
        min,
        max,
        min_inclusive,
        max_inclusive,
    })
}

fn parse_decimal_literal(value: &str) -> Option<(bool, String, usize)> {
    let (negative, unsigned) = value.strip_prefix('-').map_or((false, value), |value| (true, value));
    if unsigned.is_empty() || unsigned.starts_with('+') {
        return None;
    }
    let mut parts = unsigned.split('.');
    let integer = parts.next()?;
    let fraction = parts.next().unwrap_or("");
    if parts.next().is_some()
        || integer.is_empty()
        || !integer.bytes().all(|byte| byte.is_ascii_digit())
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    let integer = integer.trim_start_matches('0');
    let integer = if integer.is_empty() { "0" } else { integer };
    let fraction = fraction.trim_end_matches('0');
    let digits = format!("{integer}{fraction}");
    let digits = digits.trim_start_matches('0').to_owned();
    let normalized = if digits.is_empty() { "0".to_owned() } else { digits };
    let scale = fraction.len();
    Some((negative && normalized != "0", normalized, scale))
}

fn compare_decimal_literals(left: &str, right: &str) -> Option<core::cmp::Ordering> {
    let (left_negative, mut left_digits, left_scale) = parse_decimal_literal(left)?;
    let (right_negative, mut right_digits, right_scale) = parse_decimal_literal(right)?;
    let scale = left_scale.max(right_scale);
    left_digits.extend(std::iter::repeat_n('0', scale - left_scale));
    right_digits.extend(std::iter::repeat_n('0', scale - right_scale));
    let magnitude = left_digits
        .len()
        .cmp(&right_digits.len())
        .then_with(|| left_digits.cmp(&right_digits));
    Some(match (left_negative, right_negative) {
        (true, false) => core::cmp::Ordering::Less,
        (false, true) => core::cmp::Ordering::Greater,
        (true, true) => magnitude.reverse(),
        (false, false) => magnitude,
    })
}

fn parse_selector(attribute: &Attribute, position: SelectorPositionIr) -> Result<SelectorIr> {
    let Meta::List(list) = &attribute.meta else {
        return Err(Error::new_spanned(attribute, "selector requires nested declarations"));
    };
    let values = Punctuated::<Meta, Token![,]>::parse_terminated.parse2(list.tokens.clone())?;
    let mut selector = SelectorIr {
        position,
        constraints: Vec::new(),
        validators: Vec::new(),
        codec: None,
        redact: None,
    };
    for value in values {
        let nested: Attribute = parse_quote!(#[#value]);
        if is_constraint_attribute(&nested) {
            if matches!(
                nested.path().get_ident().map(ToString::to_string).as_deref(),
                Some("sequence" | "map")
            ) {
                return Err(Error::new_spanned(
                    nested,
                    "selectors cannot recursively contain collection selectors",
                ));
            }
            selector.constraints.push(parse_constraint(&nested)?);
        } else if nested.path().is_ident("validator") {
            selector.validators.push(parse_validator(&nested)?);
        } else if nested.path().is_ident("codec") {
            if selector.codec.replace(parse_codec(&nested)?).is_some() {
                return Err(Error::new_spanned(nested, "selector accepts one codec"));
            }
        } else if nested.path().is_ident("redact") {
            if selector.redact.replace(parse_redact(&nested)?).is_some() {
                return Err(Error::new_spanned(nested, "selector accepts one redact declaration"));
            }
        } else {
            return Err(Error::new_spanned(nested, "unsupported selector declaration"));
        }
    }
    Ok(selector)
}

fn parse_ident_value(expression: Expr) -> Result<String> {
    match expression {
        Expr::Path(path) if path.path.segments.len() == 1 => Ok(path.path.segments[0].ident.to_string()),
        other => Err(Error::new_spanned(other, "expected an identifier value")),
    }
}

fn parse_validator(attribute: &Attribute) -> Result<ValidatorIr> {
    let mut id = None;
    let mut params = Vec::new();
    let mut depends_on = Vec::new();
    attribute.parse_nested_meta(|meta| {
        if meta.path.is_ident("id") {
            let value: LitStr = meta.value()?.parse()?;
            if id.replace(value).is_some() {
                return Err(meta.error("duplicate validator ID"));
            }
            Ok(())
        } else if meta.path.is_ident("depends_on") {
            meta.parse_nested_meta(|path| {
                depends_on.push(path_from_syn(&path.path));
                Ok(())
            })
        } else if meta.path.is_ident("params") {
            meta.parse_nested_meta(|parameter| {
                let name = parameter
                    .path
                    .get_ident()
                    .ok_or_else(|| parameter.error("validator parameter name must be an identifier"))?
                    .to_string();
                let expression: Expr = parameter.value()?.parse()?;
                params.push((name, parse_strategy_argument(expression)?));
                Ok(())
            })
        } else {
            Err(meta.error("unsupported validator option"))
        }
    })?;
    let id = id.ok_or_else(|| Error::new_spanned(attribute, "validator requires `id = \"...\"`"))?;
    validate_ascii_id(&id, "validator ID")?;
    Ok(ValidatorIr { id, params, depends_on })
}

fn parse_codec(attribute: &Attribute) -> Result<CodecIr> {
    if let Ok(ty) = attribute.parse_args::<Type>()
        && !matches!(&ty, Type::Path(path) if path.path.is_ident("id"))
    {
        return Ok(CodecIr::RustType(Box::new(ty)));
    }
    let mut result = None;
    attribute.parse_nested_meta(|meta| {
        let value = if meta.path.is_ident("id") {
            let id: LitStr = meta.value()?.parse()?;
            validate_ascii_id(&id, "codec ID")?;
            CodecIr::DeclaredId(id)
        } else if meta.path.is_ident("type") {
            CodecIr::RustType(Box::new(meta.value()?.parse()?))
        } else {
            return Err(meta.error("unsupported codec option"));
        };
        if result.replace(value).is_some() {
            return Err(meta.error("codec accepts one reference"));
        }
        Ok(())
    })?;
    result.ok_or_else(|| Error::new_spanned(attribute, "codec requires a Rust type or `id = \"...\"`"))
}

fn parse_redact(attribute: &Attribute) -> Result<RedactIr> {
    let mut mode = None;
    attribute.parse_nested_meta(|meta| {
        let current = if meta.path.is_ident("level") {
            RedactModeIr::Level(meta.value()?.parse::<LitStr>()?.value())
        } else if meta.path.is_ident("skip") {
            RedactModeIr::Skip
        } else if meta.path.is_ident("nested") {
            RedactModeIr::Nested
        } else if meta.path.is_ident("map") {
            RedactModeIr::Map
        } else if meta.path.is_ident("keyed_by") {
            let expression: Expr = meta.value()?.parse()?;
            RedactModeIr::KeyedBy(path_text(expression)?)
        } else if meta.path.is_ident("json") {
            RedactModeIr::Json
        } else {
            return Err(meta.error("unsupported redact mode"));
        };
        if mode.replace(current).is_some() {
            return Err(meta.error("redact requires exactly one mode"));
        }
        Ok(())
    })?;
    Ok(RedactIr {
        mode: mode.ok_or_else(|| Error::new_spanned(attribute, "redact requires one mode"))?,
    })
}

fn parse_serde(attribute: &Attribute) -> Result<SerdeIr> {
    let mut serde = SerdeIr::default();
    attribute.parse_nested_meta(|meta| {
        if meta.path.is_ident("rename") {
            if meta.input.peek(Token![=]) {
                let value: LitStr = meta.value()?.parse()?;
                serde.serialize_name = Some(value.clone());
                serde.deserialize_name = Some(value);
                Ok(())
            } else {
                meta.parse_nested_meta(|direction| {
                    let value: LitStr = direction.value()?.parse()?;
                    if direction.path.is_ident("serialize") {
                        serde.serialize_name = Some(value);
                    } else if direction.path.is_ident("deserialize") {
                        serde.deserialize_name = Some(value);
                    } else {
                        return Err(direction.error("unsupported serde rename direction"));
                    }
                    Ok(())
                })
            }
        } else if meta.path.is_ident("skip") {
            serde.skip_serializing = true;
            serde.skip_deserializing = true;
            Ok(())
        } else if meta.path.is_ident("skip_serializing") {
            serde.skip_serializing = true;
            Ok(())
        } else if meta.path.is_ident("skip_deserializing") {
            serde.skip_deserializing = true;
            Ok(())
        } else if meta.path.is_ident("flatten") {
            serde.flatten = true;
            Ok(())
        } else if meta.path.is_ident("with") {
            serde.with = Some(meta.value()?.parse()?);
            Ok(())
        } else if meta.path.is_ident("default") {
            serde.default = true;
            if meta.input.peek(Token![=]) {
                let _: Expr = meta.value()?.parse()?;
            }
            Ok(())
        } else if meta.path.is_ident("skip_serializing_if") {
            serde.explicit_skip_serializing_if = true;
            let _: LitStr = meta.value()?.parse()?;
            Ok(())
        } else {
            // Serde owns its wider syntax; metadata records only the stable subset.
            if meta.input.peek(Token![=]) {
                let _: Expr = meta.value()?.parse()?;
            }
            Ok(())
        }
    })?;
    Ok(serde)
}

fn parse_strategy_argument(expression: Expr) -> Result<StrategyArgumentIr> {
    match expression {
        Expr::Lit(ExprLit {
            lit: Lit::Bool(value), ..
        }) => Ok(StrategyArgumentIr::Bool(value.value)),
        Expr::Lit(ExprLit {
            lit: Lit::Int(value), ..
        }) => {
            let text = value.base10_digits();
            if text.starts_with('-') {
                Ok(StrategyArgumentIr::Integer(value.base10_parse()?))
            } else {
                Ok(StrategyArgumentIr::Unsigned(value.base10_parse()?))
            }
        }
        Expr::Lit(ExprLit {
            lit: Lit::Str(value), ..
        }) => Ok(StrategyArgumentIr::String(value)),
        Expr::Unary(unary) if matches!(unary.op, syn::UnOp::Neg(_)) => {
            let Expr::Lit(ExprLit {
                lit: Lit::Int(value), ..
            }) = *unary.expr
            else {
                return Err(Error::new_spanned(
                    unary,
                    "negative validator parameters require an integer literal",
                ));
            };
            Ok(StrategyArgumentIr::Integer(-value.base10_parse::<i128>()?))
        }
        Expr::Array(array) => parse_strategy_array(array.elems.into_iter().collect()),
        other => Err(Error::new_spanned(
            other,
            "validator params support bool, integer, string, and homogeneous arrays",
        )),
    }
}

fn parse_strategy_array(values: Vec<Expr>) -> Result<StrategyArgumentIr> {
    if values.is_empty() {
        return Ok(StrategyArgumentIr::StringList(Vec::new()));
    }
    if values
        .iter()
        .all(|value| matches!(value, Expr::Lit(ExprLit { lit: Lit::Bool(_), .. })))
    {
        return Ok(StrategyArgumentIr::BoolList(
            values
                .into_iter()
                .map(|value| match value {
                    Expr::Lit(ExprLit {
                        lit: Lit::Bool(value), ..
                    }) => value.value,
                    _ => unreachable!(),
                })
                .collect(),
        ));
    }
    if values.iter().all(|value| strategy_signed_integer(value).is_some())
        && values.iter().any(|value| matches!(value, Expr::Unary(_)))
    {
        return Ok(StrategyArgumentIr::IntegerList(
            values
                .iter()
                .map(|value| strategy_signed_integer(value).expect("validated signed integer"))
                .collect(),
        ));
    }
    if values
        .iter()
        .all(|value| matches!(value, Expr::Lit(ExprLit { lit: Lit::Int(_), .. })))
    {
        return Ok(StrategyArgumentIr::UnsignedList(
            values
                .into_iter()
                .map(|value| match value {
                    Expr::Lit(ExprLit {
                        lit: Lit::Int(value), ..
                    }) => value.base10_parse().expect("validated integer"),
                    _ => unreachable!(),
                })
                .collect(),
        ));
    }
    if values
        .iter()
        .all(|value| matches!(value, Expr::Lit(ExprLit { lit: Lit::Str(_), .. })))
    {
        return Ok(StrategyArgumentIr::StringList(
            values
                .into_iter()
                .map(|value| match value {
                    Expr::Lit(ExprLit {
                        lit: Lit::Str(value), ..
                    }) => value,
                    _ => unreachable!(),
                })
                .collect(),
        ));
    }
    Err(Error::new(
        proc_macro2::Span::call_site(),
        "validator parameter arrays must be homogeneous",
    ))
}

fn strategy_signed_integer(value: &Expr) -> Option<i128> {
    match value {
        Expr::Lit(ExprLit {
            lit: Lit::Int(value), ..
        }) => value.base10_parse().ok(),
        Expr::Unary(unary) if matches!(unary.op, syn::UnOp::Neg(_)) => match unary.expr.as_ref() {
            Expr::Lit(ExprLit {
                lit: Lit::Int(value), ..
            }) => value.base10_parse::<i128>().ok().map(|value| -value),
            _ => None,
        },
        _ => None,
    }
}

fn parse_path_value(expression: Expr) -> Result<Vec<String>> {
    match expression {
        Expr::Path(path) => Ok(path_from_syn(&path.path)),
        Expr::Lit(ExprLit {
            lit: Lit::Str(value), ..
        }) => Ok(value.value().split('.').map(str::to_owned).collect()),
        other => Err(Error::new_spanned(other, "expected an identifier path or string path")),
    }
}

fn path_text(expression: Expr) -> Result<String> {
    parse_path_value(expression).map(|segments| segments.join("."))
}

fn path_from_syn(path: &syn::Path) -> Vec<String> {
    path.segments.iter().map(|segment| segment.ident.to_string()).collect()
}

fn validate_ascii_id(value: &LitStr, kind: &str) -> Result<()> {
    let text = value.value();
    let valid = !text.is_empty()
        && text.split('.').all(|segment| {
            let mut bytes = segment.bytes();
            bytes.next().is_some_and(|byte| byte.is_ascii_alphabetic())
                && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        });
    if valid {
        Ok(())
    } else {
        Err(Error::new(value.span(), format!("invalid {kind}")))
    }
}

fn apply_default_derives(declaration: &DeclarationIr, item: &mut DeriveInput, runtime: &TokenStream) -> Result<()> {
    let options = &declaration.options;
    if options.copy && options.no_clone {
        return Err(Error::new_spanned(
            &item.ident,
            "`copy` requires Clone; remove `no_clone`",
        ));
    }
    if options.ord && (options.no_partial_eq || options.no_eq) {
        return Err(Error::new_spanned(&item.ident, "`ord` requires PartialEq and Eq"));
    }
    let existing = existing_derive_names(&item.attrs)?;
    if !options.no_redact && !options.no_debug && existing.iter().any(|name| name == "Debug") {
        return Err(Error::new_spanned(
            &item.ident,
            "explicit Debug would bypass model redaction; use the generated safe implementation",
        ));
    }
    if !options.no_redact && !options.no_serialize && existing.iter().any(|name| name == "Serialize") {
        return Err(Error::new_spanned(
            &item.ident,
            "explicit Serialize would bypass model redaction; use the generated safe implementation",
        ));
    }
    let mut derives = Vec::new();
    let mut add = |name: &str, tokens: TokenStream| {
        if !existing.iter().any(|existing| existing == name) {
            derives.push(tokens);
        }
    };
    if !options.no_clone {
        add("Clone", quote!(Clone));
    }
    if options.no_redact && !options.no_debug {
        add("Debug", quote!(Debug));
    }
    if !options.no_partial_eq {
        add("PartialEq", quote!(PartialEq));
    }
    if !options.no_partial_eq && !options.no_eq {
        add("Eq", quote!(Eq));
    }
    if !options.no_partial_eq && !options.no_eq && !options.no_hash {
        add("Hash", quote!(Hash));
    }
    if options.ord {
        add("PartialOrd", quote!(PartialOrd));
        add("Ord", quote!(Ord));
    } else if options.partial_ord && !options.no_partial_eq {
        add("PartialOrd", quote!(PartialOrd));
    }
    let default_copy = !options.no_clone
        && !options.no_copy
        && declaration.kind == MacroKind::Enum
        && declaration.variants.iter().all(|variant| variant.fields.is_empty());
    if options.copy || default_copy {
        add("Copy", quote!(Copy));
    }
    if options.default {
        add("Default", quote!(Default));
    }
    if options.no_redact && !options.no_serialize {
        add("Serialize", quote!(#runtime::__private::serde::Serialize));
    }
    if !options.no_deserialize {
        add("Deserialize", quote!(#runtime::__private::serde::Deserialize));
    }
    if !options.no_redact {
        add("Redact", quote!(#runtime::__private::qubit_redact::Redact));
    }
    if !derives.is_empty() {
        item.attrs.push(parse_quote!(#[derive(#(#derives),*)]));
    }
    if !options.no_serialize || !options.no_deserialize {
        let path = format!("{}::__private::serde", runtime.to_string().replace(' ', ""));
        let path = LitStr::new(&path, proc_macro2::Span::call_site());
        item.attrs.push(parse_quote!(#[serde(crate = #path)]));
        if options.transparent {
            item.attrs.push(parse_quote!(#[serde(transparent)]));
        }
    }
    if !options.no_redact {
        let mut flags = Vec::new();
        if !options.no_debug && !existing.iter().any(|value| value == "Debug") {
            flags.push(quote!(debug));
        }
        if !options.no_display && !options.transparent {
            flags.push(quote!(display));
        }
        if !options.no_serialize && !existing.iter().any(|value| value == "Serialize") {
            flags.push(quote!(serde));
        }
        if options.transparent {
            flags.push(quote!(transparent));
        }
        item.attrs.push(parse_quote!(#[redact(
            crate = #runtime::__private::qubit_redact,
            #(#flags),*
        )]));
    }
    Ok(())
}

fn expand_display(declaration: &DeclarationIr, item: &DeriveInput, runtime: &TokenStream) -> TokenStream {
    let options = &declaration.options;
    if options.no_display || (!options.no_redact && !options.transparent) {
        return TokenStream::new();
    }
    let name = &item.ident;
    let mut generics = item.generics.clone();
    let transparent_field = options.transparent.then(|| match &item.data {
        Data::Struct(data) => data.fields.iter().next().expect("validated transparent field"),
        Data::Enum(_) | Data::Union(_) => unreachable!("transparent is only valid on Value structs"),
    });
    if options.no_redact {
        let where_clause = generics.make_where_clause();
        if let Some(field) = transparent_field {
            let ty = &field.ty;
            where_clause.predicates.push(parse_quote!(#ty: ::core::fmt::Display));
        } else {
            let fields: Vec<_> = match &item.data {
                Data::Struct(data) => data.fields.iter().collect(),
                Data::Enum(data) => data.variants.iter().flat_map(|variant| variant.fields.iter()).collect(),
                Data::Union(_) => Vec::new(),
            };
            for field in fields {
                let ty = &field.ty;
                where_clause.predicates.push(parse_quote!(#ty: ::core::fmt::Debug));
            }
        }
    }
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    let body = if !options.no_redact {
        let (prefix, suffix) = match transparent_field.expect("transparent field").ident.as_ref() {
            Some(field) => (format!("{} {{ {}: ", name, field), " }".to_owned()),
            None => (format!("{}(", name), ")".to_owned()),
        };
        quote! {
            let output = #runtime::__private::qubit_redact::Redactor::application_default().redact(self);
            let text = output.text().as_str();
            let text = text
                .strip_prefix(#prefix)
                .and_then(|text| text.strip_suffix(#suffix))
                .unwrap_or(text);
            let text = text
                .strip_prefix('"')
                .and_then(|text| text.strip_suffix('"'))
                .unwrap_or(text);
            formatter.write_str(text)
        }
    } else if let Some(field) = transparent_field {
        let access = field
            .ident
            .as_ref()
            .map_or_else(|| quote!(self.0), |field| quote!(self.#field));
        quote!(::core::fmt::Display::fmt(&#access, formatter))
    } else {
        plain_structured_display_body(name, &item.data)
    };
    quote! {
        impl #impl_generics ::core::fmt::Display for #name #type_generics #where_clause {
            fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                #body
            }
        }
    }
}

fn plain_structured_display_body(name: &syn::Ident, data: &Data) -> TokenStream {
    match data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let names = fields.named.iter().filter_map(|field| field.ident.as_ref());
                quote! {
                    let mut debug = formatter.debug_struct(stringify!(#name));
                    #(debug.field(stringify!(#names), &self.#names);)*
                    debug.finish()
                }
            }
            Fields::Unnamed(fields) => {
                let indexes = (0..fields.unnamed.len()).map(syn::Index::from);
                quote! {
                    let mut debug = formatter.debug_tuple(stringify!(#name));
                    #(debug.field(&self.#indexes);)*
                    debug.finish()
                }
            }
            Fields::Unit => quote!(formatter.write_str(stringify!(#name))),
        },
        Data::Enum(data) => {
            let arms = data.variants.iter().map(|variant| {
                let variant_name = &variant.ident;
                match &variant.fields {
                    Fields::Unit => quote!(Self::#variant_name => formatter.write_str(stringify!(#variant_name))),
                    Fields::Unnamed(fields) => {
                        let bindings: Vec<_> = (0..fields.unnamed.len())
                            .map(|index| format_ident!("field_{index}"))
                            .collect();
                        quote! {
                            Self::#variant_name(#(#bindings),*) => {
                                let mut debug = formatter.debug_tuple(stringify!(#variant_name));
                                #(debug.field(#bindings);)*
                                debug.finish()
                            }
                        }
                    }
                    Fields::Named(fields) => {
                        let bindings: Vec<_> = fields.named.iter().filter_map(|field| field.ident.as_ref()).collect();
                        quote! {
                            Self::#variant_name { #(#bindings),* } => {
                                let mut debug = formatter.debug_struct(stringify!(#variant_name));
                                #(debug.field(stringify!(#bindings), #bindings);)*
                                debug.finish()
                            }
                        }
                    }
                }
            });
            quote!(match self { #(#arms,)* })
        }
        Data::Union(_) => TokenStream::new(),
    }
}

fn apply_serde_defaults(declaration: &mut DeclarationIr, item: &mut DeriveInput, runtime: &TokenStream) {
    match (&mut item.data, declaration.kind) {
        (Data::Struct(data), _) => {
            for (field, ir) in data.fields.iter_mut().zip(&mut declaration.fields) {
                apply_field_serde_default(field, ir, runtime);
            }
        }
        (Data::Enum(data), MacroKind::Enum) => {
            for (variant, variant_ir) in data.variants.iter_mut().zip(&mut declaration.variants) {
                if !matches!(variant.fields, Fields::Named(_)) {
                    continue;
                }
                for (field, ir) in variant.fields.iter_mut().zip(&mut variant_ir.fields) {
                    apply_field_serde_default(field, ir, runtime);
                }
            }
        }
        _ => {}
    }
}

fn apply_field_serde_default(field: &mut syn::Field, ir: &mut FieldIr, runtime: &TokenStream) {
    if field.ident.is_none() {
        return;
    }
    let keep_serializing = ir.keep_serializing;
    let kind = omission_kind(&field.ty);
    let Some(kind) = kind else {
        return;
    };
    let serde = match ir.occurrences.iter_mut().find_map(|occurrence| match occurrence {
        FieldOccurrence::Serde(value) => Some(value),
        _ => None,
    }) {
        Some(value) => value,
        None => {
            ir.occurrences.push(FieldOccurrence::Serde(SerdeIr::default()));
            match ir.occurrences.last_mut() {
                Some(FieldOccurrence::Serde(value)) => value,
                _ => unreachable!(),
            }
        }
    };
    if !serde.default {
        field.attrs.push(parse_quote!(#[serde(default)]));
        serde.default = true;
        serde.default_from_model = true;
    }
    if keep_serializing {
        serde.omit_suppressed = true;
        return;
    }
    if serde.skip_serializing || serde.explicit_skip_serializing_if {
        return;
    }
    let suffix = match kind {
        OmissionKind::Option => "is_none",
        OmissionKind::Collection => "is_empty",
    };
    let path = format!(
        "{}::__private::serde_helpers::{suffix}",
        runtime.to_string().replace(' ', ""),
    );
    let path = LitStr::new(&path, proc_macro2::Span::call_site());
    field.attrs.push(parse_quote!(#[serde(skip_serializing_if = #path)]));
    serde.omit_from_model = true;
}

enum OmissionKind {
    Option,
    Collection,
}

fn omission_kind(ty: &Type) -> Option<OmissionKind> {
    let Type::Path(path) = ty else {
        return None;
    };
    let name = path.path.segments.last()?.ident.to_string();
    if name == "Option" {
        return Some(OmissionKind::Option);
    }
    matches!(
        name.as_str(),
        "Vec" | "VecDeque" | "LinkedList" | "BinaryHeap" | "HashSet" | "BTreeSet" | "HashMap" | "BTreeMap"
    )
    .then_some(OmissionKind::Collection)
}

fn existing_derive_names(attributes: &[Attribute]) -> Result<Vec<String>> {
    let mut result = Vec::new();
    for attribute in attributes {
        if !attribute.path().is_ident("derive") {
            continue;
        }
        let paths = attribute.parse_args_with(Punctuated::<syn::Path, Token![,]>::parse_terminated)?;
        result.extend(
            paths
                .iter()
                .filter_map(|path| path.segments.last().map(|segment| segment.ident.to_string())),
        );
    }
    Ok(result)
}

fn expand_metadata(declaration: &DeclarationIr, item: &DeriveInput, runtime: &TokenStream) -> TokenStream {
    let ident = &item.ident;
    let fields = expand_field_vector(&declaration.fields, quote!(descriptor.fields()), runtime);
    let role = expand_role(declaration, runtime);
    let declared_model_id = declaration
        .options
        .id
        .as_ref()
        .map_or_else(|| quote!(None), |id| quote!(Some(#runtime::ModelId::new(#id))));
    let has_generics = !item.generics.params.is_empty();
    let mut impl_generics_source = item.generics.clone();
    for parameter in impl_generics_source.type_params_mut() {
        parameter.bounds.push(parse_quote!(#runtime::Reflect));
        parameter.bounds.push(parse_quote!('static));
    }
    let (impl_generics, ty_generics, where_clause) = impl_generics_source.split_for_impl();
    let generic_metadata = format_ident!("__qubit_model_generic_metadata_{}", ident.to_string().to_snake_case());
    let registration = match (declaration.options.id.as_ref(), has_generics) {
        (Some(_), false) => expand_registration(ident, runtime),
        (Some(id), true) => expand_generic_registration(
            ident,
            id,
            declaration.kind,
            &generic_metadata,
            &declaration.fields,
            &item.data,
            &item.generics,
            runtime,
        ),
        (None, _) => TokenStream::new(),
    };
    let model_id = if has_generics {
        quote!(None)
    } else {
        declared_model_id.clone()
    };
    let generic_definition = if has_generics {
        declaration.options.id.as_ref().map_or_else(TokenStream::new, |_| {
            quote! {
                let metadata = metadata.with_generic_definition(#generic_metadata());
            }
        })
    } else {
        TokenStream::new()
    };
    let build_metadata = quote! {
        let descriptor = #runtime::TypeDescriptor::of::<Self>();
        #fields
        let fields: &'static [#runtime::FieldMetadata] = ::std::boxed::Box::leak(fields.into_boxed_slice());
        #role
        let properties: ::std::vec::Vec<_> = fields
            .iter()
            .filter_map(|field| field.name().map(|name| {
                #runtime::PropertyMetadata::new(
                    name,
                    field.type_ref(),
                    Some(field),
                    None,
                    None,
                )
            }))
            .collect();
        let properties: &'static [#runtime::PropertyMetadata] =
            ::std::boxed::Box::leak(properties.into_boxed_slice());
        let metadata = #runtime::TypeMetadata::new(descriptor, #model_id, fields, role)
            .with_properties(properties);
        #generic_definition
        metadata
    };
    let metadata_body = if has_generics {
        quote! {
            static CACHE: ::std::sync::OnceLock<
                ::std::sync::Mutex<
                    ::std::collections::HashMap<::std::any::TypeId, &'static #runtime::TypeMetadata>,
                >,
            > = ::std::sync::OnceLock::new();
            let cache = CACHE.get_or_init(|| ::std::sync::Mutex::new(::std::collections::HashMap::new()));
            let type_id = ::std::any::TypeId::of::<Self>();
            if let Some(metadata) = cache.lock().expect("model metadata cache lock").get(&type_id).copied() {
                return metadata;
            }
            let metadata: &'static #runtime::TypeMetadata =
                ::std::boxed::Box::leak(::std::boxed::Box::new({ #build_metadata }));
            let mut guard = cache.lock().expect("model metadata cache lock");
            *guard.entry(type_id).or_insert(metadata)
        }
    } else {
        quote! {
            static METADATA: ::std::sync::OnceLock<#runtime::TypeMetadata> = ::std::sync::OnceLock::new();
            METADATA.get_or_init(|| { #build_metadata })
        }
    };

    quote! {
        impl #impl_generics #runtime::__private::ModelTypeSeal for #ident #ty_generics #where_clause {}

        impl #impl_generics #runtime::HasTypeMetadata for #ident #ty_generics #where_clause {
            fn type_metadata() -> &'static #runtime::TypeMetadata {
                #metadata_body
            }
        }

        #registration
    }
}

#[allow(clippy::too_many_arguments)]
fn expand_generic_registration(
    ident: &syn::Ident,
    id: &LitStr,
    kind: MacroKind,
    metadata_fn: &syn::Ident,
    fields: &[FieldIr],
    data: &Data,
    generics: &syn::Generics,
    runtime: &TokenStream,
) -> TokenStream {
    let snake_name = ident.to_string().to_snake_case();
    let definition_fn = format_ident!("__qubit_reflect_generic_definition_{}", snake_name);
    let source_fn = format_ident!("__qubit_model_generic_source_{}", snake_name);
    let registration_fn = format_ident!("__qubit_model_generic_registration_{}", snake_name);
    let role = match kind {
        MacroKind::Model => quote!(#runtime::ModelRole::Model),
        MacroKind::Enum => quote!(#runtime::ModelRole::Enum),
        MacroKind::Value => quote!(#runtime::ModelRole::Value),
        MacroKind::Entity | MacroKind::Projection | MacroKind::ModelProperties => {
            unreachable!("only generic-capable roles reach generic registration")
        }
    };
    let fingerprint = stable_fingerprint(&ident.to_string());
    let template = expand_generic_template(ident, fields, data, generics, runtime);
    let template_root = format_ident!("__qubit_model_generic_template_{}", snake_name);
    let template_fields = expand_field_vector(fields, quote!(template_descriptor.fields()), runtime);
    quote! {
        #template

        #[doc(hidden)]
        fn #metadata_fn() -> &'static #runtime::GenericModelMetadata {
            static METADATA: ::std::sync::OnceLock<#runtime::GenericModelMetadata> =
                ::std::sync::OnceLock::new();
            METADATA.get_or_init(|| {
                let template_descriptor = #template_root();
                #template_fields
                let fields: &'static [#runtime::FieldMetadata] =
                    ::std::boxed::Box::leak(fields.into_boxed_slice());
                #runtime::GenericModelMetadata::new(
                    #runtime::ModelId::new(#id),
                    #role,
                    #definition_fn(),
                    fields,
                )
            })
        }

        #[doc(hidden)]
        fn #source_fn() -> &'static #runtime::identity::FragmentIdentity {
            static SOURCE: ::std::sync::OnceLock<#runtime::identity::FragmentIdentity> =
                ::std::sync::OnceLock::new();
            SOURCE.get_or_init(|| #runtime::identity::FragmentIdentity::new(
                env!("CARGO_PKG_NAME"), module_path!(), line!(), column!(), "generic-model", #fingerprint,
            ))
        }

        #[doc(hidden)]
        fn #registration_fn() -> #runtime::ModelRegistration {
            #runtime::ModelRegistration::from_generic(#metadata_fn(), #source_fn())
        }

        #runtime::__private::inventory::submit! {
            #runtime::ModelRegistrationFactory(#registration_fn)
        }
    }
}

fn expand_generic_template(
    ident: &syn::Ident,
    fields: &[FieldIr],
    data: &Data,
    generics: &syn::Generics,
    runtime: &TokenStream,
) -> TokenStream {
    let snake_name = ident.to_string().to_snake_case();
    let marker = format_ident!("__QubitModelGenericTemplate{}", ident);
    let root = format_ident!("__qubit_model_generic_template_{}", snake_name);
    let type_parameters: std::collections::HashSet<_> = generics
        .type_params()
        .map(|parameter| parameter.ident.to_string())
        .collect();
    let const_parameters: std::collections::HashSet<_> = generics
        .const_params()
        .map(|parameter| parameter.ident.to_string())
        .collect();
    let declared_fields: Vec<_> = match data {
        Data::Struct(data) => data.fields.iter().collect(),
        Data::Enum(_) | Data::Union(_) => Vec::new(),
    };
    let field_descriptors = fields.iter().zip(declared_fields).map(|(field, declared)| {
        let index = field.index;
        let name = declared
            .ident
            .as_ref()
            .map(|name| LitStr::new(&name.to_string(), name.span()));
        let rust_name = name.as_ref().map_or_else(|| quote!(None), |name| quote!(Some(#name)));
        let query_name = rust_name.clone();
        let declared_visibility = &declared.vis;
        let visibility = LitStr::new(
            &quote!(#declared_visibility).to_string().replace(' ', ""),
            proc_macro2::Span::call_site(),
        );
        let expression = expand_type_expression(&field.ty, &type_parameters, &const_parameters, runtime);
        quote! {
            {
                let field_type: &'static #runtime::descriptor::TypeRef = ::std::boxed::Box::leak(
                    ::std::boxed::Box::new(#runtime::descriptor::TypeRef::Symbolic(#expression)),
                );
                descriptors.push(#runtime::__private::descriptor::field(
                    #root,
                    #index,
                    #rust_name,
                    #query_name,
                    field_type,
                    #runtime::identity::Visibility::from_source(#visibility),
                ));
            }
        }
    });
    let struct_kind = match data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(_) => quote!(#runtime::descriptor::StructKind::Named),
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => quote!(#runtime::descriptor::StructKind::Newtype),
            Fields::Unnamed(_) => quote!(#runtime::descriptor::StructKind::Tuple),
            Fields::Unit => quote!(#runtime::descriptor::StructKind::Unit),
        },
        Data::Enum(_) | Data::Union(_) => quote!(#runtime::descriptor::StructKind::Unit),
    };
    let query_name = LitStr::new(&ident.to_string(), ident.span());
    quote! {
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        struct #marker;

        #[doc(hidden)]
        fn #root() -> &'static #runtime::TypeDescriptor {
            static DESCRIPTOR: ::std::sync::OnceLock<#runtime::TypeDescriptor> =
                ::std::sync::OnceLock::new();
            DESCRIPTOR.get_or_init(|| {
                let mut descriptors = ::std::vec::Vec::new();
                #(#field_descriptors)*
                let descriptors = ::std::boxed::Box::leak(descriptors.into_boxed_slice());
                #runtime::__private::descriptor::struct_type::<#marker>(
                    #query_name,
                    #struct_kind,
                    descriptors,
                )
            })
        }
    }
}

fn expand_type_expression(
    ty: &Type,
    type_parameters: &std::collections::HashSet<String>,
    const_parameters: &std::collections::HashSet<String>,
    runtime: &TokenStream,
) -> TokenStream {
    match ty {
        Type::Path(path) if path.qself.is_none() => {
            if path.path.segments.len() == 1 {
                let name = path.path.segments[0].ident.to_string();
                if type_parameters.contains(&name) {
                    return quote!(#runtime::expression::TypeExpression::Parameter(#name.into()));
                }
            }
            let segments: Vec<_> = path
                .path
                .segments
                .iter()
                .map(|segment| LitStr::new(&segment.ident.to_string(), segment.ident.span()))
                .collect();
            let arguments = path.path.segments.last().map_or_else(Vec::new, |segment| {
                let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
                    return Vec::new();
                };
                arguments
                    .args
                    .iter()
                    .filter_map(|argument| match argument {
                        syn::GenericArgument::Type(ty) => {
                            let expression = expand_type_expression(ty, type_parameters, const_parameters, runtime);
                            Some(quote!(#runtime::expression::GenericArgument::Type(#expression)))
                        }
                        syn::GenericArgument::Const(value) => {
                            let expression = expand_const_expression(value, const_parameters, runtime);
                            let diagnostic = LitStr::new(&quote!(#value).to_string(), proc_macro2::Span::call_site());
                            Some(quote!(#runtime::expression::GenericArgument::Const(
                                #runtime::expression::ConstGenericArgument {
                                    declared_type: ::std::boxed::Box::new(
                                        #runtime::expression::TypeExpression::Concrete(
                                            #runtime::expression::ConcreteTypeExpression {
                                                path: ::std::boxed::Box::new(["_".into()]),
                                                arguments: ::std::boxed::Box::new([]),
                                                diagnostic: #runtime::expression::DiagnosticText::default(),
                                            },
                                        ),
                                    ),
                                    value: #expression,
                                    normalized_diagnostic: #diagnostic.into(),
                                },
                            )))
                        }
                        _ => None,
                    })
                    .collect()
            });
            quote!(#runtime::expression::TypeExpression::Concrete(
                #runtime::expression::ConcreteTypeExpression {
                    path: ::std::boxed::Box::new([#(#segments.into()),*]),
                    arguments: ::std::boxed::Box::new([#(#arguments),*]),
                    diagnostic: #runtime::expression::DiagnosticText::default(),
                },
            ))
        }
        Type::Slice(slice) => {
            let element = expand_type_expression(&slice.elem, type_parameters, const_parameters, runtime);
            quote!(#runtime::expression::TypeExpression::Slice(::std::boxed::Box::new(#element)))
        }
        Type::Array(array) => {
            let element = expand_type_expression(&array.elem, type_parameters, const_parameters, runtime);
            let length = expand_const_expression(&array.len, const_parameters, runtime);
            quote!(#runtime::expression::TypeExpression::Array(
                #runtime::expression::ArrayTypeExpression {
                    element: ::std::boxed::Box::new(#element),
                    length: #length,
                    diagnostic: #runtime::expression::DiagnosticText::default(),
                },
            ))
        }
        Type::Tuple(tuple) => {
            let elements = tuple
                .elems
                .iter()
                .map(|element| expand_type_expression(element, type_parameters, const_parameters, runtime));
            quote!(#runtime::expression::TypeExpression::Tuple(
                ::std::boxed::Box::new([#(#elements),*]),
            ))
        }
        Type::Reference(reference) => {
            let target = expand_type_expression(&reference.elem, type_parameters, const_parameters, runtime);
            let lifetime = match reference
                .lifetime
                .as_ref()
                .map(|value| value.ident.to_string())
                .as_deref()
            {
                Some("static") => quote!(#runtime::expression::LifetimeExpression::Static),
                Some("_") => quote!(#runtime::expression::LifetimeExpression::Placeholder),
                Some(name) => quote!(#runtime::expression::LifetimeExpression::Named(#name.into())),
                None => quote!(#runtime::expression::LifetimeExpression::Elided),
            };
            let mutable = reference.mutability.is_some();
            quote!(#runtime::expression::TypeExpression::Reference(
                #runtime::expression::ReferenceTypeExpression {
                    lifetime: #lifetime,
                    mutable: #mutable,
                    target: ::std::boxed::Box::new(#target),
                    diagnostic: #runtime::expression::DiagnosticText::default(),
                },
            ))
        }
        _ => {
            let source = LitStr::new(&quote!(#ty).to_string(), proc_macro2::Span::call_site());
            quote!(#runtime::expression::TypeExpression::Concrete(
                #runtime::expression::ConcreteTypeExpression {
                    path: ::std::boxed::Box::new([#source.into()]),
                    arguments: ::std::boxed::Box::new([]),
                    diagnostic: #runtime::expression::DiagnosticText::from(#source),
                },
            ))
        }
    }
}

fn expand_const_expression(
    value: &Expr,
    const_parameters: &std::collections::HashSet<String>,
    runtime: &TokenStream,
) -> TokenStream {
    match value {
        Expr::Path(path) => {
            let segments: Vec<_> = path
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect();
            if segments.len() == 1 && const_parameters.contains(&segments[0]) {
                let name = &segments[0];
                quote!(#runtime::expression::ConstExpression::Parameter(#name.into()))
            } else {
                quote!(#runtime::expression::ConstExpression::Path(
                    ::std::boxed::Box::new([#(#segments.into()),*]),
                ))
            }
        }
        Expr::Lit(ExprLit {
            lit: Lit::Int(value), ..
        }) => {
            let value = value.base10_parse::<u128>().unwrap_or_default();
            quote!(#runtime::expression::ConstExpression::UnsignedInteger(#value))
        }
        Expr::Lit(ExprLit {
            lit: Lit::Bool(value), ..
        }) => {
            let value = value.value;
            quote!(#runtime::expression::ConstExpression::Boolean(#value))
        }
        Expr::Lit(ExprLit {
            lit: Lit::Char(value), ..
        }) => {
            let value = value.value();
            quote!(#runtime::expression::ConstExpression::Character(#value))
        }
        _ => {
            let source = quote!(#value).to_string();
            quote!(#runtime::expression::ConstExpression::Path(
                ::std::boxed::Box::new([#source.into()]),
            ))
        }
    }
}

fn expand_field_vector(fields: &[FieldIr], descriptor_fields: TokenStream, runtime: &TokenStream) -> TokenStream {
    let bodies = fields
        .iter()
        .map(|field| expand_field(field, &descriptor_fields, runtime));
    quote! {
        let mut fields = ::std::vec::Vec::new();
        #(#bodies)*
    }
}

fn expand_field(field: &FieldIr, descriptor_fields: &TokenStream, runtime: &TokenStream) -> TokenStream {
    let index = field.index;
    let validator_irs: Vec<_> = field
        .occurrences
        .iter()
        .filter_map(|value| match value {
            FieldOccurrence::Validator(value) => Some(value),
            _ => None,
        })
        .collect();
    let validators = validator_irs.iter().map(|value| expand_validator(value, runtime));
    let identifier_assignment = field.occurrences.iter().find_map(|value| match value {
        FieldOccurrence::Identifier(value) => Some(*value),
        _ => None,
    });
    let has_identifier = identifier_assignment.is_some();
    let has_indexed = field
        .occurrences
        .iter()
        .any(|value| matches!(value, FieldOccurrence::Indexed));
    let unique_ir = field.occurrences.iter().find_map(|value| match value {
        FieldOccurrence::Unique(value) => Some(value),
        _ => None,
    });
    let reference_ir = field.occurrences.iter().find_map(|value| match value {
        FieldOccurrence::Reference(value) => Some(value),
        _ => None,
    });
    let key_part_order = field.occurrences.iter().find_map(|value| match value {
        FieldOccurrence::KeyPart(value) => Some(*value),
        _ => None,
    });
    let codec_ir = field.occurrences.iter().find_map(|value| match value {
        FieldOccurrence::Codec(value) => Some(value),
        _ => None,
    });
    let redact_ir = field.occurrences.iter().find_map(|value| match value {
        FieldOccurrence::Redact(value) => Some(value),
        _ => None,
    });
    let serde_ir = field.occurrences.iter().find_map(|value| match value {
        FieldOccurrence::Serde(value) => Some(value),
        _ => None,
    });
    let element_ir = field.occurrences.iter().find_map(|value| match value {
        FieldOccurrence::Selector(value) if matches!(value.position, SelectorPositionIr::Element) => Some(value),
        _ => None,
    });
    let map_key_ir = field.occurrences.iter().find_map(|value| match value {
        FieldOccurrence::Selector(value) if matches!(value.position, SelectorPositionIr::MapKey) => Some(value),
        _ => None,
    });
    let map_value_ir = field.occurrences.iter().find_map(|value| match value {
        FieldOccurrence::Selector(value) if matches!(value.position, SelectorPositionIr::MapValue) => Some(value),
        _ => None,
    });
    let element_selector =
        element_ir.map(|value| expand_selector_metadata(value, format_ident!("element_selector"), runtime));
    let map_key_selector =
        map_key_ir.map(|value| expand_selector_metadata(value, format_ident!("map_key_selector"), runtime));
    let map_value_selector =
        map_value_ir.map(|value| expand_selector_metadata(value, format_ident!("map_value_selector"), runtime));
    let constraint_irs: Vec<_> = field
        .occurrences
        .iter()
        .filter_map(|value| match value {
            FieldOccurrence::Constraint(value) => Some(value),
            _ => None,
        })
        .collect();
    let constraints = constraint_irs.iter().map(|value| {
        expand_constraint(
            value,
            element_ir.is_some(),
            map_key_ir.is_some(),
            map_value_ir.is_some(),
            runtime,
        )
    });
    let identifier = identifier_assignment.map(|assignment| {
        let assignment = match assignment {
            IdentifierAssignmentIr::Application => quote!(#runtime::IdentifierAssignment::Application),
            IdentifierAssignmentIr::Database => quote!(#runtime::IdentifierAssignment::Database),
        };
        quote! {
            let identifier: &'static #runtime::IdentifierMetadata = ::std::boxed::Box::leak(::std::boxed::Box::new(
                #runtime::IdentifierMetadata::new(#assignment),
            ));
        }
    });
    let unique = unique_ir.map(|unique| {
        let paths = unique.respect_to.iter().map(|path| expand_field_path(path, runtime));
        let ignore_case = unique.ignore_case;
        quote! {
            let unique_paths: &'static [#runtime::PropertyPath] = ::std::boxed::Box::leak(::std::vec![#(#paths),*].into_boxed_slice());
            let unique: &'static #runtime::FieldUniqueMetadata = ::std::boxed::Box::leak(::std::boxed::Box::new(
                #runtime::FieldUniqueMetadata::new(unique_paths, #ignore_case),
            ));
        }
    });
    let reference = reference_ir.map(|value| expand_reference(value, runtime));
    let key_part = key_part_order.map(|order| {
        quote! {
            let key_part: &'static #runtime::KeyPartMetadata = ::std::boxed::Box::leak(::std::boxed::Box::new(
                #runtime::KeyPartMetadata::new(#order),
            ));
        }
    });
    let codec = codec_ir.map(|codec| {
        let value = match codec {
            CodecIr::DeclaredId(id) => quote!(#runtime::CodecReference::DeclaredId(#id)),
            CodecIr::RustType(ty) => quote!(#runtime::CodecReference::RustType(#runtime::StrategyTypeIdentity::of::<#ty>())),
        };
        let assertion = match codec {
            CodecIr::DeclaredId(_) => TokenStream::new(),
            CodecIr::RustType(codec_type) => {
                let value_type = codec_value_type(&field.ty);
                quote! {
                    fn assert_codec<C, V>()
                    where
                        C: ::core::default::Default
                            + #runtime::__private::qubit_codec::ValueEncoder<V, Output = ::std::string::String>
                            + #runtime::__private::qubit_codec::ValueDecoder<str, Output = V>,
                    {}
                    assert_codec::<#codec_type, #value_type>();
                }
            }
        };
        quote! {
            #assertion
            let codec_reference: &'static #runtime::CodecReference = ::std::boxed::Box::leak(::std::boxed::Box::new(#value));
            let codec: &'static #runtime::CodecMetadata = ::std::boxed::Box::leak(::std::boxed::Box::new(
                #runtime::CodecMetadata::new(codec_reference, #runtime::CodecSource::Field),
            ));
        }
    });
    let redact = redact_ir.map(|value| expand_redact(value, quote!(#runtime::RedactPosition::Field), runtime));
    let serde = serde_ir.map_or_else(
        || quote!(let serde: &'static #runtime::SerdeFieldMetadata = &#runtime::SerdeFieldMetadata::DEFAULT;),
        |value| expand_serde(value, runtime),
    );
    let mut occurrence_tokens = Vec::new();
    let mut validator_index = 0usize;
    let mut constraint_index = 0usize;
    for occurrence in &field.occurrences {
        occurrence_tokens.push(match occurrence {
            FieldOccurrence::Identifier(_) => {
                quote!(attributes.push(#runtime::FieldAttributeMetadata::Identifier(identifier));)
            }
            FieldOccurrence::Indexed => TokenStream::new(),
            FieldOccurrence::Unique(_) => quote!(attributes.push(#runtime::FieldAttributeMetadata::Unique(unique));),
            FieldOccurrence::Reference(_) => {
                quote!(attributes.push(#runtime::FieldAttributeMetadata::Reference(reference));)
            }
            FieldOccurrence::KeyPart(_) => {
                quote!(attributes.push(#runtime::FieldAttributeMetadata::KeyPart(key_part));)
            }
            FieldOccurrence::Constraint(_) => {
                let current = constraint_index;
                constraint_index += 1;
                quote!(attributes.push(#runtime::FieldAttributeMetadata::Constraint(&constraints[#current]));)
            }
            FieldOccurrence::Selector(_) => TokenStream::new(),
            FieldOccurrence::Validator(_) => {
                let current = validator_index;
                validator_index += 1;
                quote!(attributes.push(#runtime::FieldAttributeMetadata::Validator(&validators[#current]));)
            }
            FieldOccurrence::Codec(_) => quote!(attributes.push(#runtime::FieldAttributeMetadata::Codec(codec));),
            FieldOccurrence::Redact(_) => quote!(attributes.push(#runtime::FieldAttributeMetadata::Redact(redact));),
            FieldOccurrence::Serde(_) => quote!(attributes.push(#runtime::FieldAttributeMetadata::Serde(serde));),
            FieldOccurrence::Opaque => quote!(attributes.push(#runtime::FieldAttributeMetadata::Opaque);),
        });
    }
    let mut reason_parts = Vec::new();
    if has_indexed {
        reason_parts.push(quote!(#runtime::IndexingReasons::EXPLICIT));
    }
    if has_identifier {
        reason_parts.push(quote!(#runtime::IndexingReasons::IDENTIFIER));
    }
    if unique_ir.is_some() {
        reason_parts.push(quote!(#runtime::IndexingReasons::UNIQUE));
    }
    if reference_ir.is_some() {
        reason_parts.push(quote!(#runtime::IndexingReasons::REFERENCE));
    }
    let indexed = reason_parts
        .into_iter()
        .reduce(|left, right| quote!(#left | #right))
        .map(|reasons| {
            quote! {
                attributes.push(#runtime::FieldAttributeMetadata::Indexed(#reasons));
            }
        });
    quote! {
        {
            #identifier
            #unique
            #reference
            #key_part
            let validators: &'static [#runtime::ValidatorMetadata] = ::std::boxed::Box::leak(
                ::std::vec![#(#validators),*].into_boxed_slice(),
            );
            #codec
            #redact
            #serde
            #element_selector
            #map_key_selector
            #map_value_selector
            let constraints: &'static [#runtime::ConstraintMetadata] = ::std::boxed::Box::leak(
                ::std::vec![#(#constraints),*].into_boxed_slice(),
            );
            let mut attributes = ::std::vec::Vec::new();
            #(#occurrence_tokens)*
            #indexed
            let attributes: &'static [#runtime::FieldAttributeMetadata] =
                ::std::boxed::Box::leak(attributes.into_boxed_slice());
            fields.push(#runtime::FieldMetadata::with_semantics(
                &#descriptor_fields[#index],
                attributes,
                constraints,
                validators,
                serde,
            ));
        }
    }
}

fn expand_constraint(
    value: &ConstraintIr,
    has_element: bool,
    has_map_key: bool,
    has_map_value: bool,
    runtime: &TokenStream,
) -> TokenStream {
    match value {
        ConstraintIr::Text(value) => {
            let min_chars = option_number(value.min_chars);
            let max_chars = option_number(value.max_chars);
            let min_bytes = option_number(value.min_bytes);
            let max_bytes = option_number(value.max_bytes);
            let allowed = match value.allowed_chars.as_deref().unwrap_or("unicode") {
                "unicode" => quote!(#runtime::AllowedChars::Unicode),
                "printable_unicode" => quote!(#runtime::AllowedChars::PrintableUnicode),
                "ascii" => quote!(#runtime::AllowedChars::Ascii),
                "printable_ascii" => quote!(#runtime::AllowedChars::PrintableAscii),
                "code" => quote!(#runtime::AllowedChars::Code),
                _ => quote!(compile_error!("invalid allowed_chars value")),
            };
            let non_blank = value.non_blank;
            let format = value.format.as_deref().map_or_else(
                || quote!(None),
                |value| {
                    let value = match value {
                        "email" => quote!(#runtime::TextFormat::Email),
                        "cn_mobile" => quote!(#runtime::TextFormat::Mobile),
                        "uri" => quote!(#runtime::TextFormat::Uri),
                        "uuid" => quote!(#runtime::TextFormat::Uuid),
                        _ => quote!(compile_error!("invalid text format")),
                    };
                    quote!(Some(#value))
                },
            );
            quote!(#runtime::ConstraintMetadata::Text(#runtime::TextConstraint::new(
                #min_chars, #max_chars, #min_bytes, #max_bytes, #allowed, #non_blank, #format,
            )))
        }
        ConstraintIr::Decimal(value) => {
            let precision = option_number(value.precision);
            let scale = value.scale;
            let rounding = rounding_tokens(&value.rounding, runtime);
            let semantic = if value.money {
                quote!(#runtime::DecimalSemantic::Money)
            } else {
                quote!(#runtime::DecimalSemantic::Number)
            };
            let min = option_lit_str(&value.min);
            let max = option_lit_str(&value.max);
            let min_inclusive = value.min_inclusive;
            let max_inclusive = value.max_inclusive;
            quote!(#runtime::ConstraintMetadata::Decimal(
                #runtime::DecimalConstraint::new(#precision, #scale, #rounding, #semantic)
                    .with_bounds(#min, #max, #min_inclusive, #max_inclusive)
            ))
        }
        ConstraintIr::Time(value) => {
            let precision = match value.as_str() {
                "second" => quote!(#runtime::TemporalPrecision::Second),
                "millisecond" => quote!(#runtime::TemporalPrecision::Millisecond),
                "microsecond" => quote!(#runtime::TemporalPrecision::Microsecond),
                "nanosecond" => quote!(#runtime::TemporalPrecision::Nanosecond),
                _ => quote!(compile_error!("invalid time precision")),
            };
            quote!(#runtime::ConstraintMetadata::Time(#runtime::TimeConstraint::new(#precision)))
        }
        ConstraintIr::Sequence { min, max, unique } => {
            let min = option_number(*min);
            let max = option_number(*max);
            let base = quote!(#runtime::SequenceConstraint::new(#min, #max, #unique));
            let value = if has_element {
                quote!(#base.with_element(element_selector))
            } else {
                base
            };
            quote!(#runtime::ConstraintMetadata::Sequence(#value))
        }
        ConstraintIr::Map { min, max } => {
            let min = option_number(*min);
            let max = option_number(*max);
            let key = if has_map_key {
                quote!(Some(map_key_selector))
            } else {
                quote!(None)
            };
            let value = if has_map_value {
                quote!(Some(map_value_selector))
            } else {
                quote!(None)
            };
            quote!(#runtime::ConstraintMetadata::Map(#runtime::MapConstraint::new(#min, #max).with_selectors(#key, #value)))
        }
    }
}

fn expand_selector_metadata(value: &SelectorIr, name: syn::Ident, runtime: &TokenStream) -> TokenStream {
    let position = match value.position {
        SelectorPositionIr::Element => quote!(#runtime::SelectorPosition::Element),
        SelectorPositionIr::MapKey => quote!(#runtime::SelectorPosition::MapKey),
        SelectorPositionIr::MapValue => quote!(#runtime::SelectorPosition::MapValue),
    };
    let constraints = value
        .constraints
        .iter()
        .map(|constraint| expand_constraint(constraint, false, false, false, runtime));
    let validators = value
        .validators
        .iter()
        .map(|validator| expand_validator(validator, runtime));
    let codec = value.codec.as_ref().map_or_else(|| quote!(None), |codec| {
        let reference = codec_reference_expression(codec, runtime);
        quote!({
            let reference: &'static #runtime::CodecReference = ::std::boxed::Box::leak(::std::boxed::Box::new(#reference));
            Some(::std::boxed::Box::leak(::std::boxed::Box::new(
                #runtime::CodecMetadata::new(reference, #runtime::CodecSource::Selector(#position)),
            )) as &'static #runtime::CodecMetadata)
        })
    });
    let redact = value.redact.as_ref().map_or_else(|| quote!(None), |redact| {
        let expression = redact_expression(redact, match value.position {
            SelectorPositionIr::Element => quote!(#runtime::RedactPosition::Element),
            SelectorPositionIr::MapKey => quote!(#runtime::RedactPosition::MapKey),
            SelectorPositionIr::MapValue => quote!(#runtime::RedactPosition::MapValue),
        }, runtime);
        quote!(Some(::std::boxed::Box::leak(::std::boxed::Box::new(#expression)) as &'static #runtime::RedactMetadata))
    });
    quote! {
        let selector_constraints: &'static [#runtime::ConstraintMetadata] = ::std::boxed::Box::leak(::std::vec![#(#constraints),*].into_boxed_slice());
        let selector_validators: &'static [#runtime::ValidatorMetadata] = ::std::boxed::Box::leak(::std::vec![#(#validators),*].into_boxed_slice());
        let selector_codec = #codec;
        let selector_redact = #redact;
        let #name: &'static #runtime::SelectorMetadata = ::std::boxed::Box::leak(::std::boxed::Box::new(
            #runtime::SelectorMetadata::new(#position, selector_constraints, selector_validators, selector_codec, selector_redact),
        ));
    }
}

fn option_number<T: quote::ToTokens>(value: Option<T>) -> TokenStream {
    value.map_or_else(|| quote!(None), |value| quote!(Some(#value)))
}

fn rounding_tokens(value: &str, runtime: &TokenStream) -> TokenStream {
    match value {
        "down" => quote!(#runtime::RoundingMode::Down),
        "up" => quote!(#runtime::RoundingMode::Up),
        "ceiling" => quote!(#runtime::RoundingMode::Ceiling),
        "floor" => quote!(#runtime::RoundingMode::Floor),
        "half_up" => quote!(#runtime::RoundingMode::HalfUp),
        "half_down" => quote!(#runtime::RoundingMode::HalfDown),
        "half_even" => quote!(#runtime::RoundingMode::HalfEven),
        "unnecessary" => quote!(#runtime::RoundingMode::Unnecessary),
        _ => quote!(compile_error!("invalid rounding mode")),
    }
}

fn expand_validator(validator: &ValidatorIr, runtime: &TokenStream) -> TokenStream {
    let id = &validator.id;
    let params = validator.params.iter().map(|(name, value)| {
        let value = expand_strategy_argument(value, runtime);
        quote!(#runtime::NamedStrategyArgument::new(#name, #value))
    });
    let depends_on = validator.depends_on.iter().map(|path| expand_field_path(path, runtime));
    quote!({
        let params: &'static [#runtime::NamedStrategyArgument] = ::std::boxed::Box::leak(::std::vec![#(#params),*].into_boxed_slice());
        let depends_on: &'static [#runtime::PropertyPath] = ::std::boxed::Box::leak(::std::vec![#(#depends_on),*].into_boxed_slice());
        #runtime::ValidatorMetadata::new(#id, params, depends_on)
    })
}

fn expand_strategy_argument(value: &StrategyArgumentIr, runtime: &TokenStream) -> TokenStream {
    match value {
        StrategyArgumentIr::Bool(value) => quote!(#runtime::StrategyArgument::Bool(#value)),
        StrategyArgumentIr::Integer(value) => quote!(#runtime::StrategyArgument::Integer(#value)),
        StrategyArgumentIr::Unsigned(value) => quote!(#runtime::StrategyArgument::Unsigned(#value)),
        StrategyArgumentIr::String(value) => quote!(#runtime::StrategyArgument::String(#value)),
        StrategyArgumentIr::BoolList(values) => quote!(#runtime::StrategyArgument::BoolList(&[#(#values),*])),
        StrategyArgumentIr::IntegerList(values) => quote!(#runtime::StrategyArgument::IntegerList(&[#(#values),*])),
        StrategyArgumentIr::UnsignedList(values) => quote!(#runtime::StrategyArgument::UnsignedList(&[#(#values),*])),
        StrategyArgumentIr::StringList(values) => quote!(#runtime::StrategyArgument::StringList(&[#(#values),*])),
    }
}

fn expand_reference(reference: &ReferenceIr, runtime: &TokenStream) -> TokenStream {
    let target = match &reference.target {
        ReferenceTargetIr::RustType(ty) => {
            quote!(#runtime::DeclaredEntityTarget::RustType(<#ty as #runtime::HasTypeMetadata>::type_metadata))
        }
        ReferenceTargetIr::ModelId(id) => quote!(#runtime::DeclaredEntityTarget::ModelId(#runtime::ModelId::new(#id))),
    };
    let selection = reference.property.as_ref().map_or_else(
        || quote!(#runtime::ReferenceSelection::Entity),
        |path| {
            let path = expand_field_path(path, runtime);
            quote!(#runtime::ReferenceSelection::Property(#path))
        },
    );
    let same_as = reference.same_as.as_ref().map_or_else(
        || quote!(None),
        |path| {
            let path = expand_field_path(path, runtime);
            quote!(Some(::std::boxed::Box::leak(::std::boxed::Box::new(#path)) as &'static #runtime::PropertyPath))
        },
    );
    let existing = reference.existing;
    quote! {
        let reference_target: &'static #runtime::DeclaredEntityTarget = ::std::boxed::Box::leak(::std::boxed::Box::new(#target));
        let reference_selection: &'static #runtime::ReferenceSelection = ::std::boxed::Box::leak(::std::boxed::Box::new(#selection));
        let reference: &'static #runtime::FieldReferenceMetadata = ::std::boxed::Box::leak(::std::boxed::Box::new(
            #runtime::FieldReferenceMetadata::new(reference_target, reference_selection, #existing, #same_as),
        ));
    }
}

fn expand_redact(redact: &RedactIr, position: TokenStream, runtime: &TokenStream) -> TokenStream {
    let expression = redact_expression(redact, position, runtime);
    quote! {
        let redact: &'static #runtime::RedactMetadata = ::std::boxed::Box::leak(::std::boxed::Box::new(#expression));
    }
}

fn redact_expression(redact: &RedactIr, position: TokenStream, runtime: &TokenStream) -> TokenStream {
    let (sensitivity, mode) = match &redact.mode {
        RedactModeIr::Level(level) => {
            let sensitivity = match level.as_str() {
                "low" => quote!(#runtime::Sensitivity::Low),
                "medium" => quote!(#runtime::Sensitivity::Medium),
                "high" => quote!(#runtime::Sensitivity::High),
                "secret" => quote!(#runtime::Sensitivity::Secret),
                _ => quote!(compile_error!("redact level must be low, medium, high, or secret")),
            };
            (quote!(Some(#sensitivity)), quote!(#runtime::RedactModeMetadata::Level))
        }
        RedactModeIr::Skip => (quote!(None), quote!(#runtime::RedactModeMetadata::Skip)),
        RedactModeIr::Nested => (quote!(None), quote!(#runtime::RedactModeMetadata::Nested)),
        RedactModeIr::Map => (quote!(None), quote!(#runtime::RedactModeMetadata::Map)),
        RedactModeIr::KeyedBy(field) => (quote!(None), quote!(#runtime::RedactModeMetadata::KeyedBy(#field))),
        RedactModeIr::Json => (quote!(None), quote!(#runtime::RedactModeMetadata::Json)),
    };
    quote!(#runtime::RedactMetadata::new(#sensitivity, #mode, #position))
}

fn codec_reference_expression(codec: &CodecIr, runtime: &TokenStream) -> TokenStream {
    match codec {
        CodecIr::DeclaredId(id) => quote!(#runtime::CodecReference::DeclaredId(#id)),
        CodecIr::RustType(ty) => {
            quote!(#runtime::CodecReference::RustType(#runtime::StrategyTypeIdentity::of::<#ty>()))
        }
    }
}

fn expand_serde(value: &SerdeIr, runtime: &TokenStream) -> TokenStream {
    let serialize_name = option_lit_str(&value.serialize_name);
    let deserialize_name = option_lit_str(&value.deserialize_name);
    let skip_serializing = value.skip_serializing;
    let skip_deserializing = value.skip_deserializing;
    let flatten = value.flatten;
    let with = option_lit_str(&value.with);
    let default = value.default;
    let default_source = if value.default_from_model {
        quote!(#runtime::SerdeBehaviorSource::ModelDefault)
    } else if value.default {
        quote!(#runtime::SerdeBehaviorSource::Explicit)
    } else {
        quote!(#runtime::SerdeBehaviorSource::None)
    };
    let omit_source = if value.omit_from_model {
        quote!(#runtime::SerdeBehaviorSource::ModelDefault)
    } else if value.omit_suppressed {
        quote!(#runtime::SerdeBehaviorSource::Suppressed)
    } else if value.explicit_skip_serializing_if {
        quote!(#runtime::SerdeBehaviorSource::Explicit)
    } else {
        quote!(#runtime::SerdeBehaviorSource::None)
    };
    quote! {
        let serde: &'static #runtime::SerdeFieldMetadata = ::std::boxed::Box::leak(::std::boxed::Box::new(
            #runtime::SerdeFieldMetadata::new(#serialize_name, #deserialize_name, #skip_serializing, #skip_deserializing, #flatten, #with, #default)
                .with_sources(#default_source, #omit_source),
        ));
    }
}

fn expand_field_path(path: &[String], runtime: &TokenStream) -> TokenStream {
    quote!(#runtime::PropertyPath::new(&[#(#path),*]))
}

fn option_lit_str(value: &Option<LitStr>) -> TokenStream {
    value
        .as_ref()
        .map_or_else(|| quote!(None), |value| quote!(Some(#value)))
}

fn expand_role(declaration: &DeclarationIr, runtime: &TokenStream) -> TokenStream {
    match declaration.kind {
        MacroKind::Entity => {
            let index = identifier_index(&declaration.fields);
            quote! {
                let role: &'static #runtime::RoleMetadata = ::std::boxed::Box::leak(::std::boxed::Box::new(
                    #runtime::RoleMetadata::Entity(#runtime::EntityMetadata::new(&fields[#index])),
                ));
            }
        }
        MacroKind::Projection => {
            let index = identifier_index(&declaration.fields);
            let source = if let Some(source) = declaration.options.source.as_ref() {
                quote!(Some(::std::boxed::Box::leak(::std::boxed::Box::new(
                    #runtime::DeclaredEntityTarget::RustType(<#source as #runtime::HasTypeMetadata>::type_metadata),
                )) as &'static #runtime::DeclaredEntityTarget))
            } else if let Some(id) = declaration.options.source_id.as_ref() {
                quote!(Some(::std::boxed::Box::leak(::std::boxed::Box::new(
                    #runtime::DeclaredEntityTarget::ModelId(#runtime::ModelId::new(#id)),
                )) as &'static #runtime::DeclaredEntityTarget))
            } else {
                quote!(None)
            };
            quote! {
                let source = #source;
                let role: &'static #runtime::RoleMetadata = ::std::boxed::Box::leak(::std::boxed::Box::new(
                    #runtime::RoleMetadata::Projection(#runtime::ProjectionMetadata::new(&fields[#index], source)),
                ));
            }
        }
        MacroKind::Model => quote! {
            let role: &'static #runtime::RoleMetadata = ::std::boxed::Box::leak(::std::boxed::Box::new(
                #runtime::RoleMetadata::Model(#runtime::ModelMetadata),
            ));
        },
        MacroKind::Value => {
            let transparent = if declaration.options.transparent {
                quote!(Some(&fields[0]))
            } else {
                quote!(None)
            };
            let canonical_codec = declaration.options.codec.as_ref().map_or_else(
                || quote!(None),
                |codec_type| {
                    let value_type = codec_value_type(&declaration.fields[0].ty);
                    quote!({
                        fn assert_codec<C, V>()
                        where
                            C: ::core::default::Default
                                + #runtime::__private::qubit_codec::ValueEncoder<V, Output = ::std::string::String>
                                + #runtime::__private::qubit_codec::ValueDecoder<str, Output = V>,
                        {}
                        assert_codec::<#codec_type, #value_type>();
                        let reference: &'static #runtime::CodecReference = ::std::boxed::Box::leak(::std::boxed::Box::new(
                            #runtime::CodecReference::RustType(#runtime::StrategyTypeIdentity::of::<#codec_type>()),
                        ));
                        Some(::std::boxed::Box::leak(::std::boxed::Box::new(
                            #runtime::CodecMetadata::new(reference, #runtime::CodecSource::CanonicalValue),
                        )) as &'static #runtime::CodecMetadata)
                    })
                },
            );
            quote! {
                let canonical_codec = #canonical_codec;
                let role: &'static #runtime::RoleMetadata = ::std::boxed::Box::leak(::std::boxed::Box::new(
                    #runtime::RoleMetadata::Value(#runtime::ValueMetadata::new(#transparent, canonical_codec)),
                ));
            }
        }
        MacroKind::Enum => expand_enum_role(&declaration.variants, runtime),
        MacroKind::ModelProperties => unreachable!(),
    }
}

fn codec_value_type(ty: &Type) -> &Type {
    let Type::Path(path) = ty else { return ty };
    let Some(segment) = path.path.segments.last() else {
        return ty;
    };
    if segment.ident != "Option" {
        return ty;
    }
    let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return ty;
    };
    arguments
        .args
        .iter()
        .find_map(|argument| match argument {
            syn::GenericArgument::Type(ty) => Some(ty),
            _ => None,
        })
        .unwrap_or(ty)
}

fn expand_enum_role(variants: &[VariantIr], runtime: &TokenStream) -> TokenStream {
    let variants = variants.iter().enumerate().map(|(variant_index, variant)| {
        let fields = expand_field_vector(
            &variant.fields,
            quote!(descriptor.variants()[#variant_index].fields()),
            runtime,
        );
        let canonical = &variant.canonical_name;
        let serialized = &variant.serialized_name;
        let deserialized = &variant.deserialized_name;
        let default = variant.default;
        let rust_name = &variant.rust_name;
        quote! {
            {
                #fields
                let fields: &'static [#runtime::FieldMetadata] = ::std::boxed::Box::leak(fields.into_boxed_slice());
                let reflect = &descriptor.variants()[#variant_index];
                debug_assert_eq!(reflect.rust_name(), #rust_name);
                variants.push(#runtime::EnumVariantMetadata::new(
                    reflect,
                    #canonical,
                    #serialized,
                    #deserialized,
                    fields,
                    #default,
                ));
            }
        }
    });
    quote! {
        let mut variants = ::std::vec::Vec::new();
        #(#variants)*
        let variants: &'static [#runtime::EnumVariantMetadata] =
            ::std::boxed::Box::leak(variants.into_boxed_slice());
        let enum_metadata = #runtime::EnumMetadata::new(variants);
        let role: &'static #runtime::RoleMetadata = ::std::boxed::Box::leak(::std::boxed::Box::new(
            #runtime::RoleMetadata::Enum(enum_metadata),
        ));
        let fields: &'static [#runtime::FieldMetadata] = &[];
    }
}

fn identifier_index(fields: &[FieldIr]) -> usize {
    fields
        .iter()
        .position(|field| {
            field
                .occurrences
                .iter()
                .any(|value| matches!(value, FieldOccurrence::Identifier(_)))
        })
        .expect("validated role requires an identifier")
}

fn expand_registration(ident: &syn::Ident, runtime: &TokenStream) -> TokenStream {
    let snake_name = ident.to_string().to_snake_case();
    let source_fn = format_ident!("__qubit_model_source_{}", snake_name);
    let registration_fn = format_ident!("__qubit_model_registration_{}", snake_name);
    let fingerprint = stable_fingerprint(&ident.to_string());
    quote! {
        #[doc(hidden)]
        fn #source_fn() -> &'static #runtime::identity::FragmentIdentity {
            static SOURCE: ::std::sync::OnceLock<#runtime::identity::FragmentIdentity> = ::std::sync::OnceLock::new();
            SOURCE.get_or_init(|| #runtime::identity::FragmentIdentity::new(
                env!("CARGO_PKG_NAME"),
                module_path!(),
                line!(),
                column!(),
                "model",
                #fingerprint,
            ))
        }

        #[doc(hidden)]
        fn #registration_fn() -> #runtime::ModelRegistration {
            #runtime::ModelRegistration::from_concrete(
                <#ident as #runtime::HasTypeMetadata>::type_metadata(),
                #source_fn(),
            )
        }

        #runtime::__private::inventory::submit! {
            #runtime::ModelRegistrationFactory(#registration_fn)
        }
    }
}

fn stable_fingerprint(value: &str) -> u64 {
    value.bytes().fold(0xcbf29ce484222325_u64, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    })
}

fn validate_declaration(kind: MacroKind, item: &DeriveInput) -> Result<()> {
    let mut errors = None;
    for parameter in &item.generics.params {
        if matches!(parameter, GenericParam::Lifetime(_)) {
            combine(
                &mut errors,
                Error::new_spanned(parameter, "model roles do not support lifetime parameters"),
            );
        }
        if let GenericParam::Const(parameter) = parameter {
            let supported = matches!(&parameter.ty, Type::Path(path)
            if path.qself.is_none()
                && path.path.segments.len() == 1
                && matches!(
                    path.path.segments[0].ident.to_string().as_str(),
                    "bool" | "char" | "i8" | "i16" | "i32" | "i64" | "i128" | "isize"
                        | "u8" | "u16" | "u32" | "u64" | "u128" | "usize"
                ));
            if !supported {
                combine(
                    &mut errors,
                    Error::new_spanned(
                        &parameter.ty,
                        "model const parameters require a primitive integer, bool, or char type",
                    ),
                );
            }
        }
    }
    if matches!(kind, MacroKind::Entity | MacroKind::Projection)
        && (!item.generics.params.is_empty() || item.generics.where_clause.is_some())
    {
        combine(
            &mut errors,
            Error::new_spanned(&item.generics, "Entity and Projection declarations cannot be generic"),
        );
    }

    match (kind, &item.data) {
        (MacroKind::Entity | MacroKind::Projection, Data::Struct(data)) => {
            if !matches!(data.fields, Fields::Named(_)) {
                combine(
                    &mut errors,
                    Error::new_spanned(&data.fields, "Entity and Projection require named fields"),
                );
            }
        }
        (MacroKind::Model, Data::Struct(data)) => {
            if matches!(data.fields, Fields::Unnamed(_)) {
                combine(
                    &mut errors,
                    Error::new_spanned(&data.fields, "Model does not support tuple structs"),
                );
            }
        }
        (MacroKind::Enum, Data::Enum(_)) => {}
        (MacroKind::Value, Data::Struct(data)) => {
            let valid_shape = match &data.fields {
                Fields::Named(fields) => !fields.named.is_empty(),
                Fields::Unnamed(fields) => fields.unnamed.len() == 1,
                Fields::Unit => false,
            };
            if !valid_shape {
                combine(
                    &mut errors,
                    Error::new_spanned(&data.fields, "Value requires named fields or one tuple field"),
                );
            }
        }
        (_, Data::Union(data)) => combine(
            &mut errors,
            Error::new_spanned(data.union_token, "model macros do not support unions"),
        ),
        (MacroKind::Enum, _) => combine(
            &mut errors,
            Error::new_spanned(&item.ident, "Enum only supports enum declarations"),
        ),
        (_, Data::Enum(_)) => combine(
            &mut errors,
            Error::new_spanned(&item.ident, "this model role requires a struct declaration"),
        ),
        _ => {}
    }

    if let Some(error) = errors { Err(error) } else { Ok(()) }
}

fn reject_duplicate_reflect(attributes: &[Attribute]) -> Result<()> {
    for attribute in attributes {
        if !attribute.path().is_ident("derive") {
            continue;
        }
        let derives = attribute.parse_args_with(Punctuated::<syn::Path, Token![,]>::parse_terminated)?;
        if let Some(path) = derives
            .iter()
            .find(|path| path.segments.last().is_some_and(|segment| segment.ident == "Reflect"))
        {
            return Err(Error::new_spanned(
                path,
                "model macros generate Reflect; remove the duplicate derive",
            ));
        }
    }
    Ok(())
}

fn rewrite_field_helpers(data: &mut Data, declaration: &DeclarationIr) {
    let fields: Vec<_> = match data {
        Data::Struct(data) => data.fields.iter_mut().zip(&declaration.fields).collect(),
        Data::Enum(data) => data
            .variants
            .iter_mut()
            .zip(&declaration.variants)
            .flat_map(|(variant, ir)| variant.fields.iter_mut().zip(&ir.fields))
            .collect(),
        Data::Union(_) => Vec::new(),
    };
    for (field, ir) in fields {
        let opaque = field.attrs.iter().any(|attribute| attribute.path().is_ident("opaque"));
        let element_level = ir.occurrences.iter().find_map(|occurrence| match occurrence {
            FieldOccurrence::Selector(SelectorIr {
                position: SelectorPositionIr::Element,
                redact: Some(RedactIr {
                    mode: RedactModeIr::Level(level),
                }),
                ..
            }) => Some(level.clone()),
            _ => None,
        });
        let map_key_level = ir.occurrences.iter().find_map(|occurrence| match occurrence {
            FieldOccurrence::Selector(SelectorIr {
                position: SelectorPositionIr::MapKey,
                redact: Some(RedactIr {
                    mode: RedactModeIr::Level(level),
                }),
                ..
            }) => Some(level.clone()),
            _ => None,
        });
        let map_value_level = ir.occurrences.iter().find_map(|occurrence| match occurrence {
            FieldOccurrence::Selector(SelectorIr {
                position: SelectorPositionIr::MapValue,
                redact: Some(RedactIr {
                    mode: RedactModeIr::Level(level),
                }),
                ..
            }) => Some(level.clone()),
            _ => None,
        });
        field.attrs.retain(|attribute| {
            if attribute.path().is_ident("redact") {
                !declaration.options.no_redact
            } else {
                !is_model_field_helper(attribute)
            }
        });
        if let Some(level) = element_level.or_else(|| map_value_level.clone().filter(|_| map_key_level.is_none())) {
            let level = LitStr::new(&level, proc_macro2::Span::call_site());
            field.attrs.push(parse_quote!(#[redact(level = #level)]));
        }
        if let Some(key_level) = map_key_level {
            let key_level = LitStr::new(&key_level, proc_macro2::Span::call_site());
            if let Some(value_level) = map_value_level {
                let value_level = LitStr::new(&value_level, proc_macro2::Span::call_site());
                field
                    .attrs
                    .push(parse_quote!(#[redact(map_key_level = #key_level, map_value_level = #value_level)]));
            } else {
                field.attrs.push(parse_quote!(#[redact(map_key_level = #key_level)]));
            }
        }
        if opaque {
            field.attrs.push(parse_quote!(#[reflect(opaque)]));
        }
    }
}

fn is_model_field_helper(attribute: &Attribute) -> bool {
    let Some(name) = attribute.path().get_ident().map(ToString::to_string) else {
        return false;
    };
    matches!(
        name.as_str(),
        "identifier"
            | "indexed"
            | "unique"
            | "reference"
            | "key_part"
            | "text"
            | "decimal"
            | "money"
            | "time"
            | "sequence"
            | "map"
            | "element"
            | "map_key"
            | "map_value"
            | "validator"
            | "codec"
            | "redact"
            | "opaque"
            | "keep_serializing"
    )
}

fn combine(errors: &mut Option<Error>, error: Error) {
    match errors {
        Some(current) => current.combine(error),
        None => *errors = Some(error),
    }
}
