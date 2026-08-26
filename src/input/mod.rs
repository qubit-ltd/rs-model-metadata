// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Parsed model declarations supported by the derive macro.

mod model_field;
mod model_input;
mod model_shape;
mod model_variant;
mod model_variant_shape;

pub(crate) use model_field::ModelField;
pub(crate) use model_input::ModelInput;
pub(crate) use model_shape::ModelShape;
pub(crate) use model_variant::ModelVariant;
pub(crate) use model_variant_shape::ModelVariantShape;
