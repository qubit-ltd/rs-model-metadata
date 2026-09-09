// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Structural, getter-free compilation of property paths.

// qubit-style: allow multiple-public-types

use qubit_reflect::TypeDescriptor;
use qubit_reflect::descriptor::TypeKind;
use qubit_validator::BindError;
use qubit_validator::BindErrorKind;
use qubit_validator::InputType;

use super::build_error::path_error;
use crate::metadata::PropertyMetadata;
use crate::metadata::PropertyPath;
use crate::metadata::TargetMode;
use crate::metadata::TypeMetadata;
use crate::resolve::ModelGraph;

/// One property access step retained for a later executor.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PropertyStep {
    property: &'static PropertyMetadata,
    optional: bool,
}

impl PropertyStep {
    /// Returns the metadata for this path segment.
    pub(crate) const fn property(self) -> &'static PropertyMetadata {
        self.property
    }

    /// Returns whether this segment can produce no value.
    pub(crate) const fn optional(self) -> bool {
        self.optional
    }
}

/// A fully checked path; no user getter is called while constructing it.
#[derive(Clone, Debug)]
pub(crate) struct CompiledPropertyPath {
    /// Number of external containing objects needed to read this path.
    context_depth: usize,
    /// Property suffix whose owner type is supplied at execution.
    deferred: Box<[&'static str]>,
    steps: Box<[PropertyStep]>,
    input: InputType,
    optional: bool,
}

impl CompiledPropertyPath {
    /// Checks and compiles a declaration path without invoking user adapters.
    pub(crate) fn compile(
        root: &'static TypeMetadata,
        path: &PropertyPath<'_>,
        graph: &ModelGraph<'_>,
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
            path_optional |= optional;
            steps.push(PropertyStep { property, optional });
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
        let (descriptor, _optional) = match target {
            TargetMode::Value => value_descriptor(descriptor),
            TargetMode::Container => (descriptor, false),
        };
        let input = if matches!(descriptor.kind(), TypeKind::Text(_)) {
            InputType::Text
        } else {
            InputType::Typed(descriptor.type_id())
        };
        Ok(Self {
            context_depth: 0,
            deferred: Box::new([]),
            steps: steps.into_boxed_slice(),
            input,
            optional: path_optional || last.optional(),
        })
    }

    /// Compiles navigation relative to the owning object, then property
    /// selection.
    pub(crate) fn compile_dependency(
        root: &'static TypeMetadata,
        prefix: &[&'static str],
        binding: &crate::metadata::DependencyBindingMetadata,
        graph: &ModelGraph<'_>,
        ancestors: &[&'static TypeMetadata],
        expected: InputType,
    ) -> Result<Self, BindError> {
        let mut segments = prefix.to_vec();
        let mut depth = 0usize;
        for step in binding.object_path().steps() {
            match step {
                crate::metadata::NavigationStep::Property(name) => segments.push(*name),
                crate::metadata::NavigationStep::Parent => {
                    if segments.pop().is_none() {
                        depth += 1;
                    }
                }
            }
        }
        segments.extend_from_slice(binding.property().segments());
        let owner = if depth == 0 {
            root
        } else if let Some(owner) = ancestors.get(depth - 1) {
            *owner
        } else {
            return Ok(Self {
                context_depth: depth,
                deferred: segments.into_boxed_slice(),
                steps: Box::new([]),
                input: expected,
                optional: false,
            });
        };
        let mut path = Self::compile(owner, &PropertyPath::new(&segments), graph, TargetMode::Value)?;
        path.context_depth = depth;
        Ok(path)
    }

    /// Returns a suffix requiring a runtime containing-object type.
    pub(crate) fn deferred(&self) -> &[&'static str] {
        &self.deferred
    }

    /// Returns how many external parent objects precede the selected property.
    pub(crate) const fn context_depth(&self) -> usize {
        self.context_depth
    }

    /// Returns the checked path steps in traversal order.
    pub(crate) fn steps(&self) -> &[PropertyStep] {
        &self.steps
    }

    /// Returns the validator input type for the final path value.
    pub(crate) const fn input_type(&self) -> InputType {
        self.input
    }

    /// Returns whether any path segment can be absent.
    pub(crate) const fn is_optional(&self) -> bool {
        self.optional
    }
}

/// Removes transparent wrappers and records whether an optional was found.
fn value_descriptor(mut descriptor: &'static TypeDescriptor) -> (&'static TypeDescriptor, bool) {
    let mut optional = false;
    loop {
        let Some(element) = descriptor
            .as_optional()
            .map(|view| view.element_type())
            .or_else(|| descriptor.as_smart_pointer().map(|view| view.pointee_type()))
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
// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================
