mod monolithic;
pub use monolithic::Monolithic;
use probabilistic_models::StateIndex;

mod topological;
use crate::mecs::Mecs;
use crate::sccs::Scc;
use crate::sub_model::RewardsSource;
use crate::value_iteration::non_determinism::NonDeterminismResolver;
use crate::value_iteration::precomputed_states::PrecomputedStates;
use probabilistic_models::traits::{ReadPredecessors, ReadStateSpace};
pub use topological::{
    EpsAllocationScheme, SccTimingOutput, SubModelHook, SubModelIx, SubModelSolveInfo, Topological,
};
use typed_index_collections::To1;

pub trait OrderedSolver {
    fn find_and_solve_submodels<
        ND: NonDeterminismResolver,
        Solver: super::sub_model_solver::SubModelSolver,
        M: ReadStateSpace + ReadPredecessors,
        P: PrecomputedStates<Ix = M::Ix>,
        Rew: RewardsSource<M::Ix>,
    >(
        self,
        model: &M,
        precomputed_states: &P,
        rewards: Rew,
        mecs: &Mecs<M::Ix>,
        eps: f64,
    ) -> To1<StateIndex<M::Ix>, f64>;
}

#[derive(Clone)]
pub struct ModelSize {
    states: usize,
    choices: usize,
    branches: usize,
}

impl ModelSize {
    pub fn from_scc<M: ReadStateSpace>(model: &M, scc: Scc<'_, M::Ix>) -> Self {
        let states = scc.size();
        let mut choices = 0;
        let mut branches = 0;
        for state in scc.states() {
            let entry_choices = model.choices_of_state(state);
            choices += entry_choices.len();
            for choice in entry_choices {
                branches += model.branches_of_choice(choice).len();
            }
        }
        Self {
            states,
            choices,
            branches,
        }
    }

    pub fn weight(&self) -> usize {
        self.states + self.choices + self.branches
    }

    pub fn fits_u8(&self) -> bool {
        self.states < 256 && self.choices < 256 && self.branches < 256
    }
    pub fn fits_u16(&self) -> bool {
        self.states < 256 * 256 && self.choices < 256 * 256 && self.branches < 256 * 256
    }
    pub fn fits_u32(&self) -> bool {
        self.states < 256 * 256 * 256 * 256
            && self.choices < 256 * 256 * 256 * 256
            && self.branches < 256 * 256 * 256 * 256
    }
}
