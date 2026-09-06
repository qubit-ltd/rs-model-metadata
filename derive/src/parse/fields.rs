// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Parses field attributes, constraints, selectors, and strategies.

use syn::Attribute;
use syn::Error;
use syn::Expr;
use syn::ExprLit;
use syn::Lit;
use syn::LitBool;
use syn::LitInt;
use syn::LitStr;
use syn::Meta;
use syn::Path;
use syn::Result;
use syn::Token;
use syn::Type;
use syn::parse::Parser;
use syn::parse_quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;

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

impl FieldIr {
    /// Parses one field's attributes into normalized intermediate metadata.
    pub(crate) fn parse(
        index: usize,
        ty: &Type,
        attributes: &[Attribute],
        named: bool,
    ) -> Result<Self> {
        let mut occurrences = Vec::new();
        let mut keep_serializing = false;
        let mut identifier = false;
        let mut indexed = false;
        let mut opaque = false;
        let mut validate_nested = false;
        let mut diagnostics = Diagnostics::default();
        for attribute in attributes {
            let result = if attribute.path().is_ident("identifier") {
                if identifier {
                    Err(Error::new_spanned(attribute, "duplicate identifier marker"))
                } else {
                    identifier = true;
                    parse_identifier(attribute)
                        .map(|value| occurrences.push(FieldOccurrence::Identifier(value)))
                }
            } else if attribute.path().is_ident("indexed") {
                if !matches!(attribute.meta, Meta::Path(_)) {
                    Err(Error::new_spanned(
                        attribute,
                        "indexed is a marker without arguments",
                    ))
                } else if indexed {
                    Err(Error::new_spanned(attribute, "duplicate indexed marker"))
                } else {
                    indexed = true;
                    occurrences.push(FieldOccurrence::Indexed);
                    Ok(())
                }
            } else if attribute.path().is_ident("unique") {
                parse_unique(attribute)
                    .map(|value| occurrences.push(FieldOccurrence::Unique(value)))
            } else if attribute.path().is_ident("reference") {
                parse_reference(attribute)
                    .map(|value| occurrences.push(FieldOccurrence::Reference(value)))
            } else if attribute.path().is_ident("key_part") {
                parse_key_part(attribute)
                    .map(|value| occurrences.push(FieldOccurrence::KeyPart(value)))
            } else if is_constraint_attribute(attribute) {
                parse_constraint(attribute)
                    .map(|value| occurrences.push(FieldOccurrence::Constraint(value)))
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
                parse_validator(attribute)
                    .map(|value| occurrences.push(FieldOccurrence::Validator(value)))
            } else if attribute.path().is_ident("codec") {
                parse_codec(attribute).map(|value| occurrences.push(FieldOccurrence::Codec(value)))
            } else if attribute.path().is_ident("redact") {
                parse_redact(attribute)
                    .map(|value| occurrences.push(FieldOccurrence::Redact(value)))
            } else if attribute.path().is_ident("serde") {
                parse_serde(attribute).map(|value| occurrences.push(FieldOccurrence::Serde(value)))
            } else if attribute.path().is_ident("opaque") {
                if !matches!(attribute.meta, Meta::Path(_)) {
                    Err(Error::new_spanned(
                        attribute,
                        "opaque is a marker without arguments",
                    ))
                } else if opaque {
                    Err(Error::new_spanned(attribute, "duplicate opaque marker"))
                } else {
                    opaque = true;
                    occurrences.push(FieldOccurrence::Opaque);
                    Ok(())
                }
            } else if attribute.path().is_ident("validate_nested") {
                if !matches!(attribute.meta, Meta::Path(_)) {
                    Err(Error::new_spanned(
                        attribute,
                        "validate_nested is a marker without arguments",
                    ))
                } else if validate_nested {
                    Err(Error::new_spanned(
                        attribute,
                        "duplicate validate_nested marker",
                    ))
                } else {
                    validate_nested = true;
                    occurrences.push(FieldOccurrence::ValidateNested);
                    Ok(())
                }
            } else if attribute.path().is_ident("keep_serializing") {
                if !matches!(attribute.meta, Meta::Path(_)) {
                    Err(Error::new_spanned(
                        attribute,
                        "keep_serializing is a marker without arguments",
                    ))
                } else if keep_serializing {
                    Err(Error::new_spanned(
                        attribute,
                        "duplicate keep_serializing marker",
                    ))
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
        Ok(Self {
            index: Located::new(index, ty.span()),
            ty: ty.clone(),
            occurrences,
            keep_serializing,
            named,
        })
    }
}

/// Parses an identifier assignment attribute.
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
pub(crate) fn set_lit_str(slot: &mut Option<LitStr>, value: Expr, name: &str) -> Result<()> {
    if slot.is_some() {
        return Err(Error::new_spanned(
            value,
            format!("duplicate `{name}` option"),
        ));
    }
    let Expr::Lit(ExprLit {
        lit: Lit::Str(value),
        ..
    }) = value
    else {
        return Err(Error::new_spanned(
            value,
            format!("`{name}` requires a string literal"),
        ));
    };
    *slot = Some(value);
    Ok(())
}

/// Parses a uniqueness declaration and its comparison paths.
fn parse_unique(attribute: &Attribute) -> Result<UniqueIr> {
    let mut value = UniqueIr {
        respect_to: Vec::new(),
        ignore_case: true,
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
            value.ignore_case = parsed;
            Ok(())
        } else {
            Err(meta.error("unsupported unique option"))
        }
    })?;
    diagnostics.finish()?;
    Ok(value)
}

/// Parses a relationship declaration and target selector.
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
            same_as = Some(parse_path_value(meta.value()?.parse()?)?);
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
    let target = target.ok_or_else(|| {
        Error::new_spanned(attribute, "reference requires `entity` or `entity_id`")
    })?;
    Ok(ReferenceIr {
        target,
        property,
        existing,
        same_as,
    })
}

/// Parses a zero-based composite-key position.
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
fn parse_selector(attribute: &Attribute, position: SelectorPositionIr) -> Result<SelectorIr> {
    let Meta::List(list) = &attribute.meta else {
        return Err(Error::new_spanned(
            attribute,
            "selector requires nested declarations",
        ));
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
                nested
                    .path()
                    .get_ident()
                    .map(ToString::to_string)
                    .as_deref(),
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
                return Err(Error::new_spanned(
                    nested,
                    "selector accepts one redact declaration",
                ));
            }
        } else {
            return Err(Error::new_spanned(
                nested,
                "unsupported selector declaration",
            ));
        }
    }
    Ok(selector)
}

/// Converts an identifier expression into its canonical path text.
pub(crate) fn parse_ident_value(expression: Expr) -> Result<String> {
    match expression {
        Expr::Path(path) if path.path.segments.len() == 1 => {
            Ok(path.path.segments[0].ident.to_string())
        }
        other => Err(Error::new_spanned(other, "expected an identifier value")),
    }
}

/// Parses a declared codec ID or Rust codec type.
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
    result.ok_or_else(|| {
        Error::new_spanned(attribute, "codec requires a Rust type or `id = \"...\"`")
    })
}

/// Parses a field or selector redaction mode.
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
            }
            Ok(())
        }
    })?;
    Ok(serde)
}

/// Parses a field path represented as identifiers or string segments.
fn parse_path_value(expression: Expr) -> Result<Vec<String>> {
    match expression {
        Expr::Path(path) => Ok(path_from_syn(&path.path)),
        Expr::Lit(ExprLit {
            lit: Lit::Str(value),
            ..
        }) => Ok(value.value().split('.').map(str::to_owned).collect()),
        other => Err(Error::new_spanned(
            other,
            "expected an identifier path or string path",
        )),
    }
}

/// Converts one path expression into its textual segment.
fn path_text(expression: Expr) -> Result<String> {
    parse_path_value(expression).map(|segments| segments.join("."))
}

/// Converts a Syn path into owned identifier segments.
pub(crate) fn path_from_syn(path: &Path) -> Vec<String> {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect()
}

/// Validates that a model or validator ID is non-empty ASCII text.
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

#[cfg(test)]
mod tests {
    use syn::Field;
    use syn::parse_quote;

    use crate::ir::declaration::FieldIr;

    /// Exercises the complete supported field-attribute vocabulary in source
    /// order.
    #[test]
    fn test_parse_field_attribute_vocabulary() {
        let field: Field = parse_quote! {
            #[identifier(assigned_by = database)]
            #[indexed]
            #[unique(respect_to(tenant::id), ignore_case = false)]
            #[reference(entity_id = "example.Owner", property = id, path = owner::id, existing = false)]
            #[key_part(order = 0)]
            #[text(min_chars = 1, max_chars = 8, non_blank, allowed_chars = ascii, format = email)]
            #[sequence(min_items = 1, max_items = 3, unique_items)]
            #[element(text(max_chars = 4), validator(id = "example.element"), codec(id = "example.codec"), redact(level = "low"))]
            #[validator(id = "example.field", params(limit = 3))]
            #[codec(type = Codec)]
            #[redact(keyed_by = owner::id)]
            #[serde(rename(serialize = "out", deserialize = "in"), skip_serializing, flatten, with = "helper", default = "make", skip_serializing_if = "skip", unknown = true)]
            #[opaque]
            #[keep_serializing]
            value: Vec<String>
        };
        let parsed =
            FieldIr::parse(3, &field.ty, &field.attrs, true).expect("supported field attributes");

        assert_eq!(*parsed.index.value(), 3);
        assert!(parsed.named);
        assert!(parsed.keep_serializing);
        assert_eq!(parsed.occurrences.len(), 13);
    }

    /// Confirms singleton field markers and redact levels are rejected early.
    #[test]
    fn test_rejects_duplicate_markers_and_unknown_redact_level() {
        let fields: [Field; 7] = [
            parse_quote!(#[identifier] #[identifier] value: String),
            parse_quote!(#[identifier(assigned_by = application, assigned_by = database)] value: String),
            parse_quote!(#[indexed] #[indexed] value: String),
            parse_quote!(#[opaque] #[opaque] value: String),
            parse_quote!(#[indexed(unexpected)] value: String),
            parse_quote!(#[opaque(unexpected)] value: String),
            parse_quote!(#[redact(level = "unsupported")] value: String),
        ];
        for field in fields {
            assert!(FieldIr::parse(0, &field.ty, &field.attrs, true).is_err());
        }
    }

    /// Covers invalid field options and marker shapes without panicking.
    #[test]
    fn test_parse_field_attribute_errors() {
        let fields: [Field; 8] = [
            parse_quote!(#[identifier(other)] value: String),
            parse_quote!(#[identifier(assigned_by = other)] value: String),
            parse_quote!(#[reference(property = id)] value: String),
            parse_quote!(#[key_part] value: String),
            parse_quote!(#[codec] value: String),
            parse_quote!(#[redact] value: String),
            parse_quote!(#[element(sequence(min_items = 1))] value: Vec<String>),
            parse_quote!(#[keep_serializing = true] value: String),
        ];

        for field in fields {
            assert!(FieldIr::parse(0, &field.ty, &field.attrs, true).is_err());
        }
    }
}
