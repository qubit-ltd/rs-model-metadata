// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Generated metadata for one model-aware inherent implementation block.

use std::ptr::eq;
use std::sync::Arc;

use crate::metadata::LocalPropertySet;
use crate::metadata::PropertyBuildError;
use crate::metadata::PropertyBuildErrorKind;
use crate::metadata::PropertyBuildErrors;
use crate::metadata::PropertyFragment;
use crate::metadata::PropertyFragmentSource;
use crate::metadata::PropertyMetadata;
use crate::metadata::ResolvedProperties;
use crate::metadata::ResolvedPropertyFragments;
use crate::metadata::TypeMetadata;
use crate::reflect_facade::ModelImplProvider;

/// Stores raw implementation fragments and their fallible local merge.
///
/// Fragments and the local result emitted by derive are immutable static
/// metadata. Dynamic merges use `MergedModelImpl` and own their storage.
#[derive(Clone, Copy, Debug)]
pub struct ModelImplMetadata {
    /// Field/getter/setter declarations in deterministic source order.
    fragments: &'static [PropertyFragment],
    /// Cached local merge result.
    properties: Result<&'static LocalPropertySet, &'static PropertyBuildErrors>,
}

impl ModelImplMetadata {
    /// Merges the providers visible in one immutable reflection snapshot.
    ///
    /// Dynamic results are owned by the returned value and released when its
    /// final owner is dropped. Repeated lookups are cached by `ModelRegistry`.
    ///
    /// Coalesces repeated backing-field fragments and combines complementary
    /// getters and setters. Reusing the exact same accessor descriptor is
    /// allowed; distinct getter or setter descriptors for one property produce
    /// an assembly error. Failed merges retain raw fragments for diagnosis and
    /// remain isolated from other snapshot configurations.
    ///
    /// # Panics
    ///
    /// Propagates provider panics.
    pub(crate) fn merge(owner: &TypeMetadata, providers: &[ModelImplProvider]) -> MergedModelImpl {
        let overlays: Vec<_> = providers.iter().map(|provider| provider()).collect();
        let mut fragments = Vec::new();
        let mut properties: Vec<PropertyMetadata> = Vec::new();
        let mut errors = Vec::new();
        for overlay in overlays {
            for fragment in overlay.fragments() {
                if matches!(fragment.source(), PropertyFragmentSource::Field(_))
                    && fragments.iter().any(|existing: &PropertyFragment| {
                        existing.name() == fragment.name()
                            && matches!(existing.source(), PropertyFragmentSource::Field(_))
                    })
                {
                    continue;
                }
                fragments.push(*fragment);
            }
            match overlay.try_properties() {
                Err(error) => errors.extend_from_slice(error.errors()),
                Ok(local) => {
                    if let Err(error) = owner.validate_properties(local.properties()) {
                        errors.extend_from_slice(error.errors());
                    }
                    for property in local.properties() {
                        if let Some(current) = properties.iter_mut().find(|value| value.name() == property.name()) {
                            let duplicate_getter = current
                                .getter()
                                .zip(property.getter())
                                .is_some_and(|(left, right)| !eq(left, right));
                            let duplicate_setter = current
                                .setter()
                                .zip(property.setter())
                                .is_some_and(|(left, right)| !eq(left, right));
                            if duplicate_getter || duplicate_setter {
                                errors.push(PropertyBuildError::new(
                                    PropertyBuildErrorKind::InvalidName,
                                    current.name(),
                                ));
                            }
                            let selected = if current.field().is_some() || current.getter().is_some() {
                                *current
                            } else {
                                *property
                            };
                            *current = PropertyMetadata::new(
                                current.name(),
                                selected.type_ref(),
                                current.field().or(property.field()),
                                current.getter().or(property.getter()),
                                current.setter().or(property.setter()),
                            );
                        } else {
                            properties.push(*property);
                        }
                    }
                }
            }
        }
        if let Err(error) = owner.validate_properties(&properties) {
            errors.extend_from_slice(error.errors());
        }
        let properties = if errors.is_empty() {
            Ok(ResolvedProperties::Merged(Arc::from(properties)))
        } else {
            Err(Arc::new(PropertyBuildErrors::new(errors)))
        };
        MergedModelImpl {
            fragments: ResolvedPropertyFragments::Merged(Arc::from(fragments)),
            properties,
        }
    }

    /// Creates generated implementation metadata.
    ///
    /// Retains the supplied static source facts and assembly result without
    /// allocation, revalidation, or invoking any property adapters.
    #[must_use]
    #[inline]
    pub(crate) const fn new(
        fragments: &'static [PropertyFragment],
        properties: Result<&'static LocalPropertySet, &'static PropertyBuildErrors>,
    ) -> Self {
        Self { fragments, properties }
    }

    /// Returns ordered source facts retained for this implementation view.
    /// Combined views coalesce repeated field fragments while retaining
    /// accessor fragments from the selected overlays.
    #[must_use]
    #[inline]
    pub const fn fragments(&self) -> &'static [PropertyFragment] {
        self.fragments
    }

    /// Returns locally merged properties or deterministic compatibility errors.
    ///
    /// # Errors
    ///
    /// Returns the retained assembly diagnostics for incompatible types,
    /// conflicting accessors, or invalid field ownership. This accessor does
    /// not rerun assembly or invoke providers.
    #[must_use = "handle property assembly failures"]
    #[inline]
    pub const fn try_properties(&self) -> Result<&'static LocalPropertySet, &'static PropertyBuildErrors> {
        self.properties
    }
}

/// Owned property merge assembled from multiple snapshot providers.
#[derive(Clone, Debug)]
pub(crate) struct MergedModelImpl {
    fragments: ResolvedPropertyFragments,
    properties: Result<ResolvedProperties, Arc<PropertyBuildErrors>>,
}

impl MergedModelImpl {
    pub(crate) fn fragments(&self) -> ResolvedPropertyFragments {
        self.fragments.clone()
    }

    pub(crate) fn try_properties(&self) -> Result<ResolvedProperties, Arc<PropertyBuildErrors>> {
        self.properties.clone()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::MergedModelImpl;
    use super::ModelImplMetadata;
    use crate::metadata::LocalPropertySet;
    use crate::metadata::ResolvedProperties;
    use crate::metadata::ResolvedPropertyFragments;

    #[test]
    fn owned_merge_views_return_their_contents() {
        let generated = ModelImplMetadata::new(&[], Ok(Box::leak(Box::new(LocalPropertySet::new(&[])))));
        assert!(generated.fragments().is_empty());
        assert!(
            generated
                .try_properties()
                .expect("valid empty generated view")
                .properties()
                .is_empty()
        );

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
