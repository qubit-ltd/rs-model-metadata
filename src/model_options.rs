// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Parsing and normalization for `Model` capability options.

use proc_macro2::Span;
use syn::Error;
use syn::Meta;
use syn::Result;

/// Capability controls parsed from one `Model` attribute.
pub(crate) struct ModelOptions {
    /// Model metadata arguments delegated to the metadata expander.
    pub(crate) metadata: Vec<Meta>,
    /// Whether redacted formatting and serialization are requested.
    pub(crate) redact: bool,
    /// Whether each default capability is disabled.
    pub(crate) disabled: DisabledCapabilities,
}

/// Disabled default capabilities after dependency normalization.
// qubit-style: allow multiple-public-types
#[derive(Default)]
pub(crate) struct DisabledCapabilities {
    /// Disables `Clone`.
    pub(crate) clone: bool,
    /// Disables `Copy`.
    pub(crate) copy: bool,
    /// Disables `Debug`.
    pub(crate) debug: bool,
    /// Disables `Display`.
    pub(crate) display: bool,
    /// Disables `Eq`.
    pub(crate) eq: bool,
    /// Disables `PartialEq`.
    pub(crate) partial_eq: bool,
    /// Disables `PartialOrd`.
    pub(crate) partial_ord: bool,
    /// Disables `Ord`.
    pub(crate) ord: bool,
    /// Disables `Hash`.
    pub(crate) hash: bool,
    /// Disables `Serialize`.
    pub(crate) serialize: bool,
    /// Disables `Deserialize`.
    pub(crate) deserialize: bool,
}

impl ModelOptions {
    /// Separates model metadata arguments from capability controls.
    pub(crate) fn parse(attributes: impl IntoIterator<Item = Meta>) -> Result<Self> {
        let mut options = Self {
            metadata: Vec::new(),
            redact: false,
            disabled: DisabledCapabilities::default(),
        };
        let mut errors = None;

        for attribute in attributes {
            match parse_control(&attribute, &mut options) {
                Ok(true) => {}
                Ok(false) => options.metadata.push(attribute),
                Err(error) => combine_error(&mut errors, error),
            }
        }

        if let Some(error) = errors {
            Err(error)
        } else {
            options.disabled.normalize();
            Ok(options)
        }
    }
}

impl DisabledCapabilities {
    /// Applies the trait dependency rules defined by the public macro API.
    fn normalize(&mut self) {
        if self.partial_eq {
            self.eq = true;
            self.partial_ord = true;
            self.ord = true;
        }
        if self.eq || self.partial_ord {
            self.ord = true;
        }
    }
}

/// Parses one capability control and returns whether it consumed the meta item.
fn parse_control(attribute: &Meta, options: &mut ModelOptions) -> Result<bool> {
    let Meta::Path(path) = attribute else {
        return Ok(false);
    };

    let Some(ident) = path.get_ident() else {
        return Ok(false);
    };
    let name = ident.to_string();
    let consumed = match name.as_str() {
        "redact" => set_once(&mut options.redact, ident.span(), "redact")?,
        "no_clone" => set_once(&mut options.disabled.clone, ident.span(), "no_clone")?,
        "no_copy" => set_once(&mut options.disabled.copy, ident.span(), "no_copy")?,
        "no_debug" => set_once(&mut options.disabled.debug, ident.span(), "no_debug")?,
        "no_display" => set_once(&mut options.disabled.display, ident.span(), "no_display")?,
        "no_eq" => set_once(&mut options.disabled.eq, ident.span(), "no_eq")?,
        "no_partial_eq" => set_once(&mut options.disabled.partial_eq, ident.span(), "no_partial_eq")?,
        "no_partial_ord" => set_once(&mut options.disabled.partial_ord, ident.span(), "no_partial_ord")?,
        "no_ord" => set_once(&mut options.disabled.ord, ident.span(), "no_ord")?,
        "no_hash" => set_once(&mut options.disabled.hash, ident.span(), "no_hash")?,
        "no_serialize" => set_once(&mut options.disabled.serialize, ident.span(), "no_serialize")?,
        "no_deserialize" => set_once(&mut options.disabled.deserialize, ident.span(), "no_deserialize")?,
        _ if name.starts_with("no_") => {
            return Err(Error::new(ident.span(), "unknown `Model` option"));
        }
        _ => return Ok(false),
    };
    Ok(consumed)
}

/// Marks one switch and rejects repeated controls at the second occurrence.
fn set_once(value: &mut bool, span: Span, name: &str) -> Result<bool> {
    if *value {
        Err(Error::new(span, format!("duplicate `{name}` option")))
    } else {
        *value = true;
        Ok(true)
    }
}

/// Appends one independent diagnostic to an optional aggregate error.
fn combine_error(errors: &mut Option<Error>, error: Error) {
    match errors {
        Some(existing) => existing.combine(error),
        None => *errors = Some(error),
    }
}
