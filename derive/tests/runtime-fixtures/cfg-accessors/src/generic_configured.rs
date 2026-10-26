// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Generic models used to check feature-gated specialization presence.

use qubit_model_derive::Model;
use qubit_model_derive::ModelImpl;
use qubit_reflect::descriptor::Reflect;

/// Generic model used to verify specialized method registrations.
///
/// # Type Parameters
///
/// * `T` is the concrete reflected value type stored by the model.
#[Model]
struct GenericConfigured<T> {
    /// Value returned by methods enabled for each concrete specialization.
    value: T,
}

#[ModelImpl(specialize(T = String), specialize(T = u32))]
impl<T: Clone + Reflect> GenericConfigured<T> {
    /// Returns the value for specializations with the accessors feature.
    #[must_use]
    #[cfg(feature = "accessors")]
    pub fn current(&self) -> T {
        self.value.clone()
    }

    /// Provides an intentionally unavailable signature for cfg filtering.
    #[cfg(any())]
    pub fn missing(&self) -> MissingGenericType {
        unreachable!()
    }
}

#[ModelImpl(specialize(T = String), specialize(T = u32))]
#[cfg(feature = "impl-enabled")]
impl<T: Clone + Reflect> GenericConfigured<T> {
    /// Returns the value for specializations with the impl feature enabled.
    #[must_use]
    pub fn impl_current(&self) -> T {
        self.value.clone()
    }
}

#[cfg(test)]
mod tests {
    use model_runtime::metadata::TypeMetadata;
    use model_runtime::registry::ModelRegistry;
    use qubit_reflect::ReflectionContext;

    use super::GenericConfigured;

    /// Checks method and impl predicates on two concrete specializations.
    #[test]
    fn test_generic_specialization_presence() {
        let reflection = ReflectionContext::builder()
            .include_linked()
            .build()
            .expect("generic fixture reflection");
        let models = ModelRegistry::from_context(&reflection).expect("generic model registry");
        for metadata in [
            TypeMetadata::of::<GenericConfigured<String>>(),
            TypeMetadata::of::<GenericConfigured<u32>>(),
        ] {
            let properties = models.properties_for(metadata).expect("generic properties");
            assert_eq!(
                properties.property("current").is_some(),
                cfg!(feature = "accessors")
            );
            assert_eq!(
                properties.property("impl_current").is_some(),
                cfg!(feature = "impl-enabled")
            );
            assert!(properties.property("missing").is_none());
            let generic_impls = reflection
                .impl_definitions()
                .expect("generic source impl definitions");
            for (name, enabled) in [
                ("current", cfg!(feature = "accessors")),
                ("impl_current", cfg!(feature = "impl-enabled")),
                ("missing", false),
            ] {
                let present = generic_impls
                    .iter()
                    .any(|implementation| {
                        let target = implementation.target().display().to_string();
                        target.contains("GenericConfigured")
                            && implementation.methods().get(name).is_some()
                    });
                assert_eq!(present, enabled, "generic reflection method {name}");
            }
        }
    }
}
