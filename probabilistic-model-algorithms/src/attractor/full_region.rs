use super::{AttractorBuffer, AttractorCondition};
use probabilistic_models::owners::TwoPlayer;
use probabilistic_models::traits::{ReadOwners, ReadPredecessors, ReadStateSpace};
use typed_index_collections::{Index, To1};

struct FullRegionAttractorCondition<StateIdx: Index> {
    result: To1<StateIdx, bool>,
}

impl<StateIdx: Index> FullRegionAttractorCondition<StateIdx> {
    pub fn new(model_size: usize) -> Self {
        Self {
            result: To1::with_entries(vec![false; model_size]),
        }
    }
}

impl<StateIdx: Index> AttractorCondition for FullRegionAttractorCondition<StateIdx> {
    type StateIdx = StateIdx;
    type Output = To1<StateIdx, bool>;

    fn state_attracted(&mut self, index: StateIdx) -> Option<Self::Output> {
        self.result[index] = true;
        None
    }

    fn result_after_termination(self) -> Self::Output {
        self.result
    }
}

pub fn attractor<
    M: ReadStateSpace
        + ReadOwners<StateIdx = M::StateIndex, OwnerType = TwoPlayer>
        + ReadPredecessors<
            StateIdx = M::StateIndex,
            ChoiceIdx = M::ChoiceIndex,
            BranchIdx = M::BranchIndex,
        >,
    R1: Iterator<Item = M::StateIndex>,
>(
    model: &M,
    region: R1,
    attracted_player: TwoPlayer,
) -> To1<M::StateIndex, bool> {
    super::attractor_internal(
        model,
        region,
        FullRegionAttractorCondition::new(model.states().len()),
        attracted_player,
    )
}

pub fn attractor_with_buffer<
    M: ReadStateSpace
        + ReadPredecessors<
            StateIdx = M::StateIndex,
            ChoiceIdx = M::ChoiceIndex,
            BranchIdx = M::BranchIndex,
        >,
    R1: Iterator<Item = M::StateIndex>,
>(
    model: &M,
    region: R1,
    buffer: &mut AttractorBuffer<M::StateIndex>,
) -> To1<M::StateIndex, bool> {
    super::attractor_internal_with_buffer(
        model,
        region,
        FullRegionAttractorCondition::new(model.states().len()),
        buffer,
    )
}
