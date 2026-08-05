// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Resolution of the runtime metadata crate's path in consuming crates.

use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Error, Ident, Result};

/// Resolves the runtime crate path, including a dependency renamed by the
/// consuming crate.
///
/// Returns a call-site error when the consuming crate does not declare
/// `qubit-model-metadata` as a dependency.
///
/// # Returns
///
/// Returns a token stream naming the runtime crate, including a Cargo-renamed
/// dependency, or `crate` when the runtime crate is itself.
///
/// # Errors
///
/// Returns an error when the consuming crate does not declare
/// `qubit-model-metadata` as a dependency.
pub(crate) fn runtime_path() -> Result<TokenStream> {
    match crate_name("qubit-model-metadata") {
        Ok(FoundCrate::Itself) => Ok(quote!(crate)),
        Ok(FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, Span::call_site());
            Ok(quote!(::#ident))
        }
        Err(_) => Err(Error::new(
            Span::call_site(),
            "Model derive requires the `qubit-model-metadata` dependency",
        )),
    }
}
