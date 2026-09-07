// qubit-style: allow test-file-name
use qubit_reflect::FieldDefinitionDescriptor;
use qubit_reflect::identity::FragmentIdentity;
use qubit_reflect::Reflect;
use qubit_reflect::ReflectRegistry;
use qubit_reflect::TypeDefinitionDescriptor;
use qubit_reflect::TypeDefinitionId;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::expression::TypeExpression;
use qubit_reflect::descriptor::TypeRef;
use qubit_reflect::VariantDefinitionDescriptor;

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct Generic<T> {
    value: T,
}

fn main() {
    let descriptor = TypeDescriptor::of::<Generic<u8>>();
    let _: Option<TypeDefinitionId> = descriptor.definition_id();
    let _: Option<&TypeDefinitionDescriptor> = descriptor.type_definition();
    let _: Option<&FieldDefinitionDescriptor> = descriptor.type_definition().and_then(|value| value.fields()?.first());
    let _: Option<&VariantDefinitionDescriptor> = None;
    let _: Option<&TypeExpression> = None;
    let _: Option<&TypeRef> = None;
    let _: Option<&FragmentIdentity> = ReflectRegistry::initialize()
        .expect("registry")
        .definition_source(descriptor.definition_id().expect("definition"));
}
