// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow test-file-name
// The filename is part of a Cargo or trybuild fixture protocol.

//! Rejects trait and generic implementation blocks for `ModelImpl`.

use qubit_model_derive::ModelImpl;

struct Account;

trait Named {}

#[ModelImpl]
impl Named for Account {}

struct Generic<T>(T);

#[ModelImpl]
impl<T> Generic<T> {}

fn main() {}
