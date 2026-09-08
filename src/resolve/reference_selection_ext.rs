//! Resolver-specific access to declared reference selections.

use crate::metadata::PropertyPath;
use crate::metadata::ReferenceSelection;

/// Provides resolver-specific access to a selected property path.
pub(super) trait ReferenceSelectionExt {
    /// Returns the selected property path for property references.
    fn property_path(&self) -> Option<&PropertyPath<'static>>;
}

impl ReferenceSelectionExt for ReferenceSelection {
    fn property_path(&self) -> Option<&PropertyPath<'static>> {
        match self {
            Self::Entity => None,
            Self::Property(path) => Some(path),
        }
    }
}
