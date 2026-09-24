use crate::dominated_by::DominatedByRelation;
use crate::state_description::StateDescription;
use probabilistic_models::traits::{
    ReadAtomicPropositions, ReadPredecessors, ReadRewards, ReadStateSpace,
};
use typed_index_collections::To1;

mod non_determinism;
use non_determinism::{Maximise, Minimise};

pub(crate) mod precomputed_states;

mod solve_order;
pub use solve_order::{EpsAllocationScheme, SccTimingOutput};
use solve_order::{Monolithic, Topological};

mod config;
pub use config::*;

mod subgame_solver;

use crate::sub_model;
use subgame_solver::{OptimisticValueIteration, SubGameSolver, ValueIteration};

#[derive(Clone, Copy, Debug)]
pub enum NonDeterminism {
    Minimise,
    Maximise,
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
    config: ValueIterationConfig,
) -> To1<M::StateIndex, f64> {
    dispatch_non_determinism::<ValueIteration, _, ()>(model, goal, non_determinism, config, None)
}

pub fn value_iteration_rewards<
    M: ReadStateSpace
        + ReadPredecessors<
            StateIdx = M::StateIndex,
            ChoiceIdx = M::ChoiceIndex,
            BranchIdx = M::BranchIndex,
        > + ReadAtomicPropositions<StateIdx = M::StateIndex>
        + ReadRewards<StateIdx = M::StateIndex, ChoiceIdx = M::ChoiceIndex>,
>(
    model: &M,
    goal: &StateDescription<M>,
    rewards: <M as ReadRewards>::RewardIdx,
    non_determinism: NonDeterminism,
    config: ValueIterationConfig,
) -> To1<M::StateIndex, f64> {
    dispatch_non_determinism::<ValueIteration, _, _>(
        model,
        goal,
        non_determinism,
        config,
        Some(sub_model::StateAndChoiceRewards::new(model, rewards)),
    )
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
    config: ValueIterationConfig,
) -> To1<M::StateIndex, f64> {
    dispatch_non_determinism::<OptimisticValueIteration, _, ()>(
        model,
        goal,
        non_determinism,
        config,
        None,
    )
}

pub fn optimistic_value_iteration_rewards<
    M: ReadStateSpace
        + ReadPredecessors<
            StateIdx = M::StateIndex,
            ChoiceIdx = M::ChoiceIndex,
            BranchIdx = M::BranchIndex,
        > + ReadAtomicPropositions<StateIdx = M::StateIndex>
        + ReadRewards<StateIdx = M::StateIndex, ChoiceIdx = M::ChoiceIndex>,
>(
    model: &M,
    goal: &StateDescription<M>,
    rewards: <M as ReadRewards>::RewardIdx,
    non_determinism: NonDeterminism,
    config: ValueIterationConfig,
) -> To1<M::StateIndex, f64> {
    dispatch_non_determinism::<OptimisticValueIteration, _, _>(
        model,
        goal,
        non_determinism,
        config,
        Some(sub_model::StateAndChoiceRewards::new(model, rewards)),
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
    Rew: sub_model::RewardsSource<M::StateIndex, M::ChoiceIndex>,
>(
    model: &M,
    goal: &StateDescription<M>,
    non_determinism: NonDeterminism,
    config: ValueIterationConfig,
    reward_source: Option<Rew>,
) -> To1<M::StateIndex, f64> {
    match non_determinism {
        NonDeterminism::Minimise => {
            dispatch_solve_order::<Minimise, Solver, _, _>(model, goal, config, reward_source)
        }
        NonDeterminism::Maximise => {
            dispatch_solve_order::<Maximise, Solver, _, _>(model, goal, config, reward_source)
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
    Rew: sub_model::RewardsSource<M::StateIndex, M::ChoiceIndex>,
>(
    model: &M,
    goal: &StateDescription<M>,
    config: ValueIterationConfig,
    reward_source: Option<Rew>,
) -> To1<M::StateIndex, f64> {
    match config.solve_order {
        SolveOrder::Monolithic => {
            if config.write_sub_mdp_timing.is_some() {
                println!("Warning: Monolithic solver does not print per-SCC timing");
            }
            value_iteration_internal::<ND, Monolithic, Solver, _, _>(
                Monolithic {},
                model,
                goal,
                config,
                reward_source,
            )
        }
        SolveOrder::Topological {
            eps_allocation_scheme,
        } => value_iteration_internal::<ND, Topological, Solver, _, _>(
            Topological::new(config.write_sub_mdp_timing.clone(), eps_allocation_scheme),
            model,
            goal,
            config,
            reward_source,
        ),
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
    Rew: sub_model::RewardsSource<M::StateIndex, M::ChoiceIndex>,
>(
    solve_order: SolveOrder,
    model: &M,
    goal: &StateDescription<M>,
    config: ValueIterationConfig,
    rew: Option<Rew>,
) -> To1<M::StateIndex, f64> {
    if let Some(rew) = rew {
        let precomputed_states = ND::compute_s_inf(model, goal);
        let dominated_by = DominatedByRelation::empty();
        let mecs = ND::compute_reward_mecs(model, &precomputed_states, &rew);
        solve_order.find_and_solve_subgames::<ND, Solver, _, _, _>(
            model,
            &precomputed_states,
            rew,
            &dominated_by,
            &mecs,
            config.eps,
        )
    } else {
        let precomputed_states = ND::compute_s0_s1(model, goal);
        let dominated_by = DominatedByRelation::empty();
        let mecs = ND::compute_probability_mecs(model, &precomputed_states, config.collapse_mecs);
        solve_order.find_and_solve_subgames::<ND, Solver, _, _, _>(
            model,
            &precomputed_states,
            (),
            &dominated_by,
            &mecs,
            config.eps,
        )
    }
}
