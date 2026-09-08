// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Static sequences of field-name segments.
// qubit-style: allow type-file-name

/// A statically declared sequence of field-name segments.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::PropertyPath;
///
/// let path = PropertyPath::new(&["profile", "email"]);
/// assert_eq!(path.to_string(), "profile.email");
/// assert!(!path.is_empty());
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PropertyPath<'a> {
    /// The field-name segments in traversal order.
    segments: &'a [&'static str],
}

impl core::fmt::Display for PropertyPath<'_> {
    /// Formats this path with dot-separated field-name segments.
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut segments = self.segments.iter();
        if let Some(first) = segments.next() {
            write!(formatter, "{first}")?;
        }
        for segment in segments {
            write!(formatter, ".{segment}")?;
        }
        Ok(())
    }
}

impl<'a> PropertyPath<'a> {
    /// Creates a field path from statically allocated field-name segments.
    ///
    /// # Parameters
    ///
    /// - `segments`: The field-name segments in traversal order.
    ///
    /// # Returns
    ///
    /// The constructed field path.
    #[must_use]
    #[inline]
    pub const fn new(segments: &'a [&'static str]) -> Self {
        Self { segments }
    }

    /// Returns the path segments in traversal order.
    ///
    /// # Returns
    ///
    /// The statically allocated field-name segments.
    #[must_use]
    #[inline(always)]
    pub const fn segments(self) -> &'a [&'static str] {
        self.segments
    }

    /// Returns whether this path contains no field segments.
    ///
    /// # Returns
    ///
    /// `true` when the path is empty; otherwise `false`.
    #[must_use]
    #[inline(always)]
    pub const fn is_empty(self) -> bool {
        self.segments.is_empty()
    }
}
