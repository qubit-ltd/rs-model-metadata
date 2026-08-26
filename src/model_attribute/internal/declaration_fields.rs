// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Helpers that inspect or remove declaration field attributes.

use syn::Data;
use syn::DeriveInput;
use syn::Field;

use crate::attribute::is_field_level_helper_attribute;

/// Removes model field helper attributes from the declaration returned to Rust.
///
/// # Parameters
///
/// - `data`: The parsed struct or enum declaration to rewrite.
pub(crate) fn remove_field_attributes(data: &mut Data) {
    visit_fields(data, |field| {
        field
            .attrs
            .retain(|attribute| !is_field_level_helper_attribute(attribute.path()));
    });
}

/// Removes inert Serde helper attributes when no generated derive consumes
/// them.
///
/// # Parameters
///
/// - `item`: The declaration that will be returned to the Rust compiler.
///
/// This mutates the declaration only after both serialization capabilities are
/// disabled and no redaction derive remains to consume Serde helper attributes.
pub(crate) fn remove_serde_attributes(item: &mut DeriveInput) {
    item.attrs.retain(|attribute| !attribute.path().is_ident("serde"));
    match &mut item.data {
        Data::Struct(data) => {
            for field in &mut data.fields {
                field.attrs.retain(|attribute| !attribute.path().is_ident("serde"));
            }
        }
        Data::Enum(data) => {
            for variant in &mut data.variants {
                variant.attrs.retain(|attribute| !attribute.path().is_ident("serde"));
                for field in &mut variant.fields {
                    field.attrs.retain(|attribute| !attribute.path().is_ident("serde"));
                }
            }
        }
        Data::Union(_) => {}
    }
}

/// Returns whether a struct or enum payload field declares a redaction rule.
///
/// # Parameters
///
/// - `data`: The parsed declaration to inspect.
///
/// # Returns
///
/// Returns `true` when any field carries `#[redact(...)]`.
#[must_use]
#[inline]
pub(crate) fn has_redact_fields(data: &Data) -> bool {
    match data {
        Data::Struct(data) => data
            .fields
            .iter()
            .any(|field| field.attrs.iter().any(|attribute| attribute.path().is_ident("redact"))),
        Data::Enum(data) => data.variants.iter().any(|variant| {
            variant
                .fields
                .iter()
                .any(|field| field.attrs.iter().any(|attribute| attribute.path().is_ident("redact")))
        }),
        Data::Union(_) => false,
    }
}

/// Visits every struct field or enum payload field in a declaration.
///
/// # Parameters
///
/// - `data`: The parsed declaration whose fields are visited.
/// - `visit`: The operation applied to each mutable field.
fn visit_fields(data: &mut Data, mut visit: impl FnMut(&mut Field)) {
    match data {
        Data::Struct(data) => {
            for field in &mut data.fields {
                visit(field);
            }
        }
        Data::Enum(data) => {
            for variant in &mut data.variants {
                for field in &mut variant.fields {
                    visit(field);
                }
            }
        }
        Data::Union(_) => {}
    }
}
