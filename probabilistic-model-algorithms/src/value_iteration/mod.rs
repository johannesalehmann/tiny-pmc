use crate::dominated_by::DominatedByRelation;
use crate::state_description::StateDescription;
use probabilistic_models::traits::{ReadAtomicPropositions, ReadPredecessors, ReadStateSpace};
use typed_index_collections::To1;

mod non_determinism;
use non_determinism::{Maximise, Minimise};

mod solve_order;
pub use solve_order::{EpsAllocationScheme, SccTimingOutput};
use solve_order::{GlobalEpsForEachScc, Monolithic, SccTimings, Topological, UniformEpsAllocation};

mod subgame_solver;
use subgame_solver::{OptimisticValueIteration, SubGameSolver, ValueIteration};

#[derive(Clone, Copy, Debug)]
pub enum NonDeterminism {
    Minimise,
    Maximise,
}

#[derive(Clone, Debug)]
pub enum SolveOrder {
    Monolithic,
    Topological {
        eps_allocation_scheme: EpsAllocationScheme,
        // Can be used for benchmarking. Prints how long each SCC took to solve
        write_scc_timing: Option<SccTimingOutput>,
    },
}

pub fn value_iteration<
    M: ReadStateSpace
        + ReadPredecessors<
            StateIdx = M::StateIndex,
            ChoiceIdx = M::ChoiceIndex,
            BranchIdx = M::BranchIndex,
        > + ReadAtomicPropositions<StateIdx = M::StateIndex>,
>(
    model: &M,
    goal: &StateDescription<M>,
    non_determinism: NonDeterminism,
    eps: f64,
    solve_order: SolveOrder,
) -> To1<M::StateIndex, f64> {
    dispatch_non_determinism::<ValueIteration, _>(model, goal, non_determinism, eps, solve_order)
}

pub fn optimistic_value_iteration<
    M: ReadStateSpace
        + ReadPredecessors<
            StateIdx = M::StateIndex,
            ChoiceIdx = M::ChoiceIndex,
            BranchIdx = M::BranchIndex,
        > + ReadAtomicPropositions<StateIdx = M::StateIndex>,
>(
    model: &M,
    goal: &StateDescription<M>,
    non_determinism: NonDeterminism,
    eps: f64,
    solve_order: SolveOrder,
) -> To1<M::StateIndex, f64> {
    dispatch_non_determinism::<OptimisticValueIteration, _>(
        model,
        goal,
        non_determinism,
        eps,
        solve_order,
    )
}

fn dispatch_non_determinism<
    Solver: SubGameSolver,
    M: ReadStateSpace
        + ReadPredecessors<
            StateIdx = M::StateIndex,
            ChoiceIdx = M::ChoiceIndex,
            BranchIdx = M::BranchIndex,
        > + ReadAtomicPropositions<StateIdx = M::StateIndex>,
>(
    model: &M,
    goal: &StateDescription<M>,
    non_determinism: NonDeterminism,
    eps: f64,
    solve_order: SolveOrder,
) -> To1<M::StateIndex, f64> {
    match non_determinism {
        NonDeterminism::Minimise => {
            dispatch_solve_order::<Minimise, Solver, _>(model, goal, eps, solve_order)
        }
        NonDeterminism::Maximise => {
            dispatch_solve_order::<Maximise, Solver, _>(model, goal, eps, solve_order)
        }
    }
}

fn dispatch_solve_order<
    ND: non_determinism::NonDeterminism,
    Solver: SubGameSolver,
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
    solve_order: SolveOrder,
) -> To1<M::StateIndex, f64> {
    match solve_order {
        SolveOrder::Monolithic => {
            value_iteration_internal::<ND, Monolithic, Solver, _>(Monolithic {}, model, goal, eps)
        }
        SolveOrder::Topological {
            eps_allocation_scheme,
            write_scc_timing,
        } => dispatch_scc_timing::<ND, Solver, _>(
            model,
            goal,
            eps,
            eps_allocation_scheme,
            write_scc_timing,
        ),
    }
}

fn dispatch_scc_timing<
    ND: non_determinism::NonDeterminism,
    Solver: SubGameSolver,
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
    eps_allocation_scheme: EpsAllocationScheme,
    write_scc_timing: Option<SccTimingOutput>,
) -> To1<M::StateIndex, f64> {
    match write_scc_timing {
        Some(output) => dispatch_eps_allocation::<ND, Solver, _, _>(
            SccTimings::new(output),
            model,
            goal,
            eps,
            eps_allocation_scheme,
        ),
        None => {
            dispatch_eps_allocation::<ND, Solver, _, _>((), model, goal, eps, eps_allocation_scheme)
        }
    }
}

fn dispatch_eps_allocation<
    ND: non_determinism::NonDeterminism,
    Solver: SubGameSolver,
    Timing: solve_order::TopoTiming,
    M: ReadStateSpace
        + ReadPredecessors<
            StateIdx = M::StateIndex,
            ChoiceIdx = M::ChoiceIndex,
            BranchIdx = M::BranchIndex,
        > + ReadAtomicPropositions<StateIdx = M::StateIndex>,
>(
    timing: Timing,
    model: &M,
    goal: &StateDescription<M>,
    eps: f64,
    eps_allocation_scheme: EpsAllocationScheme,
) -> To1<M::StateIndex, f64> {
    match eps_allocation_scheme {
        EpsAllocationScheme::Uniform => value_iteration_internal::<
            ND,
            Topological<Timing, UniformEpsAllocation>,
            Solver,
            _,
        >(Topological::new(timing), model, goal, eps),
        EpsAllocationScheme::GlobalEpsForEach => {
            value_iteration_internal::<ND, Topological<Timing, GlobalEpsForEachScc>, Solver, _>(
                Topological::new(timing),
                model,
                goal,
                eps,
            )
        }
    }
}

fn value_iteration_internal<
    ND: non_determinism::NonDeterminism,
    SolveOrder: solve_order::SolveOrder,
    Solver: SubGameSolver,
    M: ReadStateSpace
        + ReadPredecessors<
            StateIdx = M::StateIndex,
            ChoiceIdx = M::ChoiceIndex,
            BranchIdx = M::BranchIndex,
        > + ReadAtomicPropositions<StateIdx = M::StateIndex>,
>(
    solve_order: SolveOrder,
    model: &M,
    goal: &StateDescription<M>,
    eps: f64,
) -> To1<M::StateIndex, f64> {
    let precomputed_states = ND::compute_s0_s1(model, goal);
    let dominated_by = DominatedByRelation::empty();
    solve_order.find_and_solve_subgames::<ND, Solver, _, _>(
        model,
        &precomputed_states,
        &dominated_by,
        eps,
    )
}
