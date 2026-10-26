// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Map key and value selection for generated constraints.

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

/// Projects the key and value types from a supported map shape.
///
/// Implementations cover hash and ordered maps, with optional or pointer
/// wrappers around another supported map shape. The projected key and value
/// types must be `'static` so generated constraints can use them as type
/// parameters.
#[doc(hidden)]
pub trait MapConstraintTarget {
    /// Key type selected from the map shape, required to be `'static`.
    type Key: 'static;
    /// Value type selected from the map shape, required to be `'static`.
    type Value: 'static;
}

impl<K: 'static, V: 'static, S> MapConstraintTarget for HashMap<K, V, S> {
    type Key = K;
    type Value = V;
}

impl<K: 'static, V: 'static> MapConstraintTarget for BTreeMap<K, V> {
    type Key = K;
    type Value = V;
}

impl<T: MapConstraintTarget> MapConstraintTarget for Option<T> {
    type Key = T::Key;
    type Value = T::Value;
}

macro_rules! transparent_map_target {
    ($($wrapper:ty),+ $(,)?) => {
        $(impl<T: MapConstraintTarget + ?Sized> MapConstraintTarget for $wrapper {
            type Key = T::Key;
            type Value = T::Value;
        })+
    };
}

transparent_map_target!(Box<T>, Rc<T>, Arc<T>);
