// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Generated metadata for one model-aware inherent implementation block.

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::OnceLock;

use crate::metadata::LocalPropertySet;
use crate::metadata::PropertyBuildError;
use crate::metadata::PropertyBuildErrorKind;
use crate::metadata::PropertyBuildErrors;
use crate::metadata::PropertyFragment;
use crate::metadata::PropertyFragmentSource;
use crate::metadata::PropertyMetadata;
use crate::metadata::TypeMetadata;
use crate::reflect_facade::ModelImplProvider;

/// Stores raw implementation fragments and their fallible local merge.
#[derive(Clone, Copy, Debug)]
pub struct ModelImplMetadata {
    /// Field/getter/setter declarations in deterministic source order.
    fragments: &'static [PropertyFragment],
    /// Cached local merge result.
    properties: Result<&'static LocalPropertySet, &'static PropertyBuildErrors>,
}

impl ModelImplMetadata {
    /// Merges exactly the providers visible in one immutable reflection
    /// snapshot.
    pub(crate) fn merge(owner: &TypeMetadata, providers: &[ModelImplProvider]) -> &'static Self {
        type CacheKey = (TypeId, Vec<usize>);
        type Cache = Mutex<HashMap<CacheKey, &'static OnceLock<ModelImplMetadata>>>;
        static CACHE: OnceLock<Cache> = OnceLock::new();
        let overlays: Vec<_> = providers.iter().map(|provider| provider()).collect();
        let key = (
            owner.type_id(),
            overlays
                .iter()
                .map(|value| std::ptr::from_ref(*value) as usize)
                .collect(),
        );
        let cell = {
            let mut cache = CACHE.get_or_init(Mutex::default).lock().expect("impl cache lock");
            *cache.entry(key).or_insert_with(|| Box::leak(Box::new(OnceLock::new())))
        };
        cell.get_or_init(|| {
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
                                    .is_some_and(|(left, right)| !std::ptr::eq(left, right));
                                let duplicate_setter = current
                                    .setter()
                                    .zip(property.setter())
                                    .is_some_and(|(left, right)| !std::ptr::eq(left, right));
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
            let result = if errors.is_empty() {
                Ok(&*Box::leak(Box::new(LocalPropertySet::new(Box::leak(
                    properties.into_boxed_slice(),
                )))))
            } else {
                Err(&*Box::leak(Box::new(PropertyBuildErrors::new(errors))))
            };
            Self::new(Box::leak(fragments.into_boxed_slice()), result)
        })
    }

    /// Creates generated implementation metadata.
    #[must_use]
    pub(crate) const fn new(
        fragments: &'static [PropertyFragment],
        properties: Result<&'static LocalPropertySet, &'static PropertyBuildErrors>,
    ) -> Self {
        Self { fragments, properties }
    }

    /// Returns every unmerged field/getter/setter source fact.
    #[must_use]
    #[inline(always)]
    pub const fn fragments(&self) -> &'static [PropertyFragment] {
        self.fragments
    }

    /// Returns locally merged properties or deterministic compatibility errors.
    ///
    /// # Errors
    ///
    /// Returns errors when field, getter, or setter fragments with the same
    /// name do not share a compatible value type.
    #[must_use = "handle property assembly failures"]
    pub const fn try_properties(&self) -> Result<&'static LocalPropertySet, &'static PropertyBuildErrors> {
        self.properties
    }
}
