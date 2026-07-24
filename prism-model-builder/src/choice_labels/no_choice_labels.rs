use crate::ModelBuilder;
use crate::choice_labels::{ActionNameChoiceLabels, ChoiceLabelBuilder, CommandIndexLabels};
use prism_model::Span;
use std::marker::PhantomData;
use typed_index_collections::Index;

pub struct NoChoiceLabels<ChoiceIdx> {
    _phantom_data: PhantomData<ChoiceIdx>,
}

impl<ChoiceIdx> NoChoiceLabels<ChoiceIdx> {
    pub fn new() -> Self {
        Self {
            _phantom_data: PhantomData,
        }
    }
}

impl<ChoiceIdx> Default for NoChoiceLabels<ChoiceIdx> {
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
> ModelBuilder<'a, S, Q, L, IS, B, IB, APs, NoChoiceLabels<B::ChoiceIdx>>
{
    pub fn label_choices_with_action_names<ActionIdx: Index>(
        self,
    ) -> ModelBuilder<'a, S, Q, L, IS, B, IB, APs, ActionNameChoiceLabels<B::ChoiceIdx, ActionIdx>>
    {
        self.map_choice_labels(ActionNameChoiceLabels::default())
    }
    pub fn label_choices_with_command_indices<ActionIdx: Index>(
        self,
    ) -> ModelBuilder<'a, S, Q, L, IS, B, IB, APs, CommandIndexLabels<B::ChoiceIdx, ActionIdx>>
    {
        self.map_choice_labels(CommandIndexLabels::default())
    }
}

impl<ChoiceIdx: Index> ChoiceLabelBuilder for NoChoiceLabels<ChoiceIdx> {
    type ChoiceIdx = ChoiceIdx;
    type NameIndex = ();
    type ContextType = ();
    type ChoiceLabels = ();

    fn name_to_index(&mut self, _name: Option<&str>) -> Self::NameIndex {
        ()
    }

    fn label_choice(
        &mut self,
        _choice_index: Self::ChoiceIdx,
        _name: &Self::NameIndex,
        _context: &Self::ContextType,
    ) {
        ()
    }

    fn into_choice_labels(self) -> Self::ChoiceLabels {
        ()
    }
}
