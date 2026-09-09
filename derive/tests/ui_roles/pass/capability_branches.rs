// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow test-file-name
// The filename is part of a Cargo or trybuild fixture protocol.

//! Confirms opt-in behavior and all supported role shapes.

use qubit_model_derive::Enum;
use qubit_model_derive::Model;
use qubit_model_derive::Value;

#[Model(default, ord)]
struct OrderedModel {
    value: u8,
}

#[Model(partial_ord)]
struct PartiallyOrderedModel {
    value: u8,
}

#[Model]
struct UnitModel;

#[Value]
struct TupleValue(u8);

#[Value(transparent)]
struct TransparentValue(String);

#[Enum]
enum PlainEnum {
    Unit,
    Tuple(u8),
    Named { value: u8 },
}

fn main() {
    let _ = OrderedModel::default();
    let _ = PartiallyOrderedModel { value: 1 };
    let _ = UnitModel;
    let _ = TupleValue(1);
    let _ = TransparentValue("value".to_owned());
    let _ = PlainEnum::Unit;
    let _ = PlainEnum::Tuple(1);
    let _ = PlainEnum::Named { value: 1 };
}

#[Model(no_clone)]
#[derive(Clone)]
struct ExplicitClone;
