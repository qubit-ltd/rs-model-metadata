// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::any::TypeId;

use super::ValueClosureState;

#[test]
fn test_caches_only_clean_types_without_cycle_epoch_changes() {
    let mut state = ValueClosureState::new();
    let leaf = TypeId::of::<u8>();
    assert!(state.enter(leaf));
    let epoch = state.epoch();
    state.leave(leaf, epoch, true);
    assert!(state.is_closed(leaf));
    assert_eq!(state.expansion_count(), 1);

    let failed = TypeId::of::<u16>();
    assert!(state.enter(failed));
    let epoch = state.epoch();
    state.leave(failed, epoch, false);
    assert!(!state.is_closed(failed));
}

#[test]
fn test_back_edges_invalidate_active_ancestors_and_saturation_disables_caching() {
    let mut state = ValueClosureState::new();
    let outer = TypeId::of::<u8>();
    assert!(state.enter(outer));
    let start = state.epoch();
    assert!(!state.enter(outer));
    state.leave(outer, start, true);
    assert!(!state.is_closed(outer));

    state.cycle_epoch = u64::MAX;
    let saturated = TypeId::of::<u16>();
    assert!(state.enter(saturated));
    state.leave(saturated, u64::MAX, true);
    assert!(!state.is_closed(saturated));
}

#[test]
fn test_multi_type_back_edge_invalidates_every_active_ancestor() {
    let mut state = ValueClosureState::new();
    let outer = TypeId::of::<u8>();
    let inner = TypeId::of::<u16>();

    assert!(state.enter(outer));
    let outer_epoch = state.epoch();
    assert!(state.enter(inner));
    let inner_epoch = state.epoch();
    assert!(!state.enter(outer));

    state.leave(inner, inner_epoch, true);
    state.leave(outer, outer_epoch, true);

    assert!(!state.is_closed(outer));
    assert!(!state.is_closed(inner));
    assert!(state.active_is_empty());
}
