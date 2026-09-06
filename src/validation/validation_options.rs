//! Runtime controls for model validation.

// qubit-style: allow multiple-public-types

use std::num::NonZeroUsize;

/// Selects whether validation stops at the first failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationMode {
    /// Execute every selected occurrence until a budget is reached.
    CollectAll,
    /// Stop after the first reported violation.
    FailFast,
}

/// An owned field path used to select model validation occurrences.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldPath {
    segments: Vec<String>,
}

impl FieldPath {
    /// Creates a path from dot-separated field names.
    #[must_use]
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            segments: path.into().split('.').map(str::to_owned).collect(),
        }
    }

    /// Creates a path from individual field names.
    #[must_use]
    pub fn from_segments(segments: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            segments: segments.into_iter().map(Into::into).collect(),
        }
    }

    pub(crate) fn segments(&self) -> &[String] {
        &self.segments
    }
}

/// Limits and selection for one validation call.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationOptions {
    mode: ValidationMode,
    selection: ValidationSelection,
    max_depth: NonZeroUsize,
    max_nodes: NonZeroUsize,
    max_violations: NonZeroUsize,
    max_comparisons: NonZeroUsize,
}

/// The selected model fields to validate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValidationSelection {
    /// Validate every bound occurrence.
    All,
    /// Validate occurrences whose first field path is selected.
    Fields(Vec<FieldPath>),
}

impl Default for ValidationOptions {
    fn default() -> Self {
        Self {
            mode: ValidationMode::CollectAll,
            selection: ValidationSelection::All,
            max_depth: NonZeroUsize::new(64).expect("non-zero"),
            max_nodes: NonZeroUsize::new(100_000).expect("non-zero"),
            max_violations: NonZeroUsize::new(100).expect("non-zero"),
            max_comparisons: NonZeroUsize::new(1_000_000).expect("non-zero"),
        }
    }
}

impl ValidationOptions {
    /// Creates options with the documented safe defaults.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Selects the execution mode.
    #[must_use]
    pub fn with_mode(mut self, mode: ValidationMode) -> Self {
        self.mode = mode;
        self
    }
    /// Selects fields to validate.
    #[must_use]
    pub fn with_selection(mut self, selection: ValidationSelection) -> Self {
        self.selection = selection;
        self
    }
    /// Sets the maximum traversal depth.
    #[must_use]
    pub fn with_max_depth(mut self, value: NonZeroUsize) -> Self {
        self.max_depth = value;
        self
    }
    /// Sets the maximum number of visited nodes.
    #[must_use]
    pub fn with_max_nodes(mut self, value: NonZeroUsize) -> Self {
        self.max_nodes = value;
        self
    }
    /// Sets the maximum number of violations retained.
    #[must_use]
    pub fn with_max_violations(mut self, value: NonZeroUsize) -> Self {
        self.max_violations = value;
        self
    }
    /// Sets the maximum collection comparisons.
    #[must_use]
    pub fn with_max_comparisons(mut self, value: NonZeroUsize) -> Self {
        self.max_comparisons = value;
        self
    }
    pub(crate) const fn mode(&self) -> ValidationMode {
        self.mode
    }
    pub(crate) const fn selection(&self) -> &ValidationSelection {
        &self.selection
    }
    pub(crate) const fn max_depth(&self) -> usize {
        self.max_depth.get()
    }
    pub(crate) const fn max_nodes(&self) -> usize {
        self.max_nodes.get()
    }
    pub(crate) const fn max_violations(&self) -> usize {
        self.max_violations.get()
    }
    pub(crate) const fn max_comparisons(&self) -> usize {
        self.max_comparisons.get()
    }
}
