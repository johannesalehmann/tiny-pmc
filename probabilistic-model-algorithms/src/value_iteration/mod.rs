use crate::dominated_by::DominatedByRelation;
use crate::state_description::StateDescription;
use probabilistic_models::traits::{ReadAtomicPropositions, ReadPredecessors, ReadStateSpace};
use typed_index_collections::To1;

mod non_determinism;
use non_determinism::{Maximise, Minimise, NonDeterminism};

mod solve_order;
use solve_order::{Monolithic, SolveOrder, Topological};

mod subgame_solver;
use subgame_solver::{OptimisticValueIteration, SubGameSolver, ValueIteration};

pub fn p_min_topo_vi<
    M: ReadStateSpace
        + ReadPredecessors<
            StateIdx = M::StateIndex,
            ChoiceIdx = M::ChoiceIndex,
            BranchIdx = M::BranchIndex,
        > + ReadAtomicPropositions<StateIdx = M::StateIndex>,
>(
    model: &M,
    goal: &StateDescription<M>,
    eps: f64,
) -> To1<M::StateIndex, f64> {
    value_iteration_internal::<Minimise, Topological, ValueIteration, _>(model, goal, eps)
}

pub fn p_max_topo_vi<
    M: ReadStateSpace
        + ReadPredecessors<
            StateIdx = M::StateIndex,
            ChoiceIdx = M::ChoiceIndex,
            BranchIdx = M::BranchIndex,
        > + ReadAtomicPropositions<StateIdx = M::StateIndex>,
>(
    model: &M,
    goal: &StateDescription<M>,
    eps: f64,
) -> To1<M::StateIndex, f64> {
    value_iteration_internal::<Maximise, Topological, ValueIteration, _>(model, goal, eps)
}
pub fn p_min_monolithic_vi<
    M: ReadStateSpace
        + ReadPredecessors<
            StateIdx = M::StateIndex,
            ChoiceIdx = M::ChoiceIndex,
            BranchIdx = M::BranchIndex,
        > + ReadAtomicPropositions<StateIdx = M::StateIndex>,
>(
    model: &M,
    goal: &StateDescription<M>,
    eps: f64,
) -> To1<M::StateIndex, f64> {
    value_iteration_internal::<Minimise, Monolithic, ValueIteration, _>(model, goal, eps)
}

pub fn p_max_monolithic_vi<
    M: ReadStateSpace
        + ReadPredecessors<
            StateIdx = M::StateIndex,
            ChoiceIdx = M::ChoiceIndex,
            BranchIdx = M::BranchIndex,
        > + ReadAtomicPropositions<StateIdx = M::StateIndex>,
>(
    model: &M,
    goal: &StateDescription<M>,
    eps: f64,
) -> To1<M::StateIndex, f64> {
    value_iteration_internal::<Maximise, Monolithic, ValueIteration, _>(model, goal, eps)
}

pub fn p_min_topo_ovi<
    M: ReadStateSpace
        + ReadPredecessors<
            StateIdx = M::StateIndex,
            ChoiceIdx = M::ChoiceIndex,
            BranchIdx = M::BranchIndex,
        > + ReadAtomicPropositions<StateIdx = M::StateIndex>,
>(
    model: &M,
    goal: &StateDescription<M>,
    eps: f64,
) -> To1<M::StateIndex, f64> {
    value_iteration_internal::<Minimise, Topological, OptimisticValueIteration, _>(model, goal, eps)
}

pub fn p_max_topo_ovi<
    M: ReadStateSpace
        + ReadPredecessors<
            StateIdx = M::StateIndex,
            ChoiceIdx = M::ChoiceIndex,
            BranchIdx = M::BranchIndex,
        > + ReadAtomicPropositions<StateIdx = M::StateIndex>,
>(
    model: &M,
    goal: &StateDescription<M>,
    eps: f64,
) -> To1<M::StateIndex, f64> {
    value_iteration_internal::<Maximise, Topological, OptimisticValueIteration, _>(model, goal, eps)
}
pub fn p_min_monolithic_ovi<
    M: ReadStateSpace
        + ReadPredecessors<
            StateIdx = M::StateIndex,
            ChoiceIdx = M::ChoiceIndex,
            BranchIdx = M::BranchIndex,
        > + ReadAtomicPropositions<StateIdx = M::StateIndex>,
>(
    model: &M,
    goal: &StateDescription<M>,
    eps: f64,
) -> To1<M::StateIndex, f64> {
    value_iteration_internal::<Minimise, Monolithic, OptimisticValueIteration, _>(model, goal, eps)
}

pub fn p_max_monolithic_ovi<
    M: ReadStateSpace
        + ReadPredecessors<
            StateIdx = M::StateIndex,
            ChoiceIdx = M::ChoiceIndex,
            BranchIdx = M::BranchIndex,
        > + ReadAtomicPropositions<StateIdx = M::StateIndex>,
>(
    model: &M,
    goal: &StateDescription<M>,
    eps: f64,
) -> To1<M::StateIndex, f64> {
    value_iteration_internal::<Maximise, Monolithic, OptimisticValueIteration, _>(model, goal, eps)
}

fn value_iteration_internal<
    ND: NonDeterminism,
    SolveOrder: solve_order::SolveOrder,
    Solver: subgame_solver::SubGameSolver,
    M: ReadStateSpace
        + ReadPredecessors<
            StateIdx = M::StateIndex,
            ChoiceIdx = M::ChoiceIndex,
            BranchIdx = M::BranchIndex,
        > + ReadAtomicPropositions<StateIdx = M::StateIndex>,
>(
    model: &M,
    goal: &StateDescription<M>,
    eps: f64,
) -> To1<M::StateIndex, f64> {
    let (s0, s1) = ND::compute_s0_s1(model, goal);
    let dominated_by = DominatedByRelation::empty();
    SolveOrder::find_and_solve_subgames::<ND, Solver, _>(model, &s0, &s1, &dominated_by, eps)
}
