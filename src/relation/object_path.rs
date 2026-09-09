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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObjectPath {
    steps: &'static [NavigationStep],
}

impl ObjectPath {
    /// Checks navigation steps, returning the first invalid property name.
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

    /// Returns the current-object path without allocating.
    #[must_use]
    pub const fn current() -> Self {
        Self { steps: &[] }
    }

    /// Returns every property and parent step in declaration order.
    #[must_use]
    pub const fn steps(&self) -> &'static [NavigationStep] {
        self.steps
    }

    /// Returns whether execution requires a containing-object context.
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
