// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Cache key and cell types used by
//! [`super::super::model_registry::ModelRegistry`].

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::OnceLock;

use qubit_reflect::context::ReflectionContextId;

use crate::metadata::ModelMetadataError;
use crate::metadata::PropertyResolutionError;
use crate::metadata::ResolvedProperties;
use crate::metadata::TypeMetadata;

/// Identifies cached properties by context, exact Rust type, and metadata
/// allocation.
pub(super) type PropertyCacheKey = (ReflectionContextId, TypeId, usize);

/// Shares one lazy property-resolution result; the cell retains its value or
/// original error.
pub(super) type PropertyCacheCell = Arc<OnceLock<Result<ResolvedProperties, PropertyResolutionError>>>;

/// Protects this registry's lazily initialized property-resolution cells.
pub(super) type PropertyCache = Mutex<HashMap<PropertyCacheKey, PropertyCacheCell>>;

/// Identifies metadata lookups by context, exact Rust type, and descriptor
/// allocation.
pub(super) type MetadataCacheKey = (ReflectionContextId, TypeId, usize);

/// Retains successful, missing, and failed metadata lookup results.
pub(super) type MetadataCacheValue = Result<Option<&'static TypeMetadata>, ModelMetadataError>;

/// Shares one lazy metadata lookup among concurrent callers.
pub(super) type MetadataCacheCell = Arc<OnceLock<MetadataCacheValue>>;

/// Protects this registry's lazily initialized metadata lookup cells.
pub(super) type MetadataCache = Mutex<HashMap<MetadataCacheKey, MetadataCacheCell>>;
