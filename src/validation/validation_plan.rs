//! Immutable validation binding plans.

#![allow(dead_code)]

// qubit-style: allow multiple-public-types

use qubit_validator::BindError;
use qubit_validator::BindErrorKind;
use qubit_validator::BoundValidator;
use qubit_validator::ValidatorId;

use super::ValidationBuildInputs;
use super::compiled_property_path::CompiledPropertyPath;
use crate::OnNone;
use crate::SelectorPosition;
use crate::TargetMode;
use crate::TypeMetadata;

/// One declaration bound to an executable validator and compiled paths.
#[derive(Clone, Debug)]
pub(crate) struct ModelRuleBinding {
    occurrence: usize,
    rule_id: ValidatorId,
    value: CompiledPropertyPath,
    dependencies: Box<[CompiledPropertyPath]>,
    validator: BoundValidator,
    on_none: OnNone,
    selector: Option<SelectorBinding>,
}

/// A validator bound to a sequence selector.
#[derive(Clone, Debug)]
pub(crate) struct SelectorBinding {
    position: SelectorPosition,
}

impl SelectorBinding {
    pub(crate) const fn position(&self) -> SelectorPosition {
        self.position
    }
}

/// A read-only plan containing no model instance or getter output.
pub struct ValidationPlan<'a> {
    root: &'static TypeMetadata,
    graph: &'a crate::ResolvedModelGraph<'a>,
    bindings: Box<[ModelRuleBinding]>,
}

impl<'a> ValidationPlan<'a> {
    /// Binds all direct field validator declarations on `root`.
    pub fn build(
        root: &'static TypeMetadata,
        inputs: ValidationBuildInputs<'a>,
    ) -> Result<Self, Vec<BindError>> {
        let mut bindings = Vec::new();
        let mut errors = Vec::new();
        let Some(properties) = inputs.graph.properties(root) else {
            return Err(vec![BindError::new(BindErrorKind::UnreadablePath)]);
        };
        for field in root.fields() {
            let Some(name) = field.name() else {
                continue;
            };
            let Some(property) = properties.property(name) else {
                continue;
            };
            for constraint in field.constraints() {
                match constraint {
                    crate::ConstraintMetadata::Sequence(sequence)
                        if sequence.min_items() == Some(0)
                            && sequence.max_items().is_none()
                            && !sequence.unique_items() => {}
                    crate::ConstraintMetadata::Sequence(_)
                    | crate::ConstraintMetadata::Map(_)
                    | crate::ConstraintMetadata::Text(_)
                    | crate::ConstraintMetadata::Decimal(_)
                    | crate::ConstraintMetadata::Time(_) => {
                        errors.push(BindError::new(BindErrorKind::UnsupportedConstraint));
                    }
                }
                let selectors = match constraint {
                    crate::ConstraintMetadata::Sequence(sequence) => {
                        sequence.element().into_iter().collect::<Vec<_>>()
                    }
                    crate::ConstraintMetadata::Map(map) => {
                        map.key().into_iter().chain(map.value()).collect::<Vec<_>>()
                    }
                    _ => Vec::new(),
                };
                for selector in selectors {
                    if !matches!(selector.position(), SelectorPosition::Element) {
                        errors.push(BindError::new(BindErrorKind::UnsupportedConstraint));
                        continue;
                    }
                    let Some(descriptor) =
                        selector_descriptor(property.descriptor(), selector.position())
                    else {
                        errors.push(BindError::new(BindErrorKind::UnsupportedConstraint));
                        continue;
                    };
                    if !selector.constraints().is_empty() {
                        errors.push(BindError::new(BindErrorKind::UnsupportedConstraint));
                    }
                    for declaration in selector.validators() {
                        let input = selector_input_type(descriptor, declaration.target());
                        let validator = match inputs.validators.bind(
                            declaration.declared_id(),
                            input,
                            declaration.params(),
                        ) {
                            Ok(validator) => validator,
                            Err(error) => {
                                errors.push(error);
                                continue;
                            }
                        };
                        if !validator.dependency_specs().is_empty()
                            || !declaration.depends_on().is_empty()
                        {
                            errors.push(
                                BindError::new(BindErrorKind::UnsupportedConstraint).with_rule(
                                    validator.rule_id().expect("registry binding sets rule ID"),
                                ),
                            );
                            continue;
                        }
                        bindings.push(ModelRuleBinding {
                            occurrence: bindings.len(),
                            rule_id: validator.rule_id().expect("registry binding sets rule ID"),
                            value: match CompiledPropertyPath::compile(
                                root,
                                &crate::PropertyPath::new(&[name]),
                                inputs.graph,
                                TargetMode::Container,
                            ) {
                                Ok(path) => path,
                                Err(error) => {
                                    errors.push(error);
                                    continue;
                                }
                            },
                            dependencies: Box::new([]),
                            validator,
                            on_none: declaration.on_none(),
                            selector: Some(SelectorBinding {
                                position: selector.position(),
                            }),
                        });
                    }
                }
            }
            for (occurrence, declaration) in field.validators().iter().enumerate() {
                let segments = [name];
                let path = crate::PropertyPath::new(&segments);
                let value = match CompiledPropertyPath::compile(
                    root,
                    &path,
                    inputs.graph,
                    declaration.target(),
                ) {
                    Ok(path) => path,
                    Err(error) => {
                        errors.push(error);
                        continue;
                    }
                };
                let validator = match inputs.validators.bind(
                    declaration.declared_id(),
                    value.input_type(),
                    declaration.params(),
                ) {
                    Ok(validator) => validator,
                    Err(error) => {
                        errors.push(error);
                        continue;
                    }
                };
                let mut dependencies = Vec::new();
                let specs = validator.dependency_specs();
                let declared_dependencies = declaration.dependency_bindings();
                let legacy_dependencies = declaration.depends_on();
                if specs.len()
                    != if declared_dependencies.is_empty() {
                        legacy_dependencies.len()
                    } else {
                        declared_dependencies.len()
                    }
                {
                    errors.push(
                        BindError::new(
                            if specs.len()
                                > if declared_dependencies.is_empty() {
                                    legacy_dependencies.len()
                                } else {
                                    declared_dependencies.len()
                                }
                            {
                                BindErrorKind::MissingDependencyDeclaration
                            } else {
                                BindErrorKind::UnknownDependencyDeclaration
                            },
                        )
                        .with_rule(validator.rule_id().expect("registry binding sets rule ID")),
                    );
                    continue;
                }
                for (slot, spec) in specs.iter().enumerate() {
                    let (dependency_name, dependency) = if declared_dependencies.is_empty() {
                        let dependency = legacy_dependencies[slot];
                        (dependency.to_string(), dependency)
                    } else {
                        let Some(binding) = declared_dependencies
                            .iter()
                            .find(|binding| binding.name() == spec.name())
                        else {
                            errors.push(
                                BindError::new(BindErrorKind::UnknownDependencyDeclaration)
                                    .with_rule(
                                        validator.rule_id().expect("registry binding sets rule ID"),
                                    )
                                    .with_dependency(spec.name()),
                            );
                            continue;
                        };
                        (binding.name().to_owned(), binding.path())
                    };
                    if spec.name() != dependency_name {
                        errors.push(
                            BindError::new(BindErrorKind::UnknownDependencyDeclaration)
                                .with_rule(
                                    validator.rule_id().expect("registry binding sets rule ID"),
                                )
                                .with_dependency(spec.name()),
                        );
                        continue;
                    }
                    match CompiledPropertyPath::compile(
                        root,
                        &dependency,
                        inputs.graph,
                        TargetMode::Value,
                    ) {
                        Ok(path) => dependencies.push(path),
                        Err(error) => errors.push(
                            error
                                .with_rule(
                                    validator.rule_id().expect("registry binding sets rule ID"),
                                )
                                .with_dependency(spec.name()),
                        ),
                    }
                }
                if dependencies.len() == specs.len() {
                    bindings.push(ModelRuleBinding {
                        occurrence,
                        rule_id: validator.rule_id().expect("registry binding sets rule ID"),
                        value,
                        dependencies: dependencies.into_boxed_slice(),
                        validator,
                        on_none: declaration.on_none(),
                        selector: None,
                    });
                }
            }
            let _ = property;
        }
        if errors.is_empty() {
            Ok(Self {
                root,
                graph: inputs.graph,
                bindings: bindings.into_boxed_slice(),
            })
        } else {
            Err(errors)
        }
    }

    /// Returns the number of bound validator occurrences.
    #[must_use]
    pub const fn binding_count(&self) -> usize {
        self.bindings.len()
    }

    /// Returns the model root this plan was built for.
    #[must_use]
    pub const fn root(&self) -> &'static TypeMetadata {
        self.root
    }

    /// Returns the structure graph retained by this plan.
    pub const fn graph(&self) -> &'a crate::ResolvedModelGraph<'a> {
        self.graph
    }

    /// Returns the immutable bound occurrences for the executor.
    #[must_use]
    pub(crate) fn bindings(&self) -> &[ModelRuleBinding] {
        &self.bindings
    }
}

fn selector_descriptor(
    descriptor: Option<&'static qubit_reflect::TypeDescriptor>,
    position: SelectorPosition,
) -> Option<&'static qubit_reflect::TypeDescriptor> {
    let descriptor = descriptor?;
    let descriptor = transparent_descriptor(descriptor)?;
    let type_ref = match position {
        SelectorPosition::Element => descriptor
            .as_sequence()
            .map(|view| view.element_type())
            .or_else(|| descriptor.as_set().map(|view| view.element_type()))
            .or_else(|| descriptor.as_array().map(|view| view.element_type()))
            .or_else(|| descriptor.as_slice().map(|view| view.element_type())),
        SelectorPosition::MapKey => descriptor.as_map().map(|view| view.key_type()),
        SelectorPosition::MapValue => descriptor.as_map().map(|view| view.value_type()),
    }?;
    transparent_descriptor(type_ref.as_resolved()?)
}

fn transparent_descriptor(
    mut descriptor: &'static qubit_reflect::TypeDescriptor,
) -> Option<&'static qubit_reflect::TypeDescriptor> {
    loop {
        let type_ref = descriptor
            .as_optional()
            .map(|view| view.element_type())
            .or_else(|| {
                descriptor
                    .as_smart_pointer()
                    .map(|view| view.pointee_type())
            });
        let Some(type_ref) = type_ref else {
            return Some(descriptor);
        };
        descriptor = type_ref.as_resolved()?;
    }
}

fn selector_input_type(
    descriptor: &'static qubit_reflect::TypeDescriptor,
    target: TargetMode,
) -> qubit_validator::InputType {
    let descriptor = if matches!(target, TargetMode::Value) {
        transparent_descriptor(descriptor).unwrap_or(descriptor)
    } else {
        descriptor
    };
    if matches!(
        descriptor.kind(),
        qubit_reflect::descriptor::TypeKind::Text(_)
    ) {
        qubit_validator::InputType::Text
    } else {
        qubit_validator::InputType::Typed(descriptor.type_id())
    }
}

impl ModelRuleBinding {
    pub(crate) const fn occurrence(&self) -> usize {
        self.occurrence
    }
    pub(crate) const fn rule_id(&self) -> ValidatorId {
        self.rule_id
    }
    pub(crate) const fn value(&self) -> &CompiledPropertyPath {
        &self.value
    }
    pub(crate) fn dependencies(&self) -> &[CompiledPropertyPath] {
        &self.dependencies
    }
    pub(crate) const fn validator(&self) -> &BoundValidator {
        &self.validator
    }
    pub(crate) const fn on_none(&self) -> OnNone {
        self.on_none
    }
    pub(crate) const fn selector(&self) -> Option<&SelectorBinding> {
        self.selector.as_ref()
    }
}
