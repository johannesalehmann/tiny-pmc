use crate::initial_states::SingleInitialState;
use crate::to1::To1;
use crate::{InitialStates, RawIndex, StateIndex};
use std::marker::PhantomData;

pub trait InitialStatesBuilder: Default {
    type InitialStates;
    type Index: RawIndex;

    // A new state was added (not necessarily initial). Can be used for internal bookkeeping.
    fn state_added(&mut self, index: StateIndex<Self::Index>) {}
    fn stores_initial_states() -> bool;
    fn mark_state(&mut self, state: StateIndex<Self::Index>);
    fn into_initial_states(self) -> Self::InitialStates;
}

#[derive(Default)]
pub struct UntrackedInitialStatesBuilder<I: RawIndex> {
    _phantom_data: PhantomData<I>,
}

impl<I: RawIndex> InitialStatesBuilder for UntrackedInitialStatesBuilder<I> {
    type InitialStates = ();
    type Index = I;

    fn stores_initial_states() -> bool {
        false
    }

    fn mark_state(&mut self, state: StateIndex<Self::Index>) {
        panic!("Cannot mark state as initial when using `UntrackedInitialStatesBuilder`.")
    }

    fn into_initial_states(self) -> Self::InitialStates {
        ()
    }
}

#[derive(Default)]
pub struct SingleInitialStatesBuilder<I: RawIndex> {
    state: Option<StateIndex<I>>,
}

impl<I: RawIndex> InitialStatesBuilder for SingleInitialStatesBuilder<I> {
    type InitialStates = SingleInitialState<I>;
    type Index = I;

    fn stores_initial_states() -> bool {
        true
    }

    fn mark_state(&mut self, state: StateIndex<Self::Index>) {
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

pub struct MultipleInitialStatesBuilder<I: RawIndex> {
    states: To1<StateIndex<I>, bool>,
}

impl<I: RawIndex> Default for MultipleInitialStatesBuilder<I> {
    fn default() -> Self {
        Self { states: To1::new() }
    }
}

impl<I: RawIndex> InitialStatesBuilder for MultipleInitialStatesBuilder<I> {
    type InitialStates = InitialStates<I>;
    type Index = I;

    fn state_added(&mut self, index: StateIndex<Self::Index>) {
        self.states.add_unchecked(false);
    }

    fn stores_initial_states() -> bool {
        true
    }

    fn mark_state(&mut self, state: StateIndex<Self::Index>) {
        self.states[state] = true;
    }

    fn into_initial_states(self) -> Self::InitialStates {
        self.states
    }
}
