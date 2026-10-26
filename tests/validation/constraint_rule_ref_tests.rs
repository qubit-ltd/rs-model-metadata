// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Public behavior of standard constraint rule references.

use qubit_model_metadata::validation::ConstraintRuleRef;
use qubit_validator::ValidatorId;

#[test]
fn test_rule_id_and_registry_eligibility() {
    let id = ValidatorId::new("qubit.rules.text.non_blank");
    let registry = ConstraintRuleRef::Registry(id);
    let intrinsic = ConstraintRuleRef::ModelIntrinsic(id);

    assert_eq!(registry.id(), id);
    assert_eq!(registry.registry_id(), Some(id));
    assert_eq!(intrinsic.id(), id);
    assert_eq!(intrinsic.registry_id(), None);
}
