use crate::dominated_by::DominatedByRelation;
use crate::value_iteration::non_determinism::NonDeterminism;
use crate::value_iteration::precomputed_states::PrecomputedStates;
use crate::value_iteration::subgame_solver::SubGameSolver;
use probabilistic_models::traits::{ReadPredecessors, ReadStateSpace};
use typed_index_collections::To1;

pub struct Monolithic {}

impl super::SolveOrder for Monolithic {
    fn find_and_solve_subgames<
        ND: NonDeterminism,
        Solver: SubGameSolver,
        M: ReadStateSpace + ReadPredecessors<StateIdx = M::StateIndex>,
        P: PrecomputedStates<StateIdx = M::StateIndex>,
    >(
        self,
        model: &M,
        precomputed_states: &P,
        dominated_by_relation: &DominatedByRelation<M::StateIndex>,
        eps: f64,
    ) -> To1<M::StateIndex, f64> {
        let _ = (model, precomputed_states, dominated_by_relation, eps);
        todo!()
    }
}
