// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Characterization coverage for the derive crate's public macro contract.

use model_runtime::__private::qubit_id::Id;
use model_runtime::ModelRole;
use model_runtime::TypeMetadata;
use qubit_model_derive::Entity;
use qubit_model_derive::Enum;
use qubit_model_derive::Model;
use qubit_model_derive::ModelImpl;
use qubit_model_derive::Projection;
use qubit_model_derive::Value;

#[Entity(id = "contract.Entity")]
struct ContractEntity {
    #[identifier]
    id: Id,
}

#[Projection(id = "contract.Projection", source = ContractEntity)]
struct ContractProjection {
    #[identifier]
    id: Id,
}

#[Model(id = "contract.Model")]
struct ContractModel<T> {
    #[reference(entity = "contract.Entity", property = id)]
    owner: Id,
    value: T,
}

#[Enum(id = "contract.Enum")]
enum ContractEnum<T> {
    Ready,
    Value(T),
}

#[Value(id = "contract.Value", transparent)]
struct ContractValue(String);

#[Model(id = "contract.Property")]
struct ContractProperty {
    name: String,
}

#[ModelImpl]
impl ContractProperty {
    /// Returns the property's borrowed name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Replaces the property's name.
    pub fn set_name(&mut self, value: String) {
        self.name = value;
    }
}

#[test]
fn test_public_macros_emit_metadata_for_all_roles() {
    assert_eq!(TypeMetadata::of::<ContractEntity>().role(), ModelRole::Entity);
    assert_eq!(TypeMetadata::of::<ContractProjection>().role(), ModelRole::Projection);
    assert_eq!(TypeMetadata::of::<ContractModel<String>>().role(), ModelRole::Model);
    assert_eq!(TypeMetadata::of::<ContractEnum<String>>().role(), ModelRole::Enum);
    assert_eq!(TypeMetadata::of::<ContractValue>().role(), ModelRole::Value);
}

#[test]
fn test_model_impl_contributes_property_metadata() {
    let metadata = TypeMetadata::of::<ContractProperty>();
    let properties = metadata
        .try_properties()
        .expect("ModelImpl properties must merge with fields");

    assert!(properties.property("name").is_some());
}
