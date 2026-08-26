// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Serde default injection for model declaration fields.

use proc_macro2::Span;
use syn::Attribute;
use syn::Data;
use syn::Error;
use syn::Field;
use syn::Fields;
use syn::LitStr;
use syn::Meta;
use syn::Path;
use syn::Result;
use syn::Token;
use syn::Type;
use syn::parse_quote;
use syn::punctuated::Punctuated;

/// Adds Serde defaults supported by the model's enabled serialization
/// capabilities.
///
/// # Parameters
///
/// - `data`: The declaration whose fields receive default Serde attributes.
/// - `serialize`: Whether generated serialization is enabled.
/// - `deserialize`: Whether generated deserialization is enabled.
///
/// # Errors
///
/// Returns the first combined error encountered while parsing an existing Serde
/// helper attribute.
pub(crate) fn add_default_serde_field_attributes(data: &mut Data, serialize: bool, deserialize: bool) -> Result<()> {
    let mut error: Option<Error> = None;
    match data {
        Data::Struct(data) => visit_serde_fields(&mut data.fields, true, serialize, deserialize, &mut error),
        Data::Enum(data) => {
            for variant in &mut data.variants {
                match &mut variant.fields {
                    Fields::Unit => {}
                    Fields::Unnamed(fields) => {
                        let last_index = fields.unnamed.len().saturating_sub(1);
                        let can_omit_tail = fields.unnamed.len() > 1;
                        for (index, field) in fields.unnamed.iter_mut().enumerate() {
                            add_default_serde_attributes(
                                field,
                                can_omit_tail && index == last_index,
                                serialize,
                                deserialize,
                                &mut error,
                            );
                        }
                    }
                    Fields::Named(fields) => {
                        for field in &mut fields.named {
                            add_default_serde_attributes(field, true, serialize, deserialize, &mut error);
                        }
                    }
                }
            }
        }
        Data::Union(_) => {}
    }
    error.map_or(Ok(()), Err)
}

/// Adds supported Serde defaults to a non-enum field collection.
///
/// # Parameters
///
/// - `fields`: The fields to inspect and possibly rewrite.
/// - `allow_serialization_omission`: Whether a matching field may be omitted.
/// - `serialize`: Whether generated serialization is enabled.
/// - `deserialize`: Whether generated deserialization is enabled.
/// - `error`: Accumulates parse errors from existing Serde attributes.
fn visit_serde_fields(
    fields: &mut Fields,
    allow_serialization_omission: bool,
    serialize: bool,
    deserialize: bool,
    error: &mut Option<Error>,
) {
    for field in fields {
        add_default_serde_attributes(field, allow_serialization_omission, serialize, deserialize, error);
    }
}

/// Adds Serde defaults to one field without changing tuple field positions.
///
/// A tuple variant may omit only its final field. Omitting any earlier field
/// shifts later serialized values left and prevents safe deserialization.
///
/// # Parameters
///
/// - `field`: The field to inspect and possibly rewrite.
/// - `allow_serialization_omission`: Whether this field may be omitted.
/// - `serialize`: Whether generated serialization is enabled.
/// - `deserialize`: Whether generated deserialization is enabled.
/// - `error`: Accumulates parse errors from existing Serde attributes.
fn add_default_serde_attributes(
    field: &mut Field,
    allow_serialization_omission: bool,
    serialize: bool,
    deserialize: bool,
    error: &mut Option<Error>,
) {
    let result = field_keeps_serializing(&field.attrs).and_then(|keep_serializing| {
        if keep_serializing {
            return Ok(());
        }
        let omit_serialization = serialize && allow_serialization_omission;
        if is_standard_option_type(&field.ty) {
            add_default_option_serde_attributes(field, omit_serialization, deserialize)
        } else if let Some(is_empty) = collection_is_empty_function(&field.ty) {
            add_default_collection_serde_attributes(field, is_empty, omit_serialization, deserialize)
        } else {
            Ok(())
        }
    });
    if let Err(current) = result {
        match error {
            Some(error) => error.combine(current),
            None => *error = Some(current),
        }
    }
}

/// Adds automatic Serde attributes for an optional field.
///
/// # Parameters
///
/// - `field`: The optional field to rewrite.
/// - `omit_serialization`: Whether `None` is omitted when serializing.
/// - `deserialize`: Whether missing input uses `Default::default`.
///
/// # Errors
///
/// Returns an error if an existing Serde helper attribute cannot be parsed.
fn add_default_option_serde_attributes(field: &mut Field, omit_serialization: bool, deserialize: bool) -> Result<()> {
    if omit_serialization {
        add_serde_attribute_if_absent(
            &mut field.attrs,
            &["skip_serializing_if", "skip_serializing", "skip"],
            parse_quote!(#[serde(skip_serializing_if = "::core::option::Option::is_none")]),
        )?;
    }
    if deserialize {
        add_serde_attribute_if_absent(
            &mut field.attrs,
            &["default", "skip_deserializing", "skip"],
            parse_quote!(#[serde(default)]),
        )?;
    }
    Ok(())
}

/// Adds automatic Serde attributes for a supported collection field.
///
/// # Parameters
///
/// - `field`: The collection field to rewrite.
/// - `is_empty`: The fully qualified emptiness predicate for the collection.
/// - `omit_serialization`: Whether empty values are omitted when serializing.
/// - `deserialize`: Whether missing input uses `Default::default`.
///
/// # Errors
///
/// Returns an error if an existing Serde helper attribute cannot be parsed.
fn add_default_collection_serde_attributes(
    field: &mut Field,
    is_empty: &str,
    omit_serialization: bool,
    deserialize: bool,
) -> Result<()> {
    if omit_serialization {
        add_serde_attribute_if_absent(
            &mut field.attrs,
            &["skip_serializing_if", "skip_serializing", "skip"],
            serde_skip_serializing_if_attribute(is_empty),
        )?;
    }
    if deserialize {
        add_serde_attribute_if_absent(
            &mut field.attrs,
            &["default", "skip_deserializing", "skip"],
            parse_quote!(#[serde(default)]),
        )?;
    }
    Ok(())
}

/// Returns whether a field opts out of automatic serialization omission.
///
/// # Parameters
///
/// - `attributes`: The attributes applied to the field.
///
/// # Returns
///
/// Returns `true` when `#[keep_serializing]` is present.
#[inline]
fn field_keeps_serializing(attributes: &[Attribute]) -> Result<bool> {
    Ok(attributes
        .iter()
        .any(|attribute| attribute.path().is_ident("keep_serializing")))
}

/// Adds an attribute unless an existing Serde attribute has one of its options.
///
/// # Parameters
///
/// - `attributes`: The field attributes to inspect and possibly extend.
/// - `options`: Serde options that suppress the default attribute.
/// - `default_attribute`: The Serde attribute to add when none is present.
///
/// # Errors
///
/// Returns an error if an existing Serde helper attribute cannot be parsed.
fn add_serde_attribute_if_absent(
    attributes: &mut Vec<Attribute>,
    options: &[&str],
    default_attribute: Attribute,
) -> Result<()> {
    if !has_attribute_option(attributes, "serde", options)? {
        attributes.push(default_attribute);
    }
    Ok(())
}

/// Returns whether an attribute list contains any supplied helper option.
///
/// # Parameters
///
/// - `attributes`: Attributes to inspect.
/// - `attribute_name`: The outer attribute name to match.
/// - `options`: Nested options that count as a match.
///
/// # Returns
///
/// Returns `true` when a matching nested option is present.
///
/// # Errors
///
/// Returns an error if a matching list attribute cannot be parsed as metadata.
fn has_attribute_option(attributes: &[Attribute], attribute_name: &str, options: &[&str]) -> Result<bool> {
    for attribute in attributes
        .iter()
        .filter(|attribute| attribute.path().is_ident(attribute_name))
    {
        let Meta::List(list) = &attribute.meta else {
            continue;
        };
        let serde_options = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        if serde_options
            .iter()
            .any(|serde_option| options.iter().any(|option| serde_option.path().is_ident(option)))
        {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Builds a Serde field attribute calling an inherent `is_empty` method.
///
/// # Parameters
///
/// - `is_empty`: The fully qualified function path encoded in the attribute.
///
/// # Returns
///
/// Returns a Serde `skip_serializing_if` attribute.
fn serde_skip_serializing_if_attribute(is_empty: &str) -> Attribute {
    let is_empty = LitStr::new(is_empty, Span::call_site());
    parse_quote!(#[serde(skip_serializing_if = #is_empty)])
}

/// Returns the `is_empty` function for a directly declared standard collection.
///
/// # Parameters
///
/// - `ty`: The field type to classify.
///
/// # Returns
///
/// Returns the fully qualified emptiness predicate when `ty` is supported.
fn collection_is_empty_function(ty: &Type) -> Option<&'static str> {
    match ty {
        Type::Path(_) if is_standard_type_path(ty, "Vec", &["std", "vec", "Vec"]) => Some("::std::vec::Vec::is_empty"),
        Type::Path(_) if is_standard_type_path(ty, "LinkedList", &["std", "collections", "LinkedList"]) => {
            Some("::std::collections::LinkedList::is_empty")
        }
        Type::Path(_) if is_standard_type_path(ty, "VecDeque", &["std", "collections", "VecDeque"]) => {
            Some("::std::collections::VecDeque::is_empty")
        }
        Type::Path(_) if is_standard_type_path(ty, "HashMap", &["std", "collections", "HashMap"]) => {
            Some("::std::collections::HashMap::is_empty")
        }
        Type::Path(_) if is_standard_type_path(ty, "BTreeMap", &["std", "collections", "BTreeMap"]) => {
            Some("::std::collections::BTreeMap::is_empty")
        }
        Type::Path(_) if is_standard_type_path(ty, "HashSet", &["std", "collections", "HashSet"]) => {
            Some("::std::collections::HashSet::is_empty")
        }
        Type::Path(_) if is_standard_type_path(ty, "BTreeSet", &["std", "collections", "BTreeSet"]) => {
            Some("::std::collections::BTreeSet::is_empty")
        }
        Type::Path(_) if is_standard_type_path(ty, "BinaryHeap", &["std", "collections", "BinaryHeap"]) => {
            Some("::std::collections::BinaryHeap::is_empty")
        }
        Type::Array(_) => Some("<[_]>::is_empty"),
        Type::Group(group) => collection_is_empty_function(&group.elem),
        Type::Paren(paren) => collection_is_empty_function(&paren.elem),
        _ => None,
    }
}

/// Returns whether a type names the standard-library `Option` type.
///
/// # Parameters
///
/// - `ty`: The field type to classify.
///
/// # Returns
///
/// Returns `true` when `ty` syntactically names `Option` or its canonical path.
fn is_standard_option_type(ty: &Type) -> bool {
    is_standard_type_path(ty, "Option", &["core", "option", "Option"])
        || is_standard_type_path(ty, "Option", &["std", "option", "Option"])
}

/// Returns whether a type uses an unqualified prelude spelling or a canonical
/// standard-library path for one supported type.
///
/// # Parameters
///
/// - `ty`: The field type to classify.
/// - `prelude_name`: The supported unqualified spelling.
/// - `standard_path`: The canonical standard-library path segments.
///
/// # Returns
///
/// Returns `true` when the syntax matches one supported spelling.
///
/// # Limitations
///
/// FIXME: a procedural macro sees parsed syntax rather than the consuming
/// crate's resolved names. This temporary implementation treats an
/// unqualified spelling such as `Option` as the standard-library type, even
/// when a caller import shadows it. Callers using a shadowed name must add
/// `#[keep_serializing]` to preserve their own Serde behavior. A complete
/// solution requires resolved caller type information (for example, a compiler
/// integration that exposes name resolution) or a redesigned explicit opt-in
/// for automatic omission that does not infer semantics from unqualified names.
#[must_use]
#[inline]
fn is_standard_type_path(ty: &Type, prelude_name: &str, standard_path: &[&str]) -> bool {
    match ty {
        Type::Path(type_path) => {
            type_path.qself.is_none()
                && (is_unqualified_type_path(&type_path.path, prelude_name)
                    || path_matches(&type_path.path, standard_path))
        }
        Type::Group(group) => is_standard_type_path(&group.elem, prelude_name, standard_path),
        Type::Paren(paren) => is_standard_type_path(&paren.elem, prelude_name, standard_path),
        _ => false,
    }
}

/// Returns whether a path is one unqualified type segment with the given name.
///
/// # Parameters
///
/// - `path`: The parsed type path to inspect.
/// - `expected`: The required single segment name.
///
/// # Returns
///
/// Returns `true` when `path` consists only of `expected`.
fn is_unqualified_type_path(path: &Path, expected: &str) -> bool {
    path.segments.len() == 1 && path.segments.first().is_some_and(|segment| segment.ident == expected)
}

/// Returns whether a parsed path matches supplied canonical segments.
///
/// # Parameters
///
/// - `path`: The parsed type path to inspect.
/// - `expected`: Canonical path segments to match.
///
/// # Returns
///
/// Returns `true` when `path` has exactly the expected segments.
fn path_matches(path: &Path, expected: &[&str]) -> bool {
    path.segments.len() == expected.len()
        && path
            .segments
            .iter()
            .zip(expected)
            .all(|(segment, expected)| segment.ident == *expected)
}
