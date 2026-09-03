use super::{AttractorBuffer, AttractorCondition};
use probabilistic_models::owners::TwoPlayer;
use probabilistic_models::traits::{ReadOwners, ReadPredecessors, ReadStateSpace};
use typed_index_collections::Index;

struct StateIncludedCondition<StateIdx: Index> {
    state: StateIdx,
}

impl<StateIdx: Index> StateIncludedCondition<StateIdx> {
    pub fn new(state: StateIdx) -> Self {
        Self { state }
    }
}

impl<StateIdx: Index> AttractorCondition for StateIncludedCondition<StateIdx> {
    type StateIdx = StateIdx;
    type Output = bool;

    fn state_attracted(&mut self, index: StateIdx) -> Option<Self::Output> {
        if index == self.state {
            Some(true)
        } else {
            None
        }
    }

    fn result_after_termination(self) -> Self::Output {
        false
    }
}

pub fn attractor_contains_state<
    M: ReadStateSpace
        + ReadOwners<StateIdx = <M as ReadStateSpace>::StateIdx, OwnerType = TwoPlayer>
        + ReadPredecessors<
            StateIdx = <M as ReadStateSpace>::StateIdx,
            ChoiceIdx = <M as ReadStateSpace>::ChoiceIdx,
            BranchIdx = <M as ReadStateSpace>::BranchIdx,
        >,
    R1: Iterator<Item = <M as ReadStateSpace>::StateIdx>,
>(
    model: &M,
    region: R1,
    state: <M as ReadStateSpace>::StateIdx,
    attracted_player: TwoPlayer,
) -> bool {
    super::attractor_internal(
        model,
        region,
        StateIncludedCondition::new(state),
        attracted_player,
    )
}

pub fn attractor_contains_state_with_buffer<
    M: ReadStateSpace
        + ReadPredecessors<
            StateIdx = <M as ReadStateSpace>::StateIdx,
            ChoiceIdx = <M as ReadStateSpace>::ChoiceIdx,
            BranchIdx = <M as ReadStateSpace>::BranchIdx,
        >,
    R1: Iterator<Item = <M as ReadStateSpace>::StateIdx>,
>(
    model: &M,
    region: R1,
    state: <M as ReadStateSpace>::StateIdx,
    buffer: &mut AttractorBuffer<<M as ReadStateSpace>::StateIdx>,
) -> bool {
    super::attractor_internal_with_buffer(model, region, StateIncludedCondition::new(state), buffer)
}
