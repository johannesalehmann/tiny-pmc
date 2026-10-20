use super::{AttractorBuffer, AttractorCondition};
use probabilistic_models::base_model::{ReadOwners, TwoPlayer};
use probabilistic_models::traits::{ReadPredecessors, ReadStateSpace};
use probabilistic_models::{Ix, StateIndex};

struct StateIncludedCondition<I: Ix> {
    state: StateIndex<I>,
}

impl<I: Ix> StateIncludedCondition<I> {
    pub fn new(state: StateIndex<I>) -> Self {
        Self { state }
    }
}

impl<I: Ix> AttractorCondition<I> for StateIncludedCondition<I> {
    type Output = bool;

    fn state_attracted(&mut self, index: StateIndex<I>) -> Option<Self::Output> {
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
    M: ReadStateSpace + ReadOwners<OwnerType = TwoPlayer> + ReadPredecessors,
    R1: Iterator<Item = StateIndex<M::Ix>>,
>(
    model: &M,
    region: R1,
    state: StateIndex<M::Ix>,
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
    M: ReadStateSpace + ReadPredecessors,
    R1: Iterator<Item = StateIndex<M::Ix>>,
>(
    model: &M,
    region: R1,
    state: StateIndex<M::Ix>,
    buffer: &mut AttractorBuffer<M::Ix>,
) -> bool {
    super::attractor_internal_with_buffer(model, region, StateIncludedCondition::new(state), buffer)
}
