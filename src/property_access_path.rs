// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Compiled dynamic property access paths over one explicit model registry.

#[path = "access_path/property_access_path_error.rs"]
mod property_access_path_error;
#[path = "access_path/property_access_write_failure.rs"]
mod property_access_write_failure;
#[path = "access_path/internal/property_step.rs"]
mod property_step;

use std::any::TypeId;

pub use property_access_path_error::PropertyAccessPathError;
pub use property_access_write_failure::PropertyAccessWriteFailure;
use qubit_reflect::ReflectionContext;
use qubit_reflect::descriptor::Reflect;
use qubit_reflect::descriptor::TypeKind;

use self::property_step::PropertyStep;
use crate::metadata::GetterOutputKind;
use crate::metadata::OptionalOps;
use crate::metadata::PropertyAccessError;
use crate::metadata::PropertyBorrowed;
use crate::metadata::PropertyMetadata;
use crate::metadata::PropertyMutable;
use crate::metadata::PropertyOwned;
use crate::metadata::PropertyValue;
use crate::metadata::TypeMetadata;
use crate::registry::ModelRegistry;

/// A dynamic property path compiled against a registry snapshot.
///
/// Compilation resolves every segment and its intermediate model using the
/// supplied registry. The path retains copied property metadata and never
/// consults a process-global registry after compilation.
///
/// # Examples
///
/// A path is compiled from metadata registered with a [`ModelRegistry`]. The
/// registry and metadata are supplied by the application's model setup.
///
/// ```
/// use qubit_model_metadata::metadata::TypeMetadata;
/// use qubit_model_metadata::registry::ModelRegistry;
/// use qubit_model_metadata::PropertyAccessPath;
///
/// fn compile_name_path<'ctx>(
///     registry: &ModelRegistry<'ctx>,
///     person: &'static TypeMetadata,
/// ) -> Result<PropertyAccessPath<'ctx>, qubit_model_metadata::PropertyAccessPathError> {
///     PropertyAccessPath::compile(registry, person, &["name"])
/// }
///
/// fn use_name_path<'ctx>(
///     registry: &ModelRegistry<'ctx>,
///     person: &'static TypeMetadata,
/// ) {
///     match compile_name_path(registry, person) {
///         Ok(path) => {
///             let _leaf = path.leaf_property();
///         }
///         Err(error) => eprintln!("could not compile the name path: {error}"),
///     }
/// }
///
/// // The application supplies `registry` and `person` from its model setup.
/// ```
#[derive(Clone)]
pub struct PropertyAccessPath<'ctx> {
    /// Snapshot used to bind all static property facts for this path.
    context: &'ctx ReflectionContext,
    /// Exact root Rust type expected by this path.
    root_type: TypeId,
    /// Resolved property steps in caller-supplied path order.
    steps: Box<[PropertyStep]>,
}

impl<'ctx> PropertyAccessPath<'ctx> {
    /// Compiles property names into a checked path for the supplied root model.
    ///
    /// # Parameters
    ///
    /// - `registry`: Explicit registry snapshot used to resolve properties.
    /// - `root`: Static metadata for the root object type.
    /// - `segments`: Ordered property names from root to leaf.
    ///
    /// # Returns
    ///
    /// A path retaining the resolved property metadata for every segment.
    ///
    /// # Errors
    ///
    /// Returns [`PropertyAccessPathError`] for an empty path or segment,
    /// unknown or unreadable property, unresolved intermediate type, or an
    /// intermediate representation that cannot be projected as a borrow.
    /// Compilation does not invoke property getters.
    /// This direct constructor does not cache its result; use
    /// [`ModelRegistry::compile_read_path_cached`] for snapshot-local reuse.
    #[inline]
    pub fn compile(
        registry: &ModelRegistry<'ctx>,
        root: &'static TypeMetadata,
        segments: &[&str],
    ) -> Result<Self, PropertyAccessPathError> {
        Self::compile_with_leaf_readability(registry, root, segments, true)
    }

    /// Compiles a path for writing, allowing a setter-only leaf property.
    ///
    /// Every intermediate property must still be readable so it can be
    /// traversed. The leaf is checked for writability by
    /// [`Self::check_writable`] and [`Self::write`].
    /// This direct constructor does not cache its result; use
    /// [`ModelRegistry::compile_write_path_cached`] for snapshot-local reuse.
    ///
    /// # Parameters
    ///
    /// - `registry`: Explicit registry snapshot used to resolve properties.
    /// - `root`: Static metadata for the root object type.
    /// - `segments`: Ordered property names from root to leaf.
    ///
    /// # Returns
    ///
    /// A path retaining resolved metadata for every segment.
    ///
    /// # Errors
    ///
    /// Returns [`PropertyAccessPathError`] for invalid names, unresolved
    /// properties, or unsupported intermediate representations.
    #[inline]
    pub fn compile_for_write(
        registry: &ModelRegistry<'ctx>,
        root: &'static TypeMetadata,
        segments: &[&str],
    ) -> Result<Self, PropertyAccessPathError> {
        Self::compile_with_leaf_readability(registry, root, segments, false)
    }

    /// Returns the metadata for the final property in this path.
    ///
    /// Callers can inspect the declared leaf type before converting an input
    /// value, while [`Self::write`] remains responsible for executing the
    /// setter or reflected field operation.
    ///
    /// # Returns
    ///
    /// A reference to the copied metadata for the last path segment.
    #[must_use]
    #[inline]
    pub fn leaf_property(&self) -> &PropertyMetadata {
        &self.steps[self.steps.len() - 1].property
    }

    /// Reads the final property while traversing each intermediate borrow.
    ///
    /// # Type Parameters
    ///
    /// - `T`: Reflected root type expected by this compiled path.
    ///
    /// # Parameters
    ///
    /// - `root`: Reflected root object supplied by the caller.
    ///
    /// # Returns
    ///
    /// The final property's original borrowed, optional, slice, or owned
    /// representation.
    ///
    /// # Errors
    ///
    /// Returns [`PropertyAccessPathError`] for a root type mismatch, missing
    /// optional intermediate value, or a property access failure. Intermediate
    /// owned and slice outputs are rejected rather than borrowed unsafely.
    #[inline]
    pub fn read<'a, T: Reflect>(&self, root: &'a T) -> Result<PropertyValue<'a>, PropertyAccessPathError> {
        self.read_dynamic(PropertyBorrowed::new(root))
    }

    /// Reads this path from a dynamically borrowed root value.
    ///
    /// Returns [`PropertyAccessPathError`] when the root type does not match,
    /// an optional intermediate value is absent, or a property access fails.
    /// Intermediate owned and slice outputs are rejected because they cannot
    /// be safely borrowed for the remainder of the path.
    ///
    /// # Parameters
    ///
    /// - `root`: Borrowed reflected root supplied by the caller's access flow.
    ///
    /// # Returns
    ///
    /// The leaf value in its original reflected representation.
    ///
    /// # Errors
    ///
    /// Returns the same access and traversal failures as [`Self::read`].
    #[inline]
    pub fn read_dynamic<'a>(&self, root: PropertyBorrowed<'a>) -> Result<PropertyValue<'a>, PropertyAccessPathError> {
        self.read_dynamic_impl(root, false)
    }

    /// Reads a path through private fields after external authorization.
    ///
    /// # Safety
    ///
    /// The caller must ensure access to every private field in this path is
    /// permitted by the application's privacy and encapsulation invariants.
    ///
    /// # Parameters
    ///
    /// - `root`: Borrowed reflected value supplied by the authorized caller.
    ///
    /// # Returns
    ///
    /// The leaf value in its original reflected representation.
    ///
    /// # Errors
    ///
    /// Returns access and traversal failures described by [`Self::read`].
    #[inline]
    pub unsafe fn read_dynamic_unsafe<'a>(
        &self,
        root: PropertyBorrowed<'a>,
    ) -> Result<PropertyValue<'a>, PropertyAccessPathError> {
        self.read_dynamic_impl(root, true)
    }

    /// Writes the leaf property through mutable intermediate field projections.
    ///
    /// # Type Parameters
    ///
    /// - `T`: Reflected root object type.
    /// - `V`: Reflected replacement value type.
    ///
    /// # Parameters
    ///
    /// - `root`: Mutable reflected root object supplied by the caller.
    /// - `replacement`: Owned value to pass to the leaf setter or field.
    ///
    /// # Returns
    ///
    /// `Ok(())` after the final property accepts the replacement.
    ///
    /// # Errors
    ///
    /// Returns [`PropertyAccessWriteFailure`] with the original replacement
    /// whenever failure occurs before the leaf setter begins. A leaf setter
    /// failure preserves the underlying [`crate::metadata::PropertySetFailure`]
    /// recovery rules.
    #[inline]
    pub fn write<T: Reflect, V: Reflect>(
        &self,
        root: &mut T,
        replacement: V,
    ) -> Result<(), PropertyAccessWriteFailure> {
        self.write_dynamic(PropertyMutable::new(root), PropertyOwned::new(replacement))
    }

    /// Writes an owned replacement through this path from a dynamic root.
    ///
    /// Returns [`PropertyAccessWriteFailure`] with the original replacement
    /// when failure occurs before the leaf setter begins. A leaf setter failure
    /// preserves the recovery behavior of
    /// [`crate::metadata::PropertySetFailure`].
    ///
    /// # Parameters
    ///
    /// - `root`: Mutable reflected value supplied by the caller's access flow.
    /// - `replacement`: Owned reflected value to assign to the leaf property.
    ///
    /// # Returns
    ///
    /// `Ok(())` after the leaf setter or field accepts the replacement.
    ///
    /// # Errors
    ///
    /// Returns the original replacement for failures before the leaf setter
    /// starts, and the setter's recovery result for leaf failures.
    #[inline]
    pub fn write_dynamic(
        &self,
        root: PropertyMutable<'_>,
        replacement: PropertyOwned,
    ) -> Result<(), PropertyAccessWriteFailure> {
        self.write_dynamic_impl(root, replacement, false)
    }

    /// Writes a path through private fields after external authorization.
    ///
    /// # Safety
    ///
    /// The caller must ensure mutation through every private field in this
    /// path is permitted by the application's privacy and encapsulation
    /// invariants.
    ///
    /// # Parameters
    ///
    /// - `root`: Mutable reflected value supplied by the authorized caller.
    /// - `replacement`: Owned reflected value to assign to the leaf.
    ///
    /// # Returns
    ///
    /// `Ok(())` after the leaf setter or field accepts the replacement.
    ///
    /// # Errors
    ///
    /// Returns [`PropertyAccessWriteFailure`] with the original replacement
    /// for failures before the leaf setter starts.
    #[inline]
    pub unsafe fn write_dynamic_unsafe(
        &self,
        root: PropertyMutable<'_>,
        replacement: PropertyOwned,
    ) -> Result<(), PropertyAccessWriteFailure> {
        self.write_dynamic_impl(root, replacement, true)
    }

    /// Checks whether each intermediate property is a mutable ordinary field.
    ///
    /// This validates path shape only; runtime reflection adapters are checked
    /// during [`Self::write`].
    ///
    /// # Returns
    ///
    /// `Ok(())` when mutable traversal is structurally possible.
    ///
    /// # Errors
    ///
    /// Returns [`PropertyAccessPathError::UnwritableIntermediate`] for an
    /// intermediate getter or non-field, or
    /// [`PropertyAccessPathError::NotWritable`] for a read-only leaf.
    pub fn check_writable(&self) -> Result<(), PropertyAccessPathError> {
        for (index, step) in self.steps.iter().enumerate() {
            let is_last = index + 1 == self.steps.len();
            if is_last {
                if !step.property.is_writable() {
                    return Err(PropertyAccessPathError::NotWritable {
                        index,
                        name: step.property.name().to_owned(),
                    });
                }
            } else if !step.property.is_field() || step.property.getter().is_some() {
                return Err(PropertyAccessPathError::UnwritableIntermediate {
                    index,
                    name: step.property.name().to_owned(),
                });
            }
        }
        Ok(())
    }

    /// Traverses resolved steps and optionally bypasses field visibility
    /// checks.
    ///
    /// The unsafe public entry point supplies `bypass_visibility` only after
    /// its caller accepts the privacy obligation for every private field.
    fn read_dynamic_impl<'a>(
        &self,
        root: PropertyBorrowed<'a>,
        bypass_visibility: bool,
    ) -> Result<PropertyValue<'a>, PropertyAccessPathError> {
        let mut current = root;
        for (index, step) in self.steps.iter().enumerate() {
            let actual = current.clone().type_id();
            if actual != step.owner.type_id() {
                return Err(PropertyAccessPathError::RootTypeMismatch {
                    expected: step.owner.type_id(),
                    actual,
                });
            }
            let value = (if bypass_visibility {
                unsafe { step.property.get_dynamic_unsafe_in(self.context, current) }
            } else {
                step.property.get_dynamic_in(self.context, current)
            })
            .map_err(|source| PropertyAccessPathError::AccessFailure {
                index,
                name: step.property.name().to_owned(),
                source: Box::new(source),
            })?;
            if index + 1 == self.steps.len() {
                return Ok(value);
            }
            current = match value {
                PropertyValue::Borrowed(value) if step.optional_ops.is_some() => step
                    .optional_ops
                    .expect("checked above")
                    .project_ref(value)
                    .map_err(|source| PropertyAccessPathError::AccessFailure {
                        index,
                        name: step.property.name().to_owned(),
                        source: Box::new(PropertyAccessError::ValueTypeMismatch(source)),
                    })?
                    .ok_or(PropertyAccessPathError::MissingIntermediate {
                        index,
                        name: step.property.name().to_owned(),
                    })?,
                PropertyValue::Borrowed(value) => value,
                PropertyValue::OptionalBorrowed(Some(value)) => value,
                PropertyValue::OptionalBorrowed(None) => {
                    return Err(PropertyAccessPathError::MissingIntermediate {
                        index,
                        name: step.property.name().to_owned(),
                    });
                }
                PropertyValue::BorrowedSlice(_) | PropertyValue::Owned(_) => {
                    return Err(PropertyAccessPathError::UnsupportedIntermediate {
                        index,
                        name: step.property.name().to_owned(),
                    });
                }
            };
        }
        unreachable!("compiled access paths always contain at least one step")
    }

    /// Executes the compiled path and preserves `replacement` until leaf
    /// access.
    ///
    /// `bypass_visibility` is enabled only by the unsafe public entry point,
    /// whose caller authorizes access through private fields.
    fn write_dynamic_impl(
        &self,
        root: PropertyMutable<'_>,
        replacement: PropertyOwned,
        bypass_visibility: bool,
    ) -> Result<(), PropertyAccessWriteFailure> {
        if let Err(error) = self.check_writable() {
            return Err(PropertyAccessWriteFailure::before_path(error, replacement));
        }
        if root.type_id() != self.root_type {
            return Err(PropertyAccessWriteFailure::before_path(
                PropertyAccessPathError::RootTypeMismatch {
                    expected: self.root_type,
                    actual: root.type_id(),
                },
                replacement,
            ));
        }
        let mut current = root;
        let mut replacement = Some(replacement);
        for (index, step) in self.steps.iter().enumerate() {
            if current.type_id() != step.owner.type_id() {
                return Err(PropertyAccessWriteFailure::before_path(
                    PropertyAccessPathError::RootTypeMismatch {
                        expected: step.owner.type_id(),
                        actual: current.type_id(),
                    },
                    replacement.take().expect("replacement remains unused before the leaf"),
                ));
            }
            if index + 1 == self.steps.len() {
                let value = replacement.take().expect("replacement remains unused before the leaf");
                return (if bypass_visibility {
                    // SAFETY: `write_dynamic_unsafe` requires its caller to authorize
                    // mutation through every private field in the compiled path.
                    unsafe { step.property.set_dynamic_unsafe_in(self.context, current, value) }
                } else {
                    step.property.set_dynamic_in(self.context, current, value)
                })
                .map_err(PropertyAccessWriteFailure::from_property_set);
            }
            let Some(field) = step.property.field().and_then(|field| field.reflect_in(self.context)) else {
                return Err(PropertyAccessWriteFailure::before_path(
                    PropertyAccessPathError::AccessFailure {
                        index,
                        name: step.property.name().to_owned(),
                        source: Box::new(PropertyAccessError::AdapterUnavailable),
                    },
                    replacement.take().expect("replacement remains unused before the leaf"),
                ));
            };
            let projected = if bypass_visibility {
                // SAFETY: `write_dynamic_unsafe` requires its caller to authorize
                // mutation through every private field in the compiled path.
                unsafe { field.get_dynamic_mut_unsafe(current) }
            } else {
                field.get_dynamic_mut(current)
            };
            current = projected.map_err(|source| {
                PropertyAccessWriteFailure::before_path(
                    PropertyAccessPathError::AccessFailure {
                        index,
                        name: step.property.name().to_owned(),
                        source: Box::new(PropertyAccessError::Field(source)),
                    },
                    replacement.take().expect("replacement remains unused before the leaf"),
                )
            })?;
            if let Some(ops) = step.optional_ops {
                current = ops
                    .project_mut(current)
                    .map_err(|source| {
                        PropertyAccessWriteFailure::before_path(
                            PropertyAccessPathError::AccessFailure {
                                index,
                                name: step.property.name().to_owned(),
                                source: Box::new(PropertyAccessError::ValueTypeMismatch(source)),
                            },
                            replacement.take().expect("replacement remains unused before the leaf"),
                        )
                    })?
                    .ok_or_else(|| {
                        PropertyAccessWriteFailure::before_path(
                            PropertyAccessPathError::MissingIntermediate {
                                index,
                                name: step.property.name().to_owned(),
                            },
                            replacement.take().expect("replacement remains unused before the leaf"),
                        )
                    })?;
            }
        }
        unreachable!("compiled access paths always contain at least one step")
    }

    /// Resolves each segment while optionally allowing an unreadable leaf.
    ///
    /// Intermediate properties remain readable because path traversal must
    /// inspect their values. The `require_readable_leaf` flag distinguishes
    /// read paths from write paths, whose leaf can be setter-only.
    fn compile_with_leaf_readability(
        registry: &ModelRegistry<'ctx>,
        root: &'static TypeMetadata,
        segments: &[&str],
        require_readable_leaf: bool,
    ) -> Result<Self, PropertyAccessPathError> {
        let context = registry
            .context()
            .ok_or(PropertyAccessPathError::ReflectionContextUnavailable)?;
        if segments.is_empty() {
            return Err(PropertyAccessPathError::EmptyPath);
        }
        let mut steps = Vec::with_capacity(segments.len());
        let mut owner = root;
        for (index, segment) in segments.iter().copied().enumerate() {
            if segment.is_empty() {
                return Err(PropertyAccessPathError::EmptySegment { index });
            }
            let is_last = index + 1 == segments.len();
            let property = resolve_property(registry, owner, segment, index, is_last, require_readable_leaf, context)?;
            let declaring_type = owner;
            let (optional_ops, next_owner) = if is_last {
                (None, None)
            } else {
                let (optional_ops, next_owner) = resolve_intermediate(registry, property, segment, index, context)?;
                (optional_ops, Some(next_owner))
            };
            steps.push(PropertyStep {
                owner: declaring_type,
                property,
                optional_ops,
            });
            owner = next_owner.unwrap_or(owner);
        }
        Ok(Self {
            context,
            root_type: root.type_id(),
            steps: steps.into_boxed_slice(),
        })
    }
}

/// Resolves one named property and validates its access adapter.
///
/// `is_last` and `require_readable_leaf` preserve the distinction between read
/// paths and setter-only write leaves. Resolution errors retain the original
/// segment index and property name.
fn resolve_property(
    registry: &ModelRegistry<'_>,
    owner: &'static TypeMetadata,
    segment: &str,
    index: usize,
    is_last: bool,
    require_readable_leaf: bool,
    context: &ReflectionContext,
) -> Result<PropertyMetadata, PropertyAccessPathError> {
    let properties = registry
        .properties_for(owner)
        .map_err(|source| PropertyAccessPathError::PropertyResolution {
            index,
            source: Box::new(source),
        })?;
    let property = properties
        .property(segment)
        .copied()
        .ok_or(PropertyAccessPathError::UnknownProperty {
            index,
            name: segment.to_owned(),
        })?;
    if !property.is_readable() && (require_readable_leaf || !is_last) {
        return Err(PropertyAccessPathError::UnreadableIntermediate {
            index,
            name: segment.to_owned(),
        });
    }
    if property
        .field()
        .is_some_and(|field| field.reflect_in(context).is_none())
    {
        return Err(PropertyAccessPathError::AdapterUnavailable {
            index,
            name: segment.to_owned(),
        });
    }
    Ok(property)
}

/// Resolves the model type reached through one intermediate property.
///
/// This stage validates the property's reflected output representation,
/// resolves optional element metadata when needed, and returns the generated
/// mutable/read projection operations without changing traversal state.
fn resolve_intermediate(
    registry: &ModelRegistry<'_>,
    property: PropertyMetadata,
    segment: &str,
    index: usize,
    context: &ReflectionContext,
) -> Result<(Option<OptionalOps>, &'static TypeMetadata), PropertyAccessPathError> {
    let descriptor =
        property
            .descriptor_in(context)
            .ok()
            .flatten()
            .ok_or(PropertyAccessPathError::UnsupportedIntermediate {
                index,
                name: segment.to_owned(),
            })?;
    let is_optional = matches!(descriptor.kind(), TypeKind::Optional(_));
    let optional_ops = if is_optional { property.optional_ops() } else { None };
    let next_descriptor = if is_optional {
        if let Some(ops) = optional_ops {
            context
                .find_type_by_id(ops.element_type_id())
                .map_err(|source| PropertyAccessPathError::ContextLookup { index, source })?
                .ok_or(PropertyAccessPathError::UnsupportedIntermediate {
                    index,
                    name: segment.to_owned(),
                })?
        } else if property
            .getter()
            .is_some_and(|getter| getter.output_kind() == GetterOutputKind::OptionalBorrowed)
        {
            descriptor
                .generic_arguments()
                .get_at(0)
                .and_then(|argument| argument.type_argument().ok())
                .and_then(|argument| argument.find_concrete_type())
                .ok_or(PropertyAccessPathError::UnsupportedIntermediate {
                    index,
                    name: segment.to_owned(),
                })?
        } else {
            return Err(PropertyAccessPathError::UnsupportedIntermediate {
                index,
                name: segment.to_owned(),
            });
        }
    } else {
        descriptor
    };
    if let Some(getter) = property.getter() {
        let output = getter.output_kind();
        if output == GetterOutputKind::Owned
            || output == GetterOutputKind::BorrowedSlice
            || (output == GetterOutputKind::OptionalBorrowed && !is_optional)
        {
            return Err(PropertyAccessPathError::UnsupportedIntermediate {
                index,
                name: segment.to_owned(),
            });
        }
    }
    let next_owner = registry
        .metadata_for(&next_descriptor)
        .map_err(|source| PropertyAccessPathError::MetadataLookup {
            index,
            source: Box::new(source),
        })?
        .ok_or(PropertyAccessPathError::UnsupportedIntermediate {
            index,
            name: segment.to_owned(),
        })?;
    Ok((optional_ops, next_owner))
}
