// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Direct-reference graph validation for [`ModelRegistry`].

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use super::ModelRegistry;
use crate::attribute::AttributeMetadata;
use crate::field_metadata::FieldMetadata;
use crate::metadata_resolver::MetadataResolver;
use crate::model_graph::ModelGraphError;
use crate::model_graph::ModelGraphErrors;
use crate::model_graph::project_relation_type;
use crate::model_id::ModelId;
use crate::relation::FieldPath;
use crate::relation::LookupRelationMetadata;
use crate::relation::OwnershipMetadata;
use crate::relation::ReferenceMetadata;
use crate::relation::ReferencePath;
use crate::relation::ReferencePathSegment;
use crate::relation::ReferenceTarget;
use crate::type_metadata::TypeMetadata;
use crate::type_shape::TypeRef;
use crate::type_shape::TypeShape;

impl ModelRegistry {
    /// Validates direct references among all registered models.
    ///
    /// Returns every independently discoverable missing target, missing target
    /// field, incompatible projection, invalid reference path, and
    /// required-cycle
    /// error. Registry construction intentionally does not run this method so
    /// a partial linked model collection remains usable.
    ///
    /// # Errors
    ///
    /// Returns [`ModelGraphErrors`] when one or more direct-reference graph
    /// invariants are invalid.
    #[must_use = "graph validation errors must be handled"]
    pub fn validate_graph(&self) -> Result<(), ModelGraphErrors> {
        let mut errors = Vec::new();
        let mut required_edges = self.empty_required_edges();
        let mut ownership_edges = self.empty_required_edges();

        for registration in self.registrations() {
            let source = registration.id();
            let metadata = registration.metadata();
            for field in metadata.fields() {
                for reference in direct_references(*field) {
                    self.validate_reference(source, metadata, *field, reference, &mut errors, &mut required_edges);
                }
                for lookup in lookup_relations(*field) {
                    self.validate_lookup_relation(source, *field, lookup, &mut errors);
                }
            }
            for ownership in ownership_relations(metadata) {
                self.validate_ownership(source, ownership, &mut errors, &mut ownership_edges);
            }
        }

        errors.extend(find_required_reference_cycles(&required_edges));
        errors.extend(find_ownership_cycles(&ownership_edges));
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
        reference: ReferenceMetadata,
        errors: &mut Vec<ModelGraphError>,
        required_edges: &mut BTreeMap<ModelId, Vec<ModelId>>,
    ) {
        self.validate_reference_path(source, source_metadata, source_field, reference, errors);

        let entity = reference.entity();
        let Some(target_metadata) = self.get(entity.as_str()) else {
            errors.push(ModelGraphError::MissingTarget {
                source,
                field: source_field.name(),
                target: entity,
            });
            return;
        };

        let source_projection = project_relation_type(source_field.field_type());
        let required_reference = reference.existing() && requires_reference_target(source_field);
        if required_reference {
            required_edges
                .get_mut(&source)
                .expect("all registered source models have adjacency entries")
                .push(entity);
        }

        match reference.target() {
            ReferenceTarget::WholeModel => {
                if source_projection.identity() != Some(target_metadata.identity()) {
                    errors.push(ModelGraphError::IncompatibleProjection {
                        source,
                        field: source_field.name(),
                        source_type: source_projection.type_name(),
                        target: entity,
                        target_field: FieldPath::new(&["*"]),
                        target_type: target_metadata.identity().type_name(),
                    });
                }
            }
            ReferenceTarget::Property(target_path) => {
                if is_info_method_projection(target_path) {
                    return;
                }

                let Some(target_field) = resolve_field_path(target_metadata, target_path) else {
                    errors.push(ModelGraphError::MissingTargetField {
                        source,
                        field: source_field.name(),
                        target: entity,
                        target_field: target_path,
                    });
                    return;
                };

                if source_projection.identity() == Some(target_metadata.identity()) {
                    return;
                }

                let target_projection = project_relation_type(target_field.field_type());
                if !source_projection.is_compatible_with(target_projection) {
                    errors.push(ModelGraphError::IncompatibleProjection {
                        source,
                        field: source_field.name(),
                        source_type: source_projection.type_name(),
                        target: entity,
                        target_field: target_path,
                        target_type: target_projection.type_name(),
                    });
                }
            }
        }
    }

    /// Validates one direct reference's object-graph path, when present.
    fn validate_reference_path(
        &self,
        source: ModelId,
        source_metadata: &'static TypeMetadata,
        source_field: FieldMetadata,
        reference: ReferenceMetadata,
        errors: &mut Vec<ModelGraphError>,
    ) {
        let Some(path) = reference.path() else {
            return;
        };
        let Some(target_field) = resolve_reference_path(self, source_metadata, path) else {
            errors.push(ModelGraphError::InvalidReferencePath {
                source,
                field: source_field.name(),
                path,
            });
            return;
        };
        let Some(target_reference) = target_field.reference() else {
            errors.push(ModelGraphError::IncompatibleReferencePath {
                source,
                field: source_field.name(),
                path,
            });
            return;
        };
        if target_reference.entity() != reference.entity()
            || target_reference.target() != reference.target()
            || target_reference.existing() != reference.existing()
        {
            errors.push(ModelGraphError::IncompatibleReferencePath {
                source,
                field: source_field.name(),
                path,
            });
        }
    }

    /// Validates one lookup-relation attribute.
    fn validate_lookup_relation(
        &self,
        source: ModelId,
        source_field: FieldMetadata,
        lookup: LookupRelationMetadata,
        errors: &mut Vec<ModelGraphError>,
    ) {
        let target_reference = lookup.target();
        let Some(target_metadata) = self.resolve(target_reference.identity()) else {
            let target = target_reference.metadata().map(TypeMetadata::id).unwrap_or(source);
            errors.push(ModelGraphError::MissingLookupTarget {
                source,
                field: source_field.name(),
                target,
            });
            return;
        };
        let target = target_metadata.id();
        let Some(target_field) = resolve_field_path(target_metadata, lookup.target_field()) else {
            errors.push(ModelGraphError::MissingLookupTargetField {
                source,
                field: source_field.name(),
                target,
                target_field: lookup.target_field(),
            });
            return;
        };
        let source_projection = project_relation_type(source_field.field_type());
        let target_projection = project_relation_type(target_field.field_type());
        if !source_projection.is_compatible_with(target_projection) {
            errors.push(ModelGraphError::IncompatibleLookupProjection {
                source,
                field: source_field.name(),
                source_type: source_projection.type_name(),
                target,
                target_field: lookup.target_field(),
                target_type: target_projection.type_name(),
            });
        }
    }

    /// Validates one ownership attribute and records its hierarchy edge.
    fn validate_ownership(
        &self,
        source: ModelId,
        ownership: OwnershipMetadata,
        errors: &mut Vec<ModelGraphError>,
        ownership_edges: &mut BTreeMap<ModelId, Vec<ModelId>>,
    ) {
        let owner_reference = ownership.owner();
        let Some(owner_metadata) = self.resolve(owner_reference.identity()) else {
            let owner = owner_reference.metadata().map(TypeMetadata::id).unwrap_or(source);
            errors.push(ModelGraphError::MissingOwner { source, owner });
            return;
        };
        ownership_edges
            .get_mut(&source)
            .expect("all registered source models have adjacency entries")
            .push(owner_metadata.id());
    }
}

/// Returns whether `path` denotes the conventional `info()` relation
/// projection supplied by a model trait rather than a stored field.
fn is_info_method_projection(path: FieldPath) -> bool {
    path.segments() == ["info"]
}

/// Returns whether every valid field value contains at least one reference.
fn requires_reference_target(field: FieldMetadata) -> bool {
    requires_shape_target(
        field.field_type(),
        field.sequence_constraint().and_then(|value| value.min_items()),
    )
}

/// Returns whether `field_type` necessarily contributes a leaf value.
fn requires_shape_target(field_type: TypeRef, min_items: Option<u32>) -> bool {
    match field_type.shape() {
        TypeShape::Optional(_) | TypeShape::Set(_) | TypeShape::Map { .. } => false,
        TypeShape::Sequence(inner) => min_items.is_some_and(|value| value > 0) && requires_shape_target(inner, None),
        TypeShape::Array { element, length } => length > 0 && requires_shape_target(element, None),
        TypeShape::Scalar(_) | TypeShape::Named(_) | TypeShape::Opaque => true,
    }
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
fn direct_references(field: FieldMetadata) -> impl Iterator<Item = ReferenceMetadata> {
    field.attributes().iter().filter_map(|attribute| match attribute {
        AttributeMetadata::Reference(reference) => Some(*reference),
        _ => None,
    })
}

/// Returns every lookup-relation attribute declared on `field`.
fn lookup_relations(field: FieldMetadata) -> impl Iterator<Item = LookupRelationMetadata> {
    field.attributes().iter().filter_map(|attribute| match attribute {
        AttributeMetadata::LookupRelation(lookup) => Some(*lookup),
        _ => None,
    })
}

/// Returns every ownership relation declared on `metadata`.
fn ownership_relations(metadata: &'static TypeMetadata) -> impl Iterator<Item = OwnershipMetadata> {
    metadata.attributes().iter().filter_map(|attribute| match attribute {
        AttributeMetadata::Ownership(ownership) => Some(*ownership),
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
fn resolve_field_path(metadata: &'static TypeMetadata, path: FieldPath) -> Option<&'static FieldMetadata> {
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

/// Resolves an object-graph reference path within one metadata root.
fn resolve_reference_path(
    registry: &ModelRegistry,
    metadata: &'static TypeMetadata,
    path: ReferencePath,
) -> Option<&'static FieldMetadata> {
    let mut current_metadata = metadata;
    let mut current_field = None;
    for segment in path.segments() {
        match segment {
            ReferencePathSegment::Parent => return None,
            ReferencePathSegment::Field(name) => {
                let field = current_metadata.field(name)?;
                current_field = Some(field);
                if let Some(reference) = field.reference() {
                    current_metadata = registry.get(reference.entity().as_str())?;
                } else if let Some(metadata) = field.field_type().named_metadata() {
                    current_metadata = metadata;
                }
            }
        }
    }
    current_field
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
fn find_required_reference_cycles(edges: &BTreeMap<ModelId, Vec<ModelId>>) -> Vec<ModelGraphError> {
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

/// Finds one canonical ownership cycle for every cyclic hierarchy component.
fn find_ownership_cycles(edges: &BTreeMap<ModelId, Vec<ModelId>>) -> Vec<ModelGraphError> {
    let mut edges = edges.clone();
    for targets in edges.values_mut() {
        targets.sort_unstable();
        targets.dedup();
    }
    strongly_connected_components(&edges)
        .into_iter()
        .filter_map(|component| canonical_cycle(&component, &edges))
        .map(|cycle| ModelGraphError::OwnershipCycle { cycle })
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
fn strongly_connected_components(edges: &BTreeMap<ModelId, Vec<ModelId>>) -> Vec<Vec<ModelId>> {
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
                *target, edges, next_index, indices, lowlinks, stack, on_stack, components,
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
        let member = stack.pop().expect("Tarjan root must have a matching node on the stack");
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
fn canonical_cycle(component: &[ModelId], edges: &BTreeMap<ModelId, Vec<ModelId>>) -> Option<Vec<ModelId>> {
    let start = *component.first()?;
    if component.len() == 1 {
        return edges[&start].contains(&start).then(|| vec![start, start]);
    }

    let allowed: BTreeSet<_> = component.iter().copied().collect();
    let mut path = vec![start];
    let mut visited = BTreeSet::from([start]);
    find_cycle_from(start, start, &allowed, edges, &mut visited, &mut path).then_some(path)
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
fn compare_graph_errors(left: &ModelGraphError, right: &ModelGraphError) -> core::cmp::Ordering {
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
fn graph_error_sort_key(error: &ModelGraphError) -> (Option<ModelId>, Option<&'static str>, u8, Option<ModelId>) {
    match error {
        ModelGraphError::MissingTarget { source, field, target } => (Some(*source), Some(*field), 0, Some(*target)),
        ModelGraphError::MissingTargetField {
            source, field, target, ..
        } => (Some(*source), Some(*field), 1, Some(*target)),
        ModelGraphError::IncompatibleProjection {
            source, field, target, ..
        } => (Some(*source), Some(*field), 2, Some(*target)),
        ModelGraphError::InvalidReferencePath { source, field, .. } => (Some(*source), Some(*field), 3, None),
        ModelGraphError::IncompatibleReferencePath { source, field, .. } => (Some(*source), Some(*field), 4, None),
        ModelGraphError::MissingLookupTarget { source, field, target } => {
            (Some(*source), Some(*field), 5, Some(*target))
        }
        ModelGraphError::MissingLookupTargetField {
            source, field, target, ..
        } => (Some(*source), Some(*field), 6, Some(*target)),
        ModelGraphError::IncompatibleLookupProjection {
            source, field, target, ..
        } => (Some(*source), Some(*field), 7, Some(*target)),
        ModelGraphError::MissingOwner { source, owner } => (Some(*source), None, 8, Some(*owner)),
        ModelGraphError::RequiredReferenceCycle { cycle } => {
            let source = cycle.first().copied();
            (source, None, 9, source)
        }
        ModelGraphError::OwnershipCycle { cycle } => {
            let source = cycle.first().copied();
            (source, None, 10, source)
        }
    }
}
