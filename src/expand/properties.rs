// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Validates and expands getter/setter-backed model properties.

use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::Error;
use syn::FnArg;
use syn::ImplItem;
use syn::ImplItemFn;
use syn::ItemImpl;
use syn::Result;
use syn::ReturnType;
use syn::Type;
use syn::Visibility;

pub(crate) fn validate_property_impl(item: &ItemImpl) -> Result<()> {
    if item.trait_.is_some() {
        return Err(Error::new_spanned(item, "ModelProperties requires an inherent impl"));
    }
    if !item.generics.params.is_empty() || item.generics.where_clause.is_some() {
        return Err(Error::new_spanned(
            &item.generics,
            "ModelProperties impls cannot be generic",
        ));
    }
    Ok(())
}

#[derive(Clone)]
enum GetterReturn {
    Owned(Type),
    Borrowed(Type),
    BorrowedStr,
    BorrowedSlice(Type),
    OptionalBorrowed(Type),
    OptionalBorrowedStr,
}

#[derive(Clone)]
struct GetterIr {
    property: String,
    method: syn::Ident,
    output: GetterReturn,
}

#[derive(Clone)]
struct SetterIr {
    property: String,
    method: syn::Ident,
    input: Type,
}

pub(crate) fn expand_properties(item: ItemImpl, runtime: &TokenStream) -> Result<TokenStream> {
    let target = (*item.self_ty).clone();
    let mut getters = Vec::new();
    let mut setters = Vec::new();
    let mut errors = None;
    for impl_item in &item.items {
        let ImplItem::Fn(method) = impl_item else { continue };
        match parse_property_method(method) {
            Ok(PropertyMethod::Getter(value)) => getters.push(value),
            Ok(PropertyMethod::Setter(value)) => setters.push(value),
            Err(error) => combine(&mut errors, error),
        }
    }
    validate_unique_property_methods(&getters, &setters, &mut errors);
    if let Some(error) = errors {
        return Err(error);
    }

    let target_name = quote!(#target)
        .to_string()
        .to_snake_case()
        .replace([' ', ':', '<', '>', ','], "_");
    let provider = format_ident!("__qubit_model_properties_{}", target_name);
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
    let getter_metadata: Vec<_> = getters
        .iter()
        .enumerate()
        .map(|(index, getter)| {
            let property = &getter.property;
            let method = getter.method.to_string();
            let adapter = format_ident!("__qubit_model_property_getter_{index}_{}", target_name);
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
                    let output_type = #runtime::__private::descriptor::lazy_type_ref::<#ty>().get();
                    let getter = ::std::boxed::Box::leak(::std::boxed::Box::new(
                        #runtime::GetterMetadata::new::<#target>(#method, output_type, #kind, #adapter),
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
            let adapter = format_ident!("__qubit_model_property_setter_{index}_{}", target_name);
            quote! {
                {
                    let input_type = #runtime::__private::descriptor::lazy_type_ref::<#ty>().get();
                    let setter = ::std::boxed::Box::leak(::std::boxed::Box::new(
                        #runtime::SetterMetadata::new::<#target, #ty>(#method, input_type, #adapter),
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

        #[doc(hidden)]
        fn #provider() -> &'static [#runtime::PropertyMetadata] {
            struct Entry {
                name: &'static str,
                type_ref: &'static #runtime::descriptor::TypeRef,
                field: ::core::option::Option<&'static #runtime::FieldMetadata>,
                getter: ::core::option::Option<&'static #runtime::GetterMetadata>,
                setter: ::core::option::Option<&'static #runtime::SetterMetadata>,
            }
            impl Entry {
                fn getter(name: &'static str, type_ref: &'static #runtime::descriptor::TypeRef, getter: &'static #runtime::GetterMetadata) -> Self {
                    Self { name, type_ref, field: None, getter: Some(getter), setter: None }
                }
                fn setter(name: &'static str, type_ref: &'static #runtime::descriptor::TypeRef, setter: &'static #runtime::SetterMetadata) -> Self {
                    Self { name, type_ref, field: None, getter: None, setter: Some(setter) }
                }
            }
            static PROPERTIES: ::std::sync::OnceLock<&'static [#runtime::PropertyMetadata]> =
                ::std::sync::OnceLock::new();
            PROPERTIES.get_or_init(|| {
                let metadata = #runtime::TypeMetadata::of::<#target>();
                let mut entries: ::std::vec::Vec<Entry> = metadata.fields().iter().filter_map(|field| {
                    field.reflect().query_name().map(|name| Entry {
                        name,
                        type_ref: field.type_ref(),
                        field: Some(field),
                        getter: None,
                        setter: None,
                    })
                }).collect();
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
                    #runtime::PropertyMetadata::new(
                        entry.name, entry.type_ref, entry.field, entry.getter, entry.setter,
                    )
                }).collect();
                ::std::boxed::Box::leak(properties.into_boxed_slice()) as &'static [#runtime::PropertyMetadata]
            })
        }

        impl #runtime::__private::ModelPropertiesSeal for #target {}
        #runtime::__private::v1::register_properties_capability!(
            #target,
            #provider as #runtime::ModelPropertiesProvider,
        );
    })
}

enum PropertyMethod {
    Getter(GetterIr),
    Setter(SetterIr),
}

fn parse_property_method(method: &ImplItemFn) -> Result<PropertyMethod> {
    if !matches!(method.vis, Visibility::Public(_)) {
        return Err(Error::new_spanned(
            &method.sig.ident,
            "ModelProperties methods must be public",
        ));
    }
    if method.sig.asyncness.is_some() || method.sig.unsafety.is_some() || method.sig.constness.is_some() {
        return Err(Error::new_spanned(
            &method.sig,
            "property methods must be safe synchronous non-const functions",
        ));
    }
    if !method.sig.generics.params.is_empty() || method.sig.generics.where_clause.is_some() {
        return Err(Error::new_spanned(
            &method.sig.generics,
            "property methods cannot be generic",
        ));
    }
    let name = method.sig.ident.to_string();
    if let Some(property) = name.strip_prefix("set_") {
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
        return Ok(PropertyMethod::Setter(SetterIr {
            property: property.to_owned(),
            method: method.sig.ident.clone(),
            input: (*value.ty).clone(),
        }));
    }
    let mut inputs = method.sig.inputs.iter();
    let Some(FnArg::Receiver(receiver)) = inputs.next() else {
        return Err(Error::new_spanned(&method.sig, "getter requires `&self`"));
    };
    if receiver.reference.is_none() || receiver.mutability.is_some() || inputs.next().is_some() {
        return Err(Error::new_spanned(&method.sig, "getter requires only `&self`"));
    }
    let ReturnType::Type(_, output) = &method.sig.output else {
        return Err(Error::new_spanned(
            &method.sig.output,
            "getter requires a non-unit return type",
        ));
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
            return Err(Error::new_spanned(output, "getter requires a non-unit return type"));
        }
        output => GetterReturn::Owned(output.clone()),
    };
    Ok(PropertyMethod::Getter(GetterIr {
        property: name,
        method: method.sig.ident.clone(),
        output,
    }))
}

fn option_borrowed_type(path: &syn::TypePath) -> Option<&syn::TypeReference> {
    let segment = path.path.segments.last()?;
    if segment.ident != "Option" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };
    if arguments.args.len() != 1 {
        return None;
    }
    let syn::GenericArgument::Type(Type::Reference(reference)) = arguments.args.first()? else {
        return None;
    };
    (reference.mutability.is_none()).then_some(reference)
}

fn returns_unit(output: &ReturnType) -> bool {
    matches!(output, ReturnType::Default)
        || matches!(output, ReturnType::Type(_, ty) if matches!(ty.as_ref(), Type::Tuple(tuple) if tuple.elems.is_empty()))
}

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

fn expand_getter_adapter(index: usize, getter: &GetterIr, target: &Type, runtime: &TokenStream) -> TokenStream {
    let adapter = format_ident!(
        "__qubit_model_property_getter_{index}_{}",
        quote!(#target)
            .to_string()
            .to_snake_case()
            .replace([' ', ':', '<', '>', ','], "_")
    );
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

fn expand_setter_adapter(index: usize, setter: &SetterIr, target: &Type, runtime: &TokenStream) -> TokenStream {
    let adapter = format_ident!(
        "__qubit_model_property_setter_{index}_{}",
        quote!(#target)
            .to_string()
            .to_snake_case()
            .replace([' ', ':', '<', '>', ','], "_")
    );
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

fn combine(errors: &mut Option<Error>, error: Error) {
    match errors {
        Some(current) => current.combine(error),
        None => *errors = Some(error),
    }
}
