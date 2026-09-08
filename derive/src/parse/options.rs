// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Parses options shared by the five model declaration roles.

use std::collections::HashSet;

use proc_macro2::Span;
use quote::quote;
use syn::Error;
use syn::Meta;
use syn::Path;
use syn::Result;
use syn::Token;
use syn::parse2;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;

use super::fields::set_lit_str;
use crate::compiler::diagnostics::Diagnostics;
use crate::ir::declaration::DeclarationOptions;

impl DeclarationOptions {
    /// Parses declaration-level options and rejects duplicates or bad values.
    pub(crate) fn parse(options: Punctuated<Meta, Token![,]>) -> Result<Self> {
        let mut result = Self {
            id: None,
            source: None,
            source_id: None,
            open: false,
            transparent: false,
            codec: None,
        };
        let mut diagnostics = Diagnostics::default();
        let mut markers = HashSet::new();
        for option in options {
            match option {
                Meta::NameValue(value) if value.path.is_ident("id") => {
                    if let Err(error) = set_lit_str(&mut result.id, value.value, "id") {
                        diagnostics.push(error);
                    }
                }
                Meta::NameValue(value) if value.path.is_ident("source_id") => {
                    if let Err(error) = set_lit_str(&mut result.source_id, value.value, "source_id") {
                        diagnostics.push(error);
                    }
                }
                Meta::NameValue(value) if value.path.is_ident("source") => {
                    if result.source.is_some() {
                        diagnostics.push(Error::new_spanned(value, "duplicate `source` option"));
                        continue;
                    }
                    let expression = value.value;
                    match parse2(quote!(#expression)) {
                        Ok(value) => result.source = Some(value),
                        Err(error) => diagnostics.push(error),
                    }
                }
                Meta::NameValue(value) if value.path.is_ident("codec") => {
                    if result.codec.is_some() {
                        diagnostics.push(Error::new_spanned(value, "duplicate `codec` option"));
                        continue;
                    }
                    let expression = value.value;
                    match parse2(quote!(#expression)) {
                        Ok(value) => result.codec = Some(value),
                        Err(error) => diagnostics.push(error),
                    }
                }
                Meta::Path(path) if path.is_ident("open") => {
                    set_marker_option(&mut markers, &mut diagnostics, "open", &mut result.open, path.span())
                }
                Meta::Path(path) if path.is_ident("transparent") => set_marker_option(
                    &mut markers,
                    &mut diagnostics,
                    "transparent",
                    &mut result.transparent,
                    path.span(),
                ),
                Meta::Path(path) if is_behavior_option(&path) => {
                    let name = path.get_ident().expect("behavior option identifier");
                    diagnostics.push(Error::new_spanned(
                        &path,
                        format!("unsupported model option '{name}'; derive Rust traits explicitly"),
                    ));
                }
                other => {
                    diagnostics.push(Error::new_spanned(other, "unsupported model option"));
                }
            }
        }
        diagnostics.finish()?;
        Ok(result)
    }
}

/// Returns whether `path` names a removed behavior-generating option.
fn is_behavior_option(path: &Path) -> bool {
    const OPTIONS: &[&str] = &[
        "no_clone",
        "no_debug",
        "no_display",
        "no_partial_eq",
        "no_eq",
        "no_hash",
        "no_serialize",
        "no_deserialize",
        "no_redact",
        "no_copy",
        "copy",
        "default",
        "partial_ord",
        "ord",
    ];
    OPTIONS.iter().any(|name| path.is_ident(name))
}

/// Records one declaration marker while preserving the second occurrence span.
fn set_marker_option(
    markers: &mut HashSet<&'static str>,
    diagnostics: &mut Diagnostics,
    name: &'static str,
    value: &mut bool,
    span: Span,
) {
    if !markers.insert(name) {
        diagnostics.push(Error::new(span, format!("duplicate `{name}` option")));
    } else {
        *value = true;
    }
}

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use quote::quote;
    use syn::Meta;
    use syn::Token;
    use syn::parse::Parser;
    use syn::punctuated::Punctuated;

    use crate::ir::declaration::DeclarationOptions;

    /// Covers every supported declaration option and its stored representation.
    #[test]
    fn test_parse_all_declaration_options() {
        let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
        let options = parser
            .parse2(quote!(
                id = "example.Model",
                source_id = "example.Source",
                source = Source,
                codec = Codec,
                open,
                transparent
            ))
            .expect("option syntax");
        let parsed = DeclarationOptions::parse(options).expect("supported options");

        assert_eq!(parsed.id.expect("id").value(), "example.Model");
        assert_eq!(parsed.source_id.expect("source id").value(), "example.Source");
        assert!(parsed.source.is_some());
        assert!(parsed.codec.is_some());
        assert!(parsed.open && parsed.transparent);
    }

    /// Confirms legacy behavior switches direct callers to explicit derives.
    #[test]
    fn test_reject_behavior_options() {
        let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
        for name in ["no_hash", "copy", "default", "partial_ord", "ord"] {
            let option: TokenStream = name.parse().expect("option tokens");
            let options = parser.parse2(option).expect("option syntax");
            let error = match DeclarationOptions::parse(options) {
                Ok(_) => panic!("behavior option must fail"),
                Err(error) => error,
            };
            assert!(
                error.to_string().contains(&format!(
                    "unsupported model option '{name}'; derive Rust traits explicitly"
                )),
                "unexpected error: {error}",
            );
        }
    }

    /// Confirms unsupported and duplicate options are accumulated as
    /// diagnostics.
    #[test]
    fn test_parse_declaration_option_errors() {
        let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
        let options = parser
            .parse2(quote!(
                id = "one",
                id = 2,
                source = One,
                source = Two,
                codec = A,
                codec = B,
                open,
                open,
                unknown
            ))
            .expect("option syntax");
        let error = match DeclarationOptions::parse(options) {
            Ok(_) => panic!("invalid options were accepted"),
            Err(error) => error,
        };
        let text = error.into_compile_error().to_string();

        assert!(text.contains("duplicate `id` option"));
        assert!(text.contains("duplicate `source` option"));
        assert!(text.contains("duplicate `codec` option"));
        assert!(text.contains("duplicate `open` option"));
        assert!(text.contains("unsupported model option"));
    }
}
