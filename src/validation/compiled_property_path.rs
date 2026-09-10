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
use crate::metadata::GetterOutputKind;
use crate::metadata::PropertyMetadata;
use crate::metadata::PropertyPath;
use crate::metadata::TargetMode;
use crate::metadata::TypeMetadata;
use crate::resolve::ModelGraph;

/// One property access step retained for a later executor.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PropertyStep {
    property: &'static PropertyMetadata,
}

impl PropertyStep {
    /// Returns the metadata for this path segment.
    pub(crate) const fn property(self) -> &'static PropertyMetadata {
        self.property
    }
}

/// A fully checked path; no user getter is called while constructing it.
#[derive(Clone, Debug)]
pub(crate) struct CompiledPropertyPath {
    /// Number of external containing objects needed to read this path.
    pub(super) context_depth: usize,
    /// Original named dependency navigation, when this path supplies a slot.
    pub(super) dependency: Option<crate::metadata::DependencyBindingMetadata>,
    /// Property suffix whose owner type is supplied at execution.
    pub(super) deferred: Box<[&'static str]>,
    pub(super) steps: Box<[PropertyStep]>,
    pub(super) input: InputType,
    pub(super) optional: bool,
}

impl CompiledPropertyPath {
    /// Checks and compiles a declaration path without invoking user adapters.
    pub(crate) fn compile(
        root: &'static TypeMetadata,
        path: &PropertyPath<'_>,
        graph: &ModelGraph<'_>,
        target: TargetMode,
    ) -> Result<Self, BindError> {
        if path.is_empty() || graph.model(root.type_id()).is_none() {
            return Err(path_error(BindErrorKind::UnreadablePath));
        }
        let mut current = root;
        let mut steps = Vec::with_capacity(path.segments().len());
        let mut path_optional = false;
        let mut input = InputType::of::<()>();
        for (index, segment) in path.segments().iter().enumerate() {
            let property = graph
                .properties(current)
                .and_then(|properties| properties.property(segment))
                .ok_or_else(|| path_error(BindErrorKind::UnreadablePath))?;
            if !property.is_readable() {
                return Err(path_error(BindErrorKind::UnreadablePath));
            }
            let descriptor = property
                .descriptor()
                .ok_or_else(|| path_error(BindErrorKind::UnsupportedInput))?;
            let last = index + 1 == path.segments().len();
            let value_target = !last || matches!(target, TargetMode::Value);
            let expected = if value_target {
                value_descriptor(descriptor).0
            } else {
                descriptor
            };
            let (actual, optional) = if let Some(getter) = property.getter() {
                let output = getter
                    .output_type()
                    .as_resolved()
                    .ok_or_else(|| path_error(BindErrorKind::UnsupportedInput))?;
                if !last
                    && matches!(
                        getter.output_kind(),
                        GetterOutputKind::Owned | GetterOutputKind::BorrowedSlice
                    )
                {
                    return Err(path_error(BindErrorKind::UnsupportedConstraint));
                }
                if matches!(getter.output_kind(), GetterOutputKind::OptionalBorrowed) {
                    let inner = output
                        .as_optional()
                        .and_then(|value| value.element_type().as_resolved())
                        .ok_or_else(|| path_error(BindErrorKind::UnsupportedConstraint))?;
                    (inner, true)
                } else {
                    (output, false)
                }
            } else {
                (descriptor, false)
            };
            let slice = property
                .getter()
                .is_some_and(|getter| matches!(getter.output_kind(), GetterOutputKind::BorrowedSlice));
            let compatible_text =
                matches!(expected.kind(), TypeKind::Text(_)) && matches!(actual.kind(), TypeKind::Text(_));
            if expected.type_id() != actual.type_id() && !compatible_text && !(last && slice && !value_target) {
                return Err(path_error(BindErrorKind::UnsupportedConstraint));
            }
            path_optional |= optional;
            steps.push(PropertyStep { property });
            input = if matches!(actual.kind(), TypeKind::Text(_)) {
                InputType::Text
            } else {
                InputType::Typed(actual.type_id())
            };
            if !last {
                current = graph
                    .model(actual.type_id())
                    .ok_or_else(|| path_error(BindErrorKind::UnreadablePath))?;
            }
        }
        Ok(Self {
            context_depth: 0,
            dependency: None,
            deferred: Box::new([]),
            steps: steps.into_boxed_slice(),
            input,
            optional: path_optional,
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
                dependency: Some(*binding),
                deferred: segments.into_boxed_slice(),
                steps: Box::new([]),
                input: expected,
                optional: false,
            });
        };
        let mut path = Self::compile(owner, &PropertyPath::new(&segments), graph, TargetMode::Value)?;
        path.context_depth = depth;
        path.dependency = Some(*binding);
        Ok(path)
    }

    /// Returns the original named dependency declaration, if present.
    pub(crate) const fn dependency(&self) -> Option<crate::metadata::DependencyBindingMetadata> {
        self.dependency
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

#[cfg(test)]
mod tests {
    use qubit_validator::InputType;

    use super::CompiledPropertyPath;

    #[test]
    fn compiled_path_accessors_expose_structural_state() {
        let path = CompiledPropertyPath {
            context_depth: 2,
            dependency: None,
            deferred: vec!["parent", "value"].into_boxed_slice(),
            steps: Box::new([]),
            input: InputType::of::<u8>(),
            optional: true,
        };

        assert!(path.dependency().is_none());
        assert_eq!(path.deferred(), &["parent", "value"]);
        assert_eq!(path.context_depth(), 2);
        assert!(path.steps().is_empty());
        assert_eq!(path.input_type(), InputType::of::<u8>());
        assert!(path.is_optional());
    }
}
// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================
