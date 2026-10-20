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

/// Returns the first descriptor outside optional and smart-pointer layers.
#[must_use]
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
