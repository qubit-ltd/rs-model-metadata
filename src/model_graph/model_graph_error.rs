// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Individual direct-reference graph validation errors.

use core::fmt;

use crate::model_id::ModelId;
use crate::relation::FieldPath;
use crate::relation::ReferencePath;

/// A validation error found in a model registry's direct-reference graph.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::ModelGraphError;
/// use qubit_model_metadata::ModelId;
///
/// let error = ModelGraphError::MissingOwner {
///     source: ModelId::new("example.Order"),
///     owner: ModelId::new("example.Customer"),
/// };
/// assert!(error.to_string().contains("example.Order"));
/// ```
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
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
    /// A direct reference's object-graph path does not resolve.
    InvalidReferencePath {
        /// The model declaring the reference.
        source: ModelId,
        /// The source field declaring the reference.
        field: &'static str,
        /// The invalid object-graph path.
        path: ReferencePath,
    },
    /// A direct reference's object-graph path does not end at the same
    /// reference.
    IncompatibleReferencePath {
        /// The model declaring the reference.
        source: ModelId,
        /// The source field declaring the reference.
        field: &'static str,
        /// The declared object-graph path.
        path: ReferencePath,
    },
    /// A lookup relation targets no registered model.
    MissingLookupTarget {
        /// The model declaring the lookup relation.
        source: ModelId,
        /// The source field declaring the lookup relation.
        field: &'static str,
        /// The model ID from the resolved named target metadata.
        target: ModelId,
    },
    /// A lookup relation targets a model without the requested field path.
    MissingLookupTargetField {
        /// The model declaring the lookup relation.
        source: ModelId,
        /// The source field declaring the lookup relation.
        field: &'static str,
        /// The registered target model ID.
        target: ModelId,
        /// The absent target field path.
        target_field: FieldPath,
    },
    /// A lookup relation has incompatible source and target projections.
    IncompatibleLookupProjection {
        /// The model declaring the lookup relation.
        source: ModelId,
        /// The source field declaring the lookup relation.
        field: &'static str,
        /// The source projection type.
        source_type: &'static str,
        /// The registered target model ID.
        target: ModelId,
        /// The resolved target field path.
        target_field: FieldPath,
        /// The target projection type.
        target_type: &'static str,
    },
    /// A model declares an owner that is absent from the registry.
    MissingOwner {
        /// The model declaring ownership.
        source: ModelId,
        /// The model ID from the resolved owner metadata.
        owner: ModelId,
    },
    /// Non-null, required direct references form an unsatisfiable cycle.
    RequiredReferenceCycle {
        /// The closed cycle, beginning and ending with its smallest model ID.
        cycle: Vec<ModelId>,
    },
    /// Ownership declarations form a cyclic hierarchy.
    OwnershipCycle {
        /// The closed ownership cycle, beginning and ending with its smallest
        /// model ID.
        cycle: Vec<ModelId>,
    },
}

impl fmt::Display for ModelGraphError {
    /// Formats this graph-validation error with its model IDs and field paths.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingTarget { source, field, target } => write!(
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
                target_field,
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
                target_field,
                target.as_str()
            ),
            Self::InvalidReferencePath { source, field, path } => write!(
                formatter,
                "reference {}.{field} has invalid path {}",
                source.as_str(),
                path
            ),
            Self::IncompatibleReferencePath { source, field, path } => write!(
                formatter,
                "reference {}.{field} has incompatible path {path}",
                source.as_str()
            ),
            Self::MissingLookupTarget { source, field, target } => write!(
                formatter,
                "lookup relation {}.{field} targets missing model {}",
                source.as_str(),
                target.as_str()
            ),
            Self::MissingLookupTargetField {
                source,
                field,
                target,
                target_field,
            } => write!(
                formatter,
                "lookup relation {}.{field} targets missing field {target_field} on {}",
                source.as_str(),
                target.as_str()
            ),
            Self::IncompatibleLookupProjection {
                source,
                field,
                source_type,
                target,
                target_field,
                target_type,
            } => write!(
                formatter,
                "lookup relation {}.{field} projects {source_type}, but {target_field} on {} has type {target_type}",
                source.as_str(),
                target.as_str()
            ),
            Self::MissingOwner { source, owner } => write!(
                formatter,
                "model {} declares missing owner {}",
                source.as_str(),
                owner.as_str()
            ),
            Self::RequiredReferenceCycle { cycle } => {
                write!(formatter, "required reference cycle")?;
                for id in cycle {
                    write!(formatter, " {}", id.as_str())?;
                }
                Ok(())
            }
            Self::OwnershipCycle { cycle } => {
                write!(formatter, "ownership cycle")?;
                for id in cycle {
                    write!(formatter, " {}", id.as_str())?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for ModelGraphError {}
