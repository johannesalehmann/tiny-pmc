use super::StateOrdering;
use crate::mecs::Mecs;
use crate::sccs::Scc;
use crate::sub_model::RewardsSource;
use probabilistic_models::traits::{ReadPredecessors, ReadStateSpace};
use probabilistic_models::{Ix, StateIndex};
use typed_index_collections::To1;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum IndexOrderDirection {
    FrontToBack,
    BackToFront,
}

pub struct IndexBased {
    direction: IndexOrderDirection,
}

impl IndexBased {
    pub fn new(direction: IndexOrderDirection) -> Self {
        Self { direction }
    }
}

impl StateOrdering for IndexBased {
    type Context<I: Ix> = ();

    fn create_context<M: ReadStateSpace>(&self, _model: &M) -> Self::Context<M::Ix> {}

    fn compute_ordering<
        M: ReadStateSpace + ReadPredecessors,
        NewI: Ix,
        Rew: RewardsSource<M::Ix>,
    >(
        &self,
        _model: &M,
        scc: Scc<'_, M::Ix>,
        mecs: &Mecs<M::Ix>,
        _values: &To1<StateIndex<M::Ix>, f64>,
        _rewards: &Rew,
        _context: &mut Self::Context<M::Ix>,
        to_old_state_index: &mut To1<StateIndex<NewI>, StateIndex<M::Ix>>,
    ) {
        for state in scc.states() {
            if !mecs.is_merged_away(state) {
                to_old_state_index.add(state);
            }
        }
        let states = to_old_state_index.entries_mut();
        states.sort_unstable();
        if self.direction == IndexOrderDirection::BackToFront {
            states.reverse();
        }
    }
}
