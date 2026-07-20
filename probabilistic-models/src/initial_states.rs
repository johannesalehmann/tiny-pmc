use crate::base_model::BaseModel;
use crate::{InitialStates, Model};
use typed_index_collections::Index;

pub struct SingleInitialState<StateIdx: Index> {
    pub index: StateIdx,
}

// TODO: Create model.map_initial_states() function and use that to simplify this
impl<M: BaseModel, ChLabel, BrLabel, Obs, APs, Rew, Ann, Val, Preds>
    Model<M, (), ChLabel, BrLabel, Obs, APs, Rew, Ann, Val, Preds>
{
    pub fn with_initial_state(
        self,
        initial: M::StateIndex,
    ) -> Model<M, SingleInitialState<M::StateIndex>, ChLabel, BrLabel, Obs, APs, Rew, Ann, Val, Preds>
    {
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
            predecessors: self.predecessors,
        }
    }
    pub fn with_initial_states(
        self,
        initial: InitialStates<M::StateIndex>,
    ) -> Model<M, InitialStates<M::StateIndex>, ChLabel, BrLabel, Obs, APs, Rew, Ann, Val, Preds>
    {
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
            predecessors: self.predecessors,
        }
    }
}

impl<M: BaseModel, ChLabel, BrLabel, Obs, APs, Rew, Anno, Val, Preds>
    Model<M, SingleInitialState<M::StateIndex>, ChLabel, BrLabel, Obs, APs, Rew, Anno, Val, Preds>
{
    pub fn without_initial_states(
        self,
    ) -> Model<M, (), ChLabel, BrLabel, Obs, APs, Rew, Anno, Val, Preds> {
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
            predecessors: self.predecessors,
        }
    }
}

impl<M: BaseModel, ChLabel, BrLabel, Obs, APs, Rew, Anno, Val, Preds>
    Model<M, InitialStates<M::StateIndex>, ChLabel, BrLabel, Obs, APs, Rew, Anno, Val, Preds>
{
    pub fn without_initial_states(
        self,
    ) -> Model<M, (), ChLabel, BrLabel, Obs, APs, Rew, Anno, Val, Preds> {
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
            predecessors: self.predecessors,
        }
    }
}
