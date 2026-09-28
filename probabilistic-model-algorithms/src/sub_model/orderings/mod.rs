mod attractor;
mod index_based;
mod legacy;

pub use attractor::{Attractor, AttractorChoiceMode};
pub use index_based::{IndexBased, IndexOrderDirection};
pub use legacy::Legacy;

use crate::dominated_by::DominatedByRelation;
use crate::mecs::Mecs;
use crate::sccs::Scc;
use crate::sub_model::RewardsSource;
use probabilistic_models::traits::{ReadPredecessors, ReadStateSpace};
use typed_index_collections::{Index, To1};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SubModelOrder {
    BackToFront,
    FrontToBack,
    Attractor(AttractorChoiceMode),
    Legacy,
}

pub trait StateOrdering {
    type Context<SI: Index, CI: Index>;

    fn create_context<M: ReadStateSpace>(
        &self,
        model: &M,
    ) -> Self::Context<M::StateIndex, M::ChoiceIndex>;

    // Output is stored in to_old_state_index. `context` must be clean when this function is called.
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
        model: &M,
        scc: Scc<'_, ScI, ScEI, SI>,
        dominated_by: &DominatedByRelation<SI>,
        mecs: &Mecs<SI, CI>,
        values: &To1<SI, f64>,
        rewards: &Rew,
        context: &mut Self::Context<SI, CI>,
        to_old_state_index: &mut To1<NewSI, SI>,
    );
}
