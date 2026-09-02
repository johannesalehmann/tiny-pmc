use crate::base_model::BaseModel;
use crate::base_model::transition_system::TransitionSystem;
use crate::owners::TwoPlayer;
use crate::traits::derive_read_state_space;
use crate::{ChoiceIndex, StateIndex};
use typed_index_collections::{Index, To1};

#[derive(Default)]
pub struct NonstochasticGame<StateIdx: Index, ChoiceIdx: Index> {
    pub base_transition_system: TransitionSystem<StateIdx, ChoiceIdx>,
    pub owners: To1<StateIdx, TwoPlayer>,
}

impl<StateIdx: Index, ChoiceIdx: Index> super::ReadStateSpace
    for NonstochasticGame<StateIdx, ChoiceIdx>
{
    type StateIdx = StateIdx;
    type ChoiceIdx = ChoiceIdx;
    type BranchIdx = ChoiceIdx;

    derive_read_state_space!(base_transition_system);
}

impl<SI: Index, CI: Index> BaseModel for NonstochasticGame<SI, CI> {
    type StateIndex = SI;
    type ChoiceIndex = CI;
    type BranchIndex = CI;
}

impl NonstochasticGame<StateIndex<usize>, ChoiceIndex<usize>> {
    pub fn with_default_types() -> Self {
        Self::default()
    }
}

impl<SI: Index, CI: Index> NonstochasticGame<SI, CI> {
    pub fn add_state(&mut self, owner: TwoPlayer) -> SI {
        let state_index = self.base_transition_system.add_state();
        self.owners.add_checked(state_index, owner);
        state_index
    }
    pub fn add_transition(&mut self, destination: SI) -> CI {
        self.base_transition_system.add_transition(destination)
    }
}
