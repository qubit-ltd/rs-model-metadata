// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Generates inventory registration for concrete model declarations.

use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::Ident;

/// Generates inventory registration for a concrete model declaration.
pub(super) fn expand_registration(ident: &Ident, runtime: &TokenStream) -> TokenStream {
    let snake_name = ident.to_string().to_snake_case();
    let source_fn = format_ident!("__qubit_model_source_{}", snake_name);
    let registration_fn = format_ident!("__qubit_model_registration_{}", snake_name);
    let fingerprint = stable_fingerprint(&ident.to_string());
    quote! {
        #[doc(hidden)]
        fn #source_fn() -> &'static #runtime::identity::FragmentIdentity {
            static SOURCE: ::std::sync::OnceLock<#runtime::identity::FragmentIdentity> = ::std::sync::OnceLock::new();
            SOURCE.get_or_init(|| #runtime::identity::FragmentIdentity::new(
                env!("CARGO_PKG_NAME"),
                module_path!(),
                line!(),
                column!(),
                "model",
                #fingerprint,
            ))
        }

        #[doc(hidden)]
        fn #registration_fn() -> #runtime::ModelRegistration {
            #runtime::__private::v3::concrete_registration(
                #runtime::TypeMetadata::of::<#ident>(),
                #source_fn(),
            )
        }

        #runtime::__private::inventory::submit! {
            #runtime::ModelRegistrationFactory(#registration_fn)
        }
    }
}

/// Computes the stable registration fingerprint for `value`.
pub(super) fn stable_fingerprint(value: &str) -> u64 {
    value.bytes().fold(0xcbf29ce484222325_u64, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    })
}
