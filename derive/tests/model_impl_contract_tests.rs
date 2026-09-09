//! Model implementation methods preserve reflection independently of
//! properties.

use model_runtime::metadata::TypeMetadata;
use qubit_model_derive::Model;
use qubit_model_derive::ModelImpl;
use qubit_reflect::Reflect;
use qubit_reflect::ReflectedRef;
use qubit_reflect::reflect;
use qubit_reflect::registry::ReflectRegistry;

#[Model]
struct Person {
    name: String,
}

#[ModelImpl]
impl Person {
    /// Excludes this getter while preserving its same-named stored field.
    #[model_property(skip)]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Retains reflection while excluding the computed property.
    #[model_property(skip)]
    pub fn diagnostic(&self) -> usize {
        self.name.len()
    }

    /// Provides a computed property without a separate marker.
    pub fn full_name(&self) -> String {
        self.name.clone()
    }

    /// A business operation whose name happens to start with set_.
    pub fn set_schedule(&self, _day: u32, _hour: u32) {}
}

/// Exclusion removes only the method contribution.
#[test]
fn test_exclusion_preserves_computed_and_stored_properties() {
    let meta = TypeMetadata::of::<Person>();
    assert!(meta.try_property("diagnostic").expect("properties").is_none());
    assert!(
        meta.try_property("full_name")
            .expect("properties")
            .expect("computed")
            .is_computed()
    );
    assert!(
        meta.try_property("name")
            .expect("properties")
            .expect("field")
            .is_writable()
    );
    let person = Person { name: "Ada".into() };
    assert_eq!(person.diagnostic(), 3);
    assert_eq!(person.name(), "Ada");
    person.set_schedule(1, 2);
    let reflection = ReflectRegistry::initialize().expect("reflection");
    assert!(
        reflection
            .implementations(std::any::TypeId::of::<Person>())
            .iter()
            .flat_map(|implementation| implementation.methods())
            .any(|method| method.rust_name() == "diagnostic")
    );
}

#[reflect]
trait Greeting {
    /// Provides trait behavior, independently of model properties.
    fn greeting(&self) -> String;
}

#[ModelImpl]
impl Greeting for Person {
    /// Contributes reflected trait behavior without a computed property.
    fn greeting(&self) -> String {
        self.name.clone()
    }
}

/// Trait methods remain reflected but do not become inherent properties.
#[test]
fn test_trait_methods_do_not_contribute_properties() {
    assert!(
        TypeMetadata::of::<Person>()
            .try_property("greeting")
            .expect("properties")
            .is_none()
    );
    assert_eq!(Person { name: "Ada".into() }.greeting(), "Ada");
    let reflection = ReflectRegistry::initialize().expect("reflection");
    assert!(
        reflection
            .implementations(std::any::TypeId::of::<Person>())
            .iter()
            .flat_map(|implementation| implementation.methods())
            .any(|method| method.rust_name() == "greeting")
    );
}

#[Model]
struct GenericName<T> {
    value: T,
}

#[ModelImpl(specialize(T = String), specialize(T = u32))]
impl<T: Clone + Reflect> GenericName<T> {
    /// A computed property specialized using the reflection support boundary.
    pub fn current(&self) -> T {
        self.value.clone()
    }
}

/// Generic impl specializations keep concrete property identities separate.
#[test]
fn test_generic_impl_specializations_have_exact_property_types() {
    let text = TypeMetadata::of::<GenericName<String>>();
    let number = TypeMetadata::of::<GenericName<u32>>();
    let text_property = text
        .try_property("current")
        .expect("properties")
        .expect("text property");
    let number_property = number
        .try_property("current")
        .expect("properties")
        .expect("number property");
    assert_eq!(
        text_property.descriptor().expect("type").type_id(),
        std::any::TypeId::of::<String>()
    );
    assert_eq!(
        number_property.descriptor().expect("type").type_id(),
        std::any::TypeId::of::<u32>()
    );
    let value = GenericName { value: 7u32 };
    assert!(number_property.get(ReflectedRef::new(&value)).is_ok());
}

#[Model]
struct Fixed<const N: usize> {
    bytes: [u8; N],
}

#[ModelImpl(specialize(N = 2))]
impl<const N: usize> Fixed<N> {
    /// Reports the concrete const-specialized storage length.
    pub fn length(&self) -> usize {
        self.bytes.len()
    }
}

/// Const substitutions include generic argument positions, not only arrays.
#[test]
fn test_const_impl_specialization() {
    let metadata = TypeMetadata::of::<Fixed<2>>();
    let property = metadata.try_property("length").unwrap().unwrap();
    assert!(property.get(ReflectedRef::new(&Fixed { bytes: [1, 2] })).is_ok());
}

#[Model]
struct SplitAccessors {
    value: String,
}

#[ModelImpl]
impl SplitAccessors {
    pub fn title(&self) -> String {
        self.value.clone()
    }
}

#[ModelImpl]
impl SplitAccessors {
    pub fn set_title(&mut self, value: String) {
        self.value = value;
    }
}

/// Separate inherent blocks contribute to one checked property set.
#[test]
fn test_multiple_impl_blocks_merge_accessors() {
    let metadata = TypeMetadata::of::<SplitAccessors>();
    let properties = metadata.try_properties().expect("split accessors assemble");
    let title = properties.property("title").expect("computed property");
    assert!(title.getter().is_some());
    assert!(title.setter().is_some());
    assert!(std::ptr::eq(properties, metadata.try_properties().expect("cached")));
}
