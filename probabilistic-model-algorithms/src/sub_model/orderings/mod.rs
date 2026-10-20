mod attractor;
mod index_based;
mod legacy;

use crate::mecs::Mecs;
use crate::sccs::Scc;
use crate::sub_model::RewardsSource;
pub use attractor::{Attractor, AttractorChoiceMode};
pub use index_based::{IndexBased, IndexOrderDirection};
pub use legacy::Legacy;
use probabilistic_models::traits::{ReadPredecessors, ReadStateSpace};
use probabilistic_models::{Ix, StateIndex};
use typed_index_collections::To1;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SubModelOrder {
    BackToFront,
    FrontToBack,
    Attractor(AttractorChoiceMode),
    Legacy,
}

pub trait StateOrdering {
    type Context<I: Ix>;

    fn create_context<M: ReadStateSpace>(&self, model: &M) -> Self::Context<M::Ix>;

    // Output is stored in to_old_state_index. `context` must be clean when this function is called.
    fn compute_ordering<M: ReadStateSpace + ReadPredecessors, NewI: Ix, Rew: RewardsSource<M::Ix>>(
        &self,
        model: &M,
        scc: Scc<'_, M::Ix>,
        mecs: &Mecs<M::Ix>,
        values: &To1<StateIndex<M::Ix>, f64>,
        rewards: &Rew,
        context: &mut Self::Context<M::Ix>,
        to_old_state_index: &mut To1<StateIndex<NewI>, StateIndex<M::Ix>>,
    );
}
