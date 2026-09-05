// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow test-file-name
// The filename is part of a Cargo or trybuild fixture protocol.

//! Rejects role-specific options and conflicting field semantics.

use std::collections::HashMap;

use model_runtime::__private::qubit_id::Id;
use qubit_model_derive::Enum;
use qubit_model_derive::Model;
use qubit_model_derive::Projection;

#[Model(source_id = "example.Source")]
struct SourceIdOnModel;

#[Model(source = SourceIdOnModel)]
struct SourceOnModel;

#[Projection(source = SourceIdOnModel, source_id = "example.Source", open)]
struct ConflictingProjection {
    #[identifier]
    id: Id,
}

#[Model(transparent, codec = Codec)]
struct ValueOptionsOnModel {
    value: String,
}

#[Model(no_copy)]
struct NoCopyOnModel;

#[Enum(copy, no_copy)]
enum CopyAndNoCopy {
    Value,
}

#[Model]
struct ConflictingFields {
    #[unique]
    #[indexed]
    indexed_twice: String,
    #[reference(entity_id = "example.Owner")]
    #[opaque]
    hidden_reference: Id,
    #[element(redact(skip))]
    invalid_selector_redaction: Vec<String>,
    #[map_key(text(max_chars = 4))]
    #[map_key(text(min_chars = 1))]
    duplicate_selector: HashMap<String, String>,
    #[redact(level = "low")]
    #[element(redact(level = "high"))]
    overlapping_redaction: Vec<String>,
}

struct Codec;

fn main() {}

const _: Option<Id> = None;
const _: Option<HashMap<String, String>> = None;
