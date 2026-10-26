// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! AST-based specialization of property signatures, preserving method bodies.

use proc_macro2::TokenStream;
use syn::Error;
use syn::Generics;
use syn::ImplItem;
use syn::ImplItemFn;
use syn::ItemImpl;
use syn::Meta;
use syn::Result;
use syn::Token;
use syn::Type;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::visit_mut::VisitMut;

use super::internal::Substitutions;

/// Applies explicit reflect-style specializations to property signature types.
///
/// # Errors
///
/// Returns an error when a binding is malformed, duplicated, or names an
/// unknown parameter, or when the specialization does not bind every impl
/// parameter.
pub(super) fn concrete_impls(item: &ItemImpl, options: &Punctuated<Meta, Token![,]>) -> Result<Vec<ItemImpl>> {
    let mut concrete = Vec::new();
    for option in options {
        let Meta::List(list) = option else {
            continue;
        };
        if !list.path.is_ident("specialize") {
            continue;
        }
        let mut substitutions = Substitutions::default();
        list.parse_nested_meta(|binding| {
            let name = binding
                .path
                .get_ident()
                .ok_or_else(|| binding.error("specialization parameter must be an identifier"))?
                .to_string();
            let value = binding.value()?;
            if item.generics.type_params().any(|parameter| parameter.ident == name) {
                if substitutions.types.insert(name, value.parse()?).is_some() {
                    return Err(binding.error("duplicate specialization parameter"));
                }
            } else if item.generics.const_params().any(|parameter| parameter.ident == name) {
                if substitutions.constants.insert(name, value.parse()?).is_some() {
                    return Err(binding.error("duplicate specialization parameter"));
                }
            } else {
                return Err(binding.error("unknown specialization parameter"));
            }
            Ok(())
        })?;
        if substitutions.types.len() + substitutions.constants.len() != item.generics.params.len() {
            return Err(Error::new_spanned(
                option,
                "specialization must bind every impl parameter",
            ));
        }
        let mut implementation = item.clone();
        substitutions.visit_type_mut(&mut implementation.self_ty);
        substitutions
            .types
            .insert("Self".into(), (*implementation.self_ty).clone());
        for member in &mut implementation.items {
            if let ImplItem::Fn(method) = member
                && method.sig.generics.params.is_empty()
            {
                substitutions.visit_signature_mut(&mut method.sig);
            }
        }
        implementation.generics = Generics::default();
        concrete.push(implementation);
    }
    Ok(concrete)
}

/// Parses the forwarded reflection options without assigning product policy.
///
/// # Errors
///
/// Returns an error when the tokens do not parse as a comma-separated list of
/// metadata items.
pub(super) fn options(tokens: TokenStream) -> Result<Punctuated<Meta, Token![,]>> {
    Punctuated::<Meta, Token![,]>::parse_terminated.parse2(tokens)
}

/// Resolves Self only in a property signature, preserving the original impl.
pub(super) fn replace_self(method: &mut ImplItemFn, target: &Type) {
    let mut substitutions = Substitutions::default();
    substitutions.types.insert("Self".into(), target.clone());
    substitutions.visit_signature_mut(&mut method.sig);
}
