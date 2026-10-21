// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Structural, getter-free compilation of property paths.

use qubit_reflect::ReflectionContext;
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
    owner: Option<&'static TypeMetadata>,
    /// Exact element type expected after projecting a borrowed `Option<T>`.
    optional_element: Option<std::any::TypeId>,
}

/// Active-variant check performed after the containing enum has been read.
#[derive(Clone, Copy, Debug)]
pub(crate) struct VariantGuard {
    /// Number of owner-path properties to read before checking the enum.
    pub(crate) after_steps: usize,
    /// Reflected variant whose payload field is being validated.
    pub(crate) declaring_type: std::any::TypeId,
    /// Index of the variant in its declaring enum's reflected variant list.
    pub(crate) variant_index: usize,
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

    /// Returns the metadata owner, when this step was compiled from a model.
    #[must_use]
    pub(crate) const fn owner(self) -> Option<&'static TypeMetadata> {
        self.owner
    }

    /// Returns the optional element type to project for intermediate traversal.
    #[inline]
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
    /// Optional enum payload guard inserted before the payload field step.
    pub(super) variant_guard: Option<VariantGuard>,
    /// Variant label used in reports for enum payload fields.
    pub(super) variant_name: Option<&'static str>,
}

impl CompiledPropertyPath {
    /// Compiles a direct enum payload field together with its active-variant
    /// guard.
    pub(crate) fn compile_enum_payload(
        context: &ReflectionContext,
        root: &'static TypeMetadata,
        occurrence: &crate::validation::internal::validation_occurrence::ValidationOccurrence,
        graph: &ModelGraph<'_>,
        target: TargetMode,
    ) -> Result<Self, BindError> {
        if !matches!(target, TargetMode::Value) {
            return Err(path_error(BindErrorKind::UnsupportedConstraint));
        }
        let variant = occurrence
            .enum_variant
            .and_then(|variant| {
                occurrence
                    .owner
                    .descriptor_in(context)
                    .variants()
                    .iter()
                    .find(|actual| actual.index() == variant.index())
            })
            .ok_or_else(|| path_error(BindErrorKind::UnsupportedConstraint))?;
        let field_descriptor = occurrence
            .owner
            .field_descriptor_in(occurrence.field, context)
            .ok_or_else(|| path_error(BindErrorKind::UnsupportedConstraint))?;
        let field_type = field_descriptor
            .find_concrete_type()
            .ok_or_else(|| path_error(BindErrorKind::UnsupportedInput))?;
        let prefix = &occurrence.segments[..occurrence.segments.len() - 1];
        let mut path = if prefix.is_empty() {
            Self {
                context_depth: 0,
                dependency: None,
                deferred: Box::new([]),
                steps: Box::new([]),
                optional_steps: Box::new([]),
                input: InputType::of::<()>(),
                optional: false,
                variant_guard: None,
                variant_name: None,
            }
        } else {
            Self::compile(context, root, &PropertyPath::new(prefix), graph, TargetMode::Value)?
        };
        let owner_type = path
            .steps
            .last()
            .and_then(|step| step.property().descriptor_in(context).ok().flatten())
            .map_or(root.type_id(), TypeDescriptor::type_id);
        if variant.declaring_type().type_id() != owner_type {
            return Err(path_error(BindErrorKind::UnsupportedConstraint));
        }
        let property_name = occurrence.field.name().unwrap_or("0");
        let property = crate::property::PropertyMetadata::new(
            property_name,
            occurrence.field.type_expression_data(),
            Some(occurrence.field),
            None,
            None,
        );
        let after_steps = path.steps.len();
        let mut steps = path.steps.into_vec();
        steps.push(PropertyStep {
            property,
            owner: Some(occurrence.owner),
            optional_element: None,
        });
        path.steps = steps.into_boxed_slice();
        path.input = if matches!(field_type.kind(), TypeKind::Text(_)) {
            InputType::Text
        } else {
            InputType::Typed(field_type.type_id())
        };
        path.optional = false;
        path.variant_guard = Some(VariantGuard {
            after_steps,
            declaring_type: variant.declaring_type().type_id(),
            variant_index: variant.index(),
        });
        path.variant_name = Some(
            occurrence
                .enum_variant
                .expect("variant descriptor was resolved")
                .rust_name(),
        );
        Ok(path)
    }

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
        context: &ReflectionContext,
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
                let reflect = current
                    .field_descriptor_in(field, context)
                    .ok_or_else(|| path_error(BindErrorKind::UnreadablePath))?;
                if reflect.index() != 0 {
                    return Err(path_error(BindErrorKind::UnsupportedConstraint));
                }
                PropertyMetadata::new("0", field.type_expression_data(), Some(field), None, None)
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
                .descriptor_in_owner(current, context)
                .ok()
                .flatten()
                .ok_or_else(|| path_error(BindErrorKind::UnsupportedInput))?;
            let last = index + 1 == path.segments().len();
            let value_target = !last || matches!(target, TargetMode::Value);
            let (expected, declared_optional) = if value_target {
                value_descriptor(descriptor)
            } else {
                (descriptor, false)
            };
            let (actual, optional, project_borrowed_optional) = if let Some(getter) = property.getter() {
                let output = context
                    .find_type_by_id(getter.output_type_id())
                    .ok()
                    .flatten()
                    .or_else(|| {
                        (getter.output_kind() != GetterOutputKind::Owned)
                            .then(|| property.descriptor_in_owner(current, context).ok().flatten())
                            .flatten()
                    })
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
                    let inner = generic_type_argument(output, 0)
                        .ok_or_else(|| path_error(BindErrorKind::UnsupportedConstraint))?;
                    (inner, true, false)
                } else if last && !value_target && getter.output_kind() == GetterOutputKind::BorrowedSlice {
                    (expected, false, false)
                } else if !last && declared_optional {
                    if getter.output_kind() != GetterOutputKind::Borrowed {
                        return Err(path_error(BindErrorKind::UnsupportedConstraint));
                    }
                    let actual_optional = generic_type_argument(output, 0)
                        .ok_or_else(|| path_error(BindErrorKind::UnsupportedConstraint))?;
                    if property
                        .optional_ops()
                        .is_none_or(|ops| ops.element_type_id() != expected.type_id())
                        || actual_optional.type_id() != expected.type_id()
                    {
                        return Err(path_error(BindErrorKind::UnsupportedConstraint));
                    }
                    (actual_optional, true, true)
                } else {
                    (output, false, false)
                }
            } else if declared_optional {
                if property
                    .optional_ops()
                    .is_none_or(|ops| ops.element_type_id() != expected.type_id())
                {
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
                owner: Some(current),
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
            variant_guard: None,
            variant_name: None,
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
        context: &ReflectionContext,
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
                variant_guard: None,
                variant_name: None,
            });
        };
        let mut path = Self::compile(context, owner, &PropertyPath::new(&segments), graph, TargetMode::Value)?;
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

    /// Returns the enum payload guard, when this path addresses a variant
    /// field.
    #[must_use]
    #[inline]
    pub(crate) const fn variant_guard(&self) -> Option<VariantGuard> {
        self.variant_guard
    }

    /// Returns the variant label used in reports for enum payload paths.
    #[must_use]
    #[inline]
    pub(crate) const fn variant_name(&self) -> Option<&'static str> {
        self.variant_name
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
    pub(crate) fn optionality_covered_by(&self, target: &Self, context: &ReflectionContext) -> bool {
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
                            && a.property().descriptor_in(context).ok().flatten().map(|d| d.type_id())
                                == b.property().descriptor_in(context).ok().flatten().map(|d| d.type_id())
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
    pub(crate) fn unwrap_terminal_optional(mut self, element: TypeDescriptor<'_>) -> Result<Self, BindError> {
        let element_type_id = self
            .steps
            .last()
            .and_then(|step| step.property().optional_ops())
            .map(|ops| ops.element_type_id())
            .ok_or_else(|| path_error(BindErrorKind::UnsupportedConstraint))?;
        if element.type_id() != element_type_id {
            return Err(path_error(BindErrorKind::UnsupportedConstraint));
        }
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
fn value_descriptor<'ctx>(mut descriptor: TypeDescriptor<'ctx>) -> (TypeDescriptor<'ctx>, bool) {
    let mut optional = false;
    loop {
        let kind = descriptor.kind();
        let is_optional = matches!(kind, TypeKind::Optional(_));
        let is_smart_pointer = matches!(kind, TypeKind::SmartPointer(_));
        if !is_optional && !is_smart_pointer {
            return (descriptor, optional);
        }
        optional |= is_optional;
        let Some(next) = generic_type_argument(descriptor, 0) else {
            return (descriptor, optional);
        };
        descriptor = next;
    }
}

fn generic_type_argument<'ctx>(descriptor: TypeDescriptor<'ctx>, index: usize) -> Option<TypeDescriptor<'ctx>> {
    descriptor
        .generic_arguments()
        .iter()
        .nth(index)?
        .type_argument()
        .ok()?
        .find_concrete_type()
}

#[cfg(test)]
mod tests {
    use qubit_reflect::Reflect;
    use qubit_reflect::ReflectionContext;
    use qubit_reflect::TypeDescriptor;
    use qubit_validator::InputType;

    use super::CompiledPropertyPath;
    use super::PropertyStep;
    use super::generic_type_argument;
    use super::value_descriptor;
    use crate::__private::v8;
    use crate::field_metadata::FieldMetadata;
    use crate::property::OptionalOps;
    use crate::property::PropertyMetadata;

    #[Reflect]
    struct OptionalTerminalFixture {
        value: Option<String>,
    }

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
            variant_guard: None,
            variant_name: None,
        };

        assert!(path.dependency().is_none());
        assert_eq!(path.deferred(), &["parent", "value"]);
        assert_eq!(path.context_depth(), 2);
        assert!(path.steps().is_empty());
        assert_eq!(path.input_type(), InputType::of::<u8>());
        assert!(path.is_optional());
    }

    #[test]
    fn test_optional_terminal_projection_and_prefix_coverage() {
        let context = ReflectionContext::global();
        let descriptor = TypeDescriptor::of_in::<OptionalTerminalFixture>(context);
        let field = v8::leak(
            FieldMetadata::from_reflect(
                descriptor.type_id(),
                descriptor.fields().get_at(0).expect("optional terminal field"),
            )
            .with_optional_ops(OptionalOps::for_element::<String>()),
        );
        let property = PropertyMetadata::new("value", field.type_expression_data(), Some(field), None, None);
        let path = CompiledPropertyPath {
            context_depth: 0,
            dependency: None,
            deferred: Box::new([]),
            steps: vec![PropertyStep {
                property,
                owner: None,
                optional_element: None,
            }]
            .into_boxed_slice(),
            optional_steps: vec![0].into_boxed_slice(),
            input: InputType::of::<Option<String>>(),
            optional: false,
            variant_guard: None,
            variant_name: None,
        }
        .unwrap_terminal_optional(
            generic_type_argument(descriptor.fields().get_at(0).unwrap().find_concrete_type().unwrap(), 0).unwrap(),
        )
        .expect("Option<String> projects to its reflected text element");

        assert_eq!(path.input_type(), InputType::Text);
        assert!(path.is_optional());

        let covered_target = CompiledPropertyPath {
            optional: true,
            ..path.clone()
        };
        assert!(path.optionality_covered_by(&covered_target, context));

        let uncovered_target = CompiledPropertyPath {
            optional_steps: Box::new([]),
            ..covered_target
        };
        assert!(!path.optionality_covered_by(&uncovered_target, context));

        let (element, optional) =
            value_descriptor(descriptor.fields().get_at(0).unwrap().find_concrete_type().unwrap());
        assert_eq!(element.type_id(), TypeDescriptor::of_in::<String>(context).type_id());
        assert!(optional);
    }

    #[test]
    fn test_optional_terminal_projection_rejects_an_unrelated_element_type() {
        let context = ReflectionContext::global();
        let descriptor = TypeDescriptor::of_in::<OptionalTerminalFixture>(context);
        let field = v8::leak(
            FieldMetadata::from_reflect(
                descriptor.type_id(),
                descriptor.fields().get_at(0).expect("optional terminal field"),
            )
            .with_optional_ops(OptionalOps::for_element::<String>()),
        );
        let path = CompiledPropertyPath {
            context_depth: 0,
            dependency: None,
            deferred: Box::new([]),
            steps: vec![PropertyStep {
                property: PropertyMetadata::new("value", field.type_expression_data(), Some(field), None, None),
                owner: None,
                optional_element: None,
            }]
            .into_boxed_slice(),
            optional_steps: vec![0].into_boxed_slice(),
            input: InputType::of::<Option<String>>(),
            optional: false,
            variant_guard: None,
            variant_name: None,
        };

        let error = path
            .unwrap_terminal_optional(TypeDescriptor::of_in::<u64>(context))
            .expect_err("Option<String> cannot project to an unrelated element type");
        assert_eq!(error.kind(), qubit_validator::BindErrorKind::UnsupportedConstraint);
    }

    #[test]
    fn test_value_descriptor_unwraps_optional_and_smart_pointer_layers() {
        let context = ReflectionContext::global();
        let optional = TypeDescriptor::of_in::<Option<String>>(context);
        let (value, was_optional) = value_descriptor(optional);
        assert!(was_optional);
        assert_eq!(value.type_id(), TypeDescriptor::of_in::<String>(context).type_id());

        let nested = TypeDescriptor::of_in::<Option<Box<Option<String>>>>(context);
        let (value, was_optional) = value_descriptor(nested);
        assert!(was_optional);
        assert_eq!(value.type_id(), TypeDescriptor::of_in::<String>(context).type_id());
        assert!(generic_type_argument(value, 0).is_none());
    }
}
