// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
// =============================================================================

use qubit_model_derive::Value;
use qubit_reflect::ReflectionContext;

use crate::metadata::TypeMetadata;
use crate::registry::ModelRegistry;
use crate::resolve::ResolveInputs;
use crate::resolve::StructureResolver;

#[Value]
struct ClosureLeaf {
    value: String,
}

#[Value]
struct ClosureBranch {
    left: ClosureLeaf,
    right: ClosureLeaf,
}

#[Value]
struct ClosureRoot {
    left: ClosureBranch,
    right: ClosureBranch,
}

#[test]
fn resolves_valid_shared_value_subtrees_across_roots() {
    let reflection = ReflectionContext::builder()
        .include_linked()
        .build()
        .expect("reflection context");
    let models = ModelRegistry::from_context(&reflection).expect("model registry");
    let roots = [
        TypeMetadata::of::<ClosureRoot>(),
        TypeMetadata::of::<ClosureBranch>(),
    ];
    let _graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect("shared valid value subtrees resolve");
}
