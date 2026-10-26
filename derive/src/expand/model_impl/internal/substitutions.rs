// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Rewrites generic type and const bindings in parsed syntax trees.

use std::collections::BTreeMap;

use syn::Expr;
use syn::GenericArgument;
use syn::PathSegment;
use syn::Token;
use syn::Type;
use syn::parse_quote;
use syn::punctuated::Punctuated;
use syn::visit_mut::VisitMut;
use syn::visit_mut::visit_expr_mut;
use syn::visit_mut::visit_generic_argument_mut;
use syn::visit_mut::visit_type_mut;

/// Carries type and const bindings separately, avoiding token-string rewriting.
#[derive(Default)]
pub(super) struct Substitutions {
    /// Maps type parameter names to their concrete replacement types.
    pub(in crate::expand::model_impl) types: BTreeMap<String, Type>,
    /// Maps const parameter names to expressions used at specialization sites.
    pub(in crate::expand::model_impl) constants: BTreeMap<String, Expr>,
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
