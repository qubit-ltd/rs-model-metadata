// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Private construction validators for attribute metadata.

// Validates field-name uniqueness, emptiness, and optional logical names.
mod validation;

pub(super) use self::validation::validate_named_fields;
pub(super) use self::validation::validate_optional_logical_name;
pub(super) use self::validation::validate_primary_key_fields;
pub(super) use self::validation::validate_unique_fields;
