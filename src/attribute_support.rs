// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Shared support for public model declaration attributes.

use syn::Attribute;
use syn::Error;
use syn::Expr;
use syn::Ident;
use syn::Lit;
use syn::LitStr;
use syn::Meta;
use syn::Result;
use syn::Token;
use syn::Variant;
use syn::ext::IdentExt;
use syn::punctuated::Punctuated;

/// Returns the canonical serialized name for one enum variant.
pub(crate) fn serialized_variant_name(variant: &Variant) -> Result<String> {
    let mut name = default_serialized_variant_name(&variant.ident);
    for attribute in &variant.attrs {
        if !attribute.path().is_ident("serde") {
            continue;
        }
        let attributes = attribute.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        for attribute in attributes {
            let Some(rename) = serialized_rename(&attribute)? else {
                continue;
            };
            name = rename;
        }
    }
    if name.is_empty() {
        return Err(Error::new_spanned(
            &variant.ident,
            "serialized enum name cannot be empty",
        ));
    }
    Ok(name)
}

/// Returns the implicit `SCREAMING_SNAKE_CASE` name that Serde applies to an
/// enum variant.
///
/// Serde inserts a separator before every uppercase Unicode scalar after the
/// first and performs ASCII-only case conversion. Keeping this implementation
/// aligned with Serde makes generated metadata and `name()` match the wire
/// representation for raw identifiers and acronym-heavy variant names.
fn default_serialized_variant_name(ident: &Ident) -> String {
    let variant = ident.unraw().to_string();
    let mut snake = String::with_capacity(variant.len());
    for (index, character) in variant.char_indices() {
        if index > 0 && character.is_uppercase() {
            snake.push('_');
        }
        snake.push(character.to_ascii_lowercase());
    }
    snake.to_ascii_uppercase()
}

/// Returns the serialization name selected by one Serde metadata item.
///
/// Non-rename Serde metadata is deliberately ignored after parsing so that it
/// remains available to Serde's derive macro without affecting model metadata.
///
/// # Parameters
///
/// - `attribute`: One item from a `#[serde(...)]` attribute.
///
/// # Returns
///
/// Returns `Some` for a serialization-facing `rename` value and `None` for
/// unrelated metadata or deserialize-only renames.
///
/// # Errors
///
/// Returns an error when a selected rename value is not a string literal or
/// when its nested metadata is syntactically invalid.
fn serialized_rename(attribute: &Meta) -> Result<Option<String>> {
    if !attribute.path().is_ident("rename") {
        return Ok(None);
    }
    match attribute {
        Meta::NameValue(value) => parse_string_value(&value.value).map(|value| Some(value.value())),
        Meta::List(list) => {
            let attributes = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
            for attribute in attributes {
                if !attribute.path().is_ident("serialize") {
                    continue;
                }
                if let Meta::NameValue(value) = attribute {
                    return parse_string_value(&value.value).map(|value| Some(value.value()));
                }
            }
            Ok(None)
        }
        Meta::Path(_) => Ok(None),
    }
}

/// Parses one Serde metadata value as a string literal.
///
/// # Parameters
///
/// - `value`: The expression supplied to Serde metadata such as `rename`.
///
/// # Returns
///
/// Returns the literal when `value` is a string literal.
///
/// # Errors
///
/// Returns an error at `value` when it is not a string literal.
fn parse_string_value(value: &Expr) -> Result<&LitStr> {
    let Expr::Lit(value) = value else {
        return Err(Error::new_spanned(value, "expected string literal"));
    };
    let Lit::Str(value) = &value.lit else {
        return Err(Error::new_spanned(value, "expected string literal"));
    };
    Ok(value)
}

/// Returns whether an item already declares `must_use`.
#[must_use]
#[inline(always)]
pub(crate) fn has_must_use(attributes: &[Attribute]) -> bool {
    attributes.iter().any(|attribute| attribute.path().is_ident("must_use"))
}
