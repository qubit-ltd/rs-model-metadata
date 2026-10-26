// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Generated metadata for one model-aware inherent implementation block.

use std::collections::HashMap;
use std::collections::HashSet;
use std::ptr::eq;
use std::sync::Arc;

use qubit_reflect::ReflectionContext;

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
use crate::reflect_facade::ModelImplSource;

// Owns snapshot-specific merged property storage.
mod internal;

pub(crate) use internal::merged_model_impl::MergedModelImpl;

/// Appends an overlay's fragments while retaining the first backing field.
///
/// `field_names` tracks fields already emitted, while non-field fragments are
/// always appended in provider and declaration order.
fn append_overlay_fragments(
    fragments: &mut Vec<PropertyFragment>,
    field_names: &mut HashSet<&'static str>,
    overlay: &ModelImplMetadata,
) {
    for fragment in overlay.fragments() {
        if matches!(fragment.source(), PropertyFragmentSource::Field(_)) && !field_names.insert(fragment.name()) {
            continue;
        }
        fragments.push(*fragment);
    }
}

/// Merges one overlay's successful properties and retains its diagnostics.
///
/// `property_indices` points into the ordered output vector, avoiding a scan
/// while preserving the first occurrence and provider order of each property.
fn merge_overlay_properties(
    overlay: &ModelImplMetadata,
    origin: ModelImplSource,
    properties: &mut Vec<(PropertyMetadata, Option<ModelImplSource>, Option<ModelImplSource>)>,
    property_indices: &mut HashMap<&'static str, usize>,
    errors: &mut Vec<PropertyBuildError>,
) {
    match overlay.try_properties() {
        Err(error) => errors.extend_from_slice(error.errors()),
        Ok(local) => {
            for property in local.properties() {
                if let Some(index) = property_indices.get(property.name()).copied() {
                    let (current, getter_origin, setter_origin) = &mut properties[index];
                    if let Some((first, second)) = current.getter().zip(property.getter())
                        && !eq(first, second)
                    {
                        errors.push(PropertyBuildError::with_conflict(
                            PropertyBuildErrorKind::ConflictingGetter,
                            current.name(),
                            first.rust_method_name(),
                            second.rust_method_name(),
                            (*getter_origin).expect("selected getter retains its origin"),
                            origin,
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
                            (*setter_origin).expect("selected setter retains its origin"),
                            origin,
                        ));
                    }
                    if getter_origin.is_none() && property.getter().is_some() {
                        *getter_origin = Some(origin);
                    }
                    if setter_origin.is_none() && property.setter().is_some() {
                        *setter_origin = Some(origin);
                    }
                    let selected = if current.field().is_some() || current.getter().is_some() {
                        *current
                    } else {
                        *property
                    };
                    *current = PropertyMetadata::new(
                        current.name(),
                        selected.type_expression_data(),
                        current.field().or(property.field()),
                        current.getter().or(property.getter()),
                        current.setter().or(property.setter()),
                    );
                } else {
                    property_indices.insert(property.name(), properties.len());
                    properties.push((
                        *property,
                        property.getter().map(|_| origin),
                        property.setter().map(|_| origin),
                    ));
                }
            }
        }
    }
}

/// Stores the declarations emitted for one model-aware implementation block.
///
/// `fragments` preserves the block's source declarations, including facts that
/// may not appear in the local property set. `properties` retains either the
/// successful local assembly or its diagnostics, so snapshot resolution can
/// combine providers without rerunning this block's assembly. Both values are
/// immutable static metadata; dynamic snapshot merges use `MergedModelImpl`
/// and own their storage.
///
/// # Examples
///
/// A generated provider supplies this metadata to the reflection API. Consumers
/// can inspect its original declarations and the result of local assembly:
///
/// ```
/// use qubit_model_metadata::metadata::ModelImplMetadata;
///
/// fn inspect(metadata: &ModelImplMetadata) {
///     let _fragments = metadata.fragments();
///     match metadata.try_properties() {
///         Ok(properties) => {
///             let _ = properties;
///         }
///         Err(errors) => {
///             let _ = errors;
///         }
///     }
/// }
/// ```
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
    pub(crate) fn merge(
        owner: &TypeMetadata,
        providers: &[(ModelImplProvider, ModelImplSource)],
        context: &ReflectionContext,
    ) -> MergedModelImpl {
        let overlays: Vec<_> = providers
            .iter()
            .map(|(provider, origin)| (provider(), origin))
            .collect();
        let mut fragments = Vec::new();
        let mut field_names = HashSet::new();
        let mut properties: Vec<(PropertyMetadata, Option<ModelImplSource>, Option<ModelImplSource>)> = Vec::new();
        let mut property_indices = HashMap::new();
        let mut errors = Vec::new();
        for (overlay, origin) in overlays {
            append_overlay_fragments(&mut fragments, &mut field_names, overlay);
            merge_overlay_properties(overlay, *origin, &mut properties, &mut property_indices, &mut errors);
        }
        let properties: Vec<_> = properties.into_iter().map(|(property, _, _)| property).collect();
        if let Err(error) = owner.validate_properties_in(context, &properties) {
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
    #[inline]
    pub const fn try_properties(&self) -> Result<&'static LocalPropertySet, &'static PropertyBuildErrors> {
        self.properties
    }
}
