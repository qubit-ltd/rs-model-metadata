// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Shared support for public model declaration attributes.

use heck::ToShoutySnakeCase;
use syn::Attribute;
use syn::Error;
use syn::LitStr;
use syn::Result;
use syn::Token;
use syn::Variant;

/// Returns the canonical serialized name for one enum variant.
pub(crate) fn serialized_variant_name(variant: &Variant) -> Result<String> {
    let mut name = variant.ident.to_string().to_shouty_snake_case();
    for attribute in &variant.attrs {
        if !attribute.path().is_ident("serde") {
            continue;
        }
        attribute.parse_nested_meta(|meta| {
            if !meta.path.is_ident("rename") {
                return Ok(());
            }
            if meta.input.peek(Token![=]) {
                let value: LitStr = meta.value()?.parse()?;
                name = value.value();
                return Ok(());
            }
            meta.parse_nested_meta(|rename| {
                if rename.path.is_ident("serialize") {
                    let value: LitStr = rename.value()?.parse()?;
                    name = value.value();
                }
                Ok(())
            })
        })?;
    }
    if name.is_empty() {
        return Err(Error::new_spanned(
            &variant.ident,
            "serialized enum name cannot be empty",
        ));
    }
    Ok(name)
}

/// Returns whether an item already declares `must_use`.
pub(crate) fn has_must_use(attributes: &[Attribute]) -> bool {
    attributes.iter().any(|attribute| attribute.path().is_ident("must_use"))
}
