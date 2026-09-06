//! Immutable validation binding plans.

#![allow(dead_code)]

use std::any::TypeId;
use std::collections::HashSet;

// qubit-style: allow multiple-public-types

use qubit_validator::BindError;
use qubit_validator::BindErrorKind;
use qubit_validator::BoundValidator;
use qubit_validator::ValidatorId;

use super::ValidationBuildInputs;
use super::compiled_property_path::CompiledPropertyPath;
use super::standard_constraints;
use super::standard_constraints::StandardTarget;
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
    standard_target: Option<StandardTarget>,
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
        let validators = match standard_constraints::registry(inputs.validators) {
            Ok(validators) => validators,
            Err(error) => return Err(vec![error]),
        };
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
                let standard = match standard_constraints::bind(constraint, &validators) {
                    Ok(bindings) => bindings,
                    Err(constraint_errors) => {
                        errors.extend(constraint_errors);
                        Vec::new()
                    }
                };
                for standard in standard {
                    let target = match standard.target {
                        StandardTarget::Value => TargetMode::Value,
                        StandardTarget::SequenceCount => TargetMode::Container,
                    };
                    let value = match CompiledPropertyPath::compile(
                        root,
                        &crate::PropertyPath::new(&[name]),
                        inputs.graph,
                        target,
                    ) {
                        Ok(path) => path,
                        Err(error) => {
                            errors.push(error);
                            continue;
                        }
                    };
                    let rule_id = standard
                        .validator
                        .rule_id()
                        .expect("registry binding sets rule ID");
                    bindings.push(ModelRuleBinding {
                        occurrence: bindings.len(),
                        rule_id,
                        value,
                        dependencies: Box::new([]),
                        validator: standard.validator,
                        on_none: OnNone::Skip,
                        selector: None,
                        standard_target: Some(standard.target),
                    });
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
                    let Some(getter) = property.getter() else {
                        errors.push(BindError::new(BindErrorKind::UnsupportedConstraint));
                        continue;
                    };
                    if !matches!(getter.output_kind(), crate::GetterOutputKind::Borrowed)
                        || !matches!(
                            getter.output_type().as_resolved().map(|value| value.kind()),
                            Some(qubit_reflect::descriptor::TypeKind::Slice)
                        )
                    {
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
                        let validator = match validators.bind(
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
                            standard_target: None,
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
                let validator = match validators.bind(
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
                        standard_target: None,
                    });
                }
            }
            let _ = property;
        }
        let mut stack = HashSet::from([root.type_id()]);
        collect_nested_bindings(
            root,
            root,
            &[],
            inputs.graph,
            &validators,
            &mut bindings,
            &mut errors,
            &mut stack,
        );
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

/// Adds validators declared by fields marked with `#[validate_nested]`.
///
/// Nested declarations are flattened into the root plan so execution keeps a
/// single report and can enforce one set of traversal budgets. Only direct
/// values and optional direct values are traversed; containers require an
/// explicit selector declaration and are rejected here rather than ignored.
#[allow(clippy::too_many_arguments)]
fn collect_nested_bindings<'a>(
    root: &'static TypeMetadata,
    current: &'static TypeMetadata,
    prefix: &[&'static str],
    graph: &'a crate::ResolvedModelGraph<'a>,
    validators: &qubit_validator::ValidatorRegistry,
    bindings: &mut Vec<ModelRuleBinding>,
    errors: &mut Vec<BindError>,
    stack: &mut HashSet<TypeId>,
) {
    for field in current.fields() {
        if !field.validate_nested() {
            continue;
        }
        let Some(name) = field.name() else {
            errors.push(BindError::new(BindErrorKind::UnsupportedConstraint));
            continue;
        };
        let mut path_segments = prefix.to_vec();
        path_segments.push(name);
        let Some(descriptor) = nested_value_descriptor(field.descriptor()) else {
            errors.push(BindError::new(BindErrorKind::UnsupportedConstraint));
            continue;
        };
        let Ok(Some(nested)) = graph.registry().metadata_for(descriptor) else {
            errors.push(BindError::new(BindErrorKind::UnsupportedConstraint));
            continue;
        };
        if !stack.insert(nested.type_id()) {
            errors.push(BindError::new(BindErrorKind::UnsupportedConstraint));
            continue;
        }
        for nested_field in nested.fields() {
            let Some(nested_name) = nested_field.name() else {
                continue;
            };
            let value_segments = path_segments_for(&path_segments, &[nested_name]);
            for declaration in nested_field.validators() {
                let value = match CompiledPropertyPath::compile(
                    root,
                    &crate::PropertyPath::new(&value_segments),
                    graph,
                    declaration.target(),
                ) {
                    Ok(path) => path,
                    Err(error) => {
                        errors.push(error);
                        continue;
                    }
                };
                let validator = match validators.bind(
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
                let declared = declaration.dependency_bindings();
                let legacy = declaration.depends_on();
                let specs = validator.dependency_specs();
                let declared_count = if declared.is_empty() {
                    legacy.len()
                } else {
                    declared.len()
                };
                if specs.len() != declared_count {
                    errors.push(
                        BindError::new(if specs.len() > declared_count {
                            BindErrorKind::MissingDependencyDeclaration
                        } else {
                            BindErrorKind::UnknownDependencyDeclaration
                        })
                        .with_rule(validator.rule_id().expect("registry binding sets rule ID")),
                    );
                    continue;
                }
                let mut dependencies = Vec::new();
                for (slot, spec) in specs.iter().enumerate() {
                    let (dependency_name, dependency) = if declared.is_empty() {
                        let dependency = legacy[slot];
                        (dependency.to_string(), dependency)
                    } else {
                        let Some(binding) = declared
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
                    let dependency_segments =
                        path_segments_for(&path_segments, dependency.segments());
                    match CompiledPropertyPath::compile(
                        root,
                        &crate::PropertyPath::new(&dependency_segments),
                        graph,
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
                        occurrence: bindings.len(),
                        rule_id: validator.rule_id().expect("registry binding sets rule ID"),
                        value,
                        dependencies: dependencies.into_boxed_slice(),
                        validator,
                        on_none: declaration.on_none(),
                        selector: None,
                        standard_target: None,
                    });
                }
            }
        }
        collect_nested_bindings(
            root,
            nested,
            &path_segments,
            graph,
            validators,
            bindings,
            errors,
            stack,
        );
        stack.remove(&nested.type_id());
    }
}

fn path_segments_for(prefix: &[&'static str], suffix: &[&'static str]) -> Vec<&'static str> {
    prefix.iter().chain(suffix).copied().collect()
}

fn nested_value_descriptor(
    mut descriptor: Option<&'static qubit_reflect::TypeDescriptor>,
) -> Option<&'static qubit_reflect::TypeDescriptor> {
    while let Some(value) = descriptor {
        if let Some(element) = value
            .as_optional()
            .map(|view| view.element_type())
            .or_else(|| value.as_smart_pointer().map(|view| view.pointee_type()))
        {
            descriptor = element.as_resolved();
            continue;
        }
        if matches!(
            value.kind(),
            qubit_reflect::descriptor::TypeKind::Struct(_)
                | qubit_reflect::descriptor::TypeKind::Enum
        ) {
            return Some(value);
        }
        return None;
    }
    None
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
    pub(crate) const fn occurrence(&self) -> usize { self.occurrence }
    pub(crate) const fn rule_id(&self) -> ValidatorId { self.rule_id }
    pub(crate) const fn value(&self) -> &CompiledPropertyPath { &self.value }
    pub(crate) fn dependencies(&self) -> &[CompiledPropertyPath] { &self.dependencies }
    pub(crate) const fn validator(&self) -> &BoundValidator { &self.validator }
    pub(crate) const fn on_none(&self) -> OnNone { self.on_none }
    pub(crate) const fn selector(&self) -> Option<&SelectorBinding> { self.selector.as_ref() }
    pub(crate) const fn standard_target(&self) -> Option<StandardTarget> { self.standard_target }
}
