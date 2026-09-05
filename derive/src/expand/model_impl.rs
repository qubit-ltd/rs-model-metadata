// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Validates and expands getter/setter-backed model implementation blocks.

// Groups private representations used by model implementation expansion.
mod internal;

use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::Error;
use syn::FnArg;
use syn::GenericArgument;
use syn::ImplItem;
use syn::ImplItemFn;
use syn::ItemImpl;
use syn::PathArguments;
use syn::Result;
use syn::ReturnType;
use syn::Type;
use syn::TypePath;
use syn::TypeReference;
use syn::Visibility;

use self::internal::GetterIr;
use self::internal::GetterReturn;
use self::internal::PropertyMethod;
use self::internal::SetterIr;
use crate::compiler::fingerprint::stable_fingerprint;
use crate::compiler::type_path::is_option_path;

/// Validates that `item` is a non-generic inherent implementation.
///
/// Returns a diagnostic describing the first unsupported trait or generic
/// shape; the supplied syntax tree is never modified.
pub(crate) fn validate_model_impl(item: &ItemImpl) -> Result<()> {
    if item.trait_.is_some() {
        return Err(Error::new_spanned(item, "ModelImpl requires an inherent impl"));
    }
    if !item.generics.params.is_empty() || item.generics.where_clause.is_some() {
        return Err(Error::new_spanned(&item.generics, "ModelImpl blocks cannot be generic"));
    }
    Ok(())
}

/// Expands property adapters and metadata for an already parsed implementation.
///
/// `item` is preserved in the emitted tokens and `runtime` identifies the
/// metadata facade. Returns diagnostics for invalid property-method contracts.
pub(crate) fn expand_model_impl(item: ItemImpl, runtime: &TokenStream) -> Result<TokenStream> {
    let target = (*item.self_ty).clone();
    let mut getters = Vec::new();
    let mut setters = Vec::new();
    let mut errors = None;
    for impl_item in &item.items {
        let ImplItem::Fn(method) = impl_item else {
            continue;
        };
        match parse_property_method(method) {
            Ok(Some(PropertyMethod::Getter(value))) => getters.push(value),
            Ok(Some(PropertyMethod::Setter(value))) => setters.push(value),
            Ok(None) => {}
            Err(error) => combine(&mut errors, error),
        }
    }
    validate_unique_property_methods(&getters, &setters, &mut errors);
    if let Some(error) = errors {
        return Err(error);
    }

    let target_suffix = stable_fingerprint(&quote!(#target).to_string());
    let provider = format_ident!("__qubit_model_impl_{target_suffix:016x}");
    let getter_adapters: Vec<_> = getters
        .iter()
        .enumerate()
        .map(|(index, getter)| expand_getter_adapter(index, getter, &target, runtime))
        .collect();
    let setter_adapters: Vec<_> = setters
        .iter()
        .enumerate()
        .map(|(index, setter)| expand_setter_adapter(index, setter, &target, runtime))
        .collect();
    let compatibility_assertions = expand_property_compatibility_assertions(&getters, &setters, runtime);
    let getter_metadata: Vec<_> = getters
        .iter()
        .enumerate()
        .map(|(index, getter)| {
            let property = &getter.property;
            let method = getter.method.to_string();
            let adapter = format_ident!("__qubit_model_property_getter_{index}_{target_suffix:016x}");
            let (ty, kind) = match &getter.output {
                GetterReturn::Owned(ty) => (quote!(#ty), quote!(#runtime::GetterOutputKind::Owned)),
                GetterReturn::Borrowed(ty) => (quote!(#ty), quote!(#runtime::GetterOutputKind::Borrowed)),
                GetterReturn::BorrowedStr => (quote!(str), quote!(#runtime::GetterOutputKind::Borrowed)),
                GetterReturn::BorrowedSlice(element) => {
                    (quote!([#element]), quote!(#runtime::GetterOutputKind::Borrowed))
                }
                GetterReturn::OptionalBorrowed(ty) => (
                    quote!(::core::option::Option<#ty>),
                    quote!(#runtime::GetterOutputKind::Borrowed),
                ),
                GetterReturn::OptionalBorrowedStr => (
                    quote!(::core::option::Option<::std::string::String>),
                    quote!(#runtime::GetterOutputKind::Borrowed),
                ),
            };
            quote! {
                            {
                                let output_type = #runtime::__private::codegen_v2::descriptor::lazy_type_ref::<#ty>().get();
            let getter = #runtime::__private::v4::leak(
                                    #runtime::GetterMetadata::new::<#target>(#method, output_type, #kind, #adapter),
                                );
                                fragments.push(#runtime::__private::v4::property_fragment(
                                    #property,
                                    output_type,
                                    #runtime::PropertyFragmentSource::Getter(getter),
                                ));
                                entries.push(Entry::getter(#property, output_type, getter));
                            }
                        }
        })
        .collect();
    let setter_metadata: Vec<_> = setters
        .iter()
        .enumerate()
        .map(|(index, setter)| {
            let property = &setter.property;
            let method = setter.method.to_string();
            let ty = &setter.input;
            let adapter = format_ident!("__qubit_model_property_setter_{index}_{target_suffix:016x}");
            quote! {
                {
                    let input_type = #runtime::__private::codegen_v2::descriptor::lazy_type_ref::<#ty>().get();
                    let setter = #runtime::__private::v4::leak(
                        #runtime::SetterMetadata::new::<#target, #ty>(#method, input_type, #adapter),
                    );
                    fragments.push(#runtime::__private::v4::property_fragment(
                        #property,
                        input_type,
                        #runtime::PropertyFragmentSource::Setter(setter),
                    ));
                    entries.push(Entry::setter(#property, input_type, setter));
                }
            }
        })
        .collect();

    Ok(quote! {
        #item

        #(#getter_adapters)*
        #(#setter_adapters)*
        #(#compatibility_assertions)*

        #[doc(hidden)]
        fn #provider() -> &'static #runtime::ModelImplMetadata {
            struct Entry {
                name: &'static str,
                type_ref: &'static #runtime::TypeRef,
                field: ::core::option::Option<&'static #runtime::FieldMetadata>,
                getter: ::core::option::Option<&'static #runtime::GetterMetadata>,
                setter: ::core::option::Option<&'static #runtime::SetterMetadata>,
            }
            impl Entry {
                fn getter(name: &'static str, type_ref: &'static #runtime::TypeRef, getter: &'static #runtime::GetterMetadata) -> Self {
                    Self { name, type_ref, field: None, getter: Some(getter), setter: None }
                }
                fn setter(name: &'static str, type_ref: &'static #runtime::TypeRef, setter: &'static #runtime::SetterMetadata) -> Self {
                    Self { name, type_ref, field: None, getter: None, setter: Some(setter) }
                }
            }
            static PROPERTIES: ::std::sync::OnceLock<#runtime::ModelImplMetadata> =
                ::std::sync::OnceLock::new();
            PROPERTIES.get_or_init(|| {
                let metadata = <#target as #runtime::__private::TypeMetadataProvider>::__type_metadata();
                let mut entries: ::std::vec::Vec<Entry> = ::std::vec::Vec::new();
                let mut fragments: ::std::vec::Vec<#runtime::PropertyFragment> = ::std::vec::Vec::new();
                for field in metadata.fields() {
                    if let Some(name) = field.name() {
                        fragments.push(#runtime::__private::v4::property_fragment(
                            name,
                            field.type_ref(),
                            #runtime::PropertyFragmentSource::Field(field),
                        ));
                        entries.push(Entry {
                        name,
                        type_ref: field.type_ref(),
                        field: Some(field),
                        getter: None,
                        setter: None,
                        });
                    }
                }
                #(#getter_metadata)*
                #(#setter_metadata)*
                let mut merged: ::std::vec::Vec<Entry> = ::std::vec::Vec::new();
                for entry in entries {
                    if let Some(current) = merged.iter_mut().find(|current| current.name == entry.name) {
                        if current.field.is_none() { current.field = entry.field; }
                        if current.getter.is_none() { current.getter = entry.getter; }
                        if current.setter.is_none() { current.setter = entry.setter; }
                    } else {
                        merged.push(entry);
                    }
                }
                let properties: ::std::vec::Vec<_> = merged.into_iter().map(|entry| {
                    #runtime::__private::v4::property_metadata(
                        entry.name, entry.type_ref, entry.field, entry.getter, entry.setter,
                    )
                }).collect();
                let properties = #runtime::__private::v4::leak_slice(properties);
                let properties = match metadata.validate_properties(properties) {
                    Ok(()) => Ok(#runtime::__private::v4::leak(
                        #runtime::__private::v4::local_property_set(properties),
                    )),
                    Err(errors) => Err(#runtime::__private::v4::leak(errors)),
                };
                let fragments = #runtime::__private::v4::leak_slice(fragments);
                #runtime::__private::v4::model_impl_metadata(fragments, properties)
            })
        }

        impl #runtime::__private::ModelImplSeal for #target {}
        #runtime::__private::v4::register_model_impl_capability!(
            #target,
            #provider as #runtime::ModelImplProvider,
        );
    })
}

/// Generates compile-time compatibility proofs for paired property methods.
///
/// Each assertion relates one getter's exact declared output shape to the
/// input accepted by the setter with the same canonical property name.
fn expand_property_compatibility_assertions(
    getters: &[GetterIr],
    setters: &[SetterIr],
    runtime: &TokenStream,
) -> Vec<TokenStream> {
    getters
        .iter()
        .filter_map(|getter| {
            let setter = setters.iter().find(|setter| setter.property == getter.property)?;
            let output = getter_output_type(&getter.output, runtime);
            let input = &setter.input;
            Some(quote! {
                const _: () = {
                    fn assert_property_types_are_compatible()
                    where
                        #output: #runtime::__private::v4::PropertyOutputCompatible<#input>,
                    {}
                };
            })
        })
        .collect()
}

/// Returns the exact source-level type shape produced by a getter.
fn getter_output_type(output: &GetterReturn, runtime: &TokenStream) -> TokenStream {
    match output {
        GetterReturn::Owned(ty) => quote!(#ty),
        GetterReturn::Borrowed(ty) => quote!(
            #runtime::__private::v4::BorrowedPropertyOutput<#ty>
        ),
        GetterReturn::BorrowedStr => quote!(
            #runtime::__private::v4::BorrowedPropertyOutput<str>
        ),
        GetterReturn::BorrowedSlice(element) => quote!(
            #runtime::__private::v4::BorrowedPropertyOutput<[#element]>
        ),
        GetterReturn::OptionalBorrowed(ty) => quote!(
            #runtime::__private::v4::OptionalBorrowedPropertyOutput<#ty>
        ),
        GetterReturn::OptionalBorrowedStr => quote!(
            #runtime::__private::v4::OptionalBorrowedPropertyOutput<str>
        ),
    }
}

/// Parses a property-shaped public method and ignores ordinary business
/// methods.
///
/// Setter-prefixed methods are explicit property declarations and therefore
/// receive diagnostics when they violate the setter contract.
fn parse_property_method(method: &ImplItemFn) -> Result<Option<PropertyMethod>> {
    let name = method.sig.ident.to_string();
    if let Some(property) = name.strip_prefix("set_") {
        if !matches!(method.vis, Visibility::Public(_)) {
            return Err(Error::new_spanned(&method.sig.ident, "property setters must be public"));
        }
        if method.sig.asyncness.is_some() || method.sig.unsafety.is_some() || method.sig.constness.is_some() {
            return Err(Error::new_spanned(
                &method.sig,
                "property setters must be safe synchronous non-const functions",
            ));
        }
        if !method.sig.generics.params.is_empty() || method.sig.generics.where_clause.is_some() {
            return Err(Error::new_spanned(
                &method.sig.generics,
                "property setters cannot be generic",
            ));
        }
        if property.is_empty() {
            return Err(Error::new_spanned(
                &method.sig.ident,
                "setter property name cannot be empty",
            ));
        }
        let mut inputs = method.sig.inputs.iter();
        let Some(FnArg::Receiver(receiver)) = inputs.next() else {
            return Err(Error::new_spanned(&method.sig, "setter requires `&mut self`"));
        };
        if receiver.reference.is_none() || receiver.mutability.is_none() {
            return Err(Error::new_spanned(receiver, "setter requires `&mut self`"));
        }
        let Some(FnArg::Typed(value)) = inputs.next() else {
            return Err(Error::new_spanned(
                &method.sig,
                "setter requires exactly one value parameter",
            ));
        };
        if inputs.next().is_some() || !returns_unit(&method.sig.output) {
            return Err(Error::new_spanned(
                &method.sig,
                "setter requires exactly one value parameter and unit return",
            ));
        }
        return Ok(Some(PropertyMethod::Setter(SetterIr {
            property: property.to_owned(),
            method: method.sig.ident.clone(),
            input: (*value.ty).clone(),
        })));
    }
    if !matches!(method.vis, Visibility::Public(_))
        || method.sig.asyncness.is_some()
        || method.sig.unsafety.is_some()
        || method.sig.constness.is_some()
        || !method.sig.generics.params.is_empty()
        || method.sig.generics.where_clause.is_some()
    {
        return Ok(None);
    }
    let mut inputs = method.sig.inputs.iter();
    let Some(FnArg::Receiver(receiver)) = inputs.next() else {
        return Ok(None);
    };
    if receiver.reference.is_none() || receiver.mutability.is_some() || inputs.next().is_some() {
        return Ok(None);
    }
    let ReturnType::Type(_, output) = &method.sig.output else {
        return Ok(None);
    };
    let output = match output.as_ref() {
        Type::Reference(reference) if reference.mutability.is_none() => {
            if matches!(reference.elem.as_ref(), Type::Path(path) if path.path.is_ident("str")) {
                GetterReturn::BorrowedStr
            } else if let Type::Slice(slice) = reference.elem.as_ref() {
                GetterReturn::BorrowedSlice((*slice.elem).clone())
            } else {
                GetterReturn::Borrowed((*reference.elem).clone())
            }
        }
        Type::Path(path) => {
            if let Some(reference) = option_borrowed_type(path) {
                if matches!(reference.elem.as_ref(), Type::Path(path) if path.path.is_ident("str")) {
                    GetterReturn::OptionalBorrowedStr
                } else {
                    GetterReturn::OptionalBorrowed((*reference.elem).clone())
                }
            } else {
                GetterReturn::Owned(output.as_ref().clone())
            }
        }
        Type::Tuple(tuple) if tuple.elems.is_empty() => {
            return Ok(None);
        }
        output => GetterReturn::Owned(output.clone()),
    };
    Ok(Some(PropertyMethod::Getter(GetterIr {
        property: name,
        method: method.sig.ident.clone(),
        output,
    })))
}

/// Returns the borrowed inner type when `path` is an `Option<&T>` spelling.
///
/// `None` means that the syntax does not represent the supported optional
/// borrowed getter return shape.
fn option_borrowed_type(path: &TypePath) -> Option<&TypeReference> {
    if path.qself.is_some() || !is_option_path(&path.path) {
        return None;
    }
    let segment = path.path.segments.last()?;
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };
    if arguments.args.len() != 1 {
        return None;
    }
    let GenericArgument::Type(Type::Reference(reference)) = arguments.args.first()? else {
        return None;
    };
    (reference.mutability.is_none()).then_some(reference)
}

/// Reports whether `output` is the unit return type required by setters.
fn returns_unit(output: &ReturnType) -> bool {
    matches!(output, ReturnType::Default)
        || matches!(output, ReturnType::Type(_, ty) if matches!(ty.as_ref(), Type::Tuple(tuple) if tuple.elems.is_empty()))
}

/// Adds diagnostics for duplicate getter or setter property names.
///
/// `errors` accumulates all failures so callers can return one combined
/// compiler diagnostic instead of stopping at the first duplicate.
fn validate_unique_property_methods(getters: &[GetterIr], setters: &[SetterIr], errors: &mut Option<Error>) {
    for (index, getter) in getters.iter().enumerate() {
        if getters[..index].iter().any(|other| other.property == getter.property) {
            combine(errors, Error::new_spanned(&getter.method, "duplicate property getter"));
        }
    }
    for (index, setter) in setters.iter().enumerate() {
        if setters[..index].iter().any(|other| other.property == setter.property) {
            combine(errors, Error::new_spanned(&setter.method, "duplicate property setter"));
        }
    }
}

/// Generates the adapter that exposes one getter through runtime reflection.
///
/// `index` makes the generated symbol unique; `getter`, `target`, and
/// `runtime` supply the validated method contract and emitted type paths.
fn expand_getter_adapter(index: usize, getter: &GetterIr, target: &Type, runtime: &TokenStream) -> TokenStream {
    let target_suffix = stable_fingerprint(&quote!(#target).to_string());
    let adapter = format_ident!("__qubit_model_property_getter_{index}_{target_suffix:016x}",);
    let method = &getter.method;
    let value = match &getter.output {
        GetterReturn::Owned(_) => {
            quote!(#runtime::PropertyValue::Owned(#runtime::ReflectedOwned::new(target.#method())))
        }
        GetterReturn::Borrowed(_) => {
            quote!(#runtime::PropertyValue::Borrowed(#runtime::ReflectedRef::new(target.#method())))
        }
        GetterReturn::BorrowedStr => {
            quote!(#runtime::PropertyValue::Borrowed(#runtime::ReflectedRef::new_str(target.#method())))
        }
        GetterReturn::BorrowedSlice(_) => {
            quote!(#runtime::PropertyValue::BorrowedSlice(#runtime::BorrowedPropertySlice::new(target.#method())))
        }
        GetterReturn::OptionalBorrowed(_) => {
            quote!(#runtime::PropertyValue::OptionalBorrowed(target.#method().map(#runtime::ReflectedRef::new)))
        }
        GetterReturn::OptionalBorrowedStr => {
            quote!(#runtime::PropertyValue::OptionalBorrowed(target.#method().map(#runtime::ReflectedRef::new_str)))
        }
    };
    quote! {
        #[doc(hidden)]
        fn #adapter<'a>(target: #runtime::ReflectedRef<'a>) -> ::core::result::Result<#runtime::PropertyValue<'a>, #runtime::PropertyAccessError> {
            let target = target.downcast::<#target>().map_err(|_| #runtime::PropertyAccessError::user("property target was not prevalidated"))?;
            Ok(#value)
        }
    }
}

/// Generates the adapter that exposes one setter through runtime reflection.
///
/// `index` makes the generated symbol unique; `setter`, `target`, and
/// `runtime` supply the validated method contract and emitted type paths.
fn expand_setter_adapter(index: usize, setter: &SetterIr, target: &Type, runtime: &TokenStream) -> TokenStream {
    let target_suffix = stable_fingerprint(&quote!(#target).to_string());
    let adapter = format_ident!("__qubit_model_property_setter_{index}_{target_suffix:016x}",);
    let method = &setter.method;
    let input = &setter.input;
    quote! {
        #[doc(hidden)]
        fn #adapter(target: #runtime::ReflectedMut<'_>, value: #runtime::ReflectedOwned) -> ::core::result::Result<(), #runtime::PropertySetFailure> {
            let target = target.downcast::<#target>().map_err(|_| #runtime::PropertySetFailure::after_execution(#runtime::PropertyAccessError::user("property target was not prevalidated")))?;
            let value = value.downcast::<#input>().map_err(|value| #runtime::PropertySetFailure::before_execution(#runtime::PropertyAccessError::user("property value was not prevalidated"), value))?;
            target.#method(value);
            Ok(())
        }
    }
}

/// Appends `error` to the optional combined compiler diagnostic.
fn combine(errors: &mut Option<Error>, error: Error) {
    match errors {
        Some(current) => current.combine(error),
        None => *errors = Some(error),
    }
}
