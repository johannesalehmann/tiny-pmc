use crate::initial_states::SingleInitialState;
use crate::{InitialStates, RawIndex, StateIndex};
use std::marker::PhantomData;
use typed_index_collections::{Index, To1};

pub trait InitialStatesBuilder: Default {
    type InitialStates;
    type StateIdx: Index;

    // A new state was added (not necessarily initial). Can be used for internal bookkeeping.
    fn state_added(&mut self, index: Self::StateIdx) {}
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

    fn mark_state(&mut self, state: StateIdx) {
        panic!("Cannot mark state as initial when using `UntrackedInitialStatesBuilder`.")
    }

    fn into_initial_states(self) -> Self::InitialStates {
        ()
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

#[derive(Default)]
pub struct MultipleInitialStatesBuilder<StateIdx: Index> {
    states: To1<StateIdx, bool>,
}

impl<StateIdx: Index> InitialStatesBuilder for MultipleInitialStatesBuilder<StateIdx> {
    type InitialStates = InitialStates<StateIdx>;
    type StateIdx = StateIdx;

    fn state_added(&mut self, index: StateIdx) {
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
