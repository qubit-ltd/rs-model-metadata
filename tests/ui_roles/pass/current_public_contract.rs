// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Compiles the six supported public macro entry points together.

use model_runtime::__private::qubit_id::Id;
use qubit_model_derive::Entity;
use qubit_model_derive::Enum;
use qubit_model_derive::Model;
use qubit_model_derive::ModelImpl;
use qubit_model_derive::Projection;
use qubit_model_derive::Value;

#[Entity(id = "trybuild.ContractEntity")]
struct ContractEntity {
    #[identifier]
    id: Id,
}

#[Projection(id = "trybuild.ContractProjection", source = ContractEntity)]
struct ContractProjection {
    #[identifier]
    id: Id,
}

#[Model(id = "trybuild.ContractModel")]
struct ContractModel<T> {
    #[reference(entity = "trybuild.ContractEntity", property = id)]
    owner: Id,
    value: T,
}

#[Enum(id = "trybuild.ContractEnum")]
enum ContractEnum<T> {
    Ready,
    Value(T),
}

#[Value(id = "trybuild.ContractValue", transparent)]
struct ContractValue(String);

#[Model]
struct ContractProperty {
    name: String,
}

#[ModelImpl]
impl ContractProperty {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, value: String) {
        self.name = value;
    }
}

fn main() {
    let _ = ContractEntity { id: Id::new(1) };
    let _ = ContractProjection { id: Id::new(1) };
    let _ = ContractModel::<String> {
        owner: Id::new(1),
        value: String::new(),
    };
    let _ = ContractEnum::<String>::Ready;
    let _ = ContractValue(String::new());
    let _ = ContractProperty {
        name: String::new(),
    };
}
