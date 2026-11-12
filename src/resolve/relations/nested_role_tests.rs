use std::any::TypeId;
use std::collections::HashSet;

use qubit_model_derive::Model;
use qubit_reflect::ReflectionContext;
use qubit_reflect::TypeDescriptor;

use super::super::internal::ResolutionContext;
use super::forbidden_entity_nested_role_with_visit;
use crate::metadata::ModelMetadataError;
use crate::metadata::ModelRole;
use crate::metadata::TypeMetadata;
use crate::registry::ModelRegistry;
use crate::structure::children;

#[Model]
struct SharedLeaf {
    value: u8,
}

#[Model]
struct SharedLevel1 {
    left: SharedLeaf,
    right: SharedLeaf,
}

#[Model]
struct SharedLevel2 {
    left: SharedLevel1,
    right: SharedLevel1,
}

#[Model]
struct SharedLevel4 {
    left: SharedLevel2,
    right: SharedLevel2,
}

#[Model]
struct SharedLevel8 {
    left: SharedLevel4,
    right: SharedLevel4,
}

#[Model]
struct SharedLevel12 {
    left: SharedLevel8,
    right: SharedLevel8,
}

fn reflection_context() -> ReflectionContext {
    ReflectionContext::builder()
        .include_linked()
        .build()
        .expect("linked reflection context")
}

fn previous_algorithm_visits<'ctx>(
    descriptor: TypeDescriptor<'ctx>,
    context: &ResolutionContext<'ctx>,
) -> Result<Vec<TypeId>, ModelMetadataError> {
    let mut ancestors = HashSet::new();
    let mut pending = vec![(descriptor, false)];
    let mut visits = Vec::new();
    while let Some((current, exiting)) = pending.pop() {
        if exiting {
            ancestors.remove(&current.type_id());
            continue;
        }
        if !ancestors.insert(current.type_id()) {
            continue;
        }
        visits.push(current.type_id());
        pending.push((current, true));
        let first_child = pending.len();
        if let Some(metadata) = super::metadata_for_descriptor(current, context)? {
            if matches!(metadata.role(), ModelRole::Entity | ModelRole::Projection) {
                return Ok(visits);
            }
            let variants = metadata
                .as_enum()
                .into_iter()
                .flat_map(|enumeration| enumeration.variants())
                .flat_map(|variant| variant.fields());
            for field in metadata.fields().iter().chain(variants) {
                if !field.is_opaque()
                    && field.reference().is_none()
                    && let Some(child) = metadata
                        .field_descriptor_in(field, context.reflection())
                        .and_then(|field| field.find_concrete_type())
                {
                    pending.push((child, false));
                }
            }
        } else {
            children(current, |edge| pending.push((edge.target, false)));
        }
        pending[first_child..].reverse();
    }
    Ok(visits)
}

#[test]
fn test_nested_role_shared_graph_expands_each_type_once() {
    let root = TypeMetadata::of::<SharedLevel1>();
    let reflection = reflection_context();
    let models = ModelRegistry::from_context(&reflection).expect("model registry");
    let resolution = ResolutionContext::new(&models, &[root]);
    let mut visits = Vec::<TypeId>::new();

    let result = forbidden_entity_nested_role_with_visit(
        TypeDescriptor::of::<SharedLevel1>(),
        &resolution,
        |type_id| visits.push(type_id),
    )
    .expect("metadata lookup succeeds");

    assert_eq!(result, None);
    let unique = visits.iter().copied().collect::<HashSet<_>>();
    assert_eq!(visits.len(), unique.len(), "each descriptor expands once");
    let previous = previous_algorithm_visits(TypeDescriptor::of::<SharedLevel1>(), &resolution)
        .expect("reference traversal has valid metadata");
    assert!(
        previous.len() > visits.len(),
        "the former traversal repeats shared nodes"
    );
}

#[test]
fn test_nested_role_shared_graph_visit_count_tracks_unique_depth() {
    let reflection = reflection_context();
    let models = ModelRegistry::from_context(&reflection).expect("model registry");
    for (descriptor, root, expected) in [
        (
            TypeDescriptor::of::<SharedLevel1>(),
            TypeMetadata::of::<SharedLevel1>(),
            3,
        ),
        (
            TypeDescriptor::of::<SharedLevel4>(),
            TypeMetadata::of::<SharedLevel4>(),
            5,
        ),
        (
            TypeDescriptor::of::<SharedLevel8>(),
            TypeMetadata::of::<SharedLevel8>(),
            6,
        ),
        (
            TypeDescriptor::of::<SharedLevel12>(),
            TypeMetadata::of::<SharedLevel12>(),
            7,
        ),
    ] {
        let resolution = ResolutionContext::new(&models, &[root]);
        let mut visits = Vec::new();
        assert_eq!(
            forbidden_entity_nested_role_with_visit(descriptor, &resolution, |id| visits.push(id))
                .expect("metadata lookup succeeds"),
            None,
        );
        assert_eq!(visits.len(), expected);
    }

    let root = TypeMetadata::of::<SharedLevel1>();
    let resolution = ResolutionContext::new(&models, &[root]);
    for _ in 0..2 {
        let mut visits = Vec::new();
        assert_eq!(
            forbidden_entity_nested_role_with_visit(
                TypeDescriptor::of::<SharedLevel1>(),
                &resolution,
                |id| visits.push(id),
            )
            .expect("metadata lookup succeeds"),
            None,
        );
        assert_eq!(visits.len(), 3, "each query owns a fresh seen set");
    }
}
