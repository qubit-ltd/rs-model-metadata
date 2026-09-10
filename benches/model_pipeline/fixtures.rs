// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Static workloads with one independent implementation fragment per field.

use qubit_model_derive::Model;
use qubit_model_derive::ModelImpl;

/// 1 text declarations and 1 independently registered getter fragments.
#[Model]
pub(super) struct Pipeline1 {
    #[text(non_blank)]
    field_0: String,
}

impl Pipeline1 {
    /// Creates valid input outside the measured loops.
    pub(super) fn sample() -> Self {
        Self {
            field_0: "valid input".to_owned(),
        }
    }
}

#[ModelImpl]
impl Pipeline1 {
    pub fn field_0(&self) -> &str {
        &self.field_0
    }
}

/// 8 text declarations and 8 independently registered getter fragments.
#[Model]
pub(super) struct Pipeline8 {
    #[text(non_blank)]
    field_0: String,
    #[text(non_blank)]
    field_1: String,
    #[text(non_blank)]
    field_2: String,
    #[text(non_blank)]
    field_3: String,
    #[text(non_blank)]
    field_4: String,
    #[text(non_blank)]
    field_5: String,
    #[text(non_blank)]
    field_6: String,
    #[text(non_blank)]
    field_7: String,
}

impl Pipeline8 {
    /// Creates valid input outside the measured loops.
    pub(super) fn sample() -> Self {
        Self {
            field_0: "valid input".to_owned(),
            field_1: "valid input".to_owned(),
            field_2: "valid input".to_owned(),
            field_3: "valid input".to_owned(),
            field_4: "valid input".to_owned(),
            field_5: "valid input".to_owned(),
            field_6: "valid input".to_owned(),
            field_7: "valid input".to_owned(),
        }
    }
}

#[ModelImpl]
impl Pipeline8 {
    pub fn field_0(&self) -> &str {
        &self.field_0
    }
}

#[ModelImpl]
impl Pipeline8 {
    pub fn field_1(&self) -> &str {
        &self.field_1
    }
}

#[ModelImpl]
impl Pipeline8 {
    pub fn field_2(&self) -> &str {
        &self.field_2
    }
}

#[ModelImpl]
impl Pipeline8 {
    pub fn field_3(&self) -> &str {
        &self.field_3
    }
}

#[ModelImpl]
impl Pipeline8 {
    pub fn field_4(&self) -> &str {
        &self.field_4
    }
}

#[ModelImpl]
impl Pipeline8 {
    pub fn field_5(&self) -> &str {
        &self.field_5
    }
}

#[ModelImpl]
impl Pipeline8 {
    pub fn field_6(&self) -> &str {
        &self.field_6
    }
}

#[ModelImpl]
impl Pipeline8 {
    pub fn field_7(&self) -> &str {
        &self.field_7
    }
}

/// 32 text declarations and 32 independently registered getter fragments.
#[Model]
pub(super) struct Pipeline32 {
    #[text(non_blank)]
    field_0: String,
    #[text(non_blank)]
    field_1: String,
    #[text(non_blank)]
    field_2: String,
    #[text(non_blank)]
    field_3: String,
    #[text(non_blank)]
    field_4: String,
    #[text(non_blank)]
    field_5: String,
    #[text(non_blank)]
    field_6: String,
    #[text(non_blank)]
    field_7: String,
    #[text(non_blank)]
    field_8: String,
    #[text(non_blank)]
    field_9: String,
    #[text(non_blank)]
    field_10: String,
    #[text(non_blank)]
    field_11: String,
    #[text(non_blank)]
    field_12: String,
    #[text(non_blank)]
    field_13: String,
    #[text(non_blank)]
    field_14: String,
    #[text(non_blank)]
    field_15: String,
    #[text(non_blank)]
    field_16: String,
    #[text(non_blank)]
    field_17: String,
    #[text(non_blank)]
    field_18: String,
    #[text(non_blank)]
    field_19: String,
    #[text(non_blank)]
    field_20: String,
    #[text(non_blank)]
    field_21: String,
    #[text(non_blank)]
    field_22: String,
    #[text(non_blank)]
    field_23: String,
    #[text(non_blank)]
    field_24: String,
    #[text(non_blank)]
    field_25: String,
    #[text(non_blank)]
    field_26: String,
    #[text(non_blank)]
    field_27: String,
    #[text(non_blank)]
    field_28: String,
    #[text(non_blank)]
    field_29: String,
    #[text(non_blank)]
    field_30: String,
    #[text(non_blank)]
    field_31: String,
}

impl Pipeline32 {
    /// Creates valid input outside the measured loops.
    pub(super) fn sample() -> Self {
        Self {
            field_0: "valid input".to_owned(),
            field_1: "valid input".to_owned(),
            field_2: "valid input".to_owned(),
            field_3: "valid input".to_owned(),
            field_4: "valid input".to_owned(),
            field_5: "valid input".to_owned(),
            field_6: "valid input".to_owned(),
            field_7: "valid input".to_owned(),
            field_8: "valid input".to_owned(),
            field_9: "valid input".to_owned(),
            field_10: "valid input".to_owned(),
            field_11: "valid input".to_owned(),
            field_12: "valid input".to_owned(),
            field_13: "valid input".to_owned(),
            field_14: "valid input".to_owned(),
            field_15: "valid input".to_owned(),
            field_16: "valid input".to_owned(),
            field_17: "valid input".to_owned(),
            field_18: "valid input".to_owned(),
            field_19: "valid input".to_owned(),
            field_20: "valid input".to_owned(),
            field_21: "valid input".to_owned(),
            field_22: "valid input".to_owned(),
            field_23: "valid input".to_owned(),
            field_24: "valid input".to_owned(),
            field_25: "valid input".to_owned(),
            field_26: "valid input".to_owned(),
            field_27: "valid input".to_owned(),
            field_28: "valid input".to_owned(),
            field_29: "valid input".to_owned(),
            field_30: "valid input".to_owned(),
            field_31: "valid input".to_owned(),
        }
    }
}

#[ModelImpl]
impl Pipeline32 {
    pub fn field_0(&self) -> &str {
        &self.field_0
    }
}

#[ModelImpl]
impl Pipeline32 {
    pub fn field_1(&self) -> &str {
        &self.field_1
    }
}

#[ModelImpl]
impl Pipeline32 {
    pub fn field_2(&self) -> &str {
        &self.field_2
    }
}

#[ModelImpl]
impl Pipeline32 {
    pub fn field_3(&self) -> &str {
        &self.field_3
    }
}

#[ModelImpl]
impl Pipeline32 {
    pub fn field_4(&self) -> &str {
        &self.field_4
    }
}

#[ModelImpl]
impl Pipeline32 {
    pub fn field_5(&self) -> &str {
        &self.field_5
    }
}

#[ModelImpl]
impl Pipeline32 {
    pub fn field_6(&self) -> &str {
        &self.field_6
    }
}

#[ModelImpl]
impl Pipeline32 {
    pub fn field_7(&self) -> &str {
        &self.field_7
    }
}

#[ModelImpl]
impl Pipeline32 {
    pub fn field_8(&self) -> &str {
        &self.field_8
    }
}

#[ModelImpl]
impl Pipeline32 {
    pub fn field_9(&self) -> &str {
        &self.field_9
    }
}

#[ModelImpl]
impl Pipeline32 {
    pub fn field_10(&self) -> &str {
        &self.field_10
    }
}

#[ModelImpl]
impl Pipeline32 {
    pub fn field_11(&self) -> &str {
        &self.field_11
    }
}

#[ModelImpl]
impl Pipeline32 {
    pub fn field_12(&self) -> &str {
        &self.field_12
    }
}

#[ModelImpl]
impl Pipeline32 {
    pub fn field_13(&self) -> &str {
        &self.field_13
    }
}

#[ModelImpl]
impl Pipeline32 {
    pub fn field_14(&self) -> &str {
        &self.field_14
    }
}

#[ModelImpl]
impl Pipeline32 {
    pub fn field_15(&self) -> &str {
        &self.field_15
    }
}

#[ModelImpl]
impl Pipeline32 {
    pub fn field_16(&self) -> &str {
        &self.field_16
    }
}

#[ModelImpl]
impl Pipeline32 {
    pub fn field_17(&self) -> &str {
        &self.field_17
    }
}

#[ModelImpl]
impl Pipeline32 {
    pub fn field_18(&self) -> &str {
        &self.field_18
    }
}

#[ModelImpl]
impl Pipeline32 {
    pub fn field_19(&self) -> &str {
        &self.field_19
    }
}

#[ModelImpl]
impl Pipeline32 {
    pub fn field_20(&self) -> &str {
        &self.field_20
    }
}

#[ModelImpl]
impl Pipeline32 {
    pub fn field_21(&self) -> &str {
        &self.field_21
    }
}

#[ModelImpl]
impl Pipeline32 {
    pub fn field_22(&self) -> &str {
        &self.field_22
    }
}

#[ModelImpl]
impl Pipeline32 {
    pub fn field_23(&self) -> &str {
        &self.field_23
    }
}

#[ModelImpl]
impl Pipeline32 {
    pub fn field_24(&self) -> &str {
        &self.field_24
    }
}

#[ModelImpl]
impl Pipeline32 {
    pub fn field_25(&self) -> &str {
        &self.field_25
    }
}

#[ModelImpl]
impl Pipeline32 {
    pub fn field_26(&self) -> &str {
        &self.field_26
    }
}

#[ModelImpl]
impl Pipeline32 {
    pub fn field_27(&self) -> &str {
        &self.field_27
    }
}

#[ModelImpl]
impl Pipeline32 {
    pub fn field_28(&self) -> &str {
        &self.field_28
    }
}

#[ModelImpl]
impl Pipeline32 {
    pub fn field_29(&self) -> &str {
        &self.field_29
    }
}

#[ModelImpl]
impl Pipeline32 {
    pub fn field_30(&self) -> &str {
        &self.field_30
    }
}

#[ModelImpl]
impl Pipeline32 {
    pub fn field_31(&self) -> &str {
        &self.field_31
    }
}
