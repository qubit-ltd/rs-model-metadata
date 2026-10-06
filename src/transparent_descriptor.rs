// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Shared traversal of transparent model descriptor wrappers.

use qubit_reflect::TypeDescriptor;

/// Returns the first descriptor outside optional and smart-pointer layers.
///
/// # Parameters
///
/// - `descriptor`: the descriptor to inspect, including any optional or
///   smart-pointer wrapper layers.
///
/// # Returns
///
/// Returns the first resolved descriptor outside those wrappers. Returns
/// `None` when a traversed layer is opaque or symbolic. Semantic containers
/// such as sequences and maps remain intact.
#[must_use]
pub(crate) fn transparent_descriptor(
    mut descriptor: &'static TypeDescriptor,
) -> Option<&'static TypeDescriptor> {
    loop {
        let element = descriptor
            .as_optional()
            .map(|view| view.element_type())
            .or_else(|| {
                descriptor
                    .as_smart_pointer()
                    .map(|view| view.pointee_type())
            });
        let Some(element) = element else {
            return Some(descriptor);
        };
        descriptor = element.as_resolved()?;
    }
}
