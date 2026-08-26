// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Internal helpers for declaration rewriting in the `Model` attribute macro.

mod declaration_fields;
#[path = "default_derives.rs"]
mod derive_defaults;
mod enum_api;
mod serde_fields;

pub(super) use declaration_fields::has_redact_fields;
pub(super) use declaration_fields::remove_field_attributes;
pub(super) use declaration_fields::remove_serde_attributes;
pub(super) use derive_defaults::default_derives;
pub(super) use enum_api::expand_display;
pub(super) use enum_api::expand_enum_names;
pub(super) use serde_fields::add_default_serde_field_attributes;
