use crate::{
    ModelBuilder, atomic_propositions_builder, bases, initial_states_builder,
    initial_states_source, labels, queries,
};
use prism_model::Span;
use probabilistic_models::InitialStates;
use probabilistic_models::initial_states::SingleInitialState;
use std::marker::PhantomData;
use typed_index_collections::{Index, To1};

pub trait InitialStatesBuilder: Default {
    type InitialStates;
    type StateIdx: Index;

    // A new state was added (not necessarily initial). Can be used for internal bookkeeping.
    fn state_added(&mut self, index: Self::StateIdx) {
        let _ = index;
    }
    fn stores_initial_states() -> bool;
    fn mark_state(&mut self, state: Self::StateIdx);
    fn into_initial_states(self) -> Self::InitialStates;
}

#[derive(Default)]
pub struct UntrackedInitialStatesBuilder<StateIdx: Index> {
    _phantom_data: PhantomData<StateIdx>,
}

impl<StateIdx: Index> InitialStatesBuilder for UntrackedInitialStatesBuilder<StateIdx> {
    type InitialStates = ();
    type StateIdx = StateIdx;

    fn stores_initial_states() -> bool {
        false
    }

    fn mark_state(&mut self, _state: StateIdx) {
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
    APs: atomic_propositions_builder::AtomicPropositionBuilder<StateIdx = B::StateIdx>,
> ModelBuilder<'a, S, Q, L, IS, B, UntrackedInitialStatesBuilder<B::StateIdx>, APs>
{
    pub fn with_single_initial_state(
        self,
    ) -> ModelBuilder<'a, S, Q, L, IS, B, SingleInitialStatesBuilder<B::StateIdx>, APs> {
        self.map_initial_states_builder(SingleInitialStatesBuilder::default())
    }
    pub fn with_initial_state_vector(
        self,
    ) -> ModelBuilder<'a, S, Q, L, IS, B, MultipleInitialStatesBuilder<B::StateIdx>, APs> {
        self.map_initial_states_builder(MultipleInitialStatesBuilder::default())
    }
}

#[derive(Default)]
pub struct SingleInitialStatesBuilder<StateIdx: Index> {
    state: Option<StateIdx>,
}

impl<StateIdx: Index> InitialStatesBuilder for SingleInitialStatesBuilder<StateIdx> {
    type InitialStates = SingleInitialState<StateIdx>;
    type StateIdx = StateIdx;

    fn stores_initial_states() -> bool {
        true
    }

    fn mark_state(&mut self, state: StateIdx) {
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
    APs: atomic_propositions_builder::AtomicPropositionBuilder<StateIdx = B::StateIdx>,
> ModelBuilder<'a, S, Q, L, IS, B, SingleInitialStatesBuilder<B::StateIdx>, APs>
{
    pub fn without_initial_states(
        self,
    ) -> ModelBuilder<'a, S, Q, L, IS, B, UntrackedInitialStatesBuilder<B::StateIdx>, APs> {
        self.map_initial_states_builder(UntrackedInitialStatesBuilder::default())
    }
    pub fn with_initial_state_vector(
        self,
    ) -> ModelBuilder<'a, S, Q, L, IS, B, MultipleInitialStatesBuilder<B::StateIdx>, APs> {
        self.map_initial_states_builder(MultipleInitialStatesBuilder::default())
    }
}

#[derive(Default)]
pub struct MultipleInitialStatesBuilder<StateIdx: Index> {
    states: To1<StateIdx, bool>,
}

impl<StateIdx: Index> InitialStatesBuilder for MultipleInitialStatesBuilder<StateIdx> {
    type InitialStates = InitialStates<StateIdx>;
    type StateIdx = StateIdx;

    fn state_added(&mut self, _index: StateIdx) {
        self.states.add(false);
    }

    fn stores_initial_states() -> bool {
        true
    }

    fn mark_state(&mut self, state: StateIdx) {
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
    APs: atomic_propositions_builder::AtomicPropositionBuilder<StateIdx = B::StateIdx>,
> ModelBuilder<'a, S, Q, L, IS, B, MultipleInitialStatesBuilder<B::StateIdx>, APs>
{
    pub fn without_initial_states(
        self,
    ) -> ModelBuilder<'a, S, Q, L, IS, B, UntrackedInitialStatesBuilder<B::StateIdx>, APs> {
        self.map_initial_states_builder(UntrackedInitialStatesBuilder::default())
    }
    pub fn with_single_initial_state(
        self,
    ) -> ModelBuilder<'a, S, Q, L, IS, B, SingleInitialStatesBuilder<B::StateIdx>, APs> {
        self.map_initial_states_builder(SingleInitialStatesBuilder::default())
    }
}
