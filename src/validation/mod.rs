// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Read-only validation binding for resolved model metadata.

mod build_error;
mod build_inputs;
mod compiled_property_path;
mod internal;
mod model_rule_binding;
mod model_validation_error;
mod standard_constraints;
mod validation_build_error;
mod validation_build_error_kind;
mod validation_build_errors;
mod validation_capabilities;
mod validation_options;
mod validation_plan;
mod validator_arguments;

pub use build_inputs::ValidationBuildInputs;
pub use model_rule_binding::ModelRuleBinding;
pub use model_validation_error::ModelValidationError;
pub use validation_build_error::ValidationBuildError;
pub use validation_build_error_kind::ValidationBuildErrorKind;
pub use validation_build_errors::ValidationBuildErrors;
pub use validation_capabilities::ValidationCapabilities;
pub use validation_options::FieldPath;
pub use validation_options::ValidationMode;
pub use validation_options::ValidationOptions;
pub use validation_options::ValidationOptionsBuilder;
pub use validation_options::ValidationSelection;
pub use validation_plan::ValidationPlan;
pub(crate) use validator_arguments::validator_arguments;
// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================
