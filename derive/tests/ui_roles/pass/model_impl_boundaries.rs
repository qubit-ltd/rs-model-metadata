// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// The filename is part of a Cargo or trybuild fixture protocol.

//! Describes reflected trait and generic implementation blocks.

use qubit_model_derive::Model;
use qubit_model_derive::ModelImpl;
use qubit_reflect::reflect;

#[Model]
struct Account;

#[reflect]
trait Named {
    // empty
}

#[ModelImpl]
impl Named for Account {}

#[Model]
struct Generic<T> { value: T }

#[ModelImpl]
impl<T> Generic<T> {}

fn main() {}
