use crate::{
    ModelBuilder, atomic_propositions_builder, bases, choice_labels, initial_states_source, labels,
    queries,
};
use prism_model::Span;
use probabilistic_models::HasIx;
use probabilistic_models::initial_states::{InitialStates, SingleInitialState};
use probabilistic_models::{Ix, StateIndex};
use std::marker::PhantomData;
use typed_index_collections::To1;

pub trait InitialStatesBuilder: Default + HasIx {
    type InitialStates;

    // A new state was added (not necessarily initial). Can be used for internal bookkeeping.
    fn state_added(&mut self, index: StateIndex<Self::Ix>) {
        let _ = index;
    }
    fn stores_initial_states() -> bool;
    fn mark_state(&mut self, state: StateIndex<Self::Ix>);
    fn into_initial_states(self) -> Self::InitialStates;
}

#[derive(Default)]
pub struct UntrackedInitialStatesBuilder<I: Ix> {
    _phantom_data: PhantomData<StateIndex<I>>,
}

impl<I: Ix> HasIx for UntrackedInitialStatesBuilder<I> {
    type Ix = I;
}

impl<I: Ix> InitialStatesBuilder for UntrackedInitialStatesBuilder<I> {
    type InitialStates = ();

    fn stores_initial_states() -> bool {
        false
    }

    fn mark_state(&mut self, _state: StateIndex<I>) {
        panic!("Cannot mark state as initial when using `UntrackedInitialStatesBuilder`.")
    }

    fn into_initial_states(self) -> Self::InitialStates {
        ()
    }
}

impl<
    'a,
    S: Span,
    Q: queries::QueryCollection,
    L: labels::LabelSource,
    IS: initial_states_source::InitialStateSource,
    B: bases::BaseModelBuilder,
    APs: atomic_propositions_builder::AtomicPropositionBuilder<Ix = B::Ix>,
    CL: choice_labels::ChoiceLabelBuilder<Ix = B::Ix>,
    Rew: crate::rewards_builder::RewardsBuilder<Ix = B::Ix>,
> ModelBuilder<'a, S, Q, L, IS, B, UntrackedInitialStatesBuilder<B::Ix>, APs, CL, Rew>
{
    pub fn with_single_initial_state(
        self,
    ) -> ModelBuilder<'a, S, Q, L, IS, B, SingleInitialStatesBuilder<B::Ix>, APs, CL, Rew> {
        self.map_initial_states_builder(SingleInitialStatesBuilder::default())
    }
    pub fn with_initial_state_vector(
        self,
    ) -> ModelBuilder<'a, S, Q, L, IS, B, MultipleInitialStatesBuilder<B::Ix>, APs, CL, Rew> {
        self.map_initial_states_builder(MultipleInitialStatesBuilder::default())
    }
}

#[derive(Default)]
pub struct SingleInitialStatesBuilder<I: Ix> {
    state: Option<StateIndex<I>>,
}

impl<I: Ix> HasIx for SingleInitialStatesBuilder<I> {
    type Ix = I;
}

impl<I: Ix> InitialStatesBuilder for SingleInitialStatesBuilder<I> {
    type InitialStates = SingleInitialState<I>;

    fn stores_initial_states() -> bool {
        true
    }

    fn mark_state(&mut self, state: StateIndex<I>) {
        if self.state.is_some() {
            panic!("Cannot mark a second state as initial when using `SingleInitialStatesBuilder`.")
        }
        self.state = Some(state);
    }

    fn into_initial_states(self) -> Self::InitialStates {
        match self.state {
            None => {
                panic!(
                    "No state was marked initial. This is not legal when using `SingleInitialStatesBuilder`."
                )
            }
            Some(index) => SingleInitialState { index },
        }
    }
}

impl<
    'a,
    S: Span,
    Q: queries::QueryCollection,
    L: labels::LabelSource,
    IS: initial_states_source::InitialStateSource,
    B: bases::BaseModelBuilder,
    APs: atomic_propositions_builder::AtomicPropositionBuilder<Ix = B::Ix>,
    CL: choice_labels::ChoiceLabelBuilder<Ix = B::Ix>,
    Rew: crate::rewards_builder::RewardsBuilder<Ix = B::Ix>,
> ModelBuilder<'a, S, Q, L, IS, B, SingleInitialStatesBuilder<B::Ix>, APs, CL, Rew>
{
    pub fn without_initial_states(
        self,
    ) -> ModelBuilder<'a, S, Q, L, IS, B, UntrackedInitialStatesBuilder<B::Ix>, APs, CL, Rew> {
        self.map_initial_states_builder(UntrackedInitialStatesBuilder::default())
    }
    pub fn with_initial_state_vector(
        self,
    ) -> ModelBuilder<'a, S, Q, L, IS, B, MultipleInitialStatesBuilder<B::Ix>, APs, CL, Rew> {
        self.map_initial_states_builder(MultipleInitialStatesBuilder::default())
    }
}

#[derive(Default)]
pub struct MultipleInitialStatesBuilder<I: Ix> {
    states: To1<StateIndex<I>, bool>,
}

impl<I: Ix> HasIx for MultipleInitialStatesBuilder<I> {
    type Ix = I;
}

impl<I: Ix> InitialStatesBuilder for MultipleInitialStatesBuilder<I> {
    type InitialStates = InitialStates<I>;

    fn state_added(&mut self, _index: StateIndex<I>) {
        self.states.add(false);
    }

    fn stores_initial_states() -> bool {
        true
    }

    fn mark_state(&mut self, state: StateIndex<I>) {
        self.states[state] = true;
    }

    fn into_initial_states(self) -> Self::InitialStates {
        self.states
    }
}

impl<
    'a,
    S: Span,
    Q: queries::QueryCollection,
    L: labels::LabelSource,
    IS: initial_states_source::InitialStateSource,
    B: bases::BaseModelBuilder,
    APs: atomic_propositions_builder::AtomicPropositionBuilder<Ix = B::Ix>,
    CL: choice_labels::ChoiceLabelBuilder<Ix = B::Ix>,
    Rew: crate::rewards_builder::RewardsBuilder<Ix = B::Ix>,
> ModelBuilder<'a, S, Q, L, IS, B, MultipleInitialStatesBuilder<B::Ix>, APs, CL, Rew>
{
    pub fn without_initial_states(
        self,
    ) -> ModelBuilder<'a, S, Q, L, IS, B, UntrackedInitialStatesBuilder<B::Ix>, APs, CL, Rew> {
        self.map_initial_states_builder(UntrackedInitialStatesBuilder::default())
    }
    pub fn with_single_initial_state(
        self,
    ) -> ModelBuilder<'a, S, Q, L, IS, B, SingleInitialStatesBuilder<B::Ix>, APs, CL, Rew> {
        self.map_initial_states_builder(SingleInitialStatesBuilder::default())
    }
}
