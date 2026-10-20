// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Model-owned adapters over the reflection codegen protocol.

/// Emits generic model expansion only when the runtime feature is enabled.
#[doc(hidden)]
#[macro_export]
#[cfg(feature = "generic")]
macro_rules! __qubit_model_with_generic_feature {
    ($($tokens:tt)*) => { $($tokens)* };
}

/// Reports a stable diagnostic when generic model support is disabled.
#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "generic"))]
macro_rules! __qubit_model_with_generic_feature {
    ($($tokens:tt)*) => {
        compile_error!("generic model metadata requires the 'generic' feature");
    };
}

/// Registers generated model metadata on a generic reflection definition.
#[doc(hidden)]
#[macro_export]
#[cfg(feature = "generic")]
macro_rules! __qubit_model_register_generic_model_capability {
    (
        definition_path = $definition_path:expr,
        capability = $capability:ident,
        metadata = $metadata:path,
        source = (
            $declaring_crate:expr,
            $module_path:expr,
            $line:expr,
            $column:expr,
            $fingerprint:expr
        ),
    ) => {
        const _: () = {
            fn registration() -> $crate::__private::codegen_v4::registration::Registration {
                $crate::__private::codegen_v4::registration::Registration::from(super::$capability::<
                    $crate::__private::v8::GenericModelMetadataCapability,
                >($metadata))
            }

            $crate::__private::codegen_v4::registration::submit_linked_definition_capability!(
                concat!($definition_path, "::generic-model-capability"),
                registration
            );

            const _: (&str, &str, u32, u32, u64) = ($declaring_crate, $module_path, $line, $column, $fingerprint);
        };
    };
}

/// Registers the fixed model implementation capability on a generic definition.
#[doc(hidden)]
#[macro_export]
#[cfg(feature = "generic")]
macro_rules! __qubit_model_register_generic_model_impl_capability {
    (
        definition_path = $definition_path:expr,
        capability = $capability:ident,
        source = (
            $declaring_crate:expr,
            $module_path:expr,
            $line:expr,
            $column:expr,
            $fingerprint:expr
        ),
    ) => {
        const _: () = {
            fn registration() -> $crate::__private::codegen_v4::registration::Registration {
                $crate::__private::codegen_v4::registration::Registration::from(super::$capability::<
                    $crate::__private::v8::ModelImplCapability,
                >(
                    $crate::__private::v8::all_model_impl_fragments(),
                ))
            }

            $crate::__private::codegen_v4::registration::submit_linked_definition_capability!(
                concat!($definition_path, "::model-impl-capability"),
                registration
            );

            const _: (&str, &str, u32, u32, u64) = ($declaring_crate, $module_path, $line, $column, $fingerprint);
        };
    };
}

/// Submits the generated concrete model metadata capability for `T`.
#[doc(hidden)]
#[macro_export]
macro_rules! __qubit_model_submit_metadata_capability {
    ($ty:ty $(,)?) => {
        const _: () = {
            fn registration() -> $crate::__private::codegen_v4::registration::Registration {
                $crate::__private::codegen_v4::registration::Registration::from(
                    $crate::__private::v8::model_capability::<$ty>(),
                )
            }

            $crate::__private::__inventory::submit! {
                $crate::__private::codegen_v4::registration::RegistrationFragment::for_global_registration(
                    concat!(module_path!(), "::", stringify!($ty), "::model-metadata-capability"),
                    registration,
                )
            }
        };
    };
}

/// Submits the aggregate `ModelImpl` capability for a concrete model type.
#[doc(hidden)]
#[macro_export]
macro_rules! __qubit_model_submit_model_impl_capability {
    ($ty:ty $(,)?) => {
        const _: () = {
            fn registration() -> $crate::__private::codegen_v4::registration::Registration {
                $crate::__private::codegen_v4::registration::Registration::from(
                    $crate::__private::v8::model_impl_registration::<$ty>(),
                )
            }

            $crate::__private::__inventory::submit! {
                $crate::__private::codegen_v4::registration::RegistrationFragment::for_global_registration(
                    concat!(module_path!(), "::", stringify!($ty), "::model-impl-capability"),
                    registration,
                )
            }
        };
    };
}
