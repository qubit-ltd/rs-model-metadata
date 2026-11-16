// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Shared traversal of transparent model descriptor wrappers.

use qubit_reflect::TypeDescriptor;
use qubit_reflect::descriptor::TypeKind;

/// Resolves the first descriptor outside optional and smart-pointer layers.
///
/// # Type Parameters
///
/// - `'ctx`: The lifetime of the reflection context associated with the
///   descriptor.
///
/// # Parameters
///
/// - `descriptor`: The descriptor to resolve. Optional and smart-pointer layers
///   are followed through their first generic type argument.
///
/// # Returns
///
/// Returns the first non-wrapper descriptor, still bound to the input
/// descriptor's reflection context. Returns `None` if a wrapper has no first
/// generic argument or that argument cannot be resolved to a concrete type in
/// that context.
pub(crate) fn transparent_descriptor<'ctx>(mut descriptor: TypeDescriptor<'ctx>) -> Option<TypeDescriptor<'ctx>> {
    loop {
        match descriptor.kind() {
            TypeKind::Optional(_) | TypeKind::SmartPointer(_) => {
                let argument = descriptor.generic_arguments().get_at(0)?;
                descriptor = argument.type_argument().ok()?.find_concrete_type()?;
            }
            _ => return Some(descriptor),
        }
    }
}
