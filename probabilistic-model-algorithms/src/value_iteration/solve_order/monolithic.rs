use crate::dominated_by::DominatedByRelation;
use crate::mecs::Mecs;
use crate::sub_model::RewardsSource;
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
        M: ReadStateSpace
            + ReadPredecessors<
                StateIdx = M::StateIndex,
                ChoiceIdx = M::ChoiceIndex,
                BranchIdx = M::BranchIndex,
            >,
        P: PrecomputedStates<StateIdx = M::StateIndex>,
        Rew: RewardsSource<M::StateIndex, M::ChoiceIndex>,
    >(
        self,
        model: &M,
        precomputed_states: &P,
        rew: Rew,
        dominated_by_relation: &DominatedByRelation<M::StateIndex>,
        mecs: &Mecs<M::StateIndex, M::ChoiceIndex>,
        eps: f64,
    ) -> To1<M::StateIndex, f64> {
        let _ = (
            model,
            precomputed_states,
            rew,
            dominated_by_relation,
            mecs,
            eps,
        );
        todo!()
    }
}
