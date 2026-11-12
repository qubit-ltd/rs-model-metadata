// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Preserves accessor existence predicates for the consuming compiler.

use proc_macro2::TokenStream;
use quote::quote;
use syn::Attribute;
use syn::Error;
use syn::Meta;
use syn::Result;
use syn::Token;
use syn::punctuated::Punctuated;

/// A configuration predicate evaluated only in the consuming crate.
#[derive(Clone)]
pub(crate) struct PresenceCondition {
    /// Normalized predicate, including `all()` for unconditional presence.
    predicate: TokenStream,
}

impl PresenceCondition {
    pub(crate) fn always() -> Self {
        Self { predicate: quote!(all()) }
    }
    pub(crate) fn is_unconditional(&self) -> bool {
        self.predicate.to_string().chars().filter(|character| !character.is_whitespace()).collect::<String>() == "all()"
    }
    /// Extracts all existence-changing attributes, returning syntax errors for
    /// malformed `cfg_attr` lists. Ordinary method attributes are ignored.
    ///
    /// # Parameters
    ///
    /// * `attributes` - The source attributes inspected for `cfg` and
    ///   `cfg_attr` predicates.
    ///
    /// # Returns
    ///
    /// A condition combining all existence predicates, including `all()` when
    /// none are present.
    ///
    /// # Errors
    ///
    /// Returns a syntax error when a `cfg_attr` list is malformed.
    pub(crate) fn from_attributes(attributes: &[Attribute]) -> Result<Self> {
        let predicates = attributes
            .iter()
            .map(|attribute| existence_predicate(&attribute.meta))
            .collect::<Result<Vec<_>>>()?;
        let predicates = predicates.into_iter().flatten();
        Ok(Self {
            predicate: quote!(all(#(#predicates),*)),
        })
    }

    /// Returns an attribute guarding a generated item or metadata statement.
    #[must_use]
    pub(crate) fn attribute(&self) -> TokenStream {
        let predicate = &self.predicate;
        quote!(#[cfg(#predicate)])
    }

    #[must_use]
    pub(crate) fn expression(&self) -> TokenStream {
        let predicate = &self.predicate;
        quote!(cfg!(#predicate))
    }

    #[must_use]
    pub(crate) fn predicate(&self) -> TokenStream {
        self.predicate.clone()
    }

    /// Returns the condition under which both candidates exist.
    ///
    /// # Parameters
    ///
    /// * `other` - The second condition whose predicate must also hold.
    ///
    /// # Returns
    ///
    /// The combined condition requiring both this predicate and `other`'s
    /// predicate.
    #[must_use]
    pub(crate) fn intersection(&self, other: &Self) -> Self {
        let left = &self.predicate;
        let right = &other.predicate;
        Self {
            predicate: quote!(all(#left, #right)),
        }
    }

    #[must_use]
    pub(crate) fn union(&self, other: &Self) -> Self {
        let left = &self.predicate;
        let right = &other.predicate;
        Self { predicate: quote!(any(#left, #right)) }
    }
}

/// Returns the existence predicate of a `cfg` or recursively nested
/// `cfg_attr`, or `None` for attributes that do not affect existence.
/// Malformed conditional attribute lists return a source-anchored error.
///
/// # Parameters
///
/// * `meta` - Attribute metadata inspected for `cfg` or recursively nested
///   `cfg_attr`.
///
/// # Returns
///
/// The extracted existence predicate, or `None` when the attribute does not
/// affect item presence.
///
/// # Errors
///
/// Returns a source-anchored syntax error when a conditional attribute list is
/// malformed.
fn existence_predicate(meta: &Meta) -> Result<Option<TokenStream>> {
    let Meta::List(list) = meta else {
        return Ok(None);
    };
    if list.path.is_ident("cfg") {
        return Ok(Some(list.tokens.clone()));
    }
    if !list.path.is_ident("cfg_attr") {
        return Ok(None);
    }
    let arguments = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
    let mut arguments = arguments.iter();
    let trigger = arguments
        .next()
        .ok_or_else(|| Error::new_spanned(list, "cfg_attr requires a predicate"))?;
    let conditions = arguments.map(existence_predicate).collect::<Result<Vec<_>>>()?;
    let conditions: Vec<_> = conditions.into_iter().flatten().collect();
    if conditions.is_empty() {
        return Ok(None);
    }
    Ok(Some(quote!(any(not(#trigger), all(#(#conditions),*)))))
}
