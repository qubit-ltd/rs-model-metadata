//! Explicit validation selector execution capabilities.

use crate::metadata::SelectorPosition;

/// Capabilities supported by the erased validation executor.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ValidationCapabilities;

impl ValidationCapabilities {
    /// Returns whether the executor supports `position`.
    #[must_use]
    pub const fn supports(self, position: SelectorPosition) -> bool {
        matches!(position, SelectorPosition::Element)
    }
}
