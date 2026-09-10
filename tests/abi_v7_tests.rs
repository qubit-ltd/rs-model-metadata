//! Object navigation preserves parent steps independently of property
//! selection.

#[cfg(feature = "generic")]
use qubit_model_metadata::__private::v7::generic_model_metadata;
#[cfg(feature = "generic")]
use qubit_model_metadata::metadata::ModelRole;
use qubit_model_metadata::metadata::NavigationStep;
use qubit_model_metadata::metadata::ObjectPath;
#[cfg(feature = "generic")]
use qubit_reflect::Reflect;

/// Parent navigation cannot be confused with a dotted property name.
#[test]
fn test_object_path_preserves_navigation_steps() {
    static STEPS: [NavigationStep; 3] = [
        NavigationStep::Parent,
        NavigationStep::Parent,
        NavigationStep::Property("country"),
    ];
    let path = ObjectPath::new(&STEPS).expect("valid relative navigation");
    assert_eq!(path.steps(), &STEPS);
    assert_eq!(path.to_string(), "../../country");
    assert!(path.requires_parent());
}

/// Invalid property segments cannot enter checked metadata.
#[test]
fn test_object_path_rejects_ambiguous_segments() {
    for segment in ["", ".", "..", "country/name", "country.name"] {
        let steps = Box::leak(Box::new([NavigationStep::Property(segment)]));
        assert!(ObjectPath::new(steps).is_err(), "segment: {segment}");
    }
    let current = ObjectPath::new(&[]).expect("current-object dependency");
    assert!(current.steps().is_empty());
    assert!(!current.requires_parent());
}

#[cfg(feature = "generic")]
#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata, definition_provider_v2 = anonymous_definition)]
struct Anonymous<T> {
    value: T,
}

/// Generic model identity is optional independently of reflection identity.
#[cfg(feature = "generic")]
#[test]
fn test_generic_model_definition_accepts_no_stable_id() {
    let Anonymous { value } = Anonymous { value: 7u8 };
    assert_eq!(value, 7);
    let definition = generic_model_metadata(None, ModelRole::Model, anonymous_definition(), &[], &[]);
    assert!(definition.model_id().is_none());
}

/// Returning from a child is local navigation, not external parent context.
#[test]
fn test_balanced_navigation_does_not_require_parent_context() {
    let path = ObjectPath::new(&[NavigationStep::Property("child"), NavigationStep::Parent]).unwrap();
    assert!(!path.requires_parent());
    let external = ObjectPath::new(&[
        NavigationStep::Property("child"),
        NavigationStep::Parent,
        NavigationStep::Parent,
    ])
    .unwrap();
    assert!(external.requires_parent());
}
