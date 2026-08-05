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

//! Integration tests for explicit metadata resolution.

use qubit_model_metadata::{
    MetadataRegistry, MetadataResolver, StructMetadata, TypeIdentity, TypeKind, TypeMetadata,
};

struct Account;
struct Organization;

static ACCOUNT_METADATA: TypeMetadata = TypeMetadata::new(
    TypeIdentity::of::<Account>(),
    TypeKind::Struct(StructMetadata::new(&[])),
    &[],
);
static ORGANIZATION_METADATA: TypeMetadata = TypeMetadata::new(
    TypeIdentity::of::<Organization>(),
    TypeKind::Struct(StructMetadata::new(&[])),
    &[],
);

static MODELS: [&TypeMetadata; 2] = [&ACCOUNT_METADATA, &ORGANIZATION_METADATA];

#[test]
fn test_registry_resolves_by_type_identity() {
    let registry = MetadataRegistry::new(&MODELS);

    assert_eq!(registry.models().len(), MODELS.len());
    assert!(core::ptr::eq(registry.models().as_ptr(), MODELS.as_ptr()));
    assert_eq!(
        registry
            .resolve(TypeIdentity::of::<Account>())
            .map(|metadata| metadata.identity()),
        Some(TypeIdentity::of::<Account>()),
    );
    assert_eq!(
        registry
            .resolve(TypeIdentity::of::<Organization>())
            .map(|metadata| metadata.identity()),
        Some(TypeIdentity::of::<Organization>()),
    );
}

#[test]
fn test_registry_returns_none_for_unknown_identity_and_empty_registry() {
    let empty = MetadataRegistry::new(&[]);
    assert!(empty.resolve(TypeIdentity::of::<Account>()).is_none());

    let registry = MetadataRegistry::new(&MODELS);
    assert!(registry.resolve(TypeIdentity::of::<u32>()).is_none());
}

#[test]
fn test_registry_resolves_first_duplicate_identity() {
    static DUPLICATES: [&TypeMetadata; 2] = [&ACCOUNT_METADATA, &ACCOUNT_METADATA];
    let registry = MetadataRegistry::new(&DUPLICATES);

    assert!(registry.resolve(TypeIdentity::of::<Account>()).is_some());
}
