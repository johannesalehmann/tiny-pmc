use crate::ModelBuilder;
use crate::choice_labels::{CommandIndexLabels, NoChoiceLabels};
use prism_model::Span;
use probabilistic_models::labels::Labels;
use std::collections::HashMap;
use typed_index_collections::Index;

pub struct ActionNameChoiceLabels<ChoiceIdx: Index, ActionIdx: Index> {
    labels: Labels<ChoiceIdx, ActionIdx, Option<String>>,
    name_to_action: HashMap<String, ActionIdx>,
    none_action: Option<ActionIdx>,
}

impl<ChoiceIdx: Index, ActionIdx: Index> ActionNameChoiceLabels<ChoiceIdx, ActionIdx> {
    pub fn new() -> Self {
        Self {
            labels: Labels::new(),
            name_to_action: HashMap::new(),
            none_action: None,
        }
    }
}

impl<ChoiceIdx: Index, ActionIdx: Index> Default for ActionNameChoiceLabels<ChoiceIdx, ActionIdx> {
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
> ModelBuilder<'a, S, Q, L, IS, B, IB, APs, ActionNameChoiceLabels<B::ChoiceIdx, ActionIdx>>
{
    pub fn without_choice_labels(
        self,
    ) -> ModelBuilder<'a, S, Q, L, IS, B, IB, APs, NoChoiceLabels<B::ChoiceIdx>> {
        self.map_choice_labels(NoChoiceLabels::default())
    }
    pub fn with_command_indices(
        self,
    ) -> ModelBuilder<'a, S, Q, L, IS, B, IB, APs, CommandIndexLabels<B::ChoiceIdx, ActionIdx>>
    {
        self.map_choice_labels(CommandIndexLabels::default())
    }
}

impl<ChoiceIdx: Index, ActionIdx: Index> super::ChoiceLabelBuilder
    for ActionNameChoiceLabels<ChoiceIdx, ActionIdx>
{
    type ChoiceIdx = ChoiceIdx;
    type NameIndex = ActionIdx;
    type ContextType = ();
    type ChoiceLabels = Labels<ChoiceIdx, ActionIdx, Option<String>>;

    fn name_to_index(&mut self, name: Option<&str>) -> Self::NameIndex {
        match name {
            Some(name) => {
                if let Some(action) = self.name_to_action.get(name) {
                    *action
                } else {
                    let index = self.labels.add_action(Some(name.to_string()));
                    self.name_to_action.insert(name.to_string(), index);
                    index
                }
            }
            None => {
                if let Some(action) = &self.none_action {
                    *action
                } else {
                    let index = self.labels.add_action(None);
                    self.none_action = Some(index);
                    index
                }
            }
        }
    }

    fn label_choice(
        &mut self,
        choice: Self::ChoiceIdx,
        name: &Self::NameIndex,
        _context: &Self::ContextType,
    ) {
        self.labels.label_entity(choice, *name);
    }

    fn into_choice_labels(self) -> Self::ChoiceLabels {
        self.labels
    }
}

#[cfg(test)]
mod tests {
    use crate::ModelBuilder;
    use prism_model::{
        Command, Expression, Identifier, Model, ModelType, Module, VariableInfo, VariableRange,
    };
    use probabilistic_models::labels::ReadLabels;
    use probabilistic_models::traits::ReadStateSpace;

    #[test]
    fn test_unlabelled() {
        let mut prism: Model = Model::new(ModelType::mdp());
        let var = VariableInfo::global_var(Identifier::new_unchecked("x"), VariableRange::bool());
        prism.variable_manager.add_variable(var).unwrap();
        let mut module = Module::new(Identifier::new("main").unwrap());
        module
            .commands
            .push(Command::new(None, Expression::bool(true)));
        prism.modules.add(module).unwrap();

        let model = ModelBuilder::new_mdp_builder(&mut prism).build();

        let choices = model.choices();
        assert_eq!(*model.choice_labels.label(choices.index(0)), None);
    }

    #[test]
    fn test_labelled() {
        let mut prism: Model = Model::new(ModelType::mdp());
        let var = VariableInfo::global_var(Identifier::new_unchecked("x"), VariableRange::bool());
        prism.variable_manager.add_variable(var).unwrap();
        let mut module = Module::new(Identifier::new("main").unwrap());
        module.commands.push(Command::new(
            Some(Identifier::new_unchecked("test")),
            Expression::bool(true),
        ));
        prism.modules.add(module).unwrap();

        let model = ModelBuilder::new_mdp_builder(&mut prism).build();

        let choices = model.choices();
        assert_eq!(
            *model.choice_labels.label(choices.index(0)),
            Some("test".to_string())
        );
    }

    #[test]
    fn test_labelled_synchronising() {
        let mut prism: Model = Model::new(ModelType::mdp());
        let var = VariableInfo::global_var(Identifier::new_unchecked("x"), VariableRange::bool());
        prism.variable_manager.add_variable(var).unwrap();
        let mut m1 = Module::new(Identifier::new("m1").unwrap());
        m1.commands.push(Command::new(
            Some(Identifier::new_unchecked("test")),
            Expression::bool(true),
        ));
        prism.modules.add(m1).unwrap();
        let mut m2 = Module::new(Identifier::new("m2").unwrap());
        m2.commands.push(Command::new(
            Some(Identifier::new_unchecked("test")),
            Expression::bool(true),
        ));
        prism.modules.add(m2).unwrap();

        let model = ModelBuilder::new_mdp_builder(&mut prism).build();

        let choices = model.choices();
        assert_eq!(
            *model.choice_labels.label(choices.index(0)),
            Some("test".to_string())
        );
    }
}
