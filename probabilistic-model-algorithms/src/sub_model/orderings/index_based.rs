use super::StateOrdering;
use crate::dominated_by::DominatedByRelation;
use crate::mecs::Mecs;
use crate::sccs::Scc;
use crate::sub_model::RewardsSource;
use probabilistic_models::traits::{ReadPredecessors, ReadStateSpace};
use typed_index_collections::{Index, To1};

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
    type Context<SI: Index, CI: Index> = ();

    fn create_context<M: ReadStateSpace>(
        &self,
        _model: &M,
    ) -> Self::Context<M::StateIndex, M::ChoiceIndex> {
    }

    fn compute_ordering<
        M: ReadStateSpace<StateIndex = SI, ChoiceIndex = CI, BranchIndex = BI>
            + ReadPredecessors<StateIdx = SI, ChoiceIdx = CI, BranchIdx = BI>,
        SI: Index,
        CI: Index,
        BI: Index,
        ScI: Index,
        ScEI: Index,
        NewSI: Index,
        Rew: RewardsSource<SI, CI>,
    >(
        &self,
        _model: &M,
        scc: Scc<'_, ScI, ScEI, SI>,
        dominated_by: &DominatedByRelation<SI>,
        mecs: &Mecs<SI, CI>,
        _values: &To1<SI, f64>,
        _rewards: &Rew,
        _context: &mut Self::Context<SI, CI>,
        to_old_state_index: &mut To1<NewSI, SI>,
    ) {
        for state in scc.states() {
            if dominated_by.dominated_by(state).is_none() && !mecs.is_merged_away(state) {
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
