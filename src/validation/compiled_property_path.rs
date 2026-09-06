//! Structural, getter-free compilation of property paths.

#![allow(dead_code)]

// qubit-style: allow multiple-public-types

use std::any::TypeId;

use qubit_reflect::TypeDescriptor;
use qubit_reflect::descriptor::TypeKind;
use qubit_validator::BindError;
use qubit_validator::BindErrorKind;
use qubit_validator::InputType;

use super::build_error::path_error;
use crate::PropertyMetadata;
use crate::PropertyPath;
use crate::ResolvedModelGraph;
use crate::TargetMode;
use crate::TypeMetadata;

/// One property access step retained for a later executor.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PropertyStep {
    property: &'static PropertyMetadata,
    receiver_type: TypeId,
    value_type: TypeId,
    optional: bool,
}

impl PropertyStep {
    pub(crate) const fn property(self) -> &'static PropertyMetadata {
        self.property
    }

    pub(crate) const fn receiver_type(self) -> TypeId {
        self.receiver_type
    }

    pub(crate) const fn value_type(self) -> TypeId {
        self.value_type
    }

    pub(crate) const fn optional(self) -> bool {
        self.optional
    }
}

/// A fully checked path; no user getter is called while constructing it.
#[derive(Clone, Debug)]
pub(crate) struct CompiledPropertyPath {
    steps: Box<[PropertyStep]>,
    input: InputType,
    optional: bool,
}

impl CompiledPropertyPath {
    pub(crate) fn compile(
        root: &'static TypeMetadata,
        path: &PropertyPath<'_>,
        graph: &ResolvedModelGraph<'_>,
        target: TargetMode,
    ) -> Result<Self, BindError> {
        if path.is_empty() {
            return Err(path_error(BindErrorKind::UnreadablePath));
        }
        let mut current = root;
        let mut steps = Vec::with_capacity(path.segments().len());
        let mut path_optional = false;
        for (index, segment) in path.segments().iter().enumerate() {
            let properties = graph
                .registry()
                .properties_for(current)
                .map_err(|_| path_error(BindErrorKind::UnreadablePath))?;
            let property = properties
                .property(segment)
                .ok_or_else(|| path_error(BindErrorKind::UnreadablePath))?;
            if !property.is_readable() {
                return Err(path_error(BindErrorKind::UnreadablePath));
            }
            let descriptor = property
                .descriptor()
                .ok_or_else(|| path_error(BindErrorKind::UnsupportedInput))?;
            let (value_descriptor, optional) = match target {
                TargetMode::Value => value_descriptor(descriptor),
                TargetMode::Container => (descriptor, false),
            };
            let value_type = value_descriptor.type_id();
            path_optional |= optional;
            steps.push(PropertyStep {
                property,
                receiver_type: current.type_id(),
                value_type,
                optional,
            });
            if index + 1 < path.segments().len() {
                current = graph
                    .registry()
                    .metadata_for(value_descriptor)
                    .map_err(|_| path_error(BindErrorKind::UnreadablePath))?
                    .ok_or_else(|| path_error(BindErrorKind::UnreadablePath))?;
            }
        }
        let last = steps.last().copied().expect("non-empty path");
        let descriptor = graph
            .registry()
            .properties_for(current)
            .map_err(|_| path_error(BindErrorKind::UnreadablePath))?
            .property(path.segments().last().copied().expect("non-empty path"))
            .and_then(PropertyMetadata::descriptor)
            .ok_or_else(|| path_error(BindErrorKind::UnsupportedInput))?;
        let (descriptor, optional) = match target {
            TargetMode::Value => value_descriptor(descriptor),
            TargetMode::Container => (descriptor, false),
        };
        let input = if matches!(descriptor.kind(), TypeKind::Text(_)) {
            InputType::Text
        } else {
            InputType::Typed(descriptor.type_id())
        };
        Ok(Self {
            steps: steps.into_boxed_slice(),
            input,
            optional: path_optional || last.optional(),
        })
    }

    pub(crate) fn steps(&self) -> &[PropertyStep] {
        &self.steps
    }

    pub(crate) const fn input_type(&self) -> InputType {
        self.input
    }

    pub(crate) const fn is_optional(&self) -> bool {
        self.optional
    }
}

fn value_descriptor(mut descriptor: &'static TypeDescriptor) -> (&'static TypeDescriptor, bool) {
    let mut optional = false;
    loop {
        let Some(element) = descriptor
            .as_optional()
            .map(|view| view.element_type())
            .or_else(|| {
                descriptor
                    .as_smart_pointer()
                    .map(|view| view.pointee_type())
            })
        else {
            return (descriptor, optional);
        };
        optional |= descriptor.as_optional().is_some();
        let Some(next) = element.as_resolved() else {
            return (descriptor, optional);
        };
        descriptor = next;
    }
}
