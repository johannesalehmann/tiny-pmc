use crate::dominated_by::DominatedByRelation;
use crate::value_iteration::non_determinism::NonDeterminism;
use crate::value_iteration::subgame_solver::SubGameSolver;
use probabilistic_models::traits::{ReadPredecessors, ReadStateSpace};
use typed_index_collections::To1;

pub struct Monolithic {}

impl super::SolveOrder for Monolithic {
    fn find_and_solve_subgames<
        ND: NonDeterminism,
        Solver: SubGameSolver,
        M: ReadStateSpace + ReadPredecessors<StateIdx = M::StateIndex>,
    >(
        model: &M,
        s0: &To1<M::StateIndex, bool>,
        s1: &To1<M::StateIndex, bool>,
        dominated_by_relation: &DominatedByRelation<M::StateIndex>,
        eps: f64,
    ) -> To1<M::StateIndex, f64> {
        todo!()
    }
}
