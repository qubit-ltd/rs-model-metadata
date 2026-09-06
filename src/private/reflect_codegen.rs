// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Model-owned adapters over the reflection codegen protocol.

use qubit_reflect::__private::codegen_v2::descriptor::lazy_type_ref;

/// Returns reflection's resolved root reference for `T`.
#[doc(hidden)]
#[must_use]
#[inline(always)]
pub fn reflected_type_ref<T: crate::Reflect + ?Sized>() -> &'static crate::TypeRef {
    lazy_type_ref::<T>().get()
}

/// Registers generated model metadata on a generic reflection definition.
#[doc(hidden)]
#[macro_export]
macro_rules! __qubit_model_register_generic_model_capability {
    (
        definition = $definition:path,
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
            fn runtime_identity() -> $crate::__private::codegen_v2::registration::RuntimeIdentity {
                $crate::__private::codegen_v2::registration::RuntimeIdentity::Capabilities(
                    $crate::__private::codegen_v2::registration::CapabilityTarget::TypeDefinition($definition().id()),
                )
            }

            fn payload() -> $crate::__private::codegen_v2::registration::FragmentPayload {
                $crate::__private::codegen_v2::registration::FragmentPayload::Capability(
                    $crate::__private::codegen_v2::registration::CapabilityRegistration::for_definition(
                        $definition(),
                        ::std::vec![$crate::__private::v4::generic_model_capability($metadata)],
                    ),
                )
            }

            $crate::__private::codegen_v2::inventory::submit! {
                $crate::__private::codegen_v2::registration::RegistrationFragment::new(
                    $crate::__private::codegen_v2::registration::FragmentKind::Capability,
                    $crate::__private::codegen_v2::registration::StaticFragmentIdentity::new(
                        $declaring_crate,
                        $module_path,
                        $line,
                        $column,
                        "generic-model-capability",
                        $fingerprint,
                    ),
                    runtime_identity,
                    payload,
                )
            }
        };
    };
}
