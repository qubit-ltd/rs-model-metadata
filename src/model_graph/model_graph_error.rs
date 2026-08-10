// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Individual direct-reference graph validation errors.

use core::fmt;

use crate::model_id::ModelId;
use crate::relation::FieldPath;

/// A validation error found in a model registry's direct-reference graph.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ModelGraphError {
    /// A direct reference targets no registered model.
    MissingTarget {
        /// The model declaring the reference.
        source: ModelId,
        /// The source field declaring the reference.
        field: &'static str,
        /// The unregistered target model ID.
        target: ModelId,
    },
    /// A direct reference targets a model without the requested field path.
    MissingTargetField {
        /// The model declaring the reference.
        source: ModelId,
        /// The source field declaring the reference.
        field: &'static str,
        /// The registered target model ID.
        target: ModelId,
        /// The absent field path in the target model.
        target_field: FieldPath,
    },
    /// A source field and target field have incompatible projected types.
    IncompatibleProjection {
        /// The model declaring the reference.
        source: ModelId,
        /// The source field declaring the reference.
        field: &'static str,
        /// The source field type after removing one outer `Option`.
        source_type: &'static str,
        /// The registered target model ID.
        target: ModelId,
        /// The resolved field path in the target model.
        target_field: FieldPath,
        /// The target field type after removing one outer `Option`.
        target_type: &'static str,
    },
    /// A direct reference's `same_as` path does not resolve in its source
    /// model.
    InvalidSameAs {
        /// The model declaring the reference.
        source: ModelId,
        /// The source field declaring the reference.
        field: &'static str,
        /// The invalid path in the source model.
        same_as: FieldPath,
    },
    /// Non-null, required direct references form an unsatisfiable cycle.
    RequiredReferenceCycle {
        /// The closed cycle, beginning and ending with its smallest model ID.
        cycle: Vec<ModelId>,
    },
}

impl fmt::Display for ModelGraphError {
    /// Formats this graph-validation error with its model IDs and field paths.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingTarget {
                source,
                field,
                target,
            } => write!(
                formatter,
                "reference {}.{field} targets missing model {}",
                source.as_str(),
                target.as_str()
            ),
            Self::MissingTargetField {
                source,
                field,
                target,
                target_field,
            } => write!(
                formatter,
                "reference {}.{field} targets missing field {} on {}",
                source.as_str(),
                DisplayFieldPath(*target_field),
                target.as_str()
            ),
            Self::IncompatibleProjection {
                source,
                field,
                source_type,
                target,
                target_field,
                target_type,
            } => write!(
                formatter,
                "reference {}.{field} projects {source_type}, but {} on {} has type {target_type}",
                source.as_str(),
                DisplayFieldPath(*target_field),
                target.as_str()
            ),
            Self::InvalidSameAs {
                source,
                field,
                same_as,
            } => write!(
                formatter,
                "reference {}.{field} has invalid same_as path {}",
                source.as_str(),
                DisplayFieldPath(*same_as)
            ),
            Self::RequiredReferenceCycle { cycle } => {
                write!(formatter, "required reference cycle")?;
                for id in cycle {
                    write!(formatter, " {}", id.as_str())?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for ModelGraphError {}

/// Formats a static field path with dot-separated segments.
struct DisplayFieldPath(FieldPath);

impl fmt::Display for DisplayFieldPath {
    /// Formats the wrapped path in dot-separated order.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut segments = self.0.segments().iter();
        if let Some(first) = segments.next() {
            write!(formatter, "{first}")?;
        }
        for segment in segments {
            write!(formatter, ".{segment}")?;
        }
        Ok(())
    }
}
