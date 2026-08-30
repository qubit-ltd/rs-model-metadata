// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! [`HasTypeShape`] integration for [`qubit_datatype::DataType`].

use std::sync::LazyLock;

use qubit_datatype::DataType;

use crate::ModelId;
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

static DATA_TYPE_VARIANTS: LazyLock<Vec<EnumVariantMetadata>> = LazyLock::new(|| {
    DataType::ALL
        .iter()
        .enumerate()
        .map(|(ordinal, data_type)| EnumVariantMetadata::new(ordinal, data_type.info().name()))
        .collect()
});

impl HasTypeShape for DataType {
    const TYPE_SHAPE: TypeShape = TypeShape::Named(NamedTypeRef::of::<DataType>());
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::NONE;
}

impl HasTypeMetadata for DataType {
    fn type_metadata() -> &'static TypeMetadata {
        static METADATA: LazyLock<TypeMetadata> = LazyLock::new(|| {
            TypeMetadata::new(
                ModelId::new("qubit.datatype.DataType"),
                TypeIdentity::of::<DataType>(),
                TypeKind::Enum(EnumMetadata::new(&DATA_TYPE_VARIANTS)),
                &[],
            )
        });
        &METADATA
    }
}
