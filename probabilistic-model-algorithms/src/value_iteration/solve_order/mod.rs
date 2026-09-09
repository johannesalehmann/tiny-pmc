mod monolithic;
pub use monolithic::Monolithic;

mod topological;
pub use topological::Topological;

use crate::dominated_by::DominatedByRelation;
use crate::sccs::Scc;
use crate::value_iteration::non_determinism::NonDeterminism;
use probabilistic_models::traits::{ReadPredecessors, ReadStateSpace};
use typed_index_collections::{Index, To1};

pub trait SolveOrder {
    fn find_and_solve_subgames<
        ND: NonDeterminism,
        Solver: super::subgame_solver::SubGameSolver,
        M: ReadStateSpace + ReadPredecessors<StateIdx = M::StateIndex>,
    >(
        model: &M,
        s0: &To1<M::StateIndex, bool>,
        s1: &To1<M::StateIndex, bool>,
        dominated_by_relation: &DominatedByRelation<M::StateIndex>,
        eps: f64,
    ) -> To1<M::StateIndex, f64>;
}

pub struct ModelSize {
    states: usize,
    choices: usize,
    branches: usize,
}

impl ModelSize {
    pub fn from_scc<ScI: Index, ScEI: Index, M: ReadStateSpace>(
        model: &M,
        scc: Scc<'_, ScI, ScEI, M::StateIndex>,
    ) -> Self {
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
