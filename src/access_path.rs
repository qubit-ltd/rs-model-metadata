// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
// =============================================================================

//! Compiled dynamic property access paths over one explicit model registry.

mod error;

pub use error::PropertyAccessPathError;
pub use error::PropertyAccessWriteFailure;

use std::any::TypeId;

use qubit_reflect::ReflectedMut;
use qubit_reflect::ReflectedOwned;
use qubit_reflect::ReflectedRef;

use crate::metadata::PropertyAccessError;
use crate::metadata::PropertyMetadata;
use crate::metadata::PropertyValue;
use crate::metadata::TypeMetadata;
use crate::registry::ModelRegistry;

/// One property and its declaring model in a compiled path.
#[derive(Clone, Copy)]
struct PropertyStep {
    /// Declaring metadata used to validate the runtime receiver.
    owner: &'static TypeMetadata,
    /// Resolved property to read or write.
    property: PropertyMetadata,
    /// Whether the next reflected value is an `Option<T>` wrapper.
    optional: bool,
}

/// A dynamic property path compiled against a registry snapshot.
///
/// Compilation resolves every segment and its intermediate model using the
/// supplied registry. The path retains copied property metadata and never
/// consults a process-global registry after compilation.
#[derive(Clone)]
pub struct PropertyAccessPath {
    /// Exact root Rust type expected by this path.
    root_type: TypeId,
    /// Resolved property steps in caller-supplied path order.
    steps: Box<[PropertyStep]>,
}

impl PropertyAccessPath {
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
    pub fn compile(
        registry: &ModelRegistry<'_>,
        root: &'static TypeMetadata,
        segments: &[&str],
    ) -> Result<Self, PropertyAccessPathError> {
        if segments.is_empty() {
            return Err(PropertyAccessPathError::EmptyPath);
        }
        let mut steps = Vec::with_capacity(segments.len());
        let mut owner = root;
        for (index, segment) in segments.iter().copied().enumerate() {
            if segment.is_empty() {
                return Err(PropertyAccessPathError::EmptySegment { index });
            }
            let properties = registry
                .properties_for(owner)
                .map_err(|source| PropertyAccessPathError::PropertyResolution { index, source })?;
            let property = properties
                .property(segment)
                .copied()
                .ok_or(PropertyAccessPathError::UnknownProperty { index, name: segment.to_owned() })?;
            if !property.is_readable() {
                return Err(PropertyAccessPathError::UnreadableIntermediate {
                    index,
                    name: segment.to_owned(),
                });
            }
            let is_last = index + 1 == segments.len();
            let declaring_type = owner;
            let mut optional = false;
            let mut next_owner = None;
            if !is_last {
                let descriptor = property
                    .descriptor()
                    .ok_or(PropertyAccessPathError::UnsupportedIntermediate {
                        index,
                        name: segment.to_owned(),
                    })?;
                let next_descriptor = if let Some(optional_type) = descriptor.as_optional() {
                    if !optional_type.has_ref_projection() {
                        return Err(PropertyAccessPathError::UnsupportedIntermediate {
                            index,
                            name: segment.to_owned(),
                        });
                    }
                    optional = true;
                    optional_type
                        .element_type()
                        .as_resolved()
                        .ok_or(PropertyAccessPathError::UnsupportedIntermediate {
                            index,
                            name: segment.to_owned(),
                        })?
                } else {
                    descriptor
                };
                if let Some(getter) = property.getter() {
                    use crate::metadata::GetterOutputKind;
                    let output = getter.output_kind();
                    if output == GetterOutputKind::Owned
                        || output == GetterOutputKind::BorrowedSlice
                        || (output == GetterOutputKind::OptionalBorrowed && !optional)
                    {
                        return Err(PropertyAccessPathError::UnsupportedIntermediate {
                            index,
                            name: segment.to_owned(),
                        });
                    }
                }
                next_owner = Some(registry
                    .metadata_for(next_descriptor)
                    .map_err(|source| PropertyAccessPathError::MetadataLookup { index, source })?
                    .ok_or(PropertyAccessPathError::UnsupportedIntermediate {
                        index,
                        name: segment.to_owned(),
                    })?);
            }
            steps.push(PropertyStep {
                owner: declaring_type,
                property,
                optional,
            });
            owner = next_owner.unwrap_or(owner);
        }
        Ok(Self {
            root_type: root.type_id(),
            steps: steps.into_boxed_slice(),
        })
    }

    /// Reads the final property while traversing each intermediate borrow.
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
    pub fn read<'a>(&self, root: ReflectedRef<'a>) -> Result<PropertyValue<'a>, PropertyAccessPathError> {
        if root.value_type_id() != self.root_type {
            return Err(PropertyAccessPathError::RootTypeMismatch {
                expected: self.root_type,
                actual: root.value_type_id(),
            });
        }
        let mut current = root;
        for (index, step) in self.steps.iter().enumerate() {
            if current.value_type_id() != step.owner.type_id() {
                return Err(PropertyAccessPathError::RootTypeMismatch {
                    expected: step.owner.type_id(),
                    actual: current.value_type_id(),
                });
            }
            let value = step
                .property
                .get(current)
                .map_err(|source| PropertyAccessPathError::AccessFailure {
                    index,
                    name: step.property.name().to_owned(),
                    source,
                })?;
            if index + 1 == self.steps.len() {
                return Ok(value);
            }
            current = match value {
                PropertyValue::Borrowed(value) if step.optional => {
                    let optional = step
                        .property
                        .descriptor()
                        .and_then(|descriptor| descriptor.as_optional())
                        .ok_or(PropertyAccessPathError::UnsupportedIntermediate {
                            index,
                            name: step.property.name().to_owned(),
                        })?;
                    optional
                        .project_ref(value)
                        .map_err(|_| PropertyAccessPathError::UnsupportedIntermediate {
                            index,
                            name: step.property.name().to_owned(),
                        })?
                        .ok_or(PropertyAccessPathError::MissingIntermediate {
                            index,
                            name: step.property.name().to_owned(),
                        })?
                }
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

    /// Checks whether each intermediate property is a mutable ordinary field.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the path shape can be traversed mutably; the concrete
    /// reflection adapters are checked when [`Self::write`] runs.
    ///
    /// # Errors
    ///
    /// Returns [`PropertyAccessPathError::UnwritableIntermediate`] when an
    /// intermediate step is getter-backed, optional, or lacks a field, or
    /// [`PropertyAccessPathError::NotWritable`] when the leaf has no field or
    /// setter destination.
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
            } else if !step.property.is_field() || step.property.getter().is_some() || step.optional {
                return Err(PropertyAccessPathError::UnwritableIntermediate {
                    index,
                    name: step.property.name().to_owned(),
                });
            }
        }
        Ok(())
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
    pub fn leaf_property(&self) -> &PropertyMetadata {
        &self.steps[self.steps.len() - 1].property
    }

    /// Writes the leaf property through mutable intermediate field projections.
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
    /// failure preserves the underlying [`PropertySetFailure`] recovery rules.
    pub fn write(
        &self,
        root: ReflectedMut<'_>,
        replacement: ReflectedOwned,
    ) -> Result<(), PropertyAccessWriteFailure> {
        if let Err(error) = self.check_writable() {
            return Err(PropertyAccessWriteFailure::before_path(error, replacement));
        }
        if root.value_type_id() != self.root_type {
            return Err(PropertyAccessWriteFailure::before_path(
                PropertyAccessPathError::RootTypeMismatch {
                    expected: self.root_type,
                    actual: root.value_type_id(),
                },
                replacement,
            ));
        }
        let mut current = root;
        let mut replacement = Some(replacement);
        for (index, step) in self.steps.iter().enumerate() {
            if current.value_type_id() != step.owner.type_id() {
                return Err(PropertyAccessWriteFailure::before_path(
                    PropertyAccessPathError::RootTypeMismatch {
                        expected: step.owner.type_id(),
                        actual: current.value_type_id(),
                    },
                    replacement.take().expect("replacement remains unused before the leaf"),
                ));
            }
            if index + 1 == self.steps.len() {
                return step
                    .property
                    .set(
                        current,
                        replacement.take().expect("replacement remains unused before the leaf"),
                    )
                    .map_err(PropertyAccessWriteFailure::from_property_set);
            }
            let Some(field) = step
                .property
                .field()
                .and_then(|field| field.reflect())
            else {
                return Err(PropertyAccessWriteFailure::before_path(
                    PropertyAccessPathError::AccessFailure {
                        index,
                        name: step.property.name().to_owned(),
                        source: PropertyAccessError::AdapterUnavailable,
                    },
                    replacement.take().expect("replacement remains unused before the leaf"),
                ));
            };
            current = field.get_mut(current).map_err(|source| {
                PropertyAccessWriteFailure::before_path(
                    PropertyAccessPathError::AccessFailure {
                        index,
                        name: step.property.name().to_owned(),
                        source: PropertyAccessError::Field(source),
                    },
                    replacement.take().expect("replacement remains unused before the leaf"),
                )
            })?;
        }
        unreachable!("compiled access paths always contain at least one step")
    }
}
