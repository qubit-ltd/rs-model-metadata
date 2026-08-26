// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// qubit-style: allow multiple-public-types
//! Metadata for direct field references.

use super::field_path::FieldPath;
use crate::model_id::ModelId;

/// The target selected by a direct reference.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReferenceTarget {
    /// The source field references the whole target model value.
    WholeModel,
    /// The source field references a property or projection on the target
    /// model.
    Property(FieldPath),
}

/// One segment in a reference path through the containing object graph.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReferencePathSegment {
    /// Move to the parent object in the containing object graph.
    Parent,
    /// Move to a named field on the current object.
    Field(&'static str),
}

/// A path through the containing object graph to an equivalent reference.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReferencePath {
    segments: &'static [ReferencePathSegment],
}

impl core::fmt::Display for ReferencePath {
    /// Formats this reference path with dot-separated navigation segments.
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut segments = self.segments.iter();
        if let Some(first) = segments.next() {
            write_reference_path_segment(formatter, first)?;
        }
        for segment in segments {
            formatter.write_str(".")?;
            write_reference_path_segment(formatter, segment)?;
        }
        Ok(())
    }
}

impl ReferencePath {
    /// Creates a reference path from statically allocated path segments.
    ///
    /// # Panics
    ///
    /// Panics when the path is empty or contains an empty field segment.
    #[must_use]
    #[inline]
    pub const fn new(segments: &'static [ReferencePathSegment]) -> Self {
        validate_reference_path(segments);
        Self { segments }
    }

    /// Returns the path segments in traversal order.
    #[must_use]
    #[inline(always)]
    pub const fn segments(self) -> &'static [ReferencePathSegment] {
        self.segments
    }
}

/// Writes one reference path segment.
fn write_reference_path_segment(
    formatter: &mut core::fmt::Formatter<'_>,
    segment: &ReferencePathSegment,
) -> core::fmt::Result {
    match segment {
        ReferencePathSegment::Parent => formatter.write_str(".."),
        ReferencePathSegment::Field(name) => formatter.write_str(name),
    }
}

/// A direct reference from a field to a target model or model property.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::FieldPath;
/// use qubit_model_metadata::ModelId;
/// use qubit_model_metadata::ReferenceMetadata;
/// use qubit_model_metadata::ReferenceTarget;
///
/// let reference = ReferenceMetadata::new(
///     ModelId::new("example.Account"),
///     ReferenceTarget::Property(FieldPath::new(&["id"])),
///     true,
///     None,
/// );
/// assert_eq!(reference.entity().as_str(), "example.Account");
/// assert!(reference.existing());
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReferenceMetadata {
    /// The stable ID of the referenced model.
    entity: ModelId,
    /// The referenced whole model or property.
    target: ReferenceTarget,
    /// Whether the referenced record must exist.
    existing: bool,
    /// An optional object-graph path to an equivalent reference.
    path: Option<ReferencePath>,
}

impl ReferenceMetadata {
    /// Creates direct-reference metadata for a target model and selection.
    ///
    /// # Parameters
    ///
    /// - `entity`: The stable ID of the referenced model.
    /// - `target`: The referenced whole model or property.
    /// - `existing`: Whether the referenced record must exist.
    /// - `path`: An optional object-graph path to an equivalent reference.
    ///
    /// # Returns
    ///
    /// The constructed direct-reference metadata.
    ///
    /// # Panics
    ///
    /// Panics when a property path is empty or contains an empty segment.
    #[must_use]
    #[inline]
    pub const fn new(entity: ModelId, target: ReferenceTarget, existing: bool, path: Option<ReferencePath>) -> Self {
        validate_target(target);
        Self {
            entity,
            target,
            existing,
            path,
        }
    }

    /// Returns the stable ID of the referenced model.
    #[inline(always)]
    pub const fn entity(self) -> ModelId {
        self.entity
    }

    /// Returns the referenced whole model or property.
    #[must_use]
    #[inline(always)]
    pub const fn target(self) -> ReferenceTarget {
        self.target
    }

    /// Returns whether the target record must exist.
    #[must_use]
    #[inline(always)]
    pub const fn existing(self) -> bool {
        self.existing
    }

    /// Returns the object-graph path to an equivalent reference, if configured.
    #[must_use]
    #[inline(always)]
    pub const fn path(self) -> Option<ReferencePath> {
        self.path
    }
}

/// Validates a reference target selector.
const fn validate_target(target: ReferenceTarget) {
    match target {
        ReferenceTarget::WholeModel => {}
        ReferenceTarget::Property(path) => validate_property_path(path),
    }
}

/// Validates a reference property path.
const fn validate_property_path(path: FieldPath) {
    if path.is_empty() {
        panic!("reference property path cannot be empty");
    }
    let segments = path.segments();
    let mut index = 0;
    while index < segments.len() {
        if segments[index].is_empty() {
            panic!("reference property path cannot contain empty segments");
        }
        index += 1;
    }
}

/// Validates an object-graph reference path.
const fn validate_reference_path(segments: &'static [ReferencePathSegment]) {
    if segments.is_empty() {
        panic!("reference path cannot be empty");
    }
    let mut index = 0;
    while index < segments.len() {
        match segments[index] {
            ReferencePathSegment::Parent => {}
            ReferencePathSegment::Field(name) => {
                if name.is_empty() {
                    panic!("reference path cannot contain empty field segments");
                }
            }
        }
        index += 1;
    }
}
