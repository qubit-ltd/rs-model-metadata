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

//! Explicit validation of direct-reference graphs in a model registry.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::attribute::AttributeMetadata;
use crate::field_metadata::FieldMetadata;
use crate::metadata_registry::ModelRegistry;
use crate::model_id::ModelId;
use crate::relation::FieldPath;
use crate::type_metadata::TypeMetadata;

mod model_graph_error;
mod model_graph_errors;

pub use self::model_graph_error::ModelGraphError;
pub use self::model_graph_errors::ModelGraphErrors;

impl ModelRegistry {
    /// Validates direct references among all registered models.
    ///
    /// Returns every independently discoverable missing target, missing target
    /// field, incompatible projection, invalid `same_as`, and required-cycle
    /// error. Registry construction intentionally does not run this method so
    /// a partial linked model collection remains usable.
    ///
    /// # Errors
    ///
    /// Returns [`ModelGraphErrors`] when one or more direct-reference graph
    /// invariants are invalid.
    pub fn validate_graph(&self) -> Result<(), ModelGraphErrors> {
        let mut errors = Vec::new();
        let mut required_edges = self.empty_required_edges();

        for registration in self.registrations() {
            let source = registration.id();
            let metadata = registration.metadata();
            for field in metadata.fields() {
                for reference in direct_references(*field) {
                    self.validate_reference(
                        source,
                        metadata,
                        *field,
                        reference,
                        &mut errors,
                        &mut required_edges,
                    );
                }
            }
        }

        errors.extend(find_required_reference_cycles(&required_edges));
        errors.sort_unstable_by(compare_graph_errors);
        if errors.is_empty() {
            Ok(())
        } else {
            Err(ModelGraphErrors::new(errors))
        }
    }

    /// Creates an adjacency map containing every registered model.
    ///
    /// # Returns
    ///
    /// A model-ID-sorted adjacency map with no edges.
    fn empty_required_edges(&self) -> BTreeMap<ModelId, Vec<ModelId>> {
        self.registrations()
            .map(|registration| (registration.id(), Vec::new()))
            .collect()
    }

    /// Validates one direct-reference attribute and records any graph edge.
    ///
    /// # Parameters
    ///
    /// - `source`: The model declaring the reference.
    /// - `source_metadata`: Metadata of the model declaring the reference.
    /// - `source_field`: The field declaring the reference.
    /// - `reference`: The direct-reference metadata to validate.
    /// - `errors`: The aggregate validation errors to append to.
    /// - `required_edges`: Required non-null reference edges to extend.
    fn validate_reference(
        &self,
        source: ModelId,
        source_metadata: &'static TypeMetadata,
        source_field: FieldMetadata,
        reference: crate::relation::ReferenceMetadata,
        errors: &mut Vec<ModelGraphError>,
        required_edges: &mut BTreeMap<ModelId, Vec<ModelId>>,
    ) {
        if let Some(same_as) = reference.same_as()
            && resolve_field_path(source_metadata, same_as).is_none()
        {
            errors.push(ModelGraphError::InvalidSameAs {
                source,
                field: source_field.name(),
                same_as,
            });
        }

        let target = reference.target();
        let Some(target_metadata) = self.get(target.as_str()) else {
            errors.push(ModelGraphError::MissingTarget {
                source,
                field: source_field.name(),
                target,
            });
            return;
        };

        let required_reference = reference.must_exist()
            && !is_nullable_reference_field(source_field);
        if required_reference {
            required_edges
                .get_mut(&source)
                .expect("all registered source models have adjacency entries")
                .push(target);
        }

        let Some(target_field) =
            resolve_field_path(target_metadata, reference.target_field())
        else {
            errors.push(ModelGraphError::MissingTargetField {
                source,
                field: source_field.name(),
                target,
                target_field: reference.target_field(),
            });
            return;
        };

        let source_type =
            source_field.field_type().strip_optional().type_name();
        let target_type =
            target_field.field_type().strip_optional().type_name();
        if source_type != target_type
            && !is_id_reference_projection(source_field, *target_field)
        {
            errors.push(ModelGraphError::IncompatibleProjection {
                source,
                field: source_field.name(),
                source_type,
                target,
                target_field: reference.target_field(),
                target_type,
            });
        }
    }
}

/// Returns whether a reference field is nullable, including fields whose
/// metadata is intentionally opaque.
fn is_nullable_reference_field(field: FieldMetadata) -> bool {
    field.is_nullable()
        || field.rust_type_name().starts_with("core::option::Option<")
        || field.rust_type_name().starts_with("std::option::Option<")
}

/// Returns whether an opaque ID field projects to a target ID field.
fn is_id_reference_projection(
    source: FieldMetadata,
    target: FieldMetadata,
) -> bool {
    target
        .field_type()
        .strip_optional()
        .type_name()
        .contains("qubit_id::id::Id")
        && source.field_type().type_name().contains("qubit_id::id::Id")
}

/// Returns every direct-reference attribute declared on `field`.
///
/// # Parameters
///
/// - `field`: The source field whose attributes are inspected.
///
/// # Returns
///
/// An iterator over the field's direct-reference attributes.
fn direct_references(
    field: FieldMetadata,
) -> impl Iterator<Item = crate::relation::ReferenceMetadata> {
    field
        .attributes()
        .iter()
        .filter_map(|attribute| match attribute {
            AttributeMetadata::Reference(reference) => Some(*reference),
            _ => None,
        })
}

/// Resolves a static field path through named target-model metadata.
///
/// # Parameters
///
/// - `metadata`: The model metadata from which to begin resolving.
/// - `path`: The non-empty field path to traverse.
///
/// # Returns
///
/// `Some` with the final field when every segment resolves through named
/// metadata; otherwise `None`.
fn resolve_field_path(
    metadata: &'static TypeMetadata,
    path: FieldPath,
) -> Option<&'static FieldMetadata> {
    let mut metadata = metadata;
    let mut segments = path.segments().iter();
    let mut segment = *segments.next()?;
    loop {
        let field = metadata.field(segment)?;
        let Some(next_segment) = segments.next() else {
            return Some(field);
        };
        metadata = field.field_type().named_metadata()?;
        segment = *next_segment;
    }
}

/// Finds one canonical required-reference cycle for every cyclic SCC.
///
/// # Parameters
///
/// - `edges`: The required non-null direct-reference adjacency map.
///
/// # Returns
///
/// Cycle errors in component-discovery order. The caller performs final error
/// ordering together with non-cycle graph errors.
fn find_required_reference_cycles(
    edges: &BTreeMap<ModelId, Vec<ModelId>>,
) -> Vec<ModelGraphError> {
    let mut edges = edges.clone();
    for targets in edges.values_mut() {
        targets.sort_unstable();
        targets.dedup();
    }

    let components = strongly_connected_components(&edges);
    components
        .into_iter()
        .filter_map(|component| canonical_cycle(&component, &edges))
        .map(|cycle| ModelGraphError::RequiredReferenceCycle { cycle })
        .collect()
}

/// Computes strongly connected components with deterministic Tarjan traversal.
///
/// # Parameters
///
/// - `edges`: The sorted directed graph to inspect.
///
/// # Returns
///
/// Every strongly connected component, with each component sorted by model ID.
fn strongly_connected_components(
    edges: &BTreeMap<ModelId, Vec<ModelId>>,
) -> Vec<Vec<ModelId>> {
    let mut next_index = 0;
    let mut indices = BTreeMap::new();
    let mut lowlinks = BTreeMap::new();
    let mut stack = Vec::new();
    let mut on_stack = BTreeSet::new();
    let mut components = Vec::new();

    for node in edges.keys().copied() {
        if !indices.contains_key(&node) {
            visit_component(
                node,
                edges,
                &mut next_index,
                &mut indices,
                &mut lowlinks,
                &mut stack,
                &mut on_stack,
                &mut components,
            );
        }
    }
    components
}

/// Visits one node during Tarjan strongly connected component discovery.
///
/// # Parameters
///
/// - `node`: The unvisited or partially visited node.
/// - `edges`: The sorted graph to inspect.
/// - `next_index`: The next DFS index to assign.
/// - `indices`: Assigned DFS indexes.
/// - `lowlinks`: Minimum reachable DFS indexes.
/// - `stack`: The current Tarjan node stack.
/// - `on_stack`: Nodes currently present on `stack`.
/// - `components`: Completed components to extend.
#[allow(clippy::too_many_arguments)]
fn visit_component(
    node: ModelId,
    edges: &BTreeMap<ModelId, Vec<ModelId>>,
    next_index: &mut usize,
    indices: &mut BTreeMap<ModelId, usize>,
    lowlinks: &mut BTreeMap<ModelId, usize>,
    stack: &mut Vec<ModelId>,
    on_stack: &mut BTreeSet<ModelId>,
    components: &mut Vec<Vec<ModelId>>,
) {
    let node_index = *next_index;
    *next_index += 1;
    indices.insert(node, node_index);
    lowlinks.insert(node, node_index);
    stack.push(node);
    on_stack.insert(node);

    for target in &edges[&node] {
        if !indices.contains_key(target) {
            visit_component(
                *target, edges, next_index, indices, lowlinks, stack, on_stack,
                components,
            );
            let target_lowlink = lowlinks[target];
            let node_lowlink = lowlinks[&node];
            lowlinks.insert(node, node_lowlink.min(target_lowlink));
        } else if on_stack.contains(target) {
            let target_index = indices[target];
            let node_lowlink = lowlinks[&node];
            lowlinks.insert(node, node_lowlink.min(target_index));
        }
    }

    if lowlinks[&node] != indices[&node] {
        return;
    }

    let mut component = Vec::new();
    loop {
        let member = stack
            .pop()
            .expect("Tarjan root must have a matching node on the stack");
        on_stack.remove(&member);
        component.push(member);
        if member == node {
            break;
        }
    }
    component.sort_unstable();
    components.push(component);
}

/// Finds the lexicographically first closed cycle within one SCC.
///
/// # Parameters
///
/// - `component`: A sorted strongly connected component.
/// - `edges`: The sorted required-reference graph.
///
/// # Returns
///
/// `Some` with a cycle beginning and ending at the component's smallest ID,
/// or `None` when the component is acyclic.
fn canonical_cycle(
    component: &[ModelId],
    edges: &BTreeMap<ModelId, Vec<ModelId>>,
) -> Option<Vec<ModelId>> {
    let start = *component.first()?;
    if component.len() == 1 {
        return edges[&start].contains(&start).then(|| vec![start, start]);
    }

    let allowed: BTreeSet<_> = component.iter().copied().collect();
    let mut path = vec![start];
    let mut visited = BTreeSet::from([start]);
    find_cycle_from(start, start, &allowed, edges, &mut visited, &mut path)
        .then_some(path)
}

/// Extends `path` until it returns to `start` through the selected SCC.
///
/// # Parameters
///
/// - `start`: The canonical first model ID of the cycle.
/// - `current`: The graph node whose edges are explored.
/// - `allowed`: The current strongly connected component.
/// - `edges`: The sorted required-reference graph.
/// - `visited`: The simple-path nodes already visited.
/// - `path`: The in-progress path, mutated to include a closed cycle on
///   success.
///
/// # Returns
///
/// `true` when `path` was closed by returning to `start`; otherwise `false`.
fn find_cycle_from(
    start: ModelId,
    current: ModelId,
    allowed: &BTreeSet<ModelId>,
    edges: &BTreeMap<ModelId, Vec<ModelId>>,
    visited: &mut BTreeSet<ModelId>,
    path: &mut Vec<ModelId>,
) -> bool {
    for target in &edges[&current] {
        if *target == start {
            path.push(start);
            return true;
        }
        if allowed.contains(target) && visited.insert(*target) {
            path.push(*target);
            if find_cycle_from(start, *target, allowed, edges, visited, path) {
                return true;
            }
            path.pop();
            visited.remove(target);
        }
    }
    false
}

/// Compares graph errors by source ID, field, kind, and target ID.
///
/// # Parameters
///
/// - `left`: The first graph error.
/// - `right`: The second graph error.
///
/// # Returns
///
/// The ordering required for deterministic aggregated validation output.
fn compare_graph_errors(
    left: &ModelGraphError,
    right: &ModelGraphError,
) -> core::cmp::Ordering {
    graph_error_sort_key(left).cmp(&graph_error_sort_key(right))
}

/// Returns the deterministic comparison fields for a graph error.
///
/// # Parameters
///
/// - `error`: The graph error to classify.
///
/// # Returns
///
/// Its source ID, field, kind, and target ID comparison tuple.
fn graph_error_sort_key(
    error: &ModelGraphError,
) -> (Option<ModelId>, Option<&'static str>, u8, Option<ModelId>) {
    match error {
        ModelGraphError::MissingTarget {
            source,
            field,
            target,
        } => (Some(*source), Some(*field), 0, Some(*target)),
        ModelGraphError::MissingTargetField {
            source,
            field,
            target,
            ..
        } => (Some(*source), Some(*field), 1, Some(*target)),
        ModelGraphError::IncompatibleProjection {
            source,
            field,
            target,
            ..
        } => (Some(*source), Some(*field), 2, Some(*target)),
        ModelGraphError::InvalidSameAs { source, field, .. } => {
            (Some(*source), Some(*field), 3, None)
        }
        ModelGraphError::RequiredReferenceCycle { cycle } => {
            let source = cycle.first().copied();
            (source, None, 4, source)
        }
    }
}
