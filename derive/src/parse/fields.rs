// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Parses field attributes, constraints, selectors, and strategies.

use proc_macro2::TokenStream;
use syn::Attribute;
use syn::Error;
use syn::Expr;
use syn::ExprLit;
use syn::Lit;
use syn::LitBool;
use syn::LitInt;
use syn::LitStr;
use syn::Member;
use syn::Meta;
use syn::Path;
use syn::Result;
use syn::Token;
use syn::Type;
use syn::parenthesized;
use syn::parse::Parser;
use syn::parse_quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Paren;

use super::constraints::is_constraint_attribute;
use super::constraints::parse_constraint;
use super::validator::parse_validator;
use super::vocabulary::validate_redact_level;
use crate::compiler::diagnostics::Diagnostics;
use crate::ir::Located;
use crate::ir::declaration::CodecIr;
use crate::ir::declaration::FieldIr;
use crate::ir::declaration::FieldOccurrence;
use crate::ir::declaration::IdentifierAssignmentIr;
use crate::ir::declaration::RedactIr;
use crate::ir::declaration::RedactModeIr;
use crate::ir::declaration::ReferenceIr;
use crate::ir::declaration::ReferenceTargetIr;
use crate::ir::declaration::SelectorIr;
use crate::ir::declaration::SelectorPositionIr;
use crate::ir::declaration::SerdeIr;
use crate::ir::declaration::UniqueIr;

/// Parses one field's attributes into normalized intermediate metadata.
///
/// # Parameters
/// - `index`: zero-based position of the field in its declaration.
/// - `ty`: declared Rust type, retained with its source span.
/// - `attributes`: attributes to normalize in source order.
/// - `named`: whether the field has a name rather than a tuple position.
///
/// # Returns
/// The normalized field metadata, preserving occurrence order.
///
/// # Errors
/// Returns collected diagnostics when an attribute is malformed or duplicated.
pub(crate) fn parse_field(index: usize, ty: &Type, attributes: &[Attribute], named: bool) -> Result<FieldIr> {
    let mut occurrences = Vec::new();
    let mut keep_serializing = false;
    let mut identifier = false;
    let mut indexed = false;
    let mut opaque = false;
    let mut diagnostics = Diagnostics::default();
    for attribute in attributes {
        let result = if attribute.path().is_ident("identifier") {
            if identifier {
                Err(Error::new_spanned(attribute, "duplicate identifier marker"))
            } else {
                identifier = true;
                parse_identifier(attribute).map(|value| occurrences.push(FieldOccurrence::Identifier(value)))
            }
        } else if attribute.path().is_ident("indexed") {
            if !matches!(attribute.meta, Meta::Path(_)) {
                Err(Error::new_spanned(attribute, "indexed is a marker without arguments"))
            } else if indexed {
                Err(Error::new_spanned(attribute, "duplicate indexed marker"))
            } else {
                indexed = true;
                occurrences.push(FieldOccurrence::Indexed);
                Ok(())
            }
        } else if attribute.path().is_ident("unique") {
            parse_unique(attribute).map(|value| occurrences.push(FieldOccurrence::Unique(value)))
        } else if attribute.path().is_ident("reference") {
            parse_reference(attribute).map(|value| occurrences.push(FieldOccurrence::Reference(value)))
        } else if attribute.path().is_ident("key_part") {
            parse_key_part(attribute).map(|value| occurrences.push(FieldOccurrence::KeyPart(value)))
        } else if is_constraint_attribute(attribute) {
            parse_constraint(attribute).map(|value| occurrences.push(FieldOccurrence::Constraint(value)))
        } else if attribute.path().is_ident("element") {
            parse_selector(attribute, SelectorPositionIr::Element)
                .map(|value| occurrences.push(FieldOccurrence::Selector(value)))
        } else if attribute.path().is_ident("map_key") {
            parse_selector(attribute, SelectorPositionIr::MapKey)
                .map(|value| occurrences.push(FieldOccurrence::Selector(value)))
        } else if attribute.path().is_ident("map_value") {
            parse_selector(attribute, SelectorPositionIr::MapValue)
                .map(|value| occurrences.push(FieldOccurrence::Selector(value)))
        } else if attribute.path().is_ident("validator") {
            parse_validator(attribute).map(|value| occurrences.push(FieldOccurrence::Validator(value)))
        } else if attribute.path().is_ident("codec") {
            parse_codec(attribute).map(|value| occurrences.push(FieldOccurrence::Codec(value)))
        } else if attribute.path().is_ident("redact") {
            parse_redact(attribute).map(|value| occurrences.push(FieldOccurrence::Redact(value)))
        } else if attribute.path().is_ident("serde") {
            parse_serde(attribute).map(|value| occurrences.push(FieldOccurrence::Serde(value)))
        } else if attribute.path().is_ident("opaque") {
            if !matches!(attribute.meta, Meta::Path(_)) {
                Err(Error::new_spanned(attribute, "opaque is a marker without arguments"))
            } else if opaque {
                Err(Error::new_spanned(attribute, "duplicate opaque marker"))
            } else {
                opaque = true;
                occurrences.push(FieldOccurrence::Opaque);
                Ok(())
            }
        } else if attribute.path().is_ident("validate_nested") {
            Err(Error::new_spanned(
                attribute,
                "remove validate_nested; consumers traverse nested metadata and honor opaque boundaries",
            ))
        } else if attribute.path().is_ident("keep_serializing") {
            if !matches!(attribute.meta, Meta::Path(_)) {
                Err(Error::new_spanned(
                    attribute,
                    "keep_serializing is a marker without arguments",
                ))
            } else if keep_serializing {
                Err(Error::new_spanned(attribute, "duplicate keep_serializing marker"))
            } else {
                keep_serializing = true;
                Ok(())
            }
        } else {
            Ok(())
        };
        if let Err(error) = result {
            diagnostics.push(error);
        }
    }
    diagnostics.finish()?;
    Ok(FieldIr {
        index: Located::new(index, ty.span()),
        ty: ty.clone(),
        occurrences,
        variant_index: None,
        keep_serializing,
        named,
        rust_name: None,
        visibility: syn::Visibility::Inherited,
    })
}

/// Parses an identifier assignment attribute.
///
/// # Parameters
/// - `attribute`: the identifier marker and optional assignment policy.
///
/// # Returns
/// The selected application- or database-assignment policy.
///
/// # Errors
/// Returns an error for unsupported, duplicate, or missing assignment options.
fn parse_identifier(attribute: &Attribute) -> Result<IdentifierAssignmentIr> {
    if matches!(attribute.meta, Meta::Path(_)) {
        return Ok(IdentifierAssignmentIr::Application);
    }
    let mut assignment = None;
    attribute.parse_nested_meta(|meta| {
        if !meta.path.is_ident("assigned_by") {
            return Err(meta.error("unsupported identifier option"));
        }
        if assignment.is_some() {
            return Err(meta.error("duplicate identifier `assigned_by` option"));
        }
        let value = parse_ident_value(meta.value()?.parse()?)?;
        assignment = Some(match value.as_str() {
            "application" => IdentifierAssignmentIr::Application,
            "database" => IdentifierAssignmentIr::Database,
            _ => {
                return Err(meta.error("assigned_by must be application or database"));
            }
        });
        Ok(())
    })?;
    assignment.ok_or_else(|| Error::new_spanned(attribute, "identifier requires assigned_by"))
}

/// Sets a string option while rejecting duplicate declarations.
///
/// # Parameters
/// - `slot`: destination for the parsed string, updated only on success.
/// - `value`: expression that must be a string literal.
/// - `name`: option name used in diagnostics.
///
/// # Returns
/// `()` after storing the string.
///
/// # Errors
/// Returns an error if the slot is already set or `value` is not a string.
pub(crate) fn set_lit_str(slot: &mut Option<LitStr>, value: Expr, name: &str) -> Result<()> {
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

/// Parses a uniqueness declaration and its comparison paths.
///
/// # Parameters
/// - `attribute`: the uniqueness marker and its comparison options.
///
/// # Returns
/// The case policy and ordered comparison paths.
///
/// # Errors
/// Returns diagnostics for malformed or unsupported options.
fn parse_unique(attribute: &Attribute) -> Result<UniqueIr> {
    let mut value = UniqueIr {
        respect_to: Vec::new(),
        ignore_case: None,
    };
    if matches!(attribute.meta, Meta::Path(_)) {
        return Ok(value);
    }
    let mut diagnostics = Diagnostics::default();
    let mut saw_ignore_case = false;
    attribute.parse_nested_meta(|meta| {
        if meta.path.is_ident("respect_to") {
            meta.parse_nested_meta(|path| {
                value.respect_to.push(path_from_syn(&path.path));
                Ok(())
            })
        } else if meta.path.is_ident("ignore_case") {
            let parsed = meta.value()?.parse::<LitBool>()?.value;
            if saw_ignore_case {
                diagnostics.push(meta.error("duplicate unique `ignore_case` option"));
                return Ok(());
            }
            saw_ignore_case = true;
            value.ignore_case = Some(parsed);
            Ok(())
        } else {
            Err(meta.error("unsupported unique option"))
        }
    })?;
    diagnostics.finish()?;
    Ok(value)
}

/// Parses a relationship declaration and target selector.
///
/// # Parameters
/// - `attribute`: the relationship marker and target/options.
///
/// # Returns
/// The target, optional property path, existence policy, and navigation path.
///
/// # Errors
/// Returns diagnostics when the target or any option is invalid or duplicated.
fn parse_reference(attribute: &Attribute) -> Result<ReferenceIr> {
    let mut target = None;
    let mut property = None;
    let mut existing = true;
    let mut same_as = None;
    let mut diagnostics = Diagnostics::default();
    let mut saw_property = false;
    let mut saw_path = false;
    let mut saw_existing = false;
    attribute.parse_nested_meta(|meta| {
        if meta.path.is_ident("entity") {
            let value = meta.value()?;
            let entity_target = if value.peek(LitStr) {
                let id: LitStr = value.parse()?;
                validate_ascii_id(&id, "reference entity ID")?;
                ReferenceTargetIr::ModelId(id)
            } else {
                ReferenceTargetIr::RustType(Box::new(value.parse()?))
            };
            if target.replace(entity_target).is_some() {
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
            if saw_property {
                diagnostics.push(meta.error("duplicate reference `property` option"));
                return Ok(());
            }
            saw_property = true;
            property = Some(parse_path_value(meta.value()?.parse()?)?);
            Ok(())
        } else if meta.path.is_ident("path") {
            if saw_path {
                diagnostics.push(meta.error("duplicate reference `path` option"));
                return Ok(());
            }
            saw_path = true;
            same_as = Some(parse_object_path(meta.value()?.parse()?)?);
            Ok(())
        } else if meta.path.is_ident("existing") {
            let parsed = meta.value()?.parse::<LitBool>()?.value;
            if saw_existing {
                diagnostics.push(meta.error("duplicate reference `existing` option"));
                return Ok(());
            }
            saw_existing = true;
            existing = parsed;
            Ok(())
        } else {
            Err(meta.error("unsupported reference option"))
        }
    })?;
    diagnostics.finish()?;
    let target = target.ok_or_else(|| Error::new_spanned(attribute, "reference requires `entity` or `entity_id`"))?;
    Ok(ReferenceIr {
        target,
        property,
        existing,
        same_as,
    })
}

/// Parses a zero-based composite-key position.
///
/// # Parameters
/// - `attribute`: the key-part marker containing its order.
///
/// # Returns
/// The zero-based key position.
///
/// # Errors
/// Returns an error when the order is missing, duplicated, or invalid.
fn parse_key_part(attribute: &Attribute) -> Result<usize> {
    let mut order = None;
    attribute.parse_nested_meta(|meta| {
        if meta.path.is_ident("order") {
            let value = meta.value()?.parse::<LitInt>()?.base10_parse()?;
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

/// Parses a selector and its nested constraints, validators, and redaction.
///
/// # Parameters
/// - `attribute`: selector options to parse.
/// - `position`: whether the selector addresses an element, map key, or map
///   value.
///
/// # Returns
/// The selector and its supported nested metadata.
///
/// # Errors
/// Returns an error for unsupported declarations or recursively nested
/// collection selectors.
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

/// Converts an identifier expression into its canonical path text.
///
/// # Parameters
/// - `expression`: an expression expected to contain a single identifier.
///
/// # Returns
/// The identifier's text.
///
/// # Errors
/// Returns an error unless the expression is a one-segment path.
pub(crate) fn parse_ident_value(expression: Expr) -> Result<String> {
    match expression {
        Expr::Path(path) if path.path.segments.len() == 1 => Ok(path.path.segments[0].ident.to_string()),
        other => Err(Error::new_spanned(other, "expected an identifier value")),
    }
}

/// Parses a declared codec ID or Rust codec type.
///
/// # Parameters
/// - `attribute`: the codec marker with either a type or declared ID.
///
/// # Returns
/// The codec type or declared ID.
///
/// # Errors
/// Returns an error if no valid codec reference is supplied.
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

/// Parses a field or selector redaction mode.
///
/// # Parameters
/// - `attribute`: the redaction marker and its selected mode.
///
/// # Returns
/// The normalized redaction mode.
///
/// # Errors
/// Returns an error if the mode is missing, duplicated, or unsupported.
fn parse_redact(attribute: &Attribute) -> Result<RedactIr> {
    let mut mode = None;
    attribute.parse_nested_meta(|meta| {
        let current = if meta.path.is_ident("level") {
            let level = meta.value()?.parse::<LitStr>()?;
            validate_redact_level(&level)?;
            RedactModeIr::Level(level.value())
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

/// Parses Serde rename, skip, flatten, default, and custom-handler options.
///
/// # Parameters
/// - `attribute`: the Serde helper attribute to inspect.
///
/// # Returns
/// The stable Serde metadata subset used by this crate.
///
/// # Errors
/// Returns a syntax error for malformed Serde option values.
pub(crate) fn parse_serde(attribute: &Attribute) -> Result<SerdeIr> {
    let mut serde = SerdeIr::default();
    let mut saw_rename = false;
    attribute.parse_nested_meta(|meta| {
        if meta.path.is_ident("rename") {
            if meta.input.peek(Token![=]) {
                let value: LitStr = meta.value()?.parse()?;
                if saw_rename {
                    return Err(meta.error("duplicate serde `rename` option"));
                }
                saw_rename = true;
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
            // Serde owns its wider syntax; metadata records only the stable
            // subset.
            if meta.input.peek(Token![=]) {
                let _: Expr = meta.value()?.parse()?;
            } else if meta.input.peek(Paren) {
                let nested;
                parenthesized!(nested in meta.input);
                let _: TokenStream = nested.parse()?;
            }
            Ok(())
        }
    })?;
    Ok(serde)
}

/// Parses a field path represented as identifiers or string segments.
///
/// # Parameters
/// - `expression`: a named field path, Rust path, or dotted string path.
///
/// # Returns
/// The path's ordered string segments.
///
/// # Errors
/// Returns an error for unsupported expressions or tuple-field segments.
pub(crate) fn parse_path_value(expression: Expr) -> Result<Vec<String>> {
    match expression {
        Expr::Field(field) => {
            let mut path = parse_path_value(*field.base)?;
            if let Member::Named(name) = field.member {
                path.push(name.to_string());
                Ok(path)
            } else {
                Err(Error::new_spanned(field.member, "property segments must be names"))
            }
        }
        Expr::Path(path) => Ok(path_from_syn(&path.path)),
        Expr::Lit(ExprLit {
            lit: Lit::Str(value), ..
        }) => Ok(value.value().split('.').map(str::to_owned).collect()),
        other => Err(Error::new_spanned(other, "expected an identifier path or string path")),
    }
}

/// Converts one path expression into its textual segment.
///
/// # Parameters
/// - `expression`: the path expression to normalize.
///
/// # Returns
/// The path segments joined with `.`.
///
/// # Errors
/// Returns the path parsing error when the expression is invalid.
fn path_text(expression: Expr) -> Result<String> {
    parse_path_value(expression).map(|segments| segments.join("."))
}

/// Converts a Syn path into owned identifier segments.
///
/// # Parameters
/// - `path`: the syntax-tree path to convert.
///
/// # Returns
/// The identifiers in source order, owned as strings.
#[must_use]
pub(crate) fn path_from_syn(path: &Path) -> Vec<String> {
    path.segments.iter().map(|segment| segment.ident.to_string()).collect()
}

/// Validates that a model or validator ID is non-empty ASCII text.
///
/// # Parameters
/// - `value`: the string literal containing the ID.
/// - `kind`: the ID category used to format diagnostics.
///
/// # Returns
/// `()` when every dot-separated segment has a valid identifier form.
///
/// # Errors
/// Returns an error when the ID is empty or contains an invalid segment.
pub(crate) fn validate_ascii_id(value: &LitStr, kind: &str) -> Result<()> {
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

/// Parses slash-separated object navigation independently of property paths.
///
/// # Parameters
/// - `value`: the slash-separated navigation path.
///
/// # Returns
/// The ordered path segments, including permitted `..` parent steps.
///
/// # Errors
/// Returns an error for empty segments, `.`, dotted names, or whitespace.
pub(crate) fn parse_object_path(value: LitStr) -> Result<Vec<String>> {
    let text = value.value();
    let steps: Vec<String> = text.split('/').map(str::to_owned).collect();
    if steps.iter().any(|step| {
        step.is_empty()
            || step == "."
            || (step != ".." && (step.contains('.') || step.chars().any(char::is_whitespace)))
    }) {
        return Err(Error::new_spanned(
            value,
            "object path requires slash-separated property names or .. parent steps",
        ));
    }
    Ok(steps)
}
