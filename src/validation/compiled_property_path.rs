// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Structural, getter-free compilation of property paths.

use qubit_reflect::TypeDescriptor;
use qubit_reflect::descriptor::TypeKind;
use qubit_validator::BindError;
use qubit_validator::BindErrorKind;
use qubit_validator::InputType;

use super::build_error::path_error;
use crate::metadata::DependencyBindingMetadata;
use crate::metadata::GetterOutputKind;
use crate::metadata::NavigationStep;
use crate::metadata::PropertyMetadata;
use crate::metadata::PropertyPath;
use crate::metadata::TargetMode;
use crate::metadata::TypeMetadata;
use crate::resolve::ModelGraph;
use crate::validation::internal::declaration_walker::is_transparent_unnamed_field;

/// One validated property access retained for a later executor.
///
/// The metadata is copied from the resolved property set or constructed from
/// a transparent value field. Inspecting a step never invokes a getter.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PropertyStep {
    property: PropertyMetadata,
    /// Exact element type expected after projecting a borrowed `Option<T>`.
    optional_element: Option<std::any::TypeId>,
}

impl PropertyStep {
    /// Returns the metadata for this path segment.
    ///
    /// The returned metadata is a small copy of the validated descriptor and
    /// can be inspected without executing user code.
    ///
    /// # Returns
    ///
    /// The copy of the property metadata retained for this path segment.
    #[must_use]
    #[inline]
    pub(crate) const fn property(self) -> PropertyMetadata {
        self.property
    }

    /// Returns the optional element type to project for intermediate traversal.
    pub(crate) const fn optional_element(self) -> Option<std::any::TypeId> {
        self.optional_element
    }
}

/// A structurally validated property path ready for validation execution.
///
/// Compilation resolves property ownership, access shape, and final input
/// type without invoking user getters. A path may retain a prefix for runtime
/// owner resolution when its complete owner chain is not available at compile
/// time. At execution, the stored steps guide reads, while optional-step
/// indexes preserve the absence checks required along the path.
#[derive(Clone, Debug)]
pub(crate) struct CompiledPropertyPath {
    /// Number of external containing objects to traverse before this path.
    pub(super) context_depth: usize,
    /// Original dependency binding when this path was compiled for a rule slot.
    pub(super) dependency: Option<DependencyBindingMetadata>,
    /// Uncompiled property suffix deferred until the runtime owner is known.
    pub(super) deferred: Box<[&'static str]>,
    /// Resolved property metadata in the order the executor must traverse it.
    pub(super) steps: Box<[PropertyStep]>,
    /// Indexes in `steps` whose reads may produce no value.
    pub(super) optional_steps: Box<[usize]>,
    /// Validator input representation of the final path value.
    pub(super) input: InputType,
    /// Whether any read on the complete path may be absent.
    pub(super) optional: bool,
}

impl CompiledPropertyPath {
    /// Checks and compiles a declaration path without invoking user adapters.
    ///
    /// Resolves properties from `root`, including a transparent value's field
    /// at index zero, validates their output against `target`, and records
    /// optional reads and the validator input type. No instance is read and
    /// no user getter is called.
    ///
    /// # Parameters
    ///
    /// - `root`: metadata for the model that owns the first path segment.
    /// - `path`: non-empty property names in traversal order.
    /// - `graph`: resolved model/property graph used to find each segment.
    /// - `target`: whether the final descriptor is matched as a value or as a
    ///   target projection.
    ///
    /// # Returns
    ///
    /// A compiled path containing the validated steps, input type, and
    /// optionality information.
    ///
    /// # Errors
    ///
    /// Returns `UnreadablePath` for an empty path, a missing root/property, or
    /// an unreadable property. Returns `UnsupportedInput` for unresolved
    /// descriptors, and `UnsupportedConstraint` for unsupported getter shapes
    /// or incompatible target types.
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
        let mut optional_steps = Vec::new();
        let mut path_optional = false;
        let mut input = InputType::of::<()>();
        for (index, segment) in path.segments().iter().enumerate() {
            let transparent_field = current
                .as_value()
                .and_then(|value| value.transparent_field())
                .filter(|field| *segment == "0" && is_transparent_unnamed_field(current, field));
            let property = if let Some(field) = transparent_field {
                let reflect = field.reflect().ok_or_else(|| path_error(BindErrorKind::UnreadablePath))?;
                let declared = field.descriptor().ok_or_else(|| path_error(BindErrorKind::UnsupportedInput))?;
                let reflected = reflect
                    .field_type()
                    .as_resolved()
                    .ok_or_else(|| path_error(BindErrorKind::UnsupportedInput))?;
                if reflect.index() != 0 || !std::ptr::eq(reflected, declared) {
                    return Err(path_error(BindErrorKind::UnsupportedConstraint));
                }
                PropertyMetadata::new("0", field.type_ref(), Some(field), None, None)
            } else {
                graph
                    .properties(current)
                    .and_then(|properties| properties.property(segment))
                    .copied()
                    .ok_or_else(|| path_error(BindErrorKind::UnreadablePath))?
            };
            if !property.is_readable() {
                return Err(path_error(BindErrorKind::UnreadablePath));
            }
            let descriptor = property
                .descriptor()
                .ok_or_else(|| path_error(BindErrorKind::UnsupportedInput))?;
            let last = index + 1 == path.segments().len();
            let value_target = !last || matches!(target, TargetMode::Value);
            let (expected, declared_optional) = if value_target {
                value_descriptor(descriptor)
            } else {
                (descriptor, false)
            };
            let (actual, optional, project_borrowed_optional) = if let Some(getter) = property.getter() {
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
                    (inner, true, false)
                } else if !last && declared_optional {
                    if getter.output_kind() != GetterOutputKind::Borrowed {
                        return Err(path_error(BindErrorKind::UnsupportedConstraint));
                    }
                    let declared_optional = descriptor
                        .as_optional()
                        .filter(|optional| optional.has_ref_projection())
                        .ok_or_else(|| path_error(BindErrorKind::UnsupportedConstraint))?;
                    let actual_optional = output
                        .as_optional()
                        .and_then(|value| value.element_type().as_resolved())
                        .ok_or_else(|| path_error(BindErrorKind::UnsupportedConstraint))?;
                    if declared_optional
                        .element_type()
                        .as_resolved()
                        .is_none_or(|value| value.type_id() != expected.type_id())
                        || actual_optional.type_id() != expected.type_id()
                    {
                        return Err(path_error(BindErrorKind::UnsupportedConstraint));
                    }
                    (actual_optional, true, true)
                } else {
                    (output, false, false)
                }
            } else if !last && declared_optional {
                let optional = descriptor
                    .as_optional()
                    .ok_or_else(|| path_error(BindErrorKind::UnsupportedConstraint))?;
                if !optional.has_ref_projection() {
                    return Err(path_error(BindErrorKind::UnsupportedConstraint));
                }
                (expected, true, true)
            } else {
                (descriptor, false, false)
            };
            let slice = property
                .getter()
                .is_some_and(|getter| matches!(getter.output_kind(), GetterOutputKind::BorrowedSlice));
            let compatible_text =
                matches!(expected.kind(), TypeKind::Text(_)) && matches!(actual.kind(), TypeKind::Text(_));
            if expected.type_id() != actual.type_id() && !compatible_text && !(last && slice && !value_target) {
                return Err(path_error(BindErrorKind::UnsupportedConstraint));
            }
            path_optional |= optional || declared_optional && !last;
            if optional {
                optional_steps.push(index);
            }
            steps.push(PropertyStep {
                property,
                optional_element: project_borrowed_optional.then_some(expected.type_id()),
            });
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
            optional_steps: optional_steps.into_boxed_slice(),
            input,
            optional: path_optional,
        })
    }

    /// Compiles navigation relative to the owning object, then property
    /// selection.
    ///
    /// Resolves parent/property navigation against `root` and `ancestors`.
    /// When the owner is outside the available ancestor chain, it preserves
    /// the remaining names as `deferred` for runtime resolution.
    ///
    /// # Parameters
    ///
    /// - `root`: metadata for the current owner model.
    /// - `prefix`: property names already traversed within that owner.
    /// - `binding`: dependency navigation and selected property declaration.
    /// - `graph`: resolved model/property graph for paths whose owner is known.
    /// - `ancestors`: containing models available for parent navigation.
    /// - `expected`: validator input type expected by the dependent rule.
    ///
    /// # Returns
    ///
    /// A checked path when the owner is available, or a deferred path carrying
    /// the context depth and unresolved suffix otherwise.
    ///
    /// # Errors
    ///
    /// Returns the path compilation error when a known owner path is invalid
    /// or its property types are unsupported.
    pub(crate) fn compile_dependency(
        root: &'static TypeMetadata,
        prefix: &[&'static str],
        binding: &DependencyBindingMetadata,
        graph: &ModelGraph<'_>,
        ancestors: &[&'static TypeMetadata],
        expected: InputType,
    ) -> Result<Self, BindError> {
        let mut segments = prefix.to_vec();
        let mut depth = 0usize;
        for step in binding.object_path().steps() {
            match step {
                NavigationStep::Property(name) => segments.push(*name),
                NavigationStep::Parent => {
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
                optional_steps: Box::new([]),
                input: expected,
                optional: false,
            });
        };
        let mut path = Self::compile(owner, &PropertyPath::new(&segments), graph, TargetMode::Value)?;
        path.context_depth = depth;
        path.dependency = Some(*binding);
        Ok(path)
    }

    /// Returns the original rule dependency binding, if this path came from a
    /// dependency declaration.
    ///
    /// # Returns
    ///
    /// The copied dependency binding, or `None` for a path compiled directly
    /// from a property declaration.
    #[must_use]
    #[inline]
    pub(crate) const fn dependency(&self) -> Option<DependencyBindingMetadata> {
        self.dependency
    }

    /// Returns property names that could not be compiled before runtime owner
    /// metadata becomes available.
    ///
    /// # Returns
    ///
    /// The deferred property-name suffix, empty when every segment was
    /// resolved during compilation.
    #[must_use]
    #[inline]
    pub(crate) fn deferred(&self) -> &[&'static str] {
        &self.deferred
    }

    /// Returns how many parent objects must be resolved outside the compiled
    /// root before traversing this path.
    ///
    /// # Returns
    ///
    /// The number of containing objects required before the first stored or
    /// deferred segment.
    #[must_use]
    #[inline]
    pub(crate) const fn context_depth(&self) -> usize {
        self.context_depth
    }

    /// Returns resolved property descriptors in executor traversal order.
    ///
    /// # Returns
    ///
    /// A borrowed slice of resolved property steps, in traversal order.
    #[must_use]
    #[inline]
    pub(crate) fn steps(&self) -> &[PropertyStep] {
        &self.steps
    }

    /// Returns the input representation inferred from the final property's
    /// descriptor or getter output.
    ///
    /// # Returns
    ///
    /// The validator input type associated with the path's final value.
    #[must_use]
    #[inline]
    pub(crate) const fn input_type(&self) -> InputType {
        self.input
    }

    /// Returns whether reading any segment can yield no value, including a
    /// terminal optional value.
    ///
    /// # Returns
    ///
    /// `true` if any path segment may be absent; otherwise `false`.
    #[must_use]
    #[inline]
    pub(crate) const fn is_optional(&self) -> bool {
        self.optional
    }

    /// Returns whether every optional dependency read is covered by the same
    /// optional property prefix on the validation target.
    ///
    /// Coverage requires both paths to be fully compiled and to agree on each
    /// property name and reflected type through every optional segment.
    ///
    /// # Parameters
    ///
    /// - `target`: path whose optional prefix must cover this dependency.
    ///
    /// # Returns
    ///
    /// `true` when every optional dependency read has a matching optional
    /// prefix on `target`; otherwise `false`.
    #[must_use]
    pub(crate) fn optionality_covered_by(&self, target: &Self) -> bool {
        if self.context_depth != 0
            || target.context_depth != 0
            || !self.deferred.is_empty()
            || !target.deferred.is_empty()
        {
            return false;
        }
        self.optional_steps.iter().all(|&index| {
            self.steps
                .get(..=index)
                .zip(target.steps.get(..=index))
                .is_some_and(|(left, right)| {
                    left.iter().zip(right).all(|(a, b)| {
                        a.property().name() == b.property().name()
                            && a.property().descriptor().map(|d| d.type_id())
                                == b.property().descriptor().map(|d| d.type_id())
                    }) && target.optional_steps.contains(&index)
                })
        })
    }

    /// Projects a checked, field-backed `Option<T>` onto its reflected `T`.
    /// This is used only by scalar constraints that can borrow the contained
    /// value at execution. An absent or unresolved element is unsupported.
    ///
    /// # Returns
    ///
    /// The updated path with the inner type as its validator input and the
    /// terminal step recorded as optional.
    ///
    /// # Errors
    ///
    /// Returns `UnsupportedConstraint` if the final field is not a resolved
    /// `Option<T>` descriptor.
    pub(crate) fn unwrap_terminal_optional(mut self) -> Result<Self, BindError> {
        let element = self
            .steps
            .last()
            .and_then(|step| step.property().descriptor())
            .and_then(|descriptor| descriptor.as_optional())
            .and_then(|optional| optional.element_type().as_resolved())
            .ok_or_else(|| path_error(BindErrorKind::UnsupportedConstraint))?;
        self.input = if matches!(element.kind(), TypeKind::Text(_)) {
            InputType::Text
        } else {
            InputType::Typed(element.type_id())
        };
        self.optional = true;
        if !self.optional_steps.contains(&(self.steps.len() - 1)) {
            self.optional_steps = self
                .optional_steps
                .iter()
                .copied()
                .chain(std::iter::once(self.steps.len() - 1))
                .collect::<Vec<_>>()
                .into_boxed_slice();
        }
        Ok(self)
    }
}

/// Removes transparent wrappers and records whether an optional was found.
///
/// # Parameters
///
/// - `descriptor`: reflected property type to inspect.
///
/// # Returns
///
/// The innermost resolved descriptor and whether an optional wrapper was
/// encountered. If a wrapper cannot be resolved, returns the current
/// descriptor and the optionality observed so far.
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
    fn test_compiled_path_accessors_expose_structural_state() {
        let path = CompiledPropertyPath {
            context_depth: 2,
            dependency: None,
            deferred: vec!["parent", "value"].into_boxed_slice(),
            steps: Box::new([]),
            optional_steps: Box::new([]),
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
