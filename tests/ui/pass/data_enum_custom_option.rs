// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

mod custom {
    #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Deserialize, serde::Serialize)]
    pub struct Option<T>(pub T);
}

#[qubit_model_derive::Enum(id = "test.derive.CustomOption")]
enum CustomOption {
    Value {
        #[opaque]
        value: custom::Option<String>,
    },
}

fn main() {
    let value = CustomOption::Value {
        value: custom::Option("value".to_owned()),
    };
    let _ = serde_json::to_string(&value).expect("custom option payload should serialize");
}
