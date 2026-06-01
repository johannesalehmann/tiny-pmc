#[cfg(doc)]
use crate::Model;
use crate::VariableRange;
use crate::spans::Span;

impl<V, S: Span, E, A> super::Model<V, S, E, A> {
    /// Applies mapping function `f` to every expression of the model.
    ///
    /// To map expressions to some other type, use [`Model::map_expressions_cloned()`] or
    /// [`Model::map_expressions_into()`]. However, those functions are more expensive, as they
    /// rebuild the entire model.
    pub fn map_expressions<F: Fn(&mut E)>(&mut self, f: F) {
        for variable in &mut self.variable_manager.variables {
            match &mut variable.range {
                VariableRange::BoundedInt { min, max, .. } => {
                    f(min);
                    f(max);
                }
                _ => (),
            }
            variable.initial_value.as_mut().map(&f);
        }

        for formula in &mut self.formulas.formulas {
            f(&mut formula.condition);
        }

        for module in &mut self.modules.modules {
            for command in &mut module.commands {
                f(&mut command.guard);
                for update in &mut command.updates {
                    f(&mut update.probability);
                    for assignment in &mut update.assignments {
                        f(&mut assignment.value);
                    }
                }
            }
        }

        if let Some(init_constraint) = &mut self.init_constraint {
            f(init_constraint);
        }

        for label in &mut self.labels.labels {
            f(&mut label.condition);
        }

        for reward in &mut self.rewards.rewards {
            for entry in &mut reward.entries {
                f(&mut entry.value);
                f(&mut entry.condition);
            }
        }
    }
}
