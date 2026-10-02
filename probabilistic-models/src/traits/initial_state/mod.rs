use crate::Model;
use crate::initial_states::{InitialStates, SingleInitialState};
use crate::traits::{AsStateSet, StateSet};
use typed_index_collections::Index;

pub trait ReadInitialStates {
    type StateIdx: Index;
    fn is_initial(&self, state: Self::StateIdx) -> bool;
    fn initial_states(&self) -> impl StateSet<StateIdx = Self::StateIdx>;
}

impl<T: ReadInitialStates> ReadInitialStates for &T {
    type StateIdx = T::StateIdx;

    fn is_initial(&self, state: Self::StateIdx) -> bool {
        (**self).is_initial(state)
    }

    fn initial_states(&self) -> impl StateSet<StateIdx = Self::StateIdx> {
        (**self).initial_states()
    }
}

impl<T: ReadInitialStates> ReadInitialStates for &mut T {
    type StateIdx = T::StateIdx;

    fn is_initial(&self, state: Self::StateIdx) -> bool {
        (**self).is_initial(state)
    }

    fn initial_states(&self) -> impl StateSet<StateIdx = Self::StateIdx> {
        (**self).initial_states()
    }
}

macro_rules! derive_read_initial_states {
    ($subcomponent:ident) => {
        fn is_initial(&self, state: Self::StateIdx) -> bool {
            self.$subcomponent.is_initial(state)
        }

        fn initial_states(&self) -> impl $crate::traits::StateSet<StateIdx = Self::StateIdx> {
            self.$subcomponent.initial_states()
        }
    };
}
pub(crate) use derive_read_initial_states;

impl<StateIdx: Index> ReadInitialStates for SingleInitialState<StateIdx> {
    type StateIdx = StateIdx;

    fn is_initial(&self, state: Self::StateIdx) -> bool {
        state == self.index
    }

    fn initial_states(&self) -> impl StateSet<StateIdx = StateIdx> {
        self.index.as_state_set()
    }
}

impl<StateIdx: Index> ReadInitialStates for InitialStates<StateIdx> {
    type StateIdx = StateIdx;

    fn is_initial(&self, state: Self::StateIdx) -> bool {
        self[state]
    }

    fn initial_states(&self) -> impl StateSet<StateIdx = StateIdx> {
        self
    }
}

impl<M, Ini: ReadInitialStates, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
    ReadInitialStates for Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    type StateIdx = Ini::StateIdx;

    derive_read_initial_states!(initial);
}
