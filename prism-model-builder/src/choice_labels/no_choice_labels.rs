use crate::ModelBuilder;
use crate::choice_labels::{ActionNameChoiceLabels, ChoiceLabelBuilder, CommandIndexLabels};
use prism_model::Span;
use probabilistic_models::{ChoiceIndex, HasIx, Ix};
use std::marker::PhantomData;

pub struct NoChoiceLabels<I: Ix> {
    _phantom_data: PhantomData<I>,
}

impl<I: Ix> HasIx for NoChoiceLabels<I> {
    type Ix = I;
}

impl<I: Ix> NoChoiceLabels<I> {
    pub fn new() -> Self {
        Self {
            _phantom_data: PhantomData,
        }
    }
}

impl<I: Ix> Default for NoChoiceLabels<I> {
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
    IB: crate::initial_states_builder::InitialStatesBuilder<Ix = B::Ix>,
    APs: crate::atomic_propositions_builder::AtomicPropositionBuilder<Ix = B::Ix>,
    Rew: crate::rewards_builder::RewardsBuilder<Ix = B::Ix>,
> ModelBuilder<'a, S, Q, L, IS, B, IB, APs, NoChoiceLabels<B::Ix>, Rew>
{
    pub fn label_choices_with_action_names(
        self,
    ) -> ModelBuilder<'a, S, Q, L, IS, B, IB, APs, ActionNameChoiceLabels<B::Ix>, Rew> {
        self.map_choice_labels(ActionNameChoiceLabels::default())
    }
    pub fn label_choices_with_command_indices(
        self,
    ) -> ModelBuilder<'a, S, Q, L, IS, B, IB, APs, CommandIndexLabels<B::Ix>, Rew> {
        self.map_choice_labels(CommandIndexLabels::default())
    }
}

impl<I: Ix> ChoiceLabelBuilder for NoChoiceLabels<I> {
    type NameIndex = ();
    type ContextType = ();
    type ChoiceLabels = ();

    fn name_to_index(&mut self, _name: Option<&str>) -> Self::NameIndex {
        ()
    }

    fn label_choice(
        &mut self,
        _choice_index: ChoiceIndex<Self::Ix>,
        _name: &Self::NameIndex,
        _context: &Self::ContextType,
    ) {
        ()
    }

    fn into_choice_labels(self) -> Self::ChoiceLabels {
        ()
    }
}
