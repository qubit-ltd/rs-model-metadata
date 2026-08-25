// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! [`HasTypeShape`] integration for [`qubit_datatype::DataType`].

use qubit_datatype::DataType;

use crate::type_metadata::EnumMetadata;
use crate::type_metadata::EnumVariantMetadata;
use crate::type_metadata::HasTypeMetadata;
use crate::type_metadata::NamedTypeRef;
use crate::type_metadata::TypeIdentity;
use crate::type_metadata::TypeKind;
use crate::type_metadata::TypeMetadata;
use crate::type_shape::HasTypeShape;
use crate::type_shape::TypeCapabilities;
use crate::type_shape::TypeShape;
use crate::ModelId;

static DATA_TYPE_VARIANTS: [EnumVariantMetadata; 25] = [
    EnumVariantMetadata::new(0, "bool"),
    EnumVariantMetadata::new(1, "char"),
    EnumVariantMetadata::new(2, "int8"),
    EnumVariantMetadata::new(3, "int16"),
    EnumVariantMetadata::new(4, "int32"),
    EnumVariantMetadata::new(5, "int64"),
    EnumVariantMetadata::new(6, "int128"),
    EnumVariantMetadata::new(7, "uint8"),
    EnumVariantMetadata::new(8, "uint16"),
    EnumVariantMetadata::new(9, "uint32"),
    EnumVariantMetadata::new(10, "uint64"),
    EnumVariantMetadata::new(11, "uint128"),
    EnumVariantMetadata::new(12, "float32"),
    EnumVariantMetadata::new(13, "float64"),
    EnumVariantMetadata::new(14, "string"),
    EnumVariantMetadata::new(15, "date"),
    EnumVariantMetadata::new(16, "time"),
    EnumVariantMetadata::new(17, "datetime"),
    EnumVariantMetadata::new(18, "instant"),
    EnumVariantMetadata::new(19, "biginteger"),
    EnumVariantMetadata::new(20, "bigdecimal"),
    EnumVariantMetadata::new(21, "duration"),
    EnumVariantMetadata::new(22, "url"),
    EnumVariantMetadata::new(23, "stringmap"),
    EnumVariantMetadata::new(24, "json"),
];

impl HasTypeShape for DataType {
    const TYPE_SHAPE: TypeShape = TypeShape::Named(NamedTypeRef::of::<DataType>());
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::NONE;
}

impl HasTypeMetadata for DataType {
    fn type_metadata() -> &'static TypeMetadata {
        static METADATA: TypeMetadata = TypeMetadata::new(
            ModelId::new("qubit.datatype.DataType"),
            TypeIdentity::of::<DataType>(),
            TypeKind::Enum(EnumMetadata::new(&DATA_TYPE_VARIANTS)),
            &[],
        );
        &METADATA
    }
}
