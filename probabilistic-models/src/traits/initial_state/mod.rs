use crate::initial_states::SingleInitialState;
use crate::traits::StateSet;
use crate::{InitialStates, Model};
use typed_index_collections::Index;

pub trait ReadInitialStates {
    type StateIdx: Index;
    type InitialStatesIterator<'a>: Iterator<Item = Self::StateIdx>
    where
        Self: 'a;
    fn is_initial(&self, state: Self::StateIdx) -> bool;
    fn initial_states(&self) -> impl StateSet<Self::StateIdx>;
}

impl<StateIdx: Index> ReadInitialStates for SingleInitialState<StateIdx> {
    type StateIdx = StateIdx;
    type InitialStatesIterator<'a>
        = std::iter::Once<StateIdx>
    where
        StateIdx: 'a;

    fn is_initial(&self, state: Self::StateIdx) -> bool {
        state == self.index
    }

    fn initial_states(&self) -> impl StateSet<Self::StateIdx> {
        self.index
    }
}

impl<StateIdx: Index> ReadInitialStates for InitialStates<StateIdx> {
    type StateIdx = StateIdx;
    type InitialStatesIterator<'a>
        = typed_index_collections::To1BoolValuesIterator<
        StateIdx,
        &'a typed_index_collections::To1<StateIdx, bool>,
    >
    where
        StateIdx: 'a;

    fn is_initial(&self, state: Self::StateIdx) -> bool {
        self[state]
    }

    fn initial_states(&self) -> impl StateSet<Self::StateIdx> {
        self
    }
}

impl<M, Ini: ReadInitialStates, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
    ReadInitialStates for Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    type StateIdx = Ini::StateIdx;
    type InitialStatesIterator<'a>
        = Ini::InitialStatesIterator<'a>
    where
        Self: 'a;

    fn is_initial(&self, state: Self::StateIdx) -> bool {
        self.initial.is_initial(state)
    }

    fn initial_states(&self) -> impl StateSet<Self::StateIdx> {
        self.initial.initial_states()
    }
}
