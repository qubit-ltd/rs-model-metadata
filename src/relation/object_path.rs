// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Checked, immutable object-navigation metadata.

use core::fmt;

use super::navigation_step::NavigationStep;
use super::object_path_error::ObjectPathError;

/// A static relative path through an object graph.
///
/// An empty path selects the current object for validator dependencies. An
/// omitted reference path separately means that no binding reuse was requested.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::NavigationStep;
/// use qubit_model_metadata::metadata::ObjectPath;
///
/// let path = ObjectPath::new(&[NavigationStep::Parent, NavigationStep::Property("name")])
///     .expect("valid parent and property steps");
/// assert!(path.requires_parent());
/// assert_eq!(path.to_string(), "../name");
/// assert!(ObjectPath::current().steps().is_empty());
/// assert!(ObjectPath::new(&[NavigationStep::Property("invalid/name")]).is_err());
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObjectPath {
    /// Validated navigation steps borrowed from immutable declaration metadata.
    /// Their order determines execution; an empty slice selects the current
    /// object.
    steps: &'static [NavigationStep],
}

impl ObjectPath {
    /// Checks navigation steps, returning the first invalid property name.
    ///
    /// # Parameters
    ///
    /// - `steps`: immutable declaration steps, borrowed for the process
    ///   lifetime.
    ///
    /// # Returns
    ///
    /// A checked path borrowing `steps`, without allocating or traversing
    /// objects.
    ///
    /// # Errors
    ///
    /// Returns an error for empty names, separators, whitespace, or dot markers
    /// represented as properties rather than explicit parent steps.
    pub fn new(steps: &'static [NavigationStep]) -> Result<Self, ObjectPathError> {
        for (index, step) in steps.iter().enumerate() {
            if let NavigationStep::Property(name) = step
                && (name.is_empty() || name.contains(['.', '/']) || name.chars().any(char::is_whitespace))
            {
                return Err(ObjectPathError { index, name });
            }
        }
        Ok(Self { steps })
    }

    /// Creates the current-object path without allocating.
    ///
    /// # Returns
    ///
    /// An empty path that selects the object where execution starts.
    #[must_use]
    #[inline]
    pub const fn current() -> Self {
        Self { steps: &[] }
    }

    /// Returns every property and parent step in declaration order.
    ///
    /// # Returns
    ///
    /// The immutable declaration slice; an empty slice selects the current
    /// object.
    #[must_use]
    #[inline]
    pub const fn steps(&self) -> &'static [NavigationStep] {
        self.steps
    }

    /// Returns whether execution requires a containing-object context.
    ///
    /// # Returns
    ///
    /// Returns `true` when a parent step escapes above the path's starting
    /// object. A parent step balanced by an earlier property step does not
    /// require an external containing object. This scans metadata only.
    #[must_use]
    pub fn requires_parent(&self) -> bool {
        let mut depth = 0usize;
        for step in self.steps {
            match step {
                NavigationStep::Property(_) => depth += 1,
                NavigationStep::Parent if depth == 0 => return true,
                NavigationStep::Parent => depth -= 1,
            }
        }
        false
    }
}

impl fmt::Display for ObjectPath {
    /// Renders the relative path with slash separators.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, step) in self.steps.iter().enumerate() {
            if index != 0 {
                formatter.write_str("/")?;
            }
            match step {
                NavigationStep::Property(name) => formatter.write_str(name)?,
                NavigationStep::Parent => formatter.write_str("..")?,
            }
        }
        Ok(())
    }
}
