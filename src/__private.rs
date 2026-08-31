// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Hidden, versioned ABI consumed by generated model code.
// qubit-style: allow multiple-public-types
// qubit-style: allow type-file-name

pub use inventory;
pub use qubit_codec;
pub use qubit_redact;
pub use qubit_reflect::__private::*;
pub use qubit_reflect::capability::TypeCapabilities as ReflectTypeCapabilities;
pub use qubit_reflect::capability::registered_type_capabilities;
pub use qubit_reflect::register_type_capabilities;
pub use serde;

/// Serde predicates used by generated omission defaults.
#[doc(hidden)]
pub mod serde_helpers {
    pub const fn is_none<T>(value: &Option<T>) -> bool {
        value.is_none()
    }

    pub trait IsEmpty {
        fn is_empty(&self) -> bool;
    }

    macro_rules! impl_is_empty {
        ($($type:ty),+ $(,)?) => {
            $(impl<T> IsEmpty for $type {
                fn is_empty(&self) -> bool { self.is_empty() }
            })+
        };
    }

    impl_is_empty!(
        Vec<T>,
        std::collections::VecDeque<T>,
        std::collections::LinkedList<T>,
        std::collections::BinaryHeap<T>
    );

    impl<T: Ord> IsEmpty for std::collections::BTreeSet<T> {
        fn is_empty(&self) -> bool {
            self.is_empty()
        }
    }
    impl<T, S> IsEmpty for std::collections::HashSet<T, S> {
        fn is_empty(&self) -> bool {
            self.is_empty()
        }
    }
    impl<K: Ord, V> IsEmpty for std::collections::BTreeMap<K, V> {
        fn is_empty(&self) -> bool {
            self.is_empty()
        }
    }
    impl<K, V, S> IsEmpty for std::collections::HashMap<K, V, S> {
        fn is_empty(&self) -> bool {
            self.is_empty()
        }
    }

    pub fn is_empty<T: IsEmpty>(value: &T) -> bool {
        value.is_empty()
    }
}

/// Marker implemented only by code generated from a model-role macro.
#[doc(hidden)]
pub trait ModelTypeSeal {}

/// Marker implemented once by each generated `ModelProperties` block.
#[doc(hidden)]
pub trait ModelPropertiesSeal {}

/// Versioned ABI consumed by generated model metadata code.
#[doc(hidden)]
pub mod v1 {
    use crate::TypeDescriptor;
    use crate::TypeMetadata;
    pub use crate::reflect_facade::model_capability;

    /// Creates the overlay for one already-created reflection descriptor.
    #[doc(hidden)]
    #[must_use]
    pub const fn type_metadata(descriptor: &'static TypeDescriptor) -> TypeMetadata {
        TypeMetadata::from_descriptor(descriptor)
    }

    #[doc(hidden)]
    pub use crate::__qubit_model_register_model_capability as register_model_capability;
    pub use crate::__qubit_model_register_properties_capability as register_properties_capability;
}

/// Registers a generated property provider on the shared reflection root.
#[doc(hidden)]
#[macro_export]
macro_rules! __qubit_model_register_properties_capability {
    ($target:ty, $provider:expr $(,)?) => {
        $crate::__private::register_type_capabilities!(
            $target: [$crate::model_properties_key() => $provider]
        );
    };
}

/// Registers a model metadata provider as a typed reflection capability.
#[doc(hidden)]
#[macro_export]
macro_rules! __qubit_model_register_model_capability {
    ($target:ty, $provider:expr $(,)?) => {
        $crate::__private::register_type_capabilities!(
            $target: [$crate::model_metadata_key() => $provider]
        );
    };
}
