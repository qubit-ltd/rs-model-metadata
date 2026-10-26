// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Module boundary for one registry's bounded successful property-path cache.

#[path = "path_cache/cache_entry.rs"]
mod cache_entry;
#[path = "path_cache/path_cache.rs"]
mod path_cache;
#[path = "path_cache/path_cache_key.rs"]
mod path_cache_key;

pub(super) use self::path_cache::PathCache;
pub(super) use self::path_cache_key::PathCacheKey;
