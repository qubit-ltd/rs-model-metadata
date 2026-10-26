// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Safe local erased adapters for model properties.

mod borrowed_collection;
mod borrowed_property_slice;
mod collection_items_adapter;
mod collection_ops;
mod erased_property_value;
mod getter_metadata;
mod getter_output_kind;
mod internal;
mod item_eq_adapter;
mod map_len_adapter;
mod optional_ops;
mod property_access_error;
mod property_metadata;
mod property_set_failure;
mod property_storage_kind;
mod property_type_mismatch;
mod property_value;
mod setter_metadata;
pub use borrowed_collection::BorrowedCollection;
pub use borrowed_property_slice::BorrowedPropertySlice;
pub use collection_items_adapter::CollectionItemsAdapter;
pub use collection_ops::CollectionOps;
pub use erased_property_value::PropertyBorrowed;
pub use erased_property_value::PropertyMutable;
pub use erased_property_value::PropertyOwned;
pub use getter_metadata::GetterAdapter;
pub use getter_metadata::GetterMetadata;
pub use getter_output_kind::GetterOutputKind;
pub use item_eq_adapter::ItemEqAdapter;
pub use map_len_adapter::MapLenAdapter;
pub use optional_ops::OptionalOps;
pub use property_access_error::PropertyAccessError;
pub use property_metadata::PropertyMetadata;
pub use property_set_failure::PropertySetFailure;
pub use property_storage_kind::PropertyStorageKind;
pub use property_type_mismatch::PropertyTypeMismatch;
pub use property_value::PropertyValue;
pub use setter_metadata::SetterAdapter;
pub use setter_metadata::SetterMetadata;
