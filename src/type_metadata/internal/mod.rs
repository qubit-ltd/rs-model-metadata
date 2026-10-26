// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// Field, property, and role metadata validation are kept in separate units.
mod common;
mod fields;
mod properties;
mod roles;

pub(super) use common::abi_violation;
pub(super) use common::contains_field;
pub(super) use fields::descriptor_for;
pub(super) use fields::validate_fields;
pub(super) use properties::validate_properties;
pub(super) use roles::validate_role;
