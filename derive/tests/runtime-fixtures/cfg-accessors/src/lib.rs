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
use qubit_reflect::Reflect;

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

#[Model]
pub struct GenericConfigured<T> { value: T }

#[ModelImpl(specialize(T = String), specialize(T = u32))]
impl<T: Clone + Reflect> GenericConfigured<T> {
    /// Specializations inherit method existence predicates.
    #[cfg(feature = "accessors")]
    pub fn current(&self) -> T { self.value.clone() }

    /// Inactive generic signatures must not retain unresolved types.
    #[cfg(any())]
    pub fn missing(&self) -> MissingGenericType { unreachable!() }
}

#[ModelImpl(specialize(T = String), specialize(T = u32))]
#[cfg_attr(not(feature = "impl-enabled"), cfg(any()))]
impl<T: Clone + Reflect> GenericConfigured<T> {
    /// Specializations also inherit the impl condition.
    pub fn impl_current(&self) -> T { self.value.clone() }
}

#[cfg(test)]
mod tests {
    use std::any::TypeId;

    use model_runtime::metadata::TypeMetadata;
    use qubit_reflect::ReflectedRef;
    use qubit_reflect::ReflectedMut;
    use qubit_reflect::ReflectedOwned;
    use qubit_reflect::registry::ReflectRegistry;

    use super::Configured;
    use super::GenericConfigured;

    /// Checks property presence, adapter execution, and reflected methods.
    #[test]
    fn test_real_feature_presence() {
        let metadata = TypeMetadata::of::<Configured>();
        let reflection = ReflectRegistry::initialize().expect("fixture reflection registry");
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
            let property = metadata.try_property(name).expect("valid property fragments");
            assert_eq!(property.is_some(), enabled, "property {name}");
            let method_present = reflection.implementations(TypeId::of::<Configured>()).iter()
                .flat_map(|implementation| implementation.methods())
                .any(|method| method.rust_name() == name);
            assert_eq!(method_present, enabled, "reflection method {name}");
            if let Some(property) = property {
                assert!(property.get(ReflectedRef::new(&value)).is_ok(), "adapter {name}");
            }
        }
        for (name, enabled) in [("set_absent", false), ("set_feature_value", cfg!(feature = "accessors")), ("set_paired", true)] {
            let count = reflection.implementations(TypeId::of::<Configured>()).iter()
                .flat_map(|implementation| implementation.methods()).filter(|method| method.rust_name() == name).count();
            assert_eq!(count, usize::from(enabled), "reflection setter {name}");
        }
        let paired = metadata.try_property("paired").expect("properties").expect("paired");
        let expected_type = if cfg!(feature = "alternate") { TypeId::of::<String>() } else { TypeId::of::<u32>() };
        assert_eq!(paired.descriptor().expect("paired descriptor").type_id(), expected_type);
        assert!(paired.setter().is_some());
        let mut changed = Configured { value: 0 };
        let replacement = if cfg!(feature = "alternate") {
            ReflectedOwned::new("23".to_owned())
        } else {
            ReflectedOwned::new(23u32)
        };
        paired.set(ReflectedMut::new(&mut changed), replacement).expect("enabled paired setter adapter");
        assert_eq!(changed.value, 23);
        if let Some(property) = metadata.try_property("feature_value").expect("feature property") {
            property.set(ReflectedMut::new(&mut changed), ReflectedOwned::new(29u32)).expect("feature setter adapter");
            assert_eq!(changed.value, 29);
        }
    }

    /// Checks method and impl predicates on two concrete specializations.
    #[test]
    fn test_generic_specialization_presence() {
        let reflection = ReflectRegistry::initialize().expect("generic fixture reflection");
        for metadata in [TypeMetadata::of::<GenericConfigured<String>>(), TypeMetadata::of::<GenericConfigured<u32>>()] {
            assert_eq!(metadata.try_property("current").expect("generic properties").is_some(), cfg!(feature = "accessors"));
            assert_eq!(metadata.try_property("impl_current").expect("generic impl properties").is_some(), cfg!(feature = "impl-enabled"));
            assert!(metadata.try_property("missing").expect("generic properties").is_none());
            for (name, enabled) in [("current", cfg!(feature = "accessors")), ("impl_current", cfg!(feature = "impl-enabled")), ("missing", false)] {
                let present = reflection.implementations(metadata.descriptor().type_id()).iter()
                    .flat_map(|implementation| implementation.methods()).any(|method| method.rust_name() == name);
                assert_eq!(present, enabled, "generic reflection method {name}");
            }
        }
    }
}
