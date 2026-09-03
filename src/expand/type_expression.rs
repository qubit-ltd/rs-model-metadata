// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Converts Rust types and const expressions into symbolic metadata.

use std::collections::HashSet;

use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Error;
use syn::Expr;
use syn::ExprLit;
use syn::GenericArgument;
use syn::Lit;
use syn::LitStr;
use syn::PathArguments;
use syn::Type;

/// Converts a Rust type into the runtime's symbolic type expression.
pub(super) fn expand_type_expression(
    ty: &Type,
    type_parameters: &HashSet<String>,
    const_parameters: &HashSet<String>,
    runtime: &TokenStream,
) -> TokenStream {
    match ty {
        Type::Path(path) if path.qself.is_none() => {
            if path.path.segments.len() == 1 {
                let name = path.path.segments[0].ident.to_string();
                if type_parameters.contains(&name) {
                    return quote!(#runtime::__private::reflect_codegen_v1::expression::parameter(#name));
                }
            }
            let segments: Vec<_> = path
                .path
                .segments
                .iter()
                .map(|segment| LitStr::new(&segment.ident.to_string(), segment.ident.span()))
                .collect();
            let arguments = path.path.segments.last().map_or_else(Vec::new, |segment| {
                let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
                    return Vec::new();
                };
                arguments
                    .args
                    .iter()
                    .filter_map(|argument| match argument {
                        GenericArgument::Type(ty) => {
                            let expression = expand_type_expression(ty, type_parameters, const_parameters, runtime);
                            Some(quote!(#runtime::expression::GenericArgument::Type(#expression)))
                        }
                        GenericArgument::Const(value) => {
                            let expression = expand_const_expression(value, const_parameters, runtime);
                            let diagnostic = LitStr::new(&quote!(#value).to_string(), Span::call_site());
                            Some(quote!(#runtime::expression::GenericArgument::Const(
                                #runtime::__private::reflect_codegen_v1::expression::const_argument(
                                    #runtime::expression::TypeExpression::Concrete(
                                        #runtime::__private::reflect_codegen_v1::expression::concrete(
                                            ::std::boxed::Box::new(["_".into()]),
                                            ::std::boxed::Box::new([]),
                                            #runtime::expression::DiagnosticText::default(),
                                        ),
                                    ),
                                    #expression,
                                    #diagnostic,
                                ),
                            )))
                        }
                        _ => None,
                    })
                    .collect()
            });
            quote!(#runtime::expression::TypeExpression::Concrete(
                #runtime::__private::reflect_codegen_v1::expression::concrete(
                    ::std::boxed::Box::new([#(#segments.into()),*]),
                    ::std::boxed::Box::new([#(#arguments),*]),
                    #runtime::expression::DiagnosticText::default(),
                ),
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
                #runtime::__private::reflect_codegen_v1::expression::array(
                    #element,
                    #length,
                ),
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
                Some("static") => {
                    quote!(#runtime::expression::LifetimeExpression::Static)
                }
                Some("_") => {
                    quote!(#runtime::expression::LifetimeExpression::Placeholder)
                }
                Some(name) => {
                    quote!(#runtime::__private::reflect_codegen_v1::expression::named_lifetime(#name))
                }
                None => {
                    quote!(#runtime::expression::LifetimeExpression::Elided)
                }
            };
            let mutable = reference.mutability.is_some();
            quote!(#runtime::expression::TypeExpression::Reference(
                #runtime::__private::reflect_codegen_v1::expression::reference(
                    #lifetime,
                    #mutable,
                    #target,
                ),
            ))
        }
        _ => {
            let source = LitStr::new(&quote!(#ty).to_string(), Span::call_site());
            quote!(#runtime::expression::TypeExpression::Concrete(
                #runtime::__private::reflect_codegen_v1::expression::concrete(
                    ::std::boxed::Box::new([#source.into()]),
                    ::std::boxed::Box::new([]),
                    #runtime::expression::DiagnosticText::from(#source),
                ),
            ))
        }
    }
}

/// Converts a const expression into the runtime's symbolic representation.
fn expand_const_expression(value: &Expr, const_parameters: &HashSet<String>, runtime: &TokenStream) -> TokenStream {
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
                quote!(#runtime::__private::reflect_codegen_v1::expression::const_parameter(#name))
            } else {
                quote!(#runtime::__private::reflect_codegen_v1::expression::const_path([#(#segments),*]))
            }
        }
        Expr::Lit(ExprLit {
            lit: Lit::Int(value), ..
        }) => {
            let value = match value.base10_parse::<u128>() {
                Ok(value) => value,
                Err(_) => {
                    return Error::new(value.span(), "const integer literal exceeds u128").into_compile_error();
                }
            };
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
            quote!(#runtime::__private::reflect_codegen_v1::expression::const_path([#source]))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use proc_macro2::TokenStream;
    use quote::quote;
    use syn::Type;
    use syn::parse_quote;

    use super::expand_type_expression;

    /// Exercises every supported type and const-expression representation.
    #[test]
    fn test_expand_type_expression_shapes() {
        let type_parameters = HashSet::from(["T".to_owned()]);
        let const_parameters = HashSet::from(["N".to_owned()]);
        let runtime: TokenStream = quote!(runtime);
        let cases: [Type; 13] = [
            parse_quote!(T),
            parse_quote!(std::vec::Vec<T>),
            parse_quote!(Buffer<T, N, 7, true, 'x', module::SIZE>),
            parse_quote!([T]),
            parse_quote!([T; N]),
            parse_quote!((T, String)),
            parse_quote!(&'static T),
            parse_quote!(&'_ T),
            parse_quote!(&'named mut T),
            parse_quote!(&T),
            parse_quote!(<T as Trait>::Item),
            parse_quote!(fn(T) -> T),
            parse_quote!([T; 340282366920938463463374607431768211456]),
        ];

        for ty in cases {
            assert!(!expand_type_expression(&ty, &type_parameters, &const_parameters, &runtime).is_empty());
        }
    }
}
