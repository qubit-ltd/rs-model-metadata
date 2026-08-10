// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
//    You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
//    Unless required by applicable law or agreed to in writing, software
//    distributed under the License is distributed on an "AS IS" BASIS,
//    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//    See the License for the specific language governing permissions and
//    limitations under the License.
// =============================================================================

//! Integration tests for explicit direct-reference graph validation.

use qubit_model_metadata::AttributeMetadata;
use qubit_model_metadata::FieldMetadata;
use qubit_model_metadata::FieldPath;
use qubit_model_metadata::HasTypeMetadata;
use qubit_model_metadata::HasTypeShape;
use qubit_model_metadata::ModelGraphError;
use qubit_model_metadata::ModelId;
use qubit_model_metadata::ModelRegistration;
use qubit_model_metadata::ModelRegistry;
use qubit_model_metadata::NamedTypeRef;
use qubit_model_metadata::ReferenceMetadata;
use qubit_model_metadata::SourceLocation;
use qubit_model_metadata::StructMetadata;
use qubit_model_metadata::TypeCapabilities;
use qubit_model_metadata::TypeIdentity;
use qubit_model_metadata::TypeKind;
use qubit_model_metadata::TypeMetadata;
use qubit_model_metadata::TypeRef;
use qubit_model_metadata::TypeShape;

struct Target;
struct MissingTargetSource;
struct MissingTargetFieldSource;
struct IncompatibleProjectionSource;
struct InvalidSameAsSource;
struct CycleA;
struct CycleB;
struct SelfCycle;
struct OptionalCycleA;
struct OptionalCycleB;
struct NestedInfo;
struct NestedTarget;
struct NestedSource;

static TARGET_FIELDS: [FieldMetadata; 1] = [FieldMetadata::new(
    0,
    "id",
    "i64",
    TypeRef::of::<i64>(),
    &[],
)];
static TARGET_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::from_static("test.graph.Target"),
    TypeIdentity::of::<Target>(),
    TypeKind::Struct(StructMetadata::new(&TARGET_FIELDS)),
    &[],
);

static MISSING_TARGET_ATTRIBUTES: [AttributeMetadata; 1] =
    [AttributeMetadata::Reference(ReferenceMetadata::new(
        ModelId::from_static("test.graph.Missing"),
        FieldPath::new(&["id"]),
        true,
        Some(FieldPath::new(&["unknown"])),
    ))];
static MISSING_TARGET_FIELDS: [FieldMetadata; 1] = [FieldMetadata::new(
    0,
    "target_id",
    "i64",
    TypeRef::of::<i64>(),
    &MISSING_TARGET_ATTRIBUTES,
)];
static MISSING_TARGET_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::from_static("test.graph.MissingTargetSource"),
    TypeIdentity::of::<MissingTargetSource>(),
    TypeKind::Struct(StructMetadata::new(&MISSING_TARGET_FIELDS)),
    &[],
);

static MISSING_TARGET_FIELD_ATTRIBUTES: [AttributeMetadata; 1] =
    [AttributeMetadata::Reference(ReferenceMetadata::new(
        ModelId::from_static("test.graph.Target"),
        FieldPath::new(&["unknown"]),
        true,
        None,
    ))];
static MISSING_TARGET_FIELD_FIELDS: [FieldMetadata; 1] = [FieldMetadata::new(
    0,
    "target_id",
    "i64",
    TypeRef::of::<i64>(),
    &MISSING_TARGET_FIELD_ATTRIBUTES,
)];
static MISSING_TARGET_FIELD_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::from_static("test.graph.MissingTargetFieldSource"),
    TypeIdentity::of::<MissingTargetFieldSource>(),
    TypeKind::Struct(StructMetadata::new(&MISSING_TARGET_FIELD_FIELDS)),
    &[],
);

static INCOMPATIBLE_PROJECTION_ATTRIBUTES: [AttributeMetadata; 1] =
    [AttributeMetadata::Reference(ReferenceMetadata::new(
        ModelId::from_static("test.graph.Target"),
        FieldPath::new(&["id"]),
        true,
        None,
    ))];
static INCOMPATIBLE_PROJECTION_FIELDS: [FieldMetadata; 1] =
    [FieldMetadata::new(
        0,
        "target_id",
        "String",
        TypeRef::of::<String>(),
        &INCOMPATIBLE_PROJECTION_ATTRIBUTES,
    )];
static INCOMPATIBLE_PROJECTION_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::from_static("test.graph.IncompatibleProjectionSource"),
    TypeIdentity::of::<IncompatibleProjectionSource>(),
    TypeKind::Struct(StructMetadata::new(&INCOMPATIBLE_PROJECTION_FIELDS)),
    &[],
);

static INVALID_SAME_AS_ATTRIBUTES: [AttributeMetadata; 1] =
    [AttributeMetadata::Reference(ReferenceMetadata::new(
        ModelId::from_static("test.graph.Target"),
        FieldPath::new(&["id"]),
        true,
        Some(FieldPath::new(&["unknown"])),
    ))];
static INVALID_SAME_AS_FIELDS: [FieldMetadata; 1] = [FieldMetadata::new(
    0,
    "target_id",
    "i64",
    TypeRef::of::<i64>(),
    &INVALID_SAME_AS_ATTRIBUTES,
)];
static INVALID_SAME_AS_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::from_static("test.graph.InvalidSameAsSource"),
    TypeIdentity::of::<InvalidSameAsSource>(),
    TypeKind::Struct(StructMetadata::new(&INVALID_SAME_AS_FIELDS)),
    &[],
);

static CYCLE_A_ATTRIBUTES: [AttributeMetadata; 1] =
    [AttributeMetadata::Reference(ReferenceMetadata::new(
        ModelId::from_static("test.graph.CycleB"),
        FieldPath::new(&["id"]),
        true,
        None,
    ))];
static CYCLE_A_FIELDS: [FieldMetadata; 2] = [
    FieldMetadata::new(0, "id", "i64", TypeRef::of::<i64>(), &[]),
    FieldMetadata::new(
        1,
        "cycle_b_id",
        "i64",
        TypeRef::of::<i64>(),
        &CYCLE_A_ATTRIBUTES,
    ),
];
static CYCLE_A_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::from_static("test.graph.CycleA"),
    TypeIdentity::of::<CycleA>(),
    TypeKind::Struct(StructMetadata::new(&CYCLE_A_FIELDS)),
    &[],
);

static CYCLE_B_ATTRIBUTES: [AttributeMetadata; 1] =
    [AttributeMetadata::Reference(ReferenceMetadata::new(
        ModelId::from_static("test.graph.CycleA"),
        FieldPath::new(&["id"]),
        true,
        None,
    ))];
static CYCLE_B_FIELDS: [FieldMetadata; 2] = [
    FieldMetadata::new(0, "id", "i64", TypeRef::of::<i64>(), &[]),
    FieldMetadata::new(
        1,
        "cycle_a_id",
        "i64",
        TypeRef::of::<i64>(),
        &CYCLE_B_ATTRIBUTES,
    ),
];
static CYCLE_B_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::from_static("test.graph.CycleB"),
    TypeIdentity::of::<CycleB>(),
    TypeKind::Struct(StructMetadata::new(&CYCLE_B_FIELDS)),
    &[],
);

static SELF_CYCLE_ATTRIBUTES: [AttributeMetadata; 1] =
    [AttributeMetadata::Reference(ReferenceMetadata::new(
        ModelId::from_static("test.graph.SelfCycle"),
        FieldPath::new(&["id"]),
        true,
        None,
    ))];
static SELF_CYCLE_FIELDS: [FieldMetadata; 2] = [
    FieldMetadata::new(0, "id", "i64", TypeRef::of::<i64>(), &[]),
    FieldMetadata::new(
        1,
        "parent_id",
        "i64",
        TypeRef::of::<i64>(),
        &SELF_CYCLE_ATTRIBUTES,
    ),
];
static SELF_CYCLE_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::from_static("test.graph.SelfCycle"),
    TypeIdentity::of::<SelfCycle>(),
    TypeKind::Struct(StructMetadata::new(&SELF_CYCLE_FIELDS)),
    &[],
);

static OPTIONAL_CYCLE_A_ATTRIBUTES: [AttributeMetadata; 1] =
    [AttributeMetadata::Reference(ReferenceMetadata::new(
        ModelId::from_static("test.graph.OptionalCycleB"),
        FieldPath::new(&["id"]),
        true,
        None,
    ))];
static OPTIONAL_CYCLE_A_FIELDS: [FieldMetadata; 2] = [
    FieldMetadata::new(0, "id", "i64", TypeRef::of::<i64>(), &[]),
    FieldMetadata::new(
        1,
        "cycle_b_id",
        "Option<i64>",
        TypeRef::of::<Option<i64>>(),
        &OPTIONAL_CYCLE_A_ATTRIBUTES,
    ),
];
static OPTIONAL_CYCLE_A_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::from_static("test.graph.OptionalCycleA"),
    TypeIdentity::of::<OptionalCycleA>(),
    TypeKind::Struct(StructMetadata::new(&OPTIONAL_CYCLE_A_FIELDS)),
    &[],
);

static OPTIONAL_CYCLE_B_ATTRIBUTES: [AttributeMetadata; 1] =
    [AttributeMetadata::Reference(ReferenceMetadata::new(
        ModelId::from_static("test.graph.OptionalCycleA"),
        FieldPath::new(&["id"]),
        true,
        None,
    ))];
static OPTIONAL_CYCLE_B_FIELDS: [FieldMetadata; 2] = [
    FieldMetadata::new(0, "id", "i64", TypeRef::of::<i64>(), &[]),
    FieldMetadata::new(
        1,
        "cycle_a_id",
        "i64",
        TypeRef::of::<i64>(),
        &OPTIONAL_CYCLE_B_ATTRIBUTES,
    ),
];
static OPTIONAL_CYCLE_B_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::from_static("test.graph.OptionalCycleB"),
    TypeIdentity::of::<OptionalCycleB>(),
    TypeKind::Struct(StructMetadata::new(&OPTIONAL_CYCLE_B_FIELDS)),
    &[],
);

static NESTED_INFO_FIELDS: [FieldMetadata; 1] = [FieldMetadata::new(
    0,
    "id",
    "i64",
    TypeRef::of::<i64>(),
    &[],
)];
static NESTED_INFO_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::from_static("test.graph.NestedInfo"),
    TypeIdentity::of::<NestedInfo>(),
    TypeKind::Struct(StructMetadata::new(&NESTED_INFO_FIELDS)),
    &[],
);
static NESTED_TARGET_FIELDS: [FieldMetadata; 1] = [FieldMetadata::new(
    0,
    "info",
    "NestedInfo",
    TypeRef::of::<NestedInfo>(),
    &[],
)];
static NESTED_TARGET_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::from_static("test.graph.NestedTarget"),
    TypeIdentity::of::<NestedTarget>(),
    TypeKind::Struct(StructMetadata::new(&NESTED_TARGET_FIELDS)),
    &[],
);
static NESTED_SOURCE_ATTRIBUTES: [AttributeMetadata; 1] =
    [AttributeMetadata::Reference(ReferenceMetadata::new(
        ModelId::from_static("test.graph.NestedTarget"),
        FieldPath::new(&["info", "id"]),
        true,
        None,
    ))];
static NESTED_SOURCE_FIELDS: [FieldMetadata; 1] = [FieldMetadata::new(
    0,
    "nested_id",
    "Option<i64>",
    TypeRef::of::<Option<i64>>(),
    &NESTED_SOURCE_ATTRIBUTES,
)];
static NESTED_SOURCE_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::from_static("test.graph.NestedSource"),
    TypeIdentity::of::<NestedSource>(),
    TypeKind::Struct(StructMetadata::new(&NESTED_SOURCE_FIELDS)),
    &[],
);

static TARGET_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::from_static("test.graph.Target"),
    &TARGET_METADATA,
    "Target",
    "test::graph",
    SourceLocation::new("model_graph_tests.rs", 1, 1),
);
static MISSING_TARGET_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::from_static("test.graph.MissingTargetSource"),
    &MISSING_TARGET_METADATA,
    "MissingTargetSource",
    "test::graph",
    SourceLocation::new("model_graph_tests.rs", 2, 1),
);
static MISSING_TARGET_FIELD_REGISTRATION: ModelRegistration =
    ModelRegistration::new(
        ModelId::from_static("test.graph.MissingTargetFieldSource"),
        &MISSING_TARGET_FIELD_METADATA,
        "MissingTargetFieldSource",
        "test::graph",
        SourceLocation::new("model_graph_tests.rs", 3, 1),
    );
static INCOMPATIBLE_PROJECTION_REGISTRATION: ModelRegistration =
    ModelRegistration::new(
        ModelId::from_static("test.graph.IncompatibleProjectionSource"),
        &INCOMPATIBLE_PROJECTION_METADATA,
        "IncompatibleProjectionSource",
        "test::graph",
        SourceLocation::new("model_graph_tests.rs", 4, 1),
    );
static INVALID_SAME_AS_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::from_static("test.graph.InvalidSameAsSource"),
    &INVALID_SAME_AS_METADATA,
    "InvalidSameAsSource",
    "test::graph",
    SourceLocation::new("model_graph_tests.rs", 5, 1),
);
static CYCLE_A_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::from_static("test.graph.CycleA"),
    &CYCLE_A_METADATA,
    "CycleA",
    "test::graph",
    SourceLocation::new("model_graph_tests.rs", 6, 1),
);
static CYCLE_B_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::from_static("test.graph.CycleB"),
    &CYCLE_B_METADATA,
    "CycleB",
    "test::graph",
    SourceLocation::new("model_graph_tests.rs", 7, 1),
);
static SELF_CYCLE_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::from_static("test.graph.SelfCycle"),
    &SELF_CYCLE_METADATA,
    "SelfCycle",
    "test::graph",
    SourceLocation::new("model_graph_tests.rs", 8, 1),
);
static OPTIONAL_CYCLE_A_REGISTRATION: ModelRegistration =
    ModelRegistration::new(
        ModelId::from_static("test.graph.OptionalCycleA"),
        &OPTIONAL_CYCLE_A_METADATA,
        "OptionalCycleA",
        "test::graph",
        SourceLocation::new("model_graph_tests.rs", 9, 1),
    );
static OPTIONAL_CYCLE_B_REGISTRATION: ModelRegistration =
    ModelRegistration::new(
        ModelId::from_static("test.graph.OptionalCycleB"),
        &OPTIONAL_CYCLE_B_METADATA,
        "OptionalCycleB",
        "test::graph",
        SourceLocation::new("model_graph_tests.rs", 10, 1),
    );
static NESTED_TARGET_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::from_static("test.graph.NestedTarget"),
    &NESTED_TARGET_METADATA,
    "NestedTarget",
    "test::graph",
    SourceLocation::new("model_graph_tests.rs", 11, 1),
);
static NESTED_SOURCE_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::from_static("test.graph.NestedSource"),
    &NESTED_SOURCE_METADATA,
    "NestedSource",
    "test::graph",
    SourceLocation::new("model_graph_tests.rs", 12, 1),
);

impl HasTypeShape for NestedInfo {
    const TYPE_SHAPE: TypeShape = TypeShape::Named(NamedTypeRef::of::<Self>());
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::NONE;
}

impl HasTypeMetadata for NestedInfo {
    fn type_metadata() -> &'static TypeMetadata {
        &NESTED_INFO_METADATA
    }
}

#[test]
fn test_from_registrations_allows_partial_reference_graphs() {
    ModelRegistry::from_registrations([&MISSING_TARGET_REGISTRATION])
        .expect("registry construction must not validate the reference graph");
}

#[test]
fn test_validate_graph_aggregates_direct_reference_errors_and_required_cycles()
{
    let registrations = [
        &TARGET_REGISTRATION,
        &MISSING_TARGET_REGISTRATION,
        &MISSING_TARGET_FIELD_REGISTRATION,
        &INCOMPATIBLE_PROJECTION_REGISTRATION,
        &INVALID_SAME_AS_REGISTRATION,
        &CYCLE_A_REGISTRATION,
        &CYCLE_B_REGISTRATION,
    ];
    let registry = ModelRegistry::from_registrations(registrations)
        .expect("the registrations should be valid and unique");
    let expected = [
        ModelGraphError::RequiredReferenceCycle {
            cycle: vec![
                ModelId::from_static("test.graph.CycleA"),
                ModelId::from_static("test.graph.CycleB"),
                ModelId::from_static("test.graph.CycleA"),
            ],
        },
        ModelGraphError::IncompatibleProjection {
            source: ModelId::from_static(
                "test.graph.IncompatibleProjectionSource",
            ),
            field: "target_id",
            source_type: "alloc::string::String",
            target: ModelId::from_static("test.graph.Target"),
            target_field: FieldPath::new(&["id"]),
            target_type: "i64",
        },
        ModelGraphError::InvalidSameAs {
            source: ModelId::from_static("test.graph.InvalidSameAsSource"),
            field: "target_id",
            same_as: FieldPath::new(&["unknown"]),
        },
        ModelGraphError::MissingTargetField {
            source: ModelId::from_static("test.graph.MissingTargetFieldSource"),
            field: "target_id",
            target: ModelId::from_static("test.graph.Target"),
            target_field: FieldPath::new(&["unknown"]),
        },
        ModelGraphError::MissingTarget {
            source: ModelId::from_static("test.graph.MissingTargetSource"),
            field: "target_id",
            target: ModelId::from_static("test.graph.Missing"),
        },
        ModelGraphError::InvalidSameAs {
            source: ModelId::from_static("test.graph.MissingTargetSource"),
            field: "target_id",
            same_as: FieldPath::new(&["unknown"]),
        },
    ];

    let errors = registry
        .validate_graph()
        .expect_err("the reference graph should be invalid");
    assert_eq!(errors.errors(), expected);

    let reversed_registry = ModelRegistry::from_registrations(registrations.into_iter().rev())
        .expect("the registrations should be valid and unique regardless of input order");
    let reversed_errors = reversed_registry
        .validate_graph()
        .expect_err("the reference graph should be invalid");
    assert_eq!(reversed_errors.errors(), expected);
}

#[test]
fn test_validate_graph_reports_a_canonical_self_cycle() {
    let registry =
        ModelRegistry::from_registrations([&SELF_CYCLE_REGISTRATION])
            .expect("the self-referencing registration should be valid");

    let errors = registry
        .validate_graph()
        .expect_err("the required self-reference should be a cycle");

    assert_eq!(
        errors.errors(),
        [ModelGraphError::RequiredReferenceCycle {
            cycle: vec![
                ModelId::from_static("test.graph.SelfCycle"),
                ModelId::from_static("test.graph.SelfCycle"),
            ],
        }],
    );
}

#[test]
fn test_validate_graph_ignores_optional_required_references_for_cycle_detection()
 {
    let registry = ModelRegistry::from_registrations([
        &OPTIONAL_CYCLE_A_REGISTRATION,
        &OPTIONAL_CYCLE_B_REGISTRATION,
    ])
    .expect("the optional-cycle registrations should be valid");

    registry
        .validate_graph()
        .expect("an optional source reference must not form a required cycle");
}

#[test]
fn test_validate_graph_resolves_nested_target_paths_and_optional_projections() {
    let registry = ModelRegistry::from_registrations([
        &NESTED_SOURCE_REGISTRATION,
        &NESTED_TARGET_REGISTRATION,
    ])
    .expect("the nested-path registrations should be valid");

    registry
        .validate_graph()
        .expect("the nested target path should resolve through named metadata");
}
