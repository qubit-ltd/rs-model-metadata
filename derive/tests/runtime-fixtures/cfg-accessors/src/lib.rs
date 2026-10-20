// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// Cargo's fixture entry point intentionally uses lib.rs.

//! Consumer feature matrix for property and reflection presence.

use qubit_model_derive::Model;
use qubit_model_derive::ModelImpl;

#[Model]
pub struct Configured {
    value: u32,
}

#[ModelImpl]
impl Configured {
    /// Never contributes an adapter or signature type reference.
    #[cfg(any())]
    pub fn absent(&self) -> MissingGetterType { unreachable!() }

    /// Never contributes an adapter or signature type reference.
    #[cfg(any())]
    pub fn set_absent(&mut self, _: MissingSetterType) { unreachable!() }

    /// Uses the default getter type.
    #[cfg(not(feature = "alternate"))]
    pub fn paired(&self) -> u32 { self.value }

    /// Uses a different mutually exclusive getter type.
    #[cfg(feature = "alternate")]
    pub fn paired(&self) -> String { self.value.to_string() }

    /// Is deliberately the first setter candidate, but never enabled.
    #[cfg(any())]
    pub fn set_paired(&mut self, _: MissingPairType) { unreachable!() }

    /// Matches the default getter when enabled.
    #[cfg(all(not(feature = "alternate"), not(feature = "mismatch")))]
    pub fn set_paired(&mut self, value: u32) { self.value = value; }

    /// Matches the alternate getter when enabled.
    #[cfg(all(feature = "alternate", not(feature = "mismatch")))]
    pub fn set_paired(&mut self, value: String) {
        self.value = value.parse().expect("numeric fixture value");
    }

    /// Intersects either active getter and must trigger a type diagnostic.
    #[cfg(feature = "mismatch")]
    pub fn set_paired(&mut self, _: bool) {}

    /// Adds a simultaneously enabled getter for a failure configuration.
    #[cfg(feature = "duplicate-getter")]
    pub fn paired(&self) -> bool { true }

    /// Adds a simultaneously enabled setter for a failure configuration.
    #[cfg(feature = "duplicate-setter")]
    pub fn set_paired(&mut self, _: bool) {}

    /// Is absent when accessors is enabled; ordinary attributes remain intact.
    #[cfg_attr(feature = "accessors", cfg(any()), inline)]
    pub fn conditional(&self) -> u32 { self.value }

    /// Is absent only when both real features are enabled.
    #[cfg_attr(feature = "accessors", cfg_attr(feature = "alternate", cfg(any())), doc = "conditional docs")]
    pub fn nested(&self) -> u32 { self.value }

    /// Multiple presence-changing cfg_attr attributes are conjoined.
    #[cfg_attr(feature = "accessors", cfg(all()), cfg(not(feature = "alternate")))]
    pub fn multiple(&self) -> u32 { self.value }

    /// Independent cfg attributes are conjoined.
    #[cfg(all())]
    #[cfg(not(feature = "alternate"))]
    pub fn conjunction(&self) -> u32 { self.value }

    /// A real feature controls an ordinary getter and setter pair.
    #[cfg(feature = "accessors")]
    pub fn feature_value(&self) -> u32 { self.value }

    /// A real feature controls an ordinary getter and setter pair.
    #[cfg(feature = "accessors")]
    pub fn set_feature_value(&mut self, value: u32) { self.value = value; }
}

#[Model]
struct GenericConfigured<T> {
    value: T,
}

#[ModelImpl(specialize(T = String), specialize(T = u32))]
impl<T: Clone + qubit_reflect::descriptor::Reflect> GenericConfigured<T> {
    #[cfg(feature = "accessors")]
    pub fn current(&self) -> T {
        self.value.clone()
    }

    #[cfg(any())]
    pub fn missing(&self) -> MissingGenericType {
        unreachable!()
    }
}

#[ModelImpl(specialize(T = String), specialize(T = u32))]
#[cfg(feature = "impl-enabled")]
impl<T: Clone + qubit_reflect::descriptor::Reflect> GenericConfigured<T> {
    pub fn impl_current(&self) -> T {
        self.value.clone()
    }
}

#[ModelImpl]
#[cfg_attr(not(feature = "impl-enabled"), cfg(any()))]
impl Configured {
    /// The complete provider and registration share the impl condition.
    pub fn enabled_impl(&self) -> u32 { self.value }
}

#[ModelImpl]
#[cfg(any())]
impl MissingImplTarget {
    /// No signature or provider reference survives the impl condition.
    pub fn missing_impl(&self) -> MissingImplReturn { unreachable!() }
}

#[cfg(test)]
mod tests {
    use std::any::TypeId;

    use model_runtime::metadata::TypeMetadata;
    use model_runtime::registry::ModelRegistry;
    use qubit_reflect::ReflectionContext;
    use qubit_reflect::TypeDescriptor;
    use qubit_reflect::value::DynamicMut;
    use qubit_reflect::value::DynamicOwned;
    use qubit_reflect::value::DynamicRef;
    use qubit_reflect::value::Local;

    use super::Configured;
    use super::GenericConfigured;

    /// Checks property presence, adapter execution, and reflected methods.
    #[test]
    fn test_real_feature_presence() {
        let metadata = TypeMetadata::of::<Configured>();
        let reflection = ReflectionContext::builder()
            .include_linked()
            .build()
            .expect("fixture reflection context");
        let models = ModelRegistry::from_context(&reflection).expect("fixture model registry");
        let properties = models.properties_for(metadata).expect("fixture properties");
        let descriptor = TypeDescriptor::of_in::<Configured>(&reflection);
        let expected = [
            ("absent", false),
            ("conditional", !cfg!(feature = "accessors")),
            ("nested", !cfg!(all(feature = "accessors", feature = "alternate"))),
            ("multiple", !cfg!(all(feature = "accessors", feature = "alternate"))),
            ("conjunction", !cfg!(feature = "alternate")),
            ("feature_value", cfg!(feature = "accessors")),
            ("enabled_impl", cfg!(feature = "impl-enabled")),
            ("paired", true),
        ];
        let value = Configured { value: 17 };
        for (name, enabled) in expected {
            let property = properties.property(name);
            assert_eq!(property.is_some(), enabled, "property {name}");
            let method_present = descriptor
                .inherent_impls()
                .expect("inherent impls")
                .iter()
                .any(|implementation| {
                    implementation
                        .methods()
                        .iter()
                        .any(|method| method.name() == name)
                });
            assert_eq!(method_present, enabled, "reflection method {name}");
            if let Some(property) = property {
                assert!(property.get_dynamic_in(&reflection, DynamicRef::<Local>::new(&value)).is_ok(), "adapter {name}");
            }
        }
        for (name, enabled) in [
            ("set_absent", false),
            ("set_feature_value", cfg!(feature = "accessors")),
            ("set_paired", true),
        ] {
            let count = descriptor
                .inherent_impls()
                .expect("inherent impls")
                .iter()
                .map(|implementation| {
                    implementation
                        .methods()
                        .iter()
                        .filter(|method| method.name() == name)
                        .count()
                })
                .sum::<usize>();
            assert_eq!(count, usize::from(enabled), "reflection setter {name}");
        }
        let paired = properties.property("paired").expect("paired");
        let expected_type = if cfg!(feature = "alternate") { TypeId::of::<String>() } else { TypeId::of::<u32>() };
        assert_eq!(paired.descriptor_in(&reflection).expect("paired descriptor lookup").expect("paired descriptor").type_id(), expected_type);
        assert!(paired.setter().is_some());
        let mut changed = Configured { value: 0 };
        let replacement = if cfg!(feature = "alternate") {
            DynamicOwned::<Local>::new("23".to_owned())
        } else {
            DynamicOwned::<Local>::new(23u32)
        };
        paired.set_dynamic_in(&reflection, DynamicMut::<Local>::new(&mut changed), replacement).expect("enabled paired setter adapter");
        assert_eq!(changed.value, 23);
        if let Some(property) = properties.property("feature_value") {
            property.set_dynamic_in(&reflection, DynamicMut::<Local>::new(&mut changed), DynamicOwned::<Local>::new(29u32)).expect("feature setter adapter");
            assert_eq!(changed.value, 29);
        }
    }

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
