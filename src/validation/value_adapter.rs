//! Shape helpers shared by the future validation executor.

#![allow(dead_code)]

use std::any::TypeId;

use qubit_reflect::TypeDescriptor;
use qubit_validator::next::InputType;

/// Returns the executable input shape for a reflected type in value mode.
pub(crate) fn input_type(descriptor: &'static TypeDescriptor) -> InputType {
    let descriptor = innermost_descriptor(descriptor);
    if matches!(descriptor.kind(), qubit_reflect::descriptor::TypeKind::Text(_)) {
        InputType::Text
    } else {
        InputType::Typed(descriptor.type_id())
    }
}

/// Returns the exact type identity after transparent value expansion.
pub(crate) fn type_id(descriptor: &'static TypeDescriptor) -> TypeId {
    innermost_descriptor(descriptor).type_id()
}

fn innermost_descriptor(mut descriptor: &'static TypeDescriptor) -> &'static TypeDescriptor {
    loop {
        let Some(element) = descriptor
            .as_optional()
            .map(|view| view.element_type())
            .or_else(|| descriptor.as_smart_pointer().map(|view| view.pointee_type()))
        else {
            return descriptor;
        };
        let Some(next) = element.as_resolved() else {
            return descriptor;
        };
        descriptor = next;
    }
}
