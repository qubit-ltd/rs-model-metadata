// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Successfully merged local property metadata.

use crate::metadata::PropertyMetadata;

/// A model type's locally validated field/getter/setter properties.
#[derive(Clone, Copy, Debug)]
pub struct LocalPropertySet {
    /// Properties ordered by their first declaration fragment.
    properties: &'static [PropertyMetadata],
}

impl LocalPropertySet {
    /// Creates a locally validated property collection.
    #[must_use]
    pub(crate) const fn new(properties: &'static [PropertyMetadata]) -> Self {
        Self { properties }
    }

    /// Returns merged properties in deterministic declaration order.
    #[must_use]
    #[inline]
    pub const fn properties(&self) -> &'static [PropertyMetadata] {
        self.properties
    }

    /// Finds a merged property by its canonical public name.
    #[must_use]
    pub fn property(&self, name: &str) -> Option<&'static PropertyMetadata> {
        self.properties.iter().find(|property| property.name() == name)
    }
}

#[cfg(test)]
mod tests {
    use qubit_reflect::TypeDescriptor;
    use qubit_reflect::descriptor::TypeRef;

    use super::LocalPropertySet;
    use crate::metadata::PropertyMetadata;

    #[test]
    fn empty_set_exposes_empty_slice_and_misses_every_name() {
        let set = LocalPropertySet::new(&[]);

        assert!(set.properties().is_empty());
        assert!(set.property("missing").is_none());
    }

    #[test]
    fn property_lookup_matches_the_canonical_name() {
        let type_ref = Box::leak(Box::new(TypeRef::Resolved(TypeDescriptor::of::<u32>())));
        let properties = Box::leak(vec![PropertyMetadata::new("value", type_ref, None, None, None)].into_boxed_slice());
        let set = LocalPropertySet::new(properties);

        assert_eq!(set.property("value").map(PropertyMetadata::name), Some("value"));
        assert!(set.property("other").is_none());
    }
}
