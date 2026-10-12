// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Trybuild compile-fail fixture: the removed `reflect(crate = ...)` helper is rejected.

use qubit_reflect::Reflect;

#[derive(Reflect)]
#[reflect(crate = qubit_reflect)]
struct ObsoleteRuntimeHelper;

fn main() {}
