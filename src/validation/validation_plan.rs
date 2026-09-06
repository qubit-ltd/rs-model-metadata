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
}

/// A read-only plan containing no model instance or getter output.
pub struct ValidationPlan<'a> {
    root: &'static TypeMetadata,
    graph: &'a crate::ResolvedModelGraph<'a>,
    bindings: Box<[ModelRuleBinding]>,
}

impl<'a> ValidationPlan<'a> {
    /// Binds all direct field validator declarations on `root`.
    pub fn build(root: &'static TypeMetadata, inputs: ValidationBuildInputs<'a>) -> Result<Self, Vec<BindError>> {
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
                let selectors = match constraint {
                    crate::ConstraintMetadata::Sequence(sequence) => sequence.element().into_iter().collect::<Vec<_>>(),
                    crate::ConstraintMetadata::Map(map) => map.key().into_iter().chain(map.value()).collect::<Vec<_>>(),
                    _ => Vec::new(),
                };
                for selector in selectors {
                    for declaration in selector.validators() {
                        errors.push(
                            BindError::new(BindErrorKind::UnsupportedInput)
                                .with_rule(ValidatorId::new(declaration.declared_id())),
                        );
                    }
                }
            }
            for (occurrence, declaration) in field.validators().iter().enumerate() {
                let segments = [name];
                let path = crate::PropertyPath::new(&segments);
                let value = match CompiledPropertyPath::compile(root, &path, inputs.graph, declaration.target()) {
                    Ok(path) => path,
                    Err(error) => {
                        errors.push(error);
                        continue;
                    }
                };
                let validator =
                    match inputs
                        .validators
                        .bind(declaration.declared_id(), value.input_type(), declaration.params())
                    {
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
                                    .with_rule(validator.rule_id().expect("registry binding sets rule ID"))
                                    .with_dependency(spec.name()),
                            );
                            continue;
                        };
                        (binding.name().to_owned(), binding.path())
                    };
                    if spec.name() != dependency_name {
                        errors.push(
                            BindError::new(BindErrorKind::UnknownDependencyDeclaration)
                                .with_rule(validator.rule_id().expect("registry binding sets rule ID"))
                                .with_dependency(spec.name()),
                        );
                        continue;
                    }
                    match CompiledPropertyPath::compile(root, &dependency, inputs.graph, TargetMode::Value) {
                        Ok(path) => dependencies.push(path),
                        Err(error) => errors.push(
                            error
                                .with_rule(validator.rule_id().expect("registry binding sets rule ID"))
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
}
