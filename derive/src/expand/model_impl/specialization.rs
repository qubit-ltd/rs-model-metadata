// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! AST-based specialization of property signatures, preserving method bodies.

use std::collections::BTreeMap;

use proc_macro2::TokenStream;
use syn::Error;
use syn::Expr;
use syn::GenericArgument;
use syn::Generics;
use syn::ImplItem;
use syn::ImplItemFn;
use syn::ItemImpl;
use syn::Meta;
use syn::PathSegment;
use syn::Result;
use syn::Token;
use syn::Type;
use syn::parse_quote;
use syn::punctuated::Punctuated;
use syn::visit_mut::VisitMut;
use syn::visit_mut::visit_expr_mut;
use syn::visit_mut::visit_generic_argument_mut;
use syn::visit_mut::visit_type_mut;

/// Applies explicit reflect-style specializations to property signature types.
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

/// Carries type and const bindings separately, avoiding token-string rewriting.
#[derive(Default)]
struct Substitutions {
    types: BTreeMap<String, Type>,
    constants: BTreeMap<String, Expr>,
}

impl VisitMut for Substitutions {
    /// Replaces complete type parameters and qualified associated-type
    /// prefixes.
    fn visit_type_mut(&mut self, ty: &mut Type) {
        if let Type::Path(path) = ty
            && path.qself.is_none()
            && let Some(first) = path.path.segments.first()
            && let Some(replacement) = self.types.get(&first.ident.to_string())
        {
            if path.path.segments.len() == 1 {
                *ty = replacement.clone();
                return;
            }
            let rest: Punctuated<PathSegment, Token![::]> = path.path.segments.iter().skip(1).cloned().collect();
            *ty = parse_quote!(<#replacement>::#rest);
        }
        visit_type_mut(self, ty);
    }

    /// Converts syntactically ambiguous generic arguments to const values.
    fn visit_generic_argument_mut(&mut self, argument: &mut GenericArgument) {
        if let GenericArgument::Type(Type::Path(path)) = argument
            && let Some(name) = path.path.get_ident()
            && let Some(value) = self.constants.get(&name.to_string())
        {
            *argument = GenericArgument::Const(value.clone());
            return;
        }
        visit_generic_argument_mut(self, argument);
    }

    /// Replaces const parameters in array lengths and generic const arguments.
    fn visit_expr_mut(&mut self, expression: &mut Expr) {
        if let Expr::Path(path) = expression
            && let Some(name) = path.path.get_ident()
            && let Some(value) = self.constants.get(&name.to_string())
        {
            *expression = value.clone();
            return;
        }
        visit_expr_mut(self, expression);
    }
}

/// Parses the forwarded reflection options without assigning product policy.
pub(super) fn options(tokens: TokenStream) -> Result<Punctuated<Meta, Token![,]>> {
    use syn::parse::Parser;
    Punctuated::<Meta, Token![,]>::parse_terminated.parse2(tokens)
}

/// Resolves Self only in a property signature, preserving the original impl.
pub(super) fn replace_self(method: &mut ImplItemFn, target: &Type) {
    let mut substitutions = Substitutions::default();
    substitutions.types.insert("Self".into(), target.clone());
    substitutions.visit_signature_mut(&mut method.sig);
}
