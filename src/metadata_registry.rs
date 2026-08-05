// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
//    You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
//    Unless required by applicable law or agreed to in writing, software
//    distributed under the License is distributed on an "AS IS" BASIS,
//    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//    See the License for the specific language governing permissions and
//    limitations under the License.
// =============================================================================

//! Explicit, allocation-free resolution of static model metadata.

use crate::metadata_resolver::MetadataResolver;
use crate::type_metadata::{
    TypeIdentity,
    TypeMetadata,
};

/// A caller-owned, statically declared collection of model metadata.
///
/// Registry lookup is intentionally linear and deterministic. This type does
/// not perform global registration or allocate; callers decide which model
/// metadata belongs to each registry.
#[derive(Clone, Copy, Debug)]
pub struct MetadataRegistry {
    models: &'static [&'static TypeMetadata],
}

impl MetadataRegistry {
    /// Creates a registry over a static metadata slice.
    ///
    /// # Parameters
    ///
    /// * `models` - The model metadata entries available to this registry.
    ///
    /// # Returns
    ///
    /// A registry that preserves the supplied declaration order.
    #[must_use]
    pub const fn new(models: &'static [&'static TypeMetadata]) -> Self {
        Self { models }
    }

    /// Returns the metadata entries in declaration order.
    ///
    /// # Returns
    ///
    /// The static slice owned by this registry.
    pub const fn models(self) -> &'static [&'static TypeMetadata] {
        self.models
    }
}

impl MetadataResolver for MetadataRegistry {
    /// Resolves an identity using the first matching registry entry.
    fn resolve(&self, identity: TypeIdentity) -> Option<&'static TypeMetadata> {
        self.models
            .iter()
            .copied()
            .find(|metadata| metadata.identity() == identity)
    }
}
