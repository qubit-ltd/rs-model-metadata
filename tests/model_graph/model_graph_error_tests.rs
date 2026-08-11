// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Coverage for `ModelGraphError` remains in the model-graph integration tests.

use qubit_model_metadata::FieldPath;
use qubit_model_metadata::ModelGraphError;
use qubit_model_metadata::ModelId;

#[test]
fn test_model_graph_errors_describe_each_variant() {
    let source = ModelId::new("test.Source");
    let target = ModelId::new("test.Target");
    let cases = [
        (
            ModelGraphError::MissingTarget {
                source,
                field: "target_id",
                target,
            },
            "reference test.Source.target_id targets missing model test.Target",
        ),
        (
            ModelGraphError::MissingTargetField {
                source,
                field: "target_id",
                target,
                target_field: FieldPath::new(&["nested", "id"]),
            },
            "reference test.Source.target_id targets missing field nested.id on test.Target",
        ),
        (
            ModelGraphError::IncompatibleProjection {
                source,
                field: "target_id",
                source_type: "String",
                target,
                target_field: FieldPath::new(&["nested", "id"]),
                target_type: "i64",
            },
            "reference test.Source.target_id projects String, but nested.id on test.Target has type i64",
        ),
        (
            ModelGraphError::InvalidSameAs {
                source,
                field: "target_id",
                same_as: FieldPath::new(&["account", "id"]),
            },
            "reference test.Source.target_id has invalid same_as path account.id",
        ),
        (
            ModelGraphError::RequiredReferenceCycle {
                cycle: vec![source, target, source],
            },
            "required reference cycle test.Source test.Target test.Source",
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
    }
}
