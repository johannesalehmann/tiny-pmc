use crate::Model;
use crate::initial_states::{InitialStates, SingleInitialState};
use crate::traits::{AsStateSet, StateSet};
use crate::{HasIx, Ix, StateIndex};

pub trait ReadInitialStates: HasIx {
    fn is_initial(&self, state: StateIndex<Self::Ix>) -> bool;
    fn initial_states(&self) -> impl StateSet<StateIdx = StateIndex<Self::Ix>>;
}

impl<T: ReadInitialStates> ReadInitialStates for &T {
    fn is_initial(&self, state: StateIndex<Self::Ix>) -> bool {
        (**self).is_initial(state)
    }

    fn initial_states(&self) -> impl StateSet<StateIdx = StateIndex<Self::Ix>> {
        (**self).initial_states()
    }
}

impl<T: ReadInitialStates> ReadInitialStates for &mut T {
    fn is_initial(&self, state: StateIndex<Self::Ix>) -> bool {
        (**self).is_initial(state)
    }

    fn initial_states(&self) -> impl StateSet<StateIdx = StateIndex<Self::Ix>> {
        (**self).initial_states()
    }
}

macro_rules! derive_read_initial_states {
    ($subcomponent:ident) => {
        fn is_initial(&self, state: $crate::StateIndex<Self::Ix>) -> bool {
            self.$subcomponent.is_initial(state)
        }

        fn initial_states(
            &self,
        ) -> impl $crate::traits::StateSet<StateIdx = $crate::StateIndex<Self::Ix>> {
            self.$subcomponent.initial_states()
        }
    };
}
pub(crate) use derive_read_initial_states;

impl<I: Ix> ReadInitialStates for SingleInitialState<I> {
    fn is_initial(&self, state: StateIndex<Self::Ix>) -> bool {
        state == self.index
    }

    fn initial_states(&self) -> impl StateSet<StateIdx = StateIndex<I>> {
        self.index.as_state_set()
    }
}

impl<I: Ix> ReadInitialStates for InitialStates<I> {
    fn is_initial(&self, state: StateIndex<Self::Ix>) -> bool {
        self[state]
    }

    fn initial_states(&self) -> impl StateSet<StateIdx = StateIndex<I>> {
        self
    }
}

impl<
    M: HasIx,
    Ini: ReadInitialStates<Ix = M::Ix>,
    ChLabel,
    BrLabel,
    Obs,
    APs,
    Rew,
    Ann,
    StateVals,
    Preds,
> ReadInitialStates for Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    derive_read_initial_states!(initial);
}

pub trait ReadInitialStatesMaybe: HasIx {
    type WithInitialStates: ReadInitialStates<Ix = Self::Ix>;

    fn has_initial_states(&self) -> bool;
    fn try_with_initial_states(self) -> Option<Self::WithInitialStates>;
}

impl<T: ReadInitialStates> ReadInitialStatesMaybe for T {
    type WithInitialStates = T;

    fn has_initial_states(&self) -> bool {
        true
    }

    fn try_with_initial_states(self) -> Option<Self::WithInitialStates> {
        Some(self)
    }
}

impl<T: ReadInitialStates> ReadInitialStatesMaybe for Option<T> {
    type WithInitialStates = T;

    fn has_initial_states(&self) -> bool {
        self.is_some()
    }

    fn try_with_initial_states(self) -> Option<Self::WithInitialStates> {
        self
    }
}

impl<
    M: HasIx,
    Ini: ReadInitialStates<Ix = M::Ix>,
    ChLabel,
    BrLabel,
    Obs,
    APs,
    Rew,
    Ann,
    StateVals,
    Preds,
> ReadInitialStatesMaybe
    for Model<M, Option<Ini>, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    type WithInitialStates = Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>;

    fn has_initial_states(&self) -> bool {
        self.initial.is_some()
    }

    fn try_with_initial_states(self) -> Option<Self::WithInitialStates> {
        Some(Model {
            base: self.base,
            initial: self.initial?,
            choice_labels: self.choice_labels,
            branch_labels: self.branch_labels,
            observations: self.observations,
            atomic_propositions: self.atomic_propositions,
            rewards: self.rewards,
            annotations: self.annotations,
            state_valuations: self.state_valuations,
            predecessors: self.predecessors,
        })
    }
}
