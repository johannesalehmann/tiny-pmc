use crate::{BaseModel, InitialStates, Model, StateIndex};
use std::marker::PhantomData;
use typed_index_collections::{Index, RawIndex};

pub struct SingleInitialState<StateIdx: Index> {
    pub index: StateIdx,
}

impl<M: BaseModel, ChLabel, BrLabel, Obs, APs, Rew, Ann, Val>
    Model<M, (), ChLabel, BrLabel, Obs, APs, Rew, Ann, Val>
{
    pub fn with_initial_state(
        self,
        initial: M::StateIdx,
    ) -> Model<M, SingleInitialState<M::StateIdx>, ChLabel, BrLabel, Obs, APs, Rew, Ann, Val> {
        Model {
            base: self.base,
            initial: SingleInitialState { index: initial },
            choice_labels: self.choice_labels,
            branch_labels: self.branch_labels,
            observations: self.observations,
            atomic_propositions: self.atomic_propositions,
            rewards: self.rewards,
            annotations: self.annotations,
            state_valuations: self.state_valuations,
        }
    }
    pub fn with_initial_states(
        self,
        initial: InitialStates<M::StateIdx>,
    ) -> Model<M, InitialStates<M::StateIdx>, ChLabel, BrLabel, Obs, APs, Rew, Ann, Val> {
        Model {
            base: self.base,
            initial,
            choice_labels: self.choice_labels,
            branch_labels: self.branch_labels,
            observations: self.observations,
            atomic_propositions: self.atomic_propositions,
            rewards: self.rewards,
            annotations: self.annotations,
            state_valuations: self.state_valuations,
        }
    }
}

pub trait IsInitial<StateIdx: Index> {
    fn is_initial(&self, index: StateIdx) -> bool;
}

impl<StateIdx: Index> IsInitial<StateIdx> for SingleInitialState<StateIdx> {
    fn is_initial(&self, index: StateIdx) -> bool {
        self.index == index
    }
}

impl<StateIdx: Index> IsInitial<StateIdx> for InitialStates<StateIdx> {
    fn is_initial(&self, index: StateIdx) -> bool {
        self[index]
    }
}

impl<M: BaseModel, Init: IsInitial<M::StateIdx>, ChLabel, BrLabel, Obs, APs, Rew, Anno, Val>
    Model<M, Init, ChLabel, BrLabel, Obs, APs, Rew, Anno, Val>
{
    pub fn is_initial(&self, state: M::StateIdx) -> bool {
        self.initial.is_initial(state)
    }

    pub fn without_initial_states(
        self,
    ) -> Model<M, (), ChLabel, BrLabel, Obs, APs, Rew, Anno, Val> {
        Model {
            base: self.base,
            initial: (),
            choice_labels: self.choice_labels,
            branch_labels: self.branch_labels,
            observations: self.observations,
            atomic_propositions: self.atomic_propositions,
            rewards: self.rewards,
            annotations: self.annotations,
            state_valuations: self.state_valuations,
        }
    }
}
