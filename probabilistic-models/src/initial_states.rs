use crate::{InitialStates, Model, StateIndex};
use std::marker::PhantomData;
use typed_index_collections::RawIndex;

pub struct SingleInitialState<I: RawIndex> {
    pub index: StateIndex<I>,
}

impl<I: RawIndex, M, ChLabel, BrLabel, Obs, APs, Rew, Ann, Val>
    Model<I, M, (), ChLabel, BrLabel, Obs, APs, Rew, Ann, Val>
{
    pub fn with_initial_state(
        self,
        initial: StateIndex<I>,
    ) -> Model<I, M, SingleInitialState<I>, ChLabel, BrLabel, Obs, APs, Rew, Ann, Val> {
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
            _phantom_data: PhantomData,
        }
    }
    pub fn with_initial_states(
        self,
        initial: InitialStates,
    ) -> Model<I, M, InitialStates, ChLabel, BrLabel, Obs, APs, Rew, Ann, Val> {
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
            _phantom_data: PhantomData,
        }
    }
}

pub trait IsInitial<I: RawIndex> {
    fn is_initial(&self, index: StateIndex<I>) -> bool;
}

impl<I: RawIndex> IsInitial<I> for SingleInitialState<I> {
    fn is_initial(&self, index: StateIndex<I>) -> bool {
        self.index == index
    }
}

impl<I: RawIndex> IsInitial<I> for InitialStates<I> {
    fn is_initial(&self, index: StateIndex<I>) -> bool {
        self[index]
    }
}

impl<I: RawIndex, M, Init: IsInitial<I>, ChLabel, BrLabel, Obs, APs, Rew, Anno, Val>
    Model<I, M, Init, ChLabel, BrLabel, Obs, APs, Rew, Anno, Val>
{
    pub fn is_initial(&self, state: StateIndex<I>) -> bool {
        self.initial.is_initial(state)
    }

    pub fn without_initial_states(
        self,
    ) -> Model<I, M, (), ChLabel, BrLabel, Obs, APs, Rew, Anno, Val> {
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
            _phantom_data: PhantomData,
        }
    }
}
