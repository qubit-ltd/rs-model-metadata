// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Owned storage for one snapshot-specific property assembly result.

use std::sync::Arc;

use crate::metadata::PropertyBuildErrors;
use crate::metadata::ResolvedProperties;
use crate::metadata::ResolvedPropertyFragments;

/// Retains property fragments and either assembled properties or diagnostics.
///
/// Cloned views share immutable storage; the storage is released when its final
/// owner is dropped, independently of other reflection snapshots.
#[derive(Clone, Debug)]
pub(crate) struct MergedModelImpl {
    /// Source facts in provider order, with repeated backing fields coalesced.
    /// These remain available even when property assembly fails.
    fragments: ResolvedPropertyFragments,
    /// The complete assembly result, retaining either shared property storage
    /// or deterministic diagnostics for all failed contributions.
    properties: Result<ResolvedProperties, Arc<PropertyBuildErrors>>,
}

impl MergedModelImpl {
    /// Takes ownership of the fragments and assembly result of one merge.
    ///
    /// Performs no allocation or validation and does not invoke providers.
    ///
    /// # Parameters
    ///
    /// - `fragments`: source facts retained even after a failed assembly.
    /// - `properties`: assembled properties or shared assembly diagnostics.
    ///
    /// # Returns
    ///
    /// An owned result whose views keep their immutable storage alive.
    #[must_use]
    #[inline]
    pub(in crate::model_impl_metadata) const fn new(
        fragments: ResolvedPropertyFragments,
        properties: Result<ResolvedProperties, Arc<PropertyBuildErrors>>,
    ) -> Self {
        Self { fragments, properties }
    }

    /// Returns a shared view of the original fragments without copying them.
    ///
    /// # Returns
    ///
    /// A cloned view that remains valid after this merge result is dropped.
    #[must_use]
    #[inline]
    pub(crate) fn fragments(&self) -> ResolvedPropertyFragments {
        self.fragments.clone()
    }

    /// Returns shared assembled properties without rerunning the merge.
    ///
    /// # Returns
    ///
    /// Successful views share immutable property storage and remain valid
    /// after this merge result is dropped.
    ///
    /// # Errors
    ///
    /// Returns the retained shared diagnostics for incompatible types,
    /// conflicting accessors, or invalid field ownership. Fragments remain
    /// available through [`Self::fragments`] when assembly fails.
    #[must_use = "handle property assembly failures"]
    #[inline]
    pub(crate) fn try_properties(&self) -> Result<ResolvedProperties, Arc<PropertyBuildErrors>> {
        self.properties.clone()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::MergedModelImpl;
    use crate::metadata::ResolvedProperties;
    use crate::metadata::ResolvedPropertyFragments;

    #[test]
    fn test_owned_merge_views_return_their_contents() {
        let merged = MergedModelImpl {
            fragments: ResolvedPropertyFragments::Merged(Arc::from([])),
            properties: Ok(ResolvedProperties::Merged(Arc::from([]))),
        };

        assert!(merged.fragments().fragments().is_empty());
        assert!(
            merged
                .try_properties()
                .expect("an empty overlay set has no assembly errors")
                .properties()
                .is_empty()
        );
    }
}
