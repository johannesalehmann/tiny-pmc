mod algorithm_collections;
pub use algorithm_collections::*;

mod solvable_game;
pub use solvable_game::*;

use typed_index_collections::Index;

pub trait ChangeableOwners<StateIdx: Index> {
    fn set_owner(&mut self, state: StateIdx, owner: probabilistic_models::owners::TwoPlayer);
}

pub struct ReachabilityAlgorithmCollectionWithCachedTarget<StateIdx: Index> {
    pub target_states: Vec<StateIdx>,
    pub buffer: crate::attractor::AttractorBuffer<StateIdx>,
}
