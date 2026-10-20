use super::{AttractorBuffer, AttractorCondition};
use probabilistic_models::base_model::{ReadOwners, TwoPlayer};
use probabilistic_models::traits::{ReadPredecessors, ReadStateSpace};
use probabilistic_models::{Ix, StateIndex};
use typed_index_collections::To1;

struct FullRegionAttractorCondition<I: Ix> {
    result: To1<StateIndex<I>, bool>,
}

impl<I: Ix> FullRegionAttractorCondition<I> {
    pub fn new(model_size: usize) -> Self {
        Self {
            result: To1::with_entries(vec![false; model_size]),
        }
    }
}

impl<I: Ix> AttractorCondition<I> for FullRegionAttractorCondition<I> {
    type Output = To1<StateIndex<I>, bool>;

    fn state_attracted(&mut self, index: StateIndex<I>) -> Option<Self::Output> {
        self.result[index] = true;
        None
    }

    fn result_after_termination(self) -> Self::Output {
        self.result
    }
}

pub fn attractor<
    M: ReadStateSpace + ReadOwners<OwnerType = TwoPlayer> + ReadPredecessors,
    R1: Iterator<Item = StateIndex<M::Ix>>,
>(
    model: &M,
    region: R1,
    attracted_player: TwoPlayer,
) -> To1<StateIndex<M::Ix>, bool> {
    super::attractor_internal(
        model,
        region,
        FullRegionAttractorCondition::new(model.states().len()),
        attracted_player,
    )
}

pub fn attractor_with_buffer<
    M: ReadStateSpace + ReadPredecessors,
    R1: Iterator<Item = StateIndex<M::Ix>>,
>(
    model: &M,
    region: R1,
    buffer: &mut AttractorBuffer<M::Ix>,
) -> To1<StateIndex<M::Ix>, bool> {
    super::attractor_internal_with_buffer(
        model,
        region,
        FullRegionAttractorCondition::new(model.states().len()),
        buffer,
    )
}
