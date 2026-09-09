//! Explicit anonymous roots and generic model definitions.

#![cfg(feature = "generic")]

use qubit_id::Id;
use qubit_model_derive::Entity;
use qubit_model_derive::Enum;
use qubit_model_derive::Model;
use qubit_model_derive::Value;
use qubit_model_metadata::metadata::ModelRole;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::ResolveErrorKind;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
#[cfg(feature = "validation")]
use qubit_model_metadata::validation::ValidationBuildInputs;
#[cfg(feature = "validation")]
use qubit_model_metadata::validation::ValidationPlan;
#[cfg(feature = "validation")]
use qubit_validator::ValidatorRegistry;

#[Model]
struct Page<T> {
    items: Vec<T>,
}

/// Registration identity is independent from generic model capability.
#[test]
fn test_anonymous_generic_has_definition() {
    let meta = TypeMetadata::of::<Page<String>>();
    assert!(meta.model_id().is_none());
    assert!(!meta.is_registered());
    assert!(
        meta.generic_definition()
            .expect("generic definition")
            .model_id()
            .is_none()
    );
}

/// Explicit anonymous roots participate without acquiring fabricated IDs.
#[test]
fn test_anonymous_root_is_resolved() {
    let registry = ModelRegistry::global();
    let root = TypeMetadata::of::<Page<String>>();
    let roots = [root];
    let graph = StructureResolver::new(ResolveInputs {
        models: registry,
        roots: &roots,
    })
    .resolve()
    .expect("anonymous root resolves");
    assert!(graph.properties(root).is_some());
}

/// Execution binding accepts an anonymous concrete root without a synthetic ID.
#[cfg(feature = "validation")]
#[test]
fn test_anonymous_root_validation_binds() {
    let registry = ModelRegistry::global();
    let root = TypeMetadata::of::<Page<String>>();
    let roots = [root];
    let graph = StructureResolver::new(ResolveInputs {
        models: registry,
        roots: &roots,
    })
    .resolve()
    .expect("root graph");
    let validators = ValidatorRegistry::empty();
    ValidationPlan::build(
        root,
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    )
    .expect("anonymous validation plan");
}

#[Model]
struct Owner {
    name: String,
}

#[Entity(id = "roots.Record")]
struct Record {
    #[identifier]
    id: Id,
    #[indexed]
    owner: Owner,
    #[indexed]
    owner_name: String,
}

/// Metadata retains direct declarations without imposing flattened filters.
#[test]
fn test_query_declarations_do_not_apply_product_filter_policy() {
    let models = ModelRegistry::global();
    let graph = StructureResolver::new(ResolveInputs { models, roots: &[] })
        .resolve()
        .expect("indexed model is a valid declaration");
    let root = TypeMetadata::of::<Record>();
    let declarations = graph
        .query(root.as_entity().expect("entity"))
        .expect("query declarations")
        .declarations();
    let names: Vec<_> = declarations
        .iter()
        .map(|value| value.field().name().expect("named field"))
        .collect();
    assert_eq!(names, ["id", "owner", "owner_name"]);
}

#[Model]
struct References {
    #[reference(entity = Record, property = "id")]
    records: Option<Vec<Id>>,
}

/// Reference compatibility unwraps supported containers while retaining shape.
#[test]
fn test_optional_sequence_reference_resolves() {
    let models = ModelRegistry::global();
    let root = TypeMetadata::of::<References>();
    let roots = [root];
    let graph = StructureResolver::new(ResolveInputs { models, roots: &roots })
        .resolve()
        .expect("wrapped reference");
    assert!(graph.reference(&root.fields()[0]).is_some());
    assert!(
        root.fields()[0]
            .descriptor()
            .expect("outer shape")
            .as_optional()
            .is_some()
    );
}

#[Enum]
enum ReferencedValue {
    Record(#[reference(entity = Record, property = "id")] Id),
}

#[Value]
struct ClosedValue {
    link: ReferencedValue,
}

/// A Value cannot conceal an entity reference inside an Enum payload.
#[test]
fn test_value_closure_rejects_enum_payload_references() {
    let models = ModelRegistry::global();
    let roots = [TypeMetadata::of::<ClosedValue>()];
    let result = StructureResolver::new(ResolveInputs { models, roots: &roots }).resolve();
    let errors = result.expect_err("Value closure must reject hidden references");
    let error = errors
        .errors()
        .iter()
        .find(|error| error.kind() == ResolveErrorKind::InvalidValueClosure)
        .expect("closure error");
    assert_eq!(error.owner_type_id(), Some(TypeMetadata::of::<ClosedValue>().type_id()));
    let location = error.declaration().expect("payload declaration");
    assert_eq!(location.variant, Some(0));
    assert_eq!(location.field, Some(0));
}

#[Entity(id = "roots.Nation")]
struct Nation {
    #[identifier]
    id: Id,
}

#[Entity(id = "roots.Zone")]
struct Zone {
    #[identifier]
    id: Id,
    #[reference(entity = Nation, property = id)]
    nation: Id,
}

#[Model]
struct BoundAddress {
    #[reference(entity = Zone, property = id)]
    zone: Id,
    #[reference(entity = Nation, property = id, path = "zone/nation")]
    nation: Id,
}

/// Binding paths traverse referenced entities even when fields only store IDs.
#[test]
fn test_binding_path_navigates_entity_behind_saved_id() {
    let models = ModelRegistry::global();
    let root = TypeMetadata::of::<BoundAddress>();
    let roots = [root];
    let graph = StructureResolver::new(ResolveInputs { models, roots: &roots })
        .resolve()
        .expect("binding navigation follows Entity metadata");
    assert_eq!(
        graph.reference(&root.fields()[1]).unwrap().target().type_id(),
        TypeMetadata::of::<Nation>().type_id()
    );
}

#[Model]
struct InvalidDependency {
    #[validator(id = "roots.rule", depends_on(expected(property = missing)))]
    value: String,
}

/// Known local dependency paths are checked without an execution registry.
#[test]
fn test_missing_local_dependency_is_a_structure_error() {
    let models = ModelRegistry::global();
    let roots = [TypeMetadata::of::<InvalidDependency>()];
    let errors = match StructureResolver::new(ResolveInputs { models, roots: &roots }).resolve() {
        Err(errors) => errors,
        Ok(_) => panic!("missing dependency must fail structural resolution"),
    };
    let error = errors
        .errors()
        .iter()
        .find(|error| error.owner_type_id() == Some(roots[0].type_id()))
        .unwrap();
    assert_eq!(error.kind(), ResolveErrorKind::MissingProperty);
    assert_eq!(error.path().unwrap().to_string(), "missing");
    assert!(error.declaration().is_some());
}

#[Enum]
enum UnboundEntityPayload {
    Record(Record),
}

/// Entity payloads need a reference declaration even inside an Enum.
#[test]
fn test_enum_entity_payload_requires_reference() {
    let metadata = TypeMetadata::of::<UnboundEntityPayload>();
    let roots = [metadata];
    let result = StructureResolver::new(ResolveInputs {
        models: ModelRegistry::global(),
        roots: &roots,
    })
    .resolve();
    let errors = match result {
        Err(errors) => errors,
        Ok(_) => panic!("unbound Entity payload must fail"),
    };
    assert!(
        errors
            .errors()
            .iter()
            .any(|error| error.kind() == ResolveErrorKind::InvalidEntityNesting
                && error.owner_type_id() == Some(metadata.type_id()))
    );
}

#[Model]
struct SavedIdDependency {
    #[reference(entity = Record, property = id)]
    owner: Id,
    #[validator(id = "roots.rule", depends_on(expected(path = "owner", property = owner_name)))]
    value: String,
}

/// Validator navigation reads stored values and cannot load a referenced
/// Entity.
#[test]
fn test_validator_navigation_does_not_follow_entity_bindings() {
    let metadata = TypeMetadata::of::<SavedIdDependency>();
    let roots = [metadata];
    let result = StructureResolver::new(ResolveInputs {
        models: ModelRegistry::global(),
        roots: &roots,
    })
    .resolve();
    let errors = match result {
        Err(errors) => errors,
        Ok(_) => panic!("an ID does not expose Entity properties"),
    };
    let error = errors
        .errors()
        .iter()
        .find(|error| error.owner_type_id() == Some(metadata.type_id()))
        .expect("dependency error");
    assert_eq!(error.kind(), ResolveErrorKind::MissingProperty);
    assert_eq!(error.object_path().unwrap().to_string(), "owner");
}

#[Model]
struct NestedDependency {
    child: Option<Box<DependencyChild>>,
    #[validator(id = "roots.rule", depends_on(expected(property = child.name)))]
    value: String,
}

#[Model]
struct DependencyChild {
    name: String,
}

/// Property selection unwraps optional and pointer storage, without loading
/// referenced Entities or traversing collection elements implicitly.
#[test]
fn test_dependency_property_selection_unwraps_storage() {
    let roots = [TypeMetadata::of::<NestedDependency>()];
    let graph = StructureResolver::new(ResolveInputs {
        models: ModelRegistry::global(),
        roots: &roots,
    })
    .resolve()
    .expect("optional boxed child property is structurally readable");
    assert_eq!(graph.dependencies().len(), 1);
}

#[Model]
struct WrongReferenceType {
    #[reference(entity = Record, property = owner_name)]
    value: u32,
}

/// Cross-model type mismatches retain exact identities, roles and source
/// coordinates for consumers of structured diagnostics.
#[test]
fn test_reference_type_mismatch_retains_machine_readable_context() {
    let owner = TypeMetadata::of::<WrongReferenceType>();
    let roots = [owner];
    let errors = StructureResolver::new(ResolveInputs {
        models: ModelRegistry::global(),
        roots: &roots,
    })
    .resolve()
    .expect_err("u32 cannot store the selected String property");
    assert_eq!(errors.to_string(), "1 model resolution error(s)");
    let errors = errors.into_vec();
    let error = &errors[0];
    assert_eq!(error.kind(), ResolveErrorKind::TypeMismatch);
    assert_eq!(error.expected_type(), Some(std::any::TypeId::of::<String>()));
    assert_eq!(error.actual_type(), Some(std::any::TypeId::of::<u32>()));
    assert_eq!(error.expected_role(), Some(ModelRole::Entity));
    assert_eq!(error.actual_role(), Some(ModelRole::Entity));
    assert_eq!(error.owner_type_id(), Some(owner.type_id()));
    assert_eq!(error.declaration().unwrap().field, Some(0));
    assert_eq!(error.path().unwrap().to_string(), "owner_name");
    assert!(error.to_string().contains("TypeMismatch"));
}

#[Model]
struct MissingUniqueScope {
    #[unique(respect_to(missing))]
    value: String,
}

/// A missing uniqueness scope identifies the field that declared it.
#[test]
fn test_unique_scope_error_retains_declaration_location() {
    let roots = [TypeMetadata::of::<MissingUniqueScope>()];
    let errors = StructureResolver::new(ResolveInputs {
        models: ModelRegistry::global(),
        roots: &roots,
    })
    .resolve()
    .expect_err("missing scope");
    let error = &errors.errors()[0];
    assert_eq!(error.kind(), ResolveErrorKind::MissingProperty);
    assert_eq!(error.path().unwrap().to_string(), "missing");
    assert_eq!(error.declaration().unwrap().field, Some(0));
}
