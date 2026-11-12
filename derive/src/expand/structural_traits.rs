// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Structural trait implementations with bounds on actual field shapes.

use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::format_ident;
use quote::quote;
use syn::Data;
use syn::DeriveInput;
use syn::Fields;
use syn::GenericArgument;
use syn::Ident;
use syn::Path;
use syn::PathArguments;
use syn::Token;
use syn::Type;
use syn::parse_quote;
use syn::punctuated::Punctuated;

/// Generates Clone, equality and hashing without unnecessary generic bounds.
///
/// # Parameters
/// - `item`: the model item whose structural implementation is generated.
/// - `name`: the supported structural trait name.
///
/// # Returns
/// Tokens implementing the selected trait with bounds on stored field types.
///
/// # Panics
/// Panics if `name` is not a supported structural trait.
#[must_use]
pub(super) fn expand(item: &DeriveInput, name: &str) -> TokenStream {
    let trait_path = trait_path(name);
    let shapes = field_shapes(item);
    if shapes.iter().any(|shape| {
        matches!(shape.1, Fields::Unnamed(fields)
            if fields.unnamed.iter().filter(|field| {
                !crate::expand::model_impl::internal::presence_condition::PresenceCondition::from_attributes(&field.attrs)
                    .expect("field cfg attributes were parsed by the declaration pipeline")
                    .is_unconditional()
            }).count() > crate::expand::model_impl::internal::presence_condition::MAX_CONDITIONAL_SHAPE_FIELDS)
    }) {
        return quote!(compile_error!("conditional tuple field limit exceeded (maximum 8 per shape)"););
    }
    let Some(configurations) = presence_combinations(&shapes) else {
        return quote!(compile_error!("too many cfg conditions for structural trait generation (maximum 8)"););
    };
    let ident = &item.ident;
    let implementations = configurations.iter().map(|(condition, active)| {
        let generics = bounded_generics(item, name, &trait_path, &shapes, active);
        let arms = trait_arms(item, name, &shapes, active);
        let body = trait_body(item, name, &shapes, &arms, active);
        let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
        quote!(#condition impl #impl_generics #trait_path for #ident #type_generics #where_clause { #body })
    });
    quote!(#(#implementations)*)
}

/// Enumerates target-crate cfg combinations used to build conditional impls.
///
/// # Parameters
/// * `shapes`: the declaration shapes and their presence conditions.
///
/// # Returns
/// Each generated item condition paired with the active-predicate map, or
/// `None` when the configured combination limit would be exceeded.
fn presence_combinations(
    shapes: &[(
        TokenStream,
        &Fields,
        crate::expand::model_impl::internal::presence_condition::PresenceCondition,
    )],
) -> Option<Vec<(TokenStream, HashMap<String, bool>)>> {
    use crate::expand::model_impl::internal::presence_condition::PresenceCondition;
    let mut conditions = Vec::<PresenceCondition>::new();
    for shape in shapes {
        if !shape.2.is_unconditional() {
            conditions.push(shape.2.clone());
        }
        for field in shape.1 {
            let condition = PresenceCondition::from_attributes(&field.attrs)
                .expect("field cfg attributes were parsed by the declaration pipeline");
            if !condition.is_unconditional() {
                conditions.push(condition);
            }
        }
    }
    let mut unique = Vec::<PresenceCondition>::new();
    let mut keys = std::collections::HashSet::new();
    for condition in conditions {
        if keys.insert(condition.predicate().to_string()) {
            unique.push(condition);
        }
    }
    if unique.len() > crate::expand::model_impl::internal::presence_condition::MAX_CONDITIONAL_SHAPE_FIELDS {
        return None;
    }
    let combinations = 1_usize.checked_shl(unique.len() as u32)?;
    Some(
        (0..combinations)
            .map(|mask| {
                let active: HashMap<_, _> = unique
                    .iter()
                    .enumerate()
                    .map(|(bit, condition)| (condition.predicate().to_string(), mask & (1 << bit) != 0))
                    .collect();
                let terms: Vec<_> = unique
                    .iter()
                    .enumerate()
                    .map(|(bit, condition)| {
                        let predicate = condition.predicate();
                        if mask & (1 << bit) != 0 {
                            predicate
                        } else {
                            quote!(not(#predicate))
                        }
                    })
                    .collect();
                let attribute = if terms.is_empty() {
                    TokenStream::new()
                } else {
                    quote!(#[cfg(all(#(#terms),*))])
                };
                (attribute, active)
            })
            .collect(),
    )
}

/// Reports whether a declaration shape is present in a cfg combination.
///
/// # Parameters
/// * `condition`: the condition attached to a shape or field.
/// * `active`: the predicate values for the selected combination.
///
/// # Returns
/// `true` when the declaration is unconditional or its predicate is active.
fn condition_is_active(
    condition: &crate::expand::model_impl::internal::presence_condition::PresenceCondition,
    active: &HashMap<String, bool>,
) -> bool {
    condition.is_unconditional() || active.get(&condition.predicate().to_string()).copied().unwrap_or(false)
}

/// Returns the standard trait path selected by its supported name.
///
/// # Parameters
/// - `name`: the supported structural trait name.
///
/// # Returns
/// Tokens naming the corresponding core trait.
///
/// # Panics
/// Panics when `name` is not in the supported trait set.
#[must_use]
fn trait_path(name: &str) -> TokenStream {
    match name {
        "Clone" => quote!(::core::clone::Clone),
        "PartialEq" => quote!(::core::cmp::PartialEq),
        "Eq" => quote!(::core::cmp::Eq),
        "Hash" => quote!(::core::hash::Hash),
        "Copy" => quote!(::core::marker::Copy),
        "Default" => quote!(::core::default::Default),
        "PartialOrd" => quote!(::core::cmp::PartialOrd),
        "Ord" => quote!(::core::cmp::Ord),
        "Debug" => quote!(::core::fmt::Debug),
        _ => unreachable!("supported structural trait"),
    }
}

/// Collects each struct or enum constructor together with its declared fields.
///
/// # Parameters
/// - `item`: the struct or enum syntax tree.
///
/// # Returns
/// Constructor tokens paired with references to fields borrowed from `item`,
/// in source order.
///
/// # Panics
/// Panics if called with a union after model-shape validation.
#[must_use]
fn field_shapes(
    item: &DeriveInput,
) -> Vec<(
    TokenStream,
    &Fields,
    crate::expand::model_impl::internal::presence_condition::PresenceCondition,
)> {
    match &item.data {
        Data::Struct(data) => vec![(
            quote!(Self),
            &data.fields,
            crate::expand::model_impl::internal::presence_condition::PresenceCondition::always(),
        )],
        Data::Enum(data) => data
            .variants
            .iter()
            .map(|variant| {
                let ident = &variant.ident;
                (
                    quote!(Self::#ident),
                    &variant.fields,
                    crate::expand::model_impl::internal::presence_condition::PresenceCondition::from_attributes(
                        &variant.attrs,
                    )
                    .expect("variant cfg attributes were parsed by the declaration pipeline"),
                )
            })
            .collect(),
        Data::Union(_) => unreachable!("validated model shape"),
    }
}

/// Adds structural-trait bounds for stored field types to cloned generics.
///
/// # Parameters
/// - `item`: the model whose generic parameters are cloned.
/// - `name`: the selected structural trait name.
/// - `trait_path`: tokens naming that trait.
/// - `shapes`: constructors and fields used to derive bounds.
///
/// # Returns
/// The cloned generics with bounds for stored field types and Copy's Clone
/// requirement when applicable.
#[must_use]
fn bounded_generics(
    item: &DeriveInput,
    name: &str,
    trait_path: &TokenStream,
    shapes: &[(
        TokenStream,
        &Fields,
        crate::expand::model_impl::internal::presence_condition::PresenceCondition,
    )],
    active: &HashMap<String, bool>,
) -> syn::Generics {
    let mut generics = item.generics.clone();
    let ident = &item.ident;
    let (_, arguments, _) = item.generics.split_for_impl();
    let recursive_type = quote!(#ident #arguments).to_string();
    for shape in shapes {
        if !condition_is_active(&shape.2, active) {
            continue;
        }
        for field in shape.1 {
            let presence = crate::expand::model_impl::internal::presence_condition::PresenceCondition::from_attributes(
                &field.attrs,
            )
            .expect("field cfg attributes were parsed by the declaration pipeline");
            if !condition_is_active(&presence, active) {
                continue;
            }
            for ty in recursive_bounds(&field.ty, &recursive_type).unwrap_or_else(|| vec![&field.ty]) {
                generics
                    .make_where_clause()
                    .predicates
                    .push(parse_quote!(#ty: #trait_path));
            }
        }
    }
    if name == "Copy" {
        generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(Self: ::core::clone::Clone));
    }
    generics
}

/// Generates match arms implementing the selected structural trait.
///
/// # Parameters
/// - `item`: the struct or enum being implemented.
/// - `name`: the selected structural trait name.
/// - `shapes`: constructors paired with their fields.
///
/// # Returns
/// Match-arm token streams in constructor order.
#[must_use]
fn trait_arms(
    item: &DeriveInput,
    name: &str,
    shapes: &[(
        TokenStream,
        &Fields,
        crate::expand::model_impl::internal::presence_condition::PresenceCondition,
    )],
    active: &HashMap<String, bool>,
) -> Vec<TokenStream> {
    let arms: Vec<_> = shapes
        .iter()
        .flat_map(|shape| {
            let constructor = &shape.0;
            let fields = shape.1;
            let shape_presence = shape.2.attribute();
            if let Fields::Unnamed(tuple_fields) = fields
                && tuple_fields.unnamed.iter().any(|field| {
                    !crate::expand::model_impl::internal::presence_condition::PresenceCondition::from_attributes(&field.attrs)
                        .expect("field cfg attributes were parsed by the declaration pipeline").is_unconditional()
                })
            {
                return tuple_trait_arms(name, &item.ident, constructor, fields, &shape_presence, active);
            }
            let left: Vec<_> = (0..fields.len())
                .map(|index| format_ident!("__model_left_{index}"))
                .collect();
            let right: Vec<_> = (0..fields.len())
                .map(|index| format_ident!("__model_right_{index}"))
                .collect();
            let pattern = pattern_tokens(constructor, fields, &left);
            let arm = match name {
                "Clone" => {
                    let values: Vec<_> = left
                        .iter()
                        .map(|value| quote!(::core::clone::Clone::clone(#value)))
                        .collect();
                    let body = construct(constructor, fields, &values);
                    quote!(#shape_presence #pattern => #body)
                }
                "PartialEq" => {
                    let other = pattern_tokens(constructor, fields, &right);
                    let comparisons = fields.iter().zip(&left).zip(&right).map(|((field, left), right)| {
                        let condition = crate::expand::model_impl::internal::presence_condition::PresenceCondition::from_attributes(&field.attrs)
                            .expect("field cfg attributes were parsed by the declaration pipeline").attribute();
                        quote!(#condition if !::core::cmp::PartialEq::eq(#left, #right) { return false; })
                    });
                    quote!(#shape_presence (#pattern, #other) => { #(#comparisons)* true })
                }
                "PartialOrd" | "Ord" => {
                    let other = pattern_tokens(constructor, fields, &right);
                    let comparisons = fields.iter().zip(&left).zip(&right).map(|((field, left), right)| {
                        let condition = crate::expand::model_impl::internal::presence_condition::PresenceCondition::from_attributes(&field.attrs)
                            .expect("field cfg attributes were parsed by the declaration pipeline").attribute();
                        if name == "PartialOrd" {
                            quote!(#condition match ::core::cmp::PartialOrd::partial_cmp(#left, #right) {
                                Some(::core::cmp::Ordering::Equal) => {}, ordering => return ordering,
                            })
                        } else {
                            quote!(#condition match ::core::cmp::Ord::cmp(#left, #right) {
                                ::core::cmp::Ordering::Equal => {}, ordering => return ordering,
                            })
                        }
                    });
                    if name == "PartialOrd" {
                        quote!(#shape_presence (#pattern, #other) => {
                            #(#comparisons)*
                            Some(::core::cmp::Ordering::Equal)
                        })
                    } else {
                        quote!(#shape_presence (#pattern, #other) => {
                            #(#comparisons)*
                            ::core::cmp::Ordering::Equal
                        })
                    }
                }
                "Debug" => {
                    let label = if matches!(item.data, Data::Struct(_)) {
                        item.ident.to_string()
                    } else {
                        constructor.to_string().split(" :: ").last().unwrap().to_owned()
                    };
                    let calls: Vec<_> = fields
                        .iter()
                        .zip(&left)
                        .map(|(field, binding)| {
                            let condition = crate::expand::model_impl::internal::presence_condition::PresenceCondition::from_attributes(&field.attrs)
                                .expect("field cfg attributes were parsed by the declaration pipeline").attribute();
                            if let Some(ident) = &field.ident {
                                let label = ident.to_string();
                                quote!(#condition builder.field(#label, #binding);)
                            } else {
                                quote!(#condition builder.field(#binding);)
                            }
                        })
                        .collect();
                    let builder = if matches!(fields, Fields::Named(_)) {
                        quote!(debug_struct)
                    } else {
                        quote!(debug_tuple)
                    };
                    quote!(#shape_presence #pattern => { let mut builder = formatter.#builder(#label); #(#calls)* builder.finish() })
                }
                "Hash" => {
                    let calls = fields.iter().zip(&left).map(|(field, binding)| {
                        let condition = crate::expand::model_impl::internal::presence_condition::PresenceCondition::from_attributes(&field.attrs)
                            .expect("field cfg attributes were parsed by the declaration pipeline").attribute();
                        quote!(#condition ::core::hash::Hash::hash(#binding, state);)
                    });
                    quote!(#shape_presence #pattern => { #(#calls)* })
                }
                _ => TokenStream::new(),
            };
            vec![arm]
        })
        .collect();
    arms
}

/// Generates one structural-trait arm for each active tuple field shape.
///
/// # Parameters
/// * `name`: the structural trait being generated.
/// * `struct_name`: the declared type name used by Debug output.
/// * `constructor`: tokens naming the tuple constructor.
/// * `fields`: the source tuple fields.
/// * `shape_presence`: the cfg predicate guarding the parent shape.
/// * `active`: active cfg predicates for the selected configuration.
///
/// # Returns
/// Generated match arms for the active tuple layout.
fn tuple_trait_arms(
    name: &str,
    struct_name: &Ident,
    constructor: &TokenStream,
    fields: &Fields,
    shape_presence: &TokenStream,
    active: &HashMap<String, bool>,
) -> Vec<TokenStream> {
    let Fields::Unnamed(tuple_fields) = fields else {
        unreachable!("conditional tuple trait arms require unnamed fields")
    };
    let field_conditions: Vec<_> = tuple_fields
        .unnamed
        .iter()
        .map(|field| {
            crate::expand::model_impl::internal::presence_condition::PresenceCondition::from_attributes(&field.attrs)
                .expect("field cfg attributes were parsed by the declaration pipeline")
        })
        .collect();
    let active_fields: Vec<_> = tuple_fields
        .unnamed
        .iter()
        .enumerate()
        .filter_map(|(index, _field)| condition_is_active(&field_conditions[index], active).then_some(index))
        .collect();
    let left: Vec<_> = active_fields
        .iter()
        .map(|index| format_ident!("__model_left_{index}"))
        .collect();
    let right: Vec<_> = active_fields
        .iter()
        .map(|index| format_ident!("__model_right_{index}"))
        .collect();
    let pattern = quote!(#constructor(#(#left),*));
    let arm = match name {
        "Clone" => {
            let values = left.iter().map(|binding| quote!(::core::clone::Clone::clone(#binding)));
            quote!(#shape_presence #pattern => #constructor(#(#values),*))
        }
        "PartialEq" => {
            let other = quote!(#constructor(#(#right),*));
            let comparisons = left
                .iter()
                .zip(&right)
                .map(|(left, right)| quote!(if !::core::cmp::PartialEq::eq(#left, #right) { return false; }));
            quote!(#shape_presence (#pattern, #other) => { #(#comparisons)* true })
        }
        "PartialOrd" | "Ord" => {
            let other = quote!(#constructor(#(#right),*));
            let comparisons = left.iter().zip(&right).map(|(left, right)| {
                if name == "PartialOrd" {
                    quote!(match ::core::cmp::PartialOrd::partial_cmp(#left, #right) {
                        Some(::core::cmp::Ordering::Equal) => {}, ordering => return ordering,
                    })
                } else {
                    quote!(match ::core::cmp::Ord::cmp(#left, #right) {
                        ::core::cmp::Ordering::Equal => {}, ordering => return ordering,
                    })
                }
            });
            let tail = if name == "PartialOrd" {
                quote!(Some(::core::cmp::Ordering::Equal))
            } else {
                quote!(::core::cmp::Ordering::Equal)
            };
            quote!(#shape_presence (#pattern, #other) => { #(#comparisons)* #tail })
        }
        "Debug" => {
            let rendered_constructor = constructor.to_string();
            let label = if rendered_constructor == "Self" {
                struct_name.to_string()
            } else {
                rendered_constructor.split(" :: ").last().unwrap().to_owned()
            };
            quote!(#shape_presence #pattern => {
                let mut builder = formatter.debug_tuple(#label);
                #(builder.field(#left);)*
                builder.finish()
            })
        }
        "Hash" => {
            let calls = left
                .iter()
                .map(|binding| quote!(::core::hash::Hash::hash(#binding, state);));
            quote!(#shape_presence #pattern => { #(#calls)* })
        }
        _ => TokenStream::new(),
    };
    vec![arm]
}

/// Assembles trait methods from generated arms and enum ordering metadata.
///
/// # Parameters
/// - `item`: the model receiving the implementation.
/// - `name`: the selected structural trait name.
/// - `shapes`: constructors paired with their fields.
/// - `arms`: generated match-arm tokens.
///
/// # Returns
/// Tokens for the selected trait's methods.
///
/// # Panics
/// Panics if `name` is unsupported or `Default` is requested for an item with
/// no constructor shape.
#[must_use]
fn trait_body(
    item: &DeriveInput,
    name: &str,
    shapes: &[(
        TokenStream,
        &Fields,
        crate::expand::model_impl::internal::presence_condition::PresenceCondition,
    )],
    arms: &[TokenStream],
    active: &HashMap<String, bool>,
) -> TokenStream {
    let discriminant = matches!(item.data, Data::Enum(_))
        .then(|| quote!(::core::hash::Hash::hash(&::core::mem::discriminant(self), state);));
    let ordering_prefix = if matches!(name, "PartialOrd" | "Ord") {
        enum_ordering_prefix(item, name)
    } else {
        TokenStream::new()
    };
    let ordering_fallback = matches!(&item.data, Data::Enum(data) if data.variants.len() > 1)
        .then(|| quote!(_ => unreachable!("distinct variants have distinct discriminants"),));
    match name {
        "Clone" => {
            quote!(fn clone(&self) -> Self { match self { #(#arms),* } })
        }
        "PartialEq" => {
            let fallback =
                matches!(&item.data, Data::Enum(data) if data.variants.len() > 1).then(|| quote!(_ => false,));
            quote!(fn eq(&self, other: &Self) -> bool { match (self, other) { #(#arms,)* #fallback } })
        }
        "Hash" => {
            quote!(fn hash<__ModelHasher: ::core::hash::Hasher>(&self, state: &mut __ModelHasher) {
                #discriminant
                match self { #(#arms),* }
            })
        }
        "Default" => {
            let constructor = &shapes[0].0;
            let fields = shapes[0].1;
            if let Fields::Unnamed(tuple_fields) = fields
                && tuple_fields.unnamed.iter().any(|field| {
                    !crate::expand::model_impl::internal::presence_condition::PresenceCondition::from_attributes(
                        &field.attrs,
                    )
                    .expect("field cfg attributes were parsed by the declaration pipeline")
                    .is_unconditional()
                })
            {
                let conditions: Vec<_> = tuple_fields
                    .unnamed
                    .iter()
                    .map(|field| {
                        crate::expand::model_impl::internal::presence_condition::PresenceCondition::from_attributes(
                            &field.attrs,
                        )
                        .expect("field cfg attributes were parsed by the declaration pipeline")
                    })
                    .collect();
                let values = conditions
                    .iter()
                    .filter(|condition| condition_is_active(condition, active))
                    .map(|_| quote!(::core::default::Default::default()));
                return quote!(fn default() -> Self { #constructor(#(#values),*) });
            }
            let values: Vec<_> = fields
                .iter()
                .map(|_| quote!(::core::default::Default::default()))
                .collect();
            let value = construct(constructor, fields, &values);
            quote!(fn default() -> Self { #value })
        }
        "PartialOrd" => {
            quote!(fn partial_cmp(&self, other: &Self) -> Option<::core::cmp::Ordering> {
                #ordering_prefix match (self, other) { #(#arms,)* #ordering_fallback }
            })
        }
        "Ord" => {
            quote!(fn cmp(&self, other: &Self) -> ::core::cmp::Ordering {
                #ordering_prefix match (self, other) { #(#arms,)* #ordering_fallback }
            })
        }
        "Debug" => {
            quote!(fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                match self { #(#arms),* }
            })
        }
        "Eq" | "Copy" => TokenStream::new(),
        _ => unreachable!("supported structural trait"),
    }
}

/// Converts binding names to the same shape used by structural constructors.
///
/// # Parameters
/// - `constructor`: tokens naming the struct or enum constructor.
/// - `fields`: the declared field shape.
/// - `bindings`: identifiers bound by a generated match arm.
///
/// # Returns
/// Pattern tokens matching the constructor and binding each field.
#[must_use]
fn pattern_tokens(constructor: &TokenStream, fields: &Fields, bindings: &[Ident]) -> TokenStream {
    let values: Vec<_> = bindings.iter().map(|binding| quote!(#binding)).collect();
    construct(constructor, fields, &values)
}

/// Constructs a named, tuple or unit model variant from field expressions.
///
/// # Parameters
/// - `constructor`: tokens naming the struct or enum constructor.
/// - `fields`: the declared field shape.
/// - `values`: tokens for each field expression.
///
/// # Returns
/// Tokens constructing the named, tuple, or unit value.
#[must_use]
fn construct(constructor: &TokenStream, fields: &Fields, values: &[TokenStream]) -> TokenStream {
    match fields {
        Fields::Named(fields) => {
            let entries = fields.named.iter().zip(values).map(|(field, value)| {
                let name = &field.ident;
                let condition =
                    crate::expand::model_impl::internal::presence_condition::PresenceCondition::from_attributes(
                        &field.attrs,
                    )
                    .expect("field cfg attributes were parsed by the declaration pipeline")
                    .attribute();
                quote!(#condition #name: #value)
            });
            quote!(#constructor { #(#entries),* })
        }
        Fields::Unnamed(_) => quote!(#constructor(#(#values),*)),
        Fields::Unit => quote!(#constructor),
    }
}

/// Compares Enum discriminants using their declared integer representation.
///
/// # Parameters
/// - `item`: the model syntax tree.
/// - `trait_name`: the selected ordering trait name.
///
/// # Returns
/// Tokens that compare enum discriminants, or an empty stream for structs.
#[must_use]
fn enum_ordering_prefix(item: &DeriveInput, trait_name: &str) -> TokenStream {
    let Data::Enum(data) = &item.data else {
        return TokenStream::new();
    };
    let mut representation: Type = parse_quote!(isize);
    for attribute in &item.attrs {
        if attribute.path().is_ident("repr")
            && let Ok(paths) = attribute.parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated)
        {
            for path in paths {
                if path.get_ident().is_some_and(|ident| {
                    matches!(
                        ident.to_string().as_str(),
                        "i8" | "i16"
                            | "i32"
                            | "i64"
                            | "i128"
                            | "isize"
                            | "u8"
                            | "u16"
                            | "u32"
                            | "u64"
                            | "u128"
                            | "usize"
                    )
                }) {
                    representation = parse_quote!(#path);
                }
            }
        }
    }
    let mut next = quote!(0);
    let arms: Vec<_> = data
        .variants
        .iter()
        .map(|variant| {
            let name = &variant.ident;
            let pattern = match variant.fields {
                Fields::Named(_) => quote!(Self::#name { .. }),
                Fields::Unnamed(_) => quote!(Self::#name(..)),
                Fields::Unit => quote!(Self::#name),
            };
            let value = variant
                .discriminant
                .as_ref()
                .map(|(_, value)| quote!(#value))
                .unwrap_or_else(|| next.clone());
            next = quote!((#value) + 1);
            let presence = crate::expand::model_impl::internal::presence_condition::PresenceCondition::from_attributes(
                &variant.attrs,
            )
            .expect("variant cfg attributes were parsed by the declaration pipeline")
            .attribute();
            quote!(#presence #pattern => #value)
        })
        .collect();
    let result = if trait_name == "PartialOrd" {
        quote!(Some(ordering))
    } else {
        quote!(ordering)
    };
    quote! {
        let discriminant = |value: &Self| -> #representation { match value { #(#arms),* } };
        let ordering = ::core::cmp::Ord::cmp(&discriminant(self), &discriminant(other));
        if ordering != ::core::cmp::Ordering::Equal { return #result; }
    }
}

/// Removes self-recursive obligations through standard storage wrappers while
/// retaining bounds on the other stored types.
///
/// # Parameters
/// - `ty`: the field type to inspect.
/// - `recursive_type`: the rendered recursive model type.
///
/// # Type Parameters
/// - `'a`: the lifetime of `ty` and any returned child types borrowed from it.
///
/// # Returns
/// `Some` with nonrecursive child types borrowed from `ty` when a recursive
/// occurrence is removed; otherwise `None` when no recursive occurrence is
/// found.
pub(super) fn recursive_bounds<'a>(ty: &'a Type, recursive_type: &str) -> Option<Vec<&'a Type>> {
    let rendered = ty.to_token_stream().to_string();
    if rendered == recursive_type || rendered == "Self" {
        return Some(Vec::new());
    }
    let children: Vec<&Type> = match ty {
        Type::Path(path) if path.qself.is_none() => {
            let segment = path.path.segments.last()?;
            let names: Vec<_> = path
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect();
            let names: Vec<_> = names.iter().map(String::as_str).collect();
            let standard = crate::compiler::type_path::is_option_path(&path.path)
                || crate::compiler::type_path::is_collection_path(&path.path)
                || matches!(
                    names.as_slice(),
                    ["Box" | "Rc" | "Arc"]
                        | ["std" | "alloc", "boxed", "Box"]
                        | ["std" | "alloc", "rc", "Rc"]
                        | ["std" | "alloc", "sync", "Arc"]
                );
            if !standard {
                return None;
            }
            let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
                return None;
            };
            arguments
                .args
                .iter()
                .filter_map(|argument| match argument {
                    GenericArgument::Type(ty) => Some(ty),
                    _ => None,
                })
                .collect()
        }
        Type::Tuple(tuple) => tuple.elems.iter().collect(),
        Type::Array(array) => vec![&array.elem],
        Type::Paren(paren) => vec![&paren.elem],
        _ => return None,
    };
    let mut found = false;
    let bounds = children
        .into_iter()
        .flat_map(|child| {
            if let Some(bounds) = recursive_bounds(child, recursive_type) {
                found = true;
                bounds
            } else {
                vec![child]
            }
        })
        .collect();
    found.then_some(bounds)
}

#[cfg(test)]
mod tests {
    use syn::DeriveInput;
    use syn::parse_quote;

    use super::expand;

    #[test]
    fn test_structural_traits_generate_conditional_tuple_shapes() {
        let item: DeriveInput = parse_quote! {
            struct Conditional<T>(
                #[cfg(feature = "first")] T,
                #[cfg_attr(feature = "second", cfg(feature = "second_field"))]
                String,
            );
        };
        for trait_name in [
            "Clone",
            "PartialEq",
            "Eq",
            "Hash",
            "Copy",
            "Default",
            "PartialOrd",
            "Ord",
            "Debug",
        ] {
            let output = expand(&item, trait_name).to_string();
            assert!(!output.is_empty(), "missing {trait_name} output");
            assert!(output.contains("cfg"), "missing cfg arms for {trait_name}");
        }
    }

    #[test]
    fn test_structural_traits_generate_named_and_enum_shapes() {
        let item: DeriveInput = parse_quote! {
            enum Conditional<T> {
                #[cfg(feature = "payload")]
                Tuple(#[cfg(feature = "field")] T, String),
                Named { value: T },
                Unit,
            }
        };
        for trait_name in ["Clone", "PartialEq", "Hash", "PartialOrd", "Ord", "Debug"] {
            let output = expand(&item, trait_name).to_string();
            assert!(!output.is_empty(), "missing {trait_name} output");
        }
    }

    #[test]
    fn test_structural_traits_reject_excessive_cfg_combinations() {
        let item: DeriveInput = parse_quote! {
            struct TooMany(
                #[cfg(feature = "a")] u8,
                #[cfg(feature = "b")] u8,
                #[cfg(feature = "c")] u8,
                #[cfg(feature = "d")] u8,
                #[cfg(feature = "e")] u8,
                #[cfg(feature = "f")] u8,
                #[cfg(feature = "g")] u8,
                #[cfg(feature = "h")] u8,
                #[cfg(feature = "i")] u8,
            );
        };
        assert!(
            expand(&item, "Clone")
                .to_string()
                .contains("conditional tuple field limit exceeded")
        );
    }
}
