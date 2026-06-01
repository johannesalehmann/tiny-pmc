use crate::spans::Span;
use crate::{CyclicDependency, Expression, Identifier, IdentityMapExpression, VariableManager};

impl<S: Span, A> super::Model<Identifier<S>, S, Expression<Identifier<S>, S>, A> {
    /// Expands the formulas in every expression of the model. Afterward, the model has no formulas
    /// and no expression references any formula.
    ///
    /// If there is a cyclic dependency between formulas, returns [`CyclicDependency`].
    ///
    /// # Example
    ///
    /// Consider the following model.
    ///
    /// ```prism
    /// mdp
    /// formula limit = base + 2;
    /// formula base = 3;
    /// label "at_limit" = x = limit;
    /// module m
    ///     x: [0..10] init base;
    ///     [] x < limit -> 1.0: (x'=x+1);
    /// endmodule
    /// ```
    ///
    /// Calling `substitute_formulas()` results in the following model:
    ///
    /// ```prism
    /// mdp
    /// label "at_limit" = x = 3 + 2;
    /// module m
    ///     x: [0..10] init 3;
    ///     [] x < 3 + 2 -> 1.0: (x'=x+1);
    /// endmodule
    /// ```
    pub fn substitute_formulas(&mut self) -> Result<(), CyclicDependency<S>> {
        let order = self.formulas.get_formula_replacement_ordering()?;

        for formula_index in order {
            let formula = self.formulas.get(formula_index).unwrap();
            let mut visitor = FormulaSubstitutionVisitor {
                formula_name: &formula.name,
                expression: &formula.condition,
            };

            let mut replace_expression = |e: &mut Expression<Identifier<S>, S>| {
                let condition = std::mem::replace(e, Expression::Bool(false, S::empty()));
                *e = condition.visit(&mut visitor);
            };

            for label in &mut self.labels.labels {
                replace_expression(&mut label.condition);
            }
            for reward in &mut self.rewards.rewards {
                for entry in &mut reward.entries {
                    replace_expression(&mut entry.condition);
                    replace_expression(&mut entry.value);
                }
            }
            if let Some(init_constraint) = &mut self.init_constraint {
                replace_expression(init_constraint);
            }
            for module in &mut self.modules.modules {
                for command in &mut module.commands {
                    replace_expression(&mut command.guard);
                    for update in &mut command.updates {
                        replace_expression(&mut update.probability);
                        for assignment in &mut update.assignments {
                            replace_expression(&mut assignment.value);
                        }
                    }
                }
            }
            let mut replace_in_var_defs = |vm: &mut VariableManager<
                S,
                Expression<Identifier<S>, S>,
            >| {
                for variable in &mut vm.variables {
                    if let Some(initial_value) = &mut variable.initial_value {
                        replace_expression(initial_value);
                    }
                    if let crate::VariableRange::BoundedInt { min, max, .. } = &mut variable.range {
                        replace_expression(min);
                        replace_expression(max);
                    }
                }
            };
            replace_in_var_defs(&mut self.variable_manager);
        }

        self.formulas.formulas.clear();

        Ok(())
    }
}

pub struct FormulaSubstitutionVisitor<'a, S: Span> {
    pub formula_name: &'a Identifier<S>,
    pub expression: &'a Expression<Identifier<S>, S>,
}

impl<'a, S: Span> crate::expressions::identity_map::Private for FormulaSubstitutionVisitor<'a, S> {}
impl<'a, S: Span> IdentityMapExpression<Identifier<S>, S> for FormulaSubstitutionVisitor<'a, S> {
    fn visit_var_or_const(&mut self, name: Identifier<S>, span: S) -> Expression<Identifier<S>, S> {
        if &name == self.formula_name {
            self.expression.clone()
        } else {
            Expression::VarOrConst(name, span)
        }
    }
}
