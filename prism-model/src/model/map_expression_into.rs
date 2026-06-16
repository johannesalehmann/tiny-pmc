#[cfg(doc)]
use crate::Model;
use crate::spans::Span;
use crate::{
    Assignment, Command, Formula, FormulaManager, Label, LabelManager, Module, ModuleManager,
    Rewards, RewardsElement, RewardsManager, Update, VariableInfo, VariableManager, VariableRange,
};

impl<V, S: Span, E, A> super::Model<V, S, E, A> {
    /// Constructs a new model from `self` by applying mapping function `f` to every expression.
    ///
    /// `self` is consumed in this process. This has the advantage of avoiding unnecessary clones.
    /// However, this function still performs significant allocation, as all the vectors of the
    /// model (e.g. those in [`FormulaManager`], [`VariableManager`] and [`ModuleManager`] need to
    /// be reconstructed.
    ///
    /// If you still need the original model, consider using [`Model::map_expressions_cloned()`].
    ///
    /// If `E2 = E`, i.e. `f` does not change the type of expression, consider using
    /// [`Model::map_expressions()`], which modifies the existing model instead of constructing a
    /// new model.
    pub fn map_expressions_into<E2, F: Fn(E) -> E2>(self, f: F) -> super::Model<V, S, E2, A> {
        let mut variables = Vec::new();
        for variable in self.variable_manager.variables {
            let range = match variable.range {
                VariableRange::BoundedInt { min, max, span } => VariableRange::BoundedInt {
                    min: f(min),
                    max: f(max),
                    span,
                },

                VariableRange::UnboundedInt { span } => VariableRange::UnboundedInt { span },
                VariableRange::Boolean { span } => VariableRange::Boolean { span },
                VariableRange::Float { span } => VariableRange::Float { span },
            };
            let initial_value = variable.initial_value.map(|i| f(i));
            variables.push(VariableInfo {
                scope: variable.scope,
                range,
                name: variable.name,
                initial_value,
                span: variable.span,
                attributes: variable.attributes,
            });
        }
        let variable_manager = VariableManager { variables };

        let formulas = FormulaManager::with_formulas_unchecked(
            self.formulas
                .into_iter()
                .map(|formula| Formula {
                    name: formula.name,
                    condition: f(formula.condition),
                    span: formula.span,
                    attributes: formula.attributes,
                })
                .collect::<Vec<_>>(),
        );

        let mut modules = Vec::new();

        for module in self.modules {
            let mut commands = Vec::new();
            for command in module.commands {
                let mut updates = Vec::new();
                for update in command.updates {
                    updates.push(Update {
                        probability: f(update.probability),
                        assignments: update
                            .assignments
                            .into_iter()
                            .map(|assg| Assignment {
                                target: assg.target,
                                value: f(assg.value),
                                target_span: assg.target_span,
                                span: assg.span,
                            })
                            .collect(),
                        span: update.span,
                    });
                }
                commands.push(Command {
                    action: command.action,
                    action_span: command.action_span,
                    guard: f(command.guard),
                    updates,
                    updates_span: command.updates_span,
                    span: command.span,
                    attributes: command.attributes,
                })
            }
            modules.push(Module {
                name: module.name,
                commands,
                span: module.span,
                attributes: module.attributes,
            })
        }
        let modules = ModuleManager::with_modules_unchecked(modules);

        let init_constraint = self.init_constraint.map(|i| f(i));

        let labels = LabelManager::with_labels_unchecked(
            self.labels
                .into_iter()
                .map(|label| Label {
                    name: label.name,
                    condition: f(label.condition),
                    span: label.span,
                    attributes: label.attributes,
                })
                .collect(),
        );

        let mut rewards = Vec::new();
        for reward in self.rewards {
            let entries = reward
                .entries
                .into_iter()
                .map(|reward_element| RewardsElement {
                    condition: f(reward_element.condition),
                    value: f(reward_element.value),
                    target: reward_element.target,
                    span: reward_element.span,
                })
                .collect();
            rewards.push(Rewards {
                name: reward.name,
                entries,
                span: reward.span,
                attributes: reward.attributes,
            });
        }
        let rewards = RewardsManager::with_rewards_unchecked(rewards);

        super::Model {
            model_type: self.model_type,
            variable_manager,
            formulas,
            modules,
            renamed_modules: self.renamed_modules,
            init_constraint,
            labels,
            rewards,
            span: self.span,
        }
    }
}
