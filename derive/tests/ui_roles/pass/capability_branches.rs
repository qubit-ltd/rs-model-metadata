// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow test-file-name
// The filename is part of a Cargo or trybuild fixture protocol.

//! Covers valid capability and plain-display combinations.

use qubit_model_derive::Enum;
use qubit_model_derive::Model;
use qubit_model_derive::Value;

#[Model(no_redact, ord, default)]
struct OrderedModel {
    value: u8,
}

#[Model(no_redact, partial_ord)]
struct PartiallyOrderedModel {
    value: u8,
}

#[Model(no_redact)]
struct UnitModel;

#[Value(no_redact)]
struct TupleValue(u8);

#[Value(no_redact, transparent)]
struct TransparentValue(String);

#[Enum(no_redact, no_copy)]
enum PlainEnum {
    Unit,
    Tuple(u8),
    Named { value: u8 },
}

fn main() {
    let _ = OrderedModel::default().to_string();
    let _ = PartiallyOrderedModel { value: 1 }.to_string();
    let _ = UnitModel.to_string();
    let _ = TupleValue(1).to_string();
    let _ = TransparentValue("value".to_owned()).to_string();
    let _ = PlainEnum::Unit.to_string();
    let _ = PlainEnum::Tuple(1).to_string();
    let _ = PlainEnum::Named { value: 1 }.to_string();
}
