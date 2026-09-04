// qubit-style: allow test-file-name
use qubit_model_metadata::FieldDefinitionDescriptor;
use qubit_model_metadata::FragmentIdentity;
use qubit_model_metadata::Reflect;
use qubit_model_metadata::ReflectRegistry;
use qubit_model_metadata::TypeDefinitionDescriptor;
use qubit_model_metadata::TypeDefinitionId;
use qubit_model_metadata::TypeDescriptor;
use qubit_model_metadata::TypeExpression;
use qubit_model_metadata::TypeRef;
use qubit_model_metadata::VariantDefinitionDescriptor;

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
