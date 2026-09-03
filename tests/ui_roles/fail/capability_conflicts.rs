// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow test-file-name
// The filename is part of a Cargo or trybuild fixture protocol.

//! Rejects internally inconsistent capability switches and existing derives.

use qubit_model_derive::Model;

#[Model(copy, no_clone)]
struct CopyWithoutClone;

#[Model(copy, no_copy)]
struct CopyConflict;

#[Model(partial_ord, no_partial_eq)]
struct PartialOrderConflict;

#[Model(ord, no_eq)]
struct OrderConflict;

#[Model(no_clone)]
#[derive(Clone)]
struct ExistingCloneConflict;

fn main() {}
