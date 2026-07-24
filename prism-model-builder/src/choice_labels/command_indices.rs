use crate::ModelBuilder;
use crate::choice_labels::{ActionNameChoiceLabels, NoChoiceLabels};
use prism_model::Span;
use probabilistic_models::labels::Labels;
use std::collections::HashMap;
use typed_index_collections::Index;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum CommandIndices {
    Unsynchronised {
        module_index: usize,
        command_index: usize,
    },
    Synchronised {
        components: Vec<CommandIndexComponent>,
    },
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct CommandIndexComponent {
    pub module_index: usize,
    pub command_index: usize,
}

pub struct CommandIndexLabels<ChoiceIdx: Index, ActionIdx: Index> {
    labels: Labels<ChoiceIdx, ActionIdx, CommandIndices>,
    label_to_index: HashMap<CommandIndices, ActionIdx>,
}

impl<ChoiceIdx: Index, ActionIdx: Index> CommandIndexLabels<ChoiceIdx, ActionIdx> {
    pub fn new() -> Self {
        Self {
            labels: Labels::new(),
            label_to_index: HashMap::new(),
        }
    }
}

impl<ChoiceIdx: Index, ActionIdx: Index> Default for CommandIndexLabels<ChoiceIdx, ActionIdx> {
    fn default() -> Self {
        Self::new()
    }
}

impl<
    'a,
    S: Span,
    Q: crate::queries::QueryCollection,
    L: crate::labels::LabelSource,
    IS: crate::initial_states_source::InitialStateSource,
    B: crate::bases::BaseModelBuilder,
    IB: crate::initial_states_builder::InitialStatesBuilder<StateIdx = B::StateIdx>,
    APs: crate::atomic_propositions_builder::AtomicPropositionBuilder<StateIdx = B::StateIdx>,
    ActionIdx: Index,
> ModelBuilder<'a, S, Q, L, IS, B, IB, APs, CommandIndexLabels<B::ChoiceIdx, ActionIdx>>
{
    pub fn without_choice_labels(
        self,
    ) -> ModelBuilder<'a, S, Q, L, IS, B, IB, APs, NoChoiceLabels<B::ChoiceIdx>> {
        self.map_choice_labels(NoChoiceLabels::default())
    }
    pub fn with_action_names(
        self,
    ) -> ModelBuilder<'a, S, Q, L, IS, B, IB, APs, ActionNameChoiceLabels<B::ChoiceIdx, ActionIdx>>
    {
        self.map_choice_labels(ActionNameChoiceLabels::default())
    }
}

impl<ChoiceIdx: Index, ActionIdx: Index> super::ChoiceLabelBuilder
    for CommandIndexLabels<ChoiceIdx, ActionIdx>
{
    type ChoiceIdx = ChoiceIdx;
    type NameIndex = ();
    type ContextType = CommandIndices;
    type ChoiceLabels = Labels<ChoiceIdx, ActionIdx, CommandIndices>;

    fn name_to_index(&mut self, _name: Option<&str>) -> Self::NameIndex {
        ()
    }

    fn label_choice(
        &mut self,
        choice: Self::ChoiceIdx,
        _name: &Self::NameIndex,
        context: &Self::ContextType,
    ) {
        let action = if let Some(action) = self.label_to_index.get(context) {
            *action
        } else {
            let action = self.labels.add_action(context.clone());
            self.label_to_index.insert(context.clone(), action);
            action
        };
        self.labels.label_entity(choice, action);
    }

    fn into_choice_labels(self) -> Self::ChoiceLabels {
        self.labels
    }
}

impl super::Context for CommandIndices {
    fn new_unsynchronised(module_index: usize, command_index: usize) -> Self {
        CommandIndices::Unsynchronised {
            module_index,
            command_index,
        }
    }

    fn new_synchronised(component_count: usize) -> Self {
        CommandIndices::Synchronised {
            components: vec![
                CommandIndexComponent {
                    module_index: 0,
                    command_index: 0,
                };
                component_count
            ],
        }
    }

    fn set_synchronised_component(
        &mut self,
        component_index: usize,
        module_index: usize,
        command_index: usize,
    ) {
        if let CommandIndices::Synchronised { components } = self {
            components[component_index] = CommandIndexComponent {
                module_index,
                command_index,
            };
        } else {
            panic!("Cannot set synchronised component on unsynchronised choice label component");
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ModelBuilder;
    use crate::choice_labels::{CommandIndexComponent, CommandIndices};
    use prism_model::{
        Assignment, Command, Expression, Identifier, Model, ModelType, Module, Update,
        VariableInfo, VariableRange,
    };
    use probabilistic_models::labels::ReadLabels;
    use probabilistic_models::traits::ReadStateSpace;

    #[test]
    fn test_unsynchronised() {
        let mut prism: Model = Model::new(ModelType::mdp());
        let var = VariableInfo::global_var(Identifier::new_unchecked("x"), VariableRange::bool());
        let var_ref = prism.variable_manager.add_variable(var).unwrap();
        prism
            .modules
            .add(Module::new(Identifier::new_unchecked("dummy")))
            .unwrap();
        let mut module = Module::new(Identifier::new_unchecked("main"));
        module.commands.push(Command::new(
            None,
            Expression::var_or_const(var_ref).equals_to(Expression::bool(true)),
        ));
        module.commands.push(Command::with_updates(
            None,
            Expression::var_or_const(var_ref).equals_to(Expression::bool(false)),
            vec![Update::with_assignments(
                Expression::float(1.0),
                vec![Assignment::new(var_ref, Expression::bool(true))],
            )],
        ));
        prism.modules.add(module).unwrap();

        let model = ModelBuilder::new_mdp_builder(&mut prism)
            .with_command_indices()
            .build();
        let states = model.states();
        assert_eq!(states.len(), 2);
        let choices = model.choices_of_state(states.index(0));
        assert_eq!(choices.len(), 1);
        assert_eq!(
            *model.choice_labels.label(choices.index(0)),
            CommandIndices::Unsynchronised {
                module_index: 1,
                command_index: 1
            }
        );
        let choices = model.choices_of_state(states.index(1));
        assert_eq!(choices.len(), 1);
        assert_eq!(
            *model.choice_labels.label(choices.index(0)),
            CommandIndices::Unsynchronised {
                module_index: 1,
                command_index: 0
            }
        );
    }

    #[test]
    fn test_synchronised() {
        let mut prism: Model = Model::new(ModelType::mdp());
        let var = VariableInfo::global_var(Identifier::new_unchecked("x"), VariableRange::bool());
        let var_ref = prism.variable_manager.add_variable(var).unwrap();
        prism
            .modules
            .add(Module::new(Identifier::new_unchecked("dummy")))
            .unwrap();

        let mut m1 = Module::new(Identifier::new_unchecked("m1"));
        m1.commands.push(Command::new(
            Some(Identifier::new_unchecked("alpha")),
            Expression::var_or_const(var_ref).equals_to(Expression::bool(true)),
        ));
        m1.commands.push(Command::with_updates(
            Some(Identifier::new_unchecked("beta")),
            Expression::bool(true),
            vec![Update::with_assignments(
                Expression::float(1.0),
                vec![Assignment::new(var_ref, Expression::bool(true))],
            )],
        ));
        prism.modules.add(m1).unwrap();

        let mut m2 = Module::new(Identifier::new_unchecked("m2"));
        m2.commands.push(Command::new(
            Some(Identifier::new_unchecked("beta")),
            Expression::var_or_const(var_ref).equals_to(Expression::bool(false)),
        ));
        m2.commands.push(Command::new(
            Some(Identifier::new_unchecked("alpha")),
            Expression::bool(true),
        ));
        prism.modules.add(m2).unwrap();

        let model = ModelBuilder::new_mdp_builder(&mut prism)
            .with_command_indices()
            .build();
        let states = model.states();
        assert_eq!(states.len(), 2);
        let choices = model.choices_of_state(states.index(0));
        assert_eq!(choices.len(), 1);
        assert_eq!(
            *model.choice_labels.label(choices.index(0)),
            CommandIndices::Synchronised {
                components: vec![
                    CommandIndexComponent {
                        module_index: 1,
                        command_index: 1,
                    },
                    CommandIndexComponent {
                        module_index: 2,
                        command_index: 0,
                    }
                ]
            }
        );
        let choices = model.choices_of_state(states.index(1));
        assert_eq!(choices.len(), 1);
        assert_eq!(
            *model.choice_labels.label(choices.index(0)),
            CommandIndices::Synchronised {
                components: vec![
                    CommandIndexComponent {
                        module_index: 1,
                        command_index: 0,
                    },
                    CommandIndexComponent {
                        module_index: 2,
                        command_index: 1,
                    }
                ]
            }
        );
    }
}
