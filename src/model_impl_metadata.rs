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

use qubit_reflect::capability::CapabilityOrigin;

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

// Owns snapshot-specific merged property storage.
mod internal;

pub(crate) use internal::merged_model_impl::MergedModelImpl;

/// Stores the declarations emitted for one model-aware implementation block.
///
/// `fragments` preserves the block's source declarations, including facts that
/// may not appear in the local property set. `properties` retains either the
/// successful local assembly or its diagnostics, so snapshot resolution can
/// combine providers without rerunning this block's assembly. Both values are
/// immutable static metadata; dynamic snapshot merges use `MergedModelImpl`
/// and own their storage.
#[derive(Clone, Copy, Debug)]
pub struct ModelImplMetadata {
    /// Original field, getter, and setter declarations in deterministic source
    /// order, including declarations omitted from the local merged properties.
    fragments: &'static [PropertyFragment],
    /// The result of assembling this block alone; on failure, retains borrowed
    /// diagnostics for incompatible or conflicting property facts.
    properties: Result<&'static LocalPropertySet, &'static PropertyBuildErrors>,
}

impl ModelImplMetadata {
    /// Creates generated implementation metadata.
    ///
    /// Retains the supplied static source facts and assembly result without
    /// allocation, revalidation, or invoking any property adapters.
    ///
    /// # Parameters
    ///
    /// - `fragments`: original field and accessor declarations in source order.
    /// - `properties`: local properties or the retained assembly diagnostics.
    ///
    /// # Returns
    ///
    /// Immutable metadata borrowing the supplied generated storage.
    #[must_use]
    #[inline]
    pub(crate) const fn new(
        fragments: &'static [PropertyFragment],
        properties: Result<&'static LocalPropertySet, &'static PropertyBuildErrors>,
    ) -> Self {
        Self { fragments, properties }
    }

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
    /// # Parameters
    ///
    /// - `owner`: model whose field ownership and property types are checked.
    /// - `providers`: snapshot-selected implementation providers and their
    ///   origins.
    ///
    /// # Returns
    ///
    /// An owned merge retaining fragments and either effective properties or
    /// deterministic assembly diagnostics. No provider output is globally
    /// cached.
    ///
    /// # Panics
    ///
    /// Propagates provider panics.
    #[must_use]
    pub(crate) fn merge(owner: &TypeMetadata, providers: &[(ModelImplProvider, CapabilityOrigin)]) -> MergedModelImpl {
        let overlays: Vec<_> = providers
            .iter()
            .map(|(provider, origin)| (provider(), origin))
            .collect();
        let mut fragments = Vec::new();
        let mut properties: Vec<(PropertyMetadata, Option<CapabilityOrigin>, Option<CapabilityOrigin>)> = Vec::new();
        let mut errors = Vec::new();
        for (overlay, origin) in overlays {
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
                        if let Some((current, getter_origin, setter_origin)) = properties
                            .iter_mut()
                            .find(|(value, _, _)| value.name() == property.name())
                        {
                            if let Some((first, second)) = current.getter().zip(property.getter())
                                && !eq(first, second)
                            {
                                errors.push(PropertyBuildError::with_conflict(
                                    PropertyBuildErrorKind::ConflictingGetter,
                                    current.name(),
                                    first.rust_method_name(),
                                    second.rust_method_name(),
                                    getter_origin.clone().expect("selected getter retains its origin"),
                                    origin.clone(),
                                ));
                            }
                            if let Some((first, second)) = current.setter().zip(property.setter())
                                && !eq(first, second)
                            {
                                errors.push(PropertyBuildError::with_conflict(
                                    PropertyBuildErrorKind::ConflictingSetter,
                                    current.name(),
                                    first.rust_method_name(),
                                    second.rust_method_name(),
                                    setter_origin.clone().expect("selected setter retains its origin"),
                                    origin.clone(),
                                ));
                            }
                            if getter_origin.is_none() && property.getter().is_some() {
                                *getter_origin = Some(origin.clone());
                            }
                            if setter_origin.is_none() && property.setter().is_some() {
                                *setter_origin = Some(origin.clone());
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
                            properties.push((
                                *property,
                                property.getter().map(|_| origin.clone()),
                                property.setter().map(|_| origin.clone()),
                            ));
                        }
                    }
                }
            }
        }
        let properties: Vec<_> = properties.into_iter().map(|(property, _, _)| property).collect();
        if let Err(error) = owner.validate_properties(&properties) {
            errors.extend_from_slice(error.errors());
        }
        let properties = if errors.is_empty() {
            Ok(ResolvedProperties::Merged(Arc::from(properties)))
        } else {
            Err(Arc::new(PropertyBuildErrors::new(errors)))
        };
        MergedModelImpl::new(ResolvedPropertyFragments::Merged(Arc::from(fragments)), properties)
    }

    /// Returns this implementation's original declarations in source order.
    ///
    /// # Returns
    ///
    /// The generated static declarations, without copying or merging them.
    /// Snapshot-specific coalescing is performed by the owned merge view.
    #[must_use]
    #[inline]
    pub const fn fragments(&self) -> &'static [PropertyFragment] {
        self.fragments
    }

    /// Returns locally merged properties or deterministic compatibility errors.
    ///
    /// # Returns
    ///
    /// The retained static local properties when assembly succeeded.
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
