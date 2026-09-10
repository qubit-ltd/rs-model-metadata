// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Context of one executable declaration at a concrete usage path.

use super::execution_declaration::ExecutionDeclaration;
use crate::metadata::FieldMetadata;
use crate::metadata::SelectorPosition;
use crate::metadata::TypeMetadata;

/// One source declaration reached through one root-relative field path.
#[derive(Clone)]
pub(crate) struct ValidationOccurrence {
    /// Root requested by the caller.
    pub(crate) root: &'static TypeMetadata,
    /// Concrete type declaring the field.
    pub(crate) owner: &'static TypeMetadata,
    /// Original field overlay, including its checked identity and source.
    pub(crate) field: &'static FieldMetadata,
    /// Full property path from the root to this field.
    pub(crate) segments: Vec<&'static str>,
    /// Structural path for rejected shapes, including variant and tuple
    /// positions. Supported property paths use `segments` without
    /// allocating this string.
    pub(crate) unsupported_path: Option<String>,
    /// Container position, if this declaration belongs to a selector.
    pub(crate) selector: Option<SelectorPosition>,
    /// Declaration-order identity within the root's traversal.
    pub(crate) ordinal: usize,
    /// Original executable declaration.
    pub(crate) declaration: ExecutionDeclaration,
}

impl ValidationOccurrence {
    /// Returns the declared user rule ID even when no registration exists.
    pub(crate) const fn declared_rule_id(&self) -> Option<&'static str> {
        match self.declaration {
            ExecutionDeclaration::Validator(value) => Some(value.declared_id()),
            ExecutionDeclaration::Constraint(_) | ExecutionDeclaration::Traversal => None,
        }
    }
}

impl std::fmt::Debug for ValidationOccurrence {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ValidationOccurrence")
            .field("root", &self.root.type_name())
            .field("owner", &self.owner.type_name())
            .field("field", &self.field.location())
            .field("selector", &self.selector)
            .field("ordinal", &self.ordinal)
            .field("declared_rule_id", &self.declared_rule_id())
            .finish()
    }
}
