// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Integration tests for explicit direct-reference graph validation.

mod model_graph;

use qubit_model_metadata::AttributeMetadata;
use qubit_model_metadata::FieldMetadata;
use qubit_model_metadata::FieldPath;
use qubit_model_metadata::HasTypeMetadata;
use qubit_model_metadata::HasTypeShape;
use qubit_model_metadata::LookupRelationMetadata;
use qubit_model_metadata::ModelGraphError;
use qubit_model_metadata::ModelId;
use qubit_model_metadata::ModelRegistration;
use qubit_model_metadata::ModelRegistry;
use qubit_model_metadata::NamedTypeRef;
use qubit_model_metadata::OwnershipMetadata;
use qubit_model_metadata::ReferenceMetadata;
use qubit_model_metadata::ReferencePath;
use qubit_model_metadata::ReferencePathSegment;
use qubit_model_metadata::ReferenceTarget;
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
struct DirectTargetSource;
struct InfoProjectionSource;
struct InvalidReferencePathSource;
struct CycleA;
struct CycleB;
struct SelfCycle;
struct VecCycle;
struct OptionalCycleA;
struct OptionalCycleB;
struct NestedInfo;
struct NestedTarget;
struct NestedSource;
struct LookupSource;
struct OwnedModel;
struct OwnershipCycleA;
struct OwnershipCycleB;
struct Country;
struct Province;
struct AddressPath;

static TARGET_FIELDS: [FieldMetadata; 1] = [FieldMetadata::new(0, "id", "i64", TypeRef::of::<i64>(), &[])];
static TARGET_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::new("test.graph.Target"),
    TypeIdentity::of::<Target>(),
    TypeKind::Struct(StructMetadata::new(&TARGET_FIELDS)),
    &[],
);

static MISSING_TARGET_ATTRIBUTES: [AttributeMetadata; 1] = [AttributeMetadata::Reference(ReferenceMetadata::new(
    ModelId::new("test.graph.Missing"),
    ReferenceTarget::Property(FieldPath::new(&["id"])),
    true,
    Some(ReferencePath::new(&[ReferencePathSegment::Field("unknown")])),
))];
static MISSING_TARGET_FIELDS: [FieldMetadata; 1] = [FieldMetadata::new(
    0,
    "target_id",
    "i64",
    TypeRef::of::<i64>(),
    &MISSING_TARGET_ATTRIBUTES,
)];
static MISSING_TARGET_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::new("test.graph.MissingTargetSource"),
    TypeIdentity::of::<MissingTargetSource>(),
    TypeKind::Struct(StructMetadata::new(&MISSING_TARGET_FIELDS)),
    &[],
);

static MISSING_TARGET_FIELD_ATTRIBUTES: [AttributeMetadata; 1] =
    [AttributeMetadata::Reference(ReferenceMetadata::new(
        ModelId::new("test.graph.Target"),
        ReferenceTarget::Property(FieldPath::new(&["unknown"])),
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
    ModelId::new("test.graph.MissingTargetFieldSource"),
    TypeIdentity::of::<MissingTargetFieldSource>(),
    TypeKind::Struct(StructMetadata::new(&MISSING_TARGET_FIELD_FIELDS)),
    &[],
);

static INCOMPATIBLE_PROJECTION_ATTRIBUTES: [AttributeMetadata; 1] =
    [AttributeMetadata::Reference(ReferenceMetadata::new(
        ModelId::new("test.graph.Target"),
        ReferenceTarget::Property(FieldPath::new(&["id"])),
        true,
        None,
    ))];
static INCOMPATIBLE_PROJECTION_FIELDS: [FieldMetadata; 1] = [FieldMetadata::new(
    0,
    "target_id",
    "String",
    TypeRef::of::<String>(),
    &INCOMPATIBLE_PROJECTION_ATTRIBUTES,
)];
static INCOMPATIBLE_PROJECTION_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::new("test.graph.IncompatibleProjectionSource"),
    TypeIdentity::of::<IncompatibleProjectionSource>(),
    TypeKind::Struct(StructMetadata::new(&INCOMPATIBLE_PROJECTION_FIELDS)),
    &[],
);

static DIRECT_TARGET_ATTRIBUTES: [AttributeMetadata; 1] = [AttributeMetadata::Reference(ReferenceMetadata::new(
    ModelId::new("test.graph.Target"),
    ReferenceTarget::Property(FieldPath::new(&["id"])),
    true,
    None,
))];
static DIRECT_TARGET_FIELDS: [FieldMetadata; 1] = [FieldMetadata::new(
    0,
    "target",
    "Target",
    TypeRef::of::<Target>(),
    &DIRECT_TARGET_ATTRIBUTES,
)];
static DIRECT_TARGET_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::new("test.graph.DirectTargetSource"),
    TypeIdentity::of::<DirectTargetSource>(),
    TypeKind::Struct(StructMetadata::new(&DIRECT_TARGET_FIELDS)),
    &[],
);

static INFO_PROJECTION_ATTRIBUTES: [AttributeMetadata; 1] = [AttributeMetadata::Reference(ReferenceMetadata::new(
    ModelId::new("test.graph.Target"),
    ReferenceTarget::Property(FieldPath::new(&["info"])),
    true,
    None,
))];
static INFO_PROJECTION_FIELDS: [FieldMetadata; 1] = [FieldMetadata::new(
    0,
    "target_info",
    "String",
    TypeRef::of::<String>(),
    &INFO_PROJECTION_ATTRIBUTES,
)];
static INFO_PROJECTION_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::new("test.graph.InfoProjectionSource"),
    TypeIdentity::of::<InfoProjectionSource>(),
    TypeKind::Struct(StructMetadata::new(&INFO_PROJECTION_FIELDS)),
    &[],
);

static INVALID_REFERENCE_PATH_ATTRIBUTES: [AttributeMetadata; 1] =
    [AttributeMetadata::Reference(ReferenceMetadata::new(
        ModelId::new("test.graph.Target"),
        ReferenceTarget::Property(FieldPath::new(&["id"])),
        true,
        Some(ReferencePath::new(&[ReferencePathSegment::Field("unknown")])),
    ))];
static INVALID_REFERENCE_PATH_FIELDS: [FieldMetadata; 1] = [FieldMetadata::new(
    0,
    "target_id",
    "i64",
    TypeRef::of::<i64>(),
    &INVALID_REFERENCE_PATH_ATTRIBUTES,
)];
static INVALID_REFERENCE_PATH_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::new("test.graph.InvalidReferencePathSource"),
    TypeIdentity::of::<InvalidReferencePathSource>(),
    TypeKind::Struct(StructMetadata::new(&INVALID_REFERENCE_PATH_FIELDS)),
    &[],
);

static CYCLE_A_ATTRIBUTES: [AttributeMetadata; 1] = [AttributeMetadata::Reference(ReferenceMetadata::new(
    ModelId::new("test.graph.CycleB"),
    ReferenceTarget::Property(FieldPath::new(&["id"])),
    true,
    None,
))];
static CYCLE_A_FIELDS: [FieldMetadata; 2] = [
    FieldMetadata::new(0, "id", "i64", TypeRef::of::<i64>(), &[]),
    FieldMetadata::new(1, "cycle_b_id", "i64", TypeRef::of::<i64>(), &CYCLE_A_ATTRIBUTES),
];
static CYCLE_A_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::new("test.graph.CycleA"),
    TypeIdentity::of::<CycleA>(),
    TypeKind::Struct(StructMetadata::new(&CYCLE_A_FIELDS)),
    &[],
);

static CYCLE_B_ATTRIBUTES: [AttributeMetadata; 1] = [AttributeMetadata::Reference(ReferenceMetadata::new(
    ModelId::new("test.graph.CycleA"),
    ReferenceTarget::Property(FieldPath::new(&["id"])),
    true,
    None,
))];
static CYCLE_B_FIELDS: [FieldMetadata; 2] = [
    FieldMetadata::new(0, "id", "i64", TypeRef::of::<i64>(), &[]),
    FieldMetadata::new(1, "cycle_a_id", "i64", TypeRef::of::<i64>(), &CYCLE_B_ATTRIBUTES),
];
static CYCLE_B_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::new("test.graph.CycleB"),
    TypeIdentity::of::<CycleB>(),
    TypeKind::Struct(StructMetadata::new(&CYCLE_B_FIELDS)),
    &[],
);

static SELF_CYCLE_ATTRIBUTES: [AttributeMetadata; 1] = [AttributeMetadata::Reference(ReferenceMetadata::new(
    ModelId::new("test.graph.SelfCycle"),
    ReferenceTarget::Property(FieldPath::new(&["id"])),
    true,
    None,
))];
static SELF_CYCLE_FIELDS: [FieldMetadata; 2] = [
    FieldMetadata::new(0, "id", "i64", TypeRef::of::<i64>(), &[]),
    FieldMetadata::new(1, "parent_id", "i64", TypeRef::of::<i64>(), &SELF_CYCLE_ATTRIBUTES),
];
static SELF_CYCLE_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::new("test.graph.SelfCycle"),
    TypeIdentity::of::<SelfCycle>(),
    TypeKind::Struct(StructMetadata::new(&SELF_CYCLE_FIELDS)),
    &[],
);

static VEC_CYCLE_ATTRIBUTES: [AttributeMetadata; 1] = [AttributeMetadata::Reference(ReferenceMetadata::new(
    ModelId::new("test.graph.VecCycle"),
    ReferenceTarget::Property(FieldPath::new(&["id"])),
    true,
    None,
))];
static VEC_CYCLE_FIELDS: [FieldMetadata; 2] = [
    FieldMetadata::new(0, "id", "i64", TypeRef::of::<i64>(), &[]),
    FieldMetadata::new(
        1,
        "related_ids",
        "Vec<i64>",
        TypeRef::of::<Vec<i64>>(),
        &VEC_CYCLE_ATTRIBUTES,
    ),
];
static VEC_CYCLE_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::new("test.graph.VecCycle"),
    TypeIdentity::of::<VecCycle>(),
    TypeKind::Struct(StructMetadata::new(&VEC_CYCLE_FIELDS)),
    &[],
);

static OPTIONAL_CYCLE_A_ATTRIBUTES: [AttributeMetadata; 1] = [AttributeMetadata::Reference(ReferenceMetadata::new(
    ModelId::new("test.graph.OptionalCycleB"),
    ReferenceTarget::Property(FieldPath::new(&["id"])),
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
    ModelId::new("test.graph.OptionalCycleA"),
    TypeIdentity::of::<OptionalCycleA>(),
    TypeKind::Struct(StructMetadata::new(&OPTIONAL_CYCLE_A_FIELDS)),
    &[],
);

static OPTIONAL_CYCLE_B_ATTRIBUTES: [AttributeMetadata; 1] = [AttributeMetadata::Reference(ReferenceMetadata::new(
    ModelId::new("test.graph.OptionalCycleA"),
    ReferenceTarget::Property(FieldPath::new(&["id"])),
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
    ModelId::new("test.graph.OptionalCycleB"),
    TypeIdentity::of::<OptionalCycleB>(),
    TypeKind::Struct(StructMetadata::new(&OPTIONAL_CYCLE_B_FIELDS)),
    &[],
);

static NESTED_INFO_FIELDS: [FieldMetadata; 1] = [FieldMetadata::new(0, "id", "i64", TypeRef::of::<i64>(), &[])];
static NESTED_INFO_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::new("test.graph.NestedInfo"),
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
    ModelId::new("test.graph.NestedTarget"),
    TypeIdentity::of::<NestedTarget>(),
    TypeKind::Struct(StructMetadata::new(&NESTED_TARGET_FIELDS)),
    &[],
);
static NESTED_SOURCE_ATTRIBUTES: [AttributeMetadata; 1] = [AttributeMetadata::Reference(ReferenceMetadata::new(
    ModelId::new("test.graph.NestedTarget"),
    ReferenceTarget::Property(FieldPath::new(&["info", "id"])),
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
    ModelId::new("test.graph.NestedSource"),
    TypeIdentity::of::<NestedSource>(),
    TypeKind::Struct(StructMetadata::new(&NESTED_SOURCE_FIELDS)),
    &[],
);

static COUNTRY_FIELDS: [FieldMetadata; 1] = [FieldMetadata::new(0, "info", "String", TypeRef::of::<String>(), &[])];
static COUNTRY_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::new("test.graph.Country"),
    TypeIdentity::of::<Country>(),
    TypeKind::Struct(StructMetadata::new(&COUNTRY_FIELDS)),
    &[],
);
static PROVINCE_COUNTRY_ATTRIBUTES: [AttributeMetadata; 1] = [AttributeMetadata::Reference(ReferenceMetadata::new(
    ModelId::new("test.graph.Country"),
    ReferenceTarget::Property(FieldPath::new(&["info"])),
    true,
    None,
))];
static PROVINCE_FIELDS: [FieldMetadata; 1] = [FieldMetadata::new(
    0,
    "country",
    "String",
    TypeRef::of::<String>(),
    &PROVINCE_COUNTRY_ATTRIBUTES,
)];
static PROVINCE_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::new("test.graph.Province"),
    TypeIdentity::of::<Province>(),
    TypeKind::Struct(StructMetadata::new(&PROVINCE_FIELDS)),
    &[],
);
static ADDRESS_COUNTRY_ATTRIBUTES: [AttributeMetadata; 1] = [AttributeMetadata::Reference(ReferenceMetadata::new(
    ModelId::new("test.graph.Country"),
    ReferenceTarget::Property(FieldPath::new(&["info"])),
    true,
    Some(ReferencePath::new(&[
        ReferencePathSegment::Field("province"),
        ReferencePathSegment::Field("country"),
    ])),
))];
static ADDRESS_PROVINCE_ATTRIBUTES: [AttributeMetadata; 1] = [AttributeMetadata::Reference(ReferenceMetadata::new(
    ModelId::new("test.graph.Province"),
    ReferenceTarget::Property(FieldPath::new(&["info"])),
    true,
    None,
))];
static ADDRESS_PATH_FIELDS: [FieldMetadata; 2] = [
    FieldMetadata::new(
        0,
        "country",
        "String",
        TypeRef::of::<String>(),
        &ADDRESS_COUNTRY_ATTRIBUTES,
    ),
    FieldMetadata::new(
        1,
        "province",
        "String",
        TypeRef::of::<String>(),
        &ADDRESS_PROVINCE_ATTRIBUTES,
    ),
];
static ADDRESS_PATH_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::new("test.graph.AddressPath"),
    TypeIdentity::of::<AddressPath>(),
    TypeKind::Struct(StructMetadata::new(&ADDRESS_PATH_FIELDS)),
    &[],
);
static LOOKUP_SOURCE_ATTRIBUTES: [AttributeMetadata; 1] = [AttributeMetadata::LookupRelation(
    LookupRelationMetadata::new(NamedTypeRef::of::<Target>(), FieldPath::new(&["id"])),
)];
static LOOKUP_SOURCE_FIELDS: [FieldMetadata; 1] = [FieldMetadata::new(
    0,
    "target_id",
    "i64",
    TypeRef::of::<i64>(),
    &LOOKUP_SOURCE_ATTRIBUTES,
)];
static LOOKUP_SOURCE_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::new("test.graph.LookupSource"),
    TypeIdentity::of::<LookupSource>(),
    TypeKind::Struct(StructMetadata::new(&LOOKUP_SOURCE_FIELDS)),
    &[],
);
static OWNED_MODEL_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::new("test.graph.OwnedModel"),
    TypeIdentity::of::<OwnedModel>(),
    TypeKind::Struct(StructMetadata::new(&[])),
    &[AttributeMetadata::Ownership(OwnershipMetadata::new(
        NamedTypeRef::of::<Target>(),
    ))],
);
static OWNERSHIP_CYCLE_A_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::new("test.graph.OwnershipCycleA"),
    TypeIdentity::of::<OwnershipCycleA>(),
    TypeKind::Struct(StructMetadata::new(&[])),
    &[AttributeMetadata::Ownership(OwnershipMetadata::new(
        NamedTypeRef::of::<OwnershipCycleB>(),
    ))],
);
static OWNERSHIP_CYCLE_B_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::new("test.graph.OwnershipCycleB"),
    TypeIdentity::of::<OwnershipCycleB>(),
    TypeKind::Struct(StructMetadata::new(&[])),
    &[AttributeMetadata::Ownership(OwnershipMetadata::new(
        NamedTypeRef::of::<OwnershipCycleA>(),
    ))],
);

static TARGET_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::new("test.graph.Target"),
    &TARGET_METADATA,
    "Target",
    "test::graph",
    SourceLocation::new("model_graph_tests.rs", 1, 1),
);
static MISSING_TARGET_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::new("test.graph.MissingTargetSource"),
    &MISSING_TARGET_METADATA,
    "MissingTargetSource",
    "test::graph",
    SourceLocation::new("model_graph_tests.rs", 2, 1),
);
static MISSING_TARGET_FIELD_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::new("test.graph.MissingTargetFieldSource"),
    &MISSING_TARGET_FIELD_METADATA,
    "MissingTargetFieldSource",
    "test::graph",
    SourceLocation::new("model_graph_tests.rs", 3, 1),
);
static INCOMPATIBLE_PROJECTION_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::new("test.graph.IncompatibleProjectionSource"),
    &INCOMPATIBLE_PROJECTION_METADATA,
    "IncompatibleProjectionSource",
    "test::graph",
    SourceLocation::new("model_graph_tests.rs", 4, 1),
);
static DIRECT_TARGET_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::new("test.graph.DirectTargetSource"),
    &DIRECT_TARGET_METADATA,
    "DirectTargetSource",
    "test::graph",
    SourceLocation::new("model_graph_tests.rs", 17, 1),
);
static INFO_PROJECTION_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::new("test.graph.InfoProjectionSource"),
    &INFO_PROJECTION_METADATA,
    "InfoProjectionSource",
    "test::graph",
    SourceLocation::new("model_graph_tests.rs", 18, 1),
);
static INVALID_REFERENCE_PATH_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::new("test.graph.InvalidReferencePathSource"),
    &INVALID_REFERENCE_PATH_METADATA,
    "InvalidReferencePathSource",
    "test::graph",
    SourceLocation::new("model_graph_tests.rs", 5, 1),
);
static CYCLE_A_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::new("test.graph.CycleA"),
    &CYCLE_A_METADATA,
    "CycleA",
    "test::graph",
    SourceLocation::new("model_graph_tests.rs", 6, 1),
);
static CYCLE_B_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::new("test.graph.CycleB"),
    &CYCLE_B_METADATA,
    "CycleB",
    "test::graph",
    SourceLocation::new("model_graph_tests.rs", 7, 1),
);
static SELF_CYCLE_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::new("test.graph.SelfCycle"),
    &SELF_CYCLE_METADATA,
    "SelfCycle",
    "test::graph",
    SourceLocation::new("model_graph_tests.rs", 8, 1),
);
static VEC_CYCLE_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::new("test.graph.VecCycle"),
    &VEC_CYCLE_METADATA,
    "test::VecCycle",
    "test::graph",
    SourceLocation::new("vec_cycle.rs", 1, 1),
);
static OPTIONAL_CYCLE_A_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::new("test.graph.OptionalCycleA"),
    &OPTIONAL_CYCLE_A_METADATA,
    "OptionalCycleA",
    "test::graph",
    SourceLocation::new("model_graph_tests.rs", 9, 1),
);
static OPTIONAL_CYCLE_B_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::new("test.graph.OptionalCycleB"),
    &OPTIONAL_CYCLE_B_METADATA,
    "OptionalCycleB",
    "test::graph",
    SourceLocation::new("model_graph_tests.rs", 10, 1),
);
static NESTED_TARGET_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::new("test.graph.NestedTarget"),
    &NESTED_TARGET_METADATA,
    "NestedTarget",
    "test::graph",
    SourceLocation::new("model_graph_tests.rs", 11, 1),
);
static NESTED_SOURCE_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::new("test.graph.NestedSource"),
    &NESTED_SOURCE_METADATA,
    "NestedSource",
    "test::graph",
    SourceLocation::new("model_graph_tests.rs", 12, 1),
);
static COUNTRY_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::new("test.graph.Country"),
    &COUNTRY_METADATA,
    "Country",
    "test::graph",
    SourceLocation::new("model_graph_tests.rs", 13, 1),
);
static PROVINCE_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::new("test.graph.Province"),
    &PROVINCE_METADATA,
    "Province",
    "test::graph",
    SourceLocation::new("model_graph_tests.rs", 14, 1),
);
static ADDRESS_PATH_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::new("test.graph.AddressPath"),
    &ADDRESS_PATH_METADATA,
    "AddressPath",
    "test::graph",
    SourceLocation::new("model_graph_tests.rs", 15, 1),
);
static LOOKUP_SOURCE_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::new("test.graph.LookupSource"),
    &LOOKUP_SOURCE_METADATA,
    "LookupSource",
    "test::graph",
    SourceLocation::new("model_graph_tests.rs", 13, 1),
);
static OWNED_MODEL_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::new("test.graph.OwnedModel"),
    &OWNED_MODEL_METADATA,
    "OwnedModel",
    "test::graph",
    SourceLocation::new("model_graph_tests.rs", 14, 1),
);
static OWNERSHIP_CYCLE_A_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::new("test.graph.OwnershipCycleA"),
    &OWNERSHIP_CYCLE_A_METADATA,
    "OwnershipCycleA",
    "test::graph",
    SourceLocation::new("model_graph_tests.rs", 15, 1),
);
static OWNERSHIP_CYCLE_B_REGISTRATION: ModelRegistration = ModelRegistration::new(
    ModelId::new("test.graph.OwnershipCycleB"),
    &OWNERSHIP_CYCLE_B_METADATA,
    "OwnershipCycleB",
    "test::graph",
    SourceLocation::new("model_graph_tests.rs", 16, 1),
);

impl HasTypeMetadata for Target {
    fn type_metadata() -> &'static TypeMetadata {
        &TARGET_METADATA
    }
}

impl HasTypeMetadata for OwnershipCycleA {
    fn type_metadata() -> &'static TypeMetadata {
        &OWNERSHIP_CYCLE_A_METADATA
    }
}

impl HasTypeMetadata for OwnershipCycleB {
    fn type_metadata() -> &'static TypeMetadata {
        &OWNERSHIP_CYCLE_B_METADATA
    }
}

impl HasTypeShape for NestedInfo {
    const TYPE_SHAPE: TypeShape = TypeShape::Named(NamedTypeRef::of::<Self>());
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::NONE;
}

impl HasTypeShape for Target {
    const TYPE_SHAPE: TypeShape = TypeShape::Named(NamedTypeRef::of::<Self>());
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::NONE;
}

impl HasTypeShape for OwnershipCycleA {
    const TYPE_SHAPE: TypeShape = TypeShape::Named(NamedTypeRef::of::<Self>());
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::NONE;
}

impl HasTypeShape for OwnershipCycleB {
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
fn test_validate_graph_aggregates_direct_reference_errors_and_required_cycles() {
    let registrations = [
        &TARGET_REGISTRATION,
        &MISSING_TARGET_REGISTRATION,
        &MISSING_TARGET_FIELD_REGISTRATION,
        &INCOMPATIBLE_PROJECTION_REGISTRATION,
        &INVALID_REFERENCE_PATH_REGISTRATION,
        &CYCLE_A_REGISTRATION,
        &CYCLE_B_REGISTRATION,
    ];
    let registry =
        ModelRegistry::from_registrations(registrations).expect("the registrations should be valid and unique");
    let expected = [
        ModelGraphError::RequiredReferenceCycle {
            cycle: vec![
                ModelId::new("test.graph.CycleA"),
                ModelId::new("test.graph.CycleB"),
                ModelId::new("test.graph.CycleA"),
            ],
        },
        ModelGraphError::IncompatibleProjection {
            source: ModelId::new("test.graph.IncompatibleProjectionSource"),
            field: "target_id",
            source_type: "alloc::string::String",
            target: ModelId::new("test.graph.Target"),
            target_field: FieldPath::new(&["id"]),
            target_type: "i64",
        },
        ModelGraphError::InvalidReferencePath {
            source: ModelId::new("test.graph.InvalidReferencePathSource"),
            field: "target_id",
            path: ReferencePath::new(&[ReferencePathSegment::Field("unknown")]),
        },
        ModelGraphError::MissingTargetField {
            source: ModelId::new("test.graph.MissingTargetFieldSource"),
            field: "target_id",
            target: ModelId::new("test.graph.Target"),
            target_field: FieldPath::new(&["unknown"]),
        },
        ModelGraphError::MissingTarget {
            source: ModelId::new("test.graph.MissingTargetSource"),
            field: "target_id",
            target: ModelId::new("test.graph.Missing"),
        },
        ModelGraphError::InvalidReferencePath {
            source: ModelId::new("test.graph.MissingTargetSource"),
            field: "target_id",
            path: ReferencePath::new(&[ReferencePathSegment::Field("unknown")]),
        },
    ];

    let errors = registry
        .validate_graph()
        .expect_err("the reference graph should be invalid");
    assert_eq!(errors.errors(), expected);
    assert!(errors.to_string().starts_with("model graph validation failed; "));

    let reversed_registry = ModelRegistry::from_registrations(registrations.into_iter().rev())
        .expect("the registrations should be valid and unique regardless of input order");
    let reversed_errors = reversed_registry
        .validate_graph()
        .expect_err("the reference graph should be invalid");
    assert_eq!(reversed_errors.errors(), expected);
}

#[test]
fn test_validate_graph_reports_a_canonical_self_cycle() {
    let registry = ModelRegistry::from_registrations([&SELF_CYCLE_REGISTRATION])
        .expect("the self-referencing registration should be valid");

    let errors = registry
        .validate_graph()
        .expect_err("the required self-reference should be a cycle");

    assert_eq!(
        errors.errors(),
        [ModelGraphError::RequiredReferenceCycle {
            cycle: vec![
                ModelId::new("test.graph.SelfCycle"),
                ModelId::new("test.graph.SelfCycle"),
            ],
        }],
    );
}

#[test]
fn test_validate_graph_allows_empty_vector_reference_cycles() {
    ModelRegistry::from_registrations([&VEC_CYCLE_REGISTRATION])
        .expect("the registry should be valid")
        .validate_graph()
        .expect("an unconstrained vector can be empty");
}

#[test]
fn test_validate_graph_ignores_optional_required_references_for_cycle_detection() {
    let registry = ModelRegistry::from_registrations([&OPTIONAL_CYCLE_A_REGISTRATION, &OPTIONAL_CYCLE_B_REGISTRATION])
        .expect("the optional-cycle registrations should be valid");

    registry
        .validate_graph()
        .expect("an optional source reference must not form a required cycle");
}

#[test]
fn test_validate_graph_resolves_nested_target_paths_and_optional_projections() {
    let registry = ModelRegistry::from_registrations([&NESTED_SOURCE_REGISTRATION, &NESTED_TARGET_REGISTRATION])
        .expect("the nested-path registrations should be valid");

    registry
        .validate_graph()
        .expect("the nested target path should resolve through named metadata");
}

#[test]
fn test_validate_graph_resolves_reference_path_through_projection_entities() {
    let registry = ModelRegistry::from_registrations([
        &COUNTRY_REGISTRATION,
        &PROVINCE_REGISTRATION,
        &ADDRESS_PATH_REGISTRATION,
    ])
    .expect("the address-path registrations should be valid");

    registry
        .validate_graph()
        .expect("reference path should traverse through referenced projection entities");
}

#[test]
fn test_validate_graph_accepts_model_and_info_method_reference_projections() {
    let registry = ModelRegistry::from_registrations([
        &TARGET_REGISTRATION,
        &DIRECT_TARGET_REGISTRATION,
        &INFO_PROJECTION_REGISTRATION,
    ])
    .expect("the registrations should be valid");

    registry
        .validate_graph()
        .expect("model values and trait-provided info projections are valid references");
}

#[test]
fn test_validate_graph_validates_lookup_relations_and_acyclic_ownership() {
    let registry = ModelRegistry::from_registrations([
        &TARGET_REGISTRATION,
        &LOOKUP_SOURCE_REGISTRATION,
        &OWNED_MODEL_REGISTRATION,
    ])
    .expect("the lookup and ownership registrations should be valid");

    registry
        .validate_graph()
        .expect("lookup relation and acyclic ownership should be valid");
}

#[test]
fn test_validate_graph_reports_ownership_cycles() {
    let registry =
        ModelRegistry::from_registrations([&OWNERSHIP_CYCLE_A_REGISTRATION, &OWNERSHIP_CYCLE_B_REGISTRATION])
            .expect("the ownership registrations should be structurally valid");

    assert_eq!(
        registry
            .validate_graph()
            .expect_err("cyclic ownership must be rejected")
            .errors(),
        [ModelGraphError::OwnershipCycle {
            cycle: vec![
                ModelId::new("test.graph.OwnershipCycleA"),
                ModelId::new("test.graph.OwnershipCycleB"),
                ModelId::new("test.graph.OwnershipCycleA"),
            ],
        }]
    );
}
