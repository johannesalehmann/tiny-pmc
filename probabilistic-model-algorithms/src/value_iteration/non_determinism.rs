use crate::mecs::Mecs;
use crate::sccs::ExcludeStatesAndChoices;
use crate::state_description::StateDescription;
use crate::sub_model::RewardsSource;
use crate::value_iteration::CollapseMecs;
use crate::value_iteration::precomputed_states::{PrecomputedStates, S0S1, SInfinity};
use probabilistic_models::traits::{ReadAtomicPropositions, ReadPredecessors, ReadStateSpace};
use typed_index_collections::To1;

pub trait NonDeterminism {
    fn compute_s0_s1<
        M: ReadStateSpace
            + ReadAtomicPropositions<StateIdx = M::StateIndex>
            + ReadPredecessors<
                StateIdx = M::StateIndex,
                ChoiceIdx = M::ChoiceIndex,
                BranchIdx = M::BranchIndex,
            >,
    >(
        model: &M,
        goal: &StateDescription<M>,
    ) -> S0S1<M::StateIndex>;

    fn compute_s_inf<
        M: ReadStateSpace
            + ReadAtomicPropositions<StateIdx = M::StateIndex>
            + ReadPredecessors<
                StateIdx = M::StateIndex,
                ChoiceIdx = M::ChoiceIndex,
                BranchIdx = M::BranchIndex,
            >,
    >(
        model: &M,
        goal: &StateDescription<M>,
    ) -> SInfinity<M::StateIndex>;

    fn compute_probability_mecs<
        M: ReadStateSpace
            + ReadPredecessors<
                StateIdx = M::StateIndex,
                ChoiceIdx = M::ChoiceIndex,
                BranchIdx = M::BranchIndex,
            >,
    >(
        model: &M,
        s0_s1: &S0S1<M::StateIndex>,
        collapse_mecs: CollapseMecs,
    ) -> Mecs<M::StateIndex, M::ChoiceIndex>;

    fn compute_reward_mecs<
        M: ReadStateSpace
            + ReadPredecessors<
                StateIdx = M::StateIndex,
                ChoiceIdx = M::ChoiceIndex,
                BranchIdx = M::BranchIndex,
            >,
        Rew: RewardsSource<M::StateIndex, M::ChoiceIndex>,
    >(
        model: &M,
        s_inf: &SInfinity<M::StateIndex>,
        rewards: &Rew,
    ) -> Mecs<M::StateIndex, M::ChoiceIndex>;

    fn neutral_value() -> f64;
    fn is_better(before: f64, new: f64) -> bool;
}

pub struct Maximise {}

impl NonDeterminism for Maximise {
    fn compute_s0_s1<
        M: ReadStateSpace
            + ReadAtomicPropositions<StateIdx = M::StateIndex>
            + ReadPredecessors<
                StateIdx = M::StateIndex,
                ChoiceIdx = M::ChoiceIndex,
                BranchIdx = M::BranchIndex,
            >,
    >(
        model: &M,
        goal: &StateDescription<M>,
    ) -> S0S1<M::StateIndex> {
        let s0 = crate::qualitative_reachability::s0_max(model, goal);
        let s1 = crate::qualitative_reachability::s1_max(model, goal);
        S0S1::new(s0, s1)
    }

    fn compute_s_inf<
        M: ReadStateSpace
            + ReadAtomicPropositions<StateIdx = M::StateIndex>
            + ReadPredecessors<
                StateIdx = M::StateIndex,
                ChoiceIdx = M::ChoiceIndex,
                BranchIdx = M::BranchIndex,
            >,
    >(
        model: &M,
        goal: &StateDescription<M>,
    ) -> SInfinity<M::StateIndex> {
        let s0 = crate::qualitative_reachability::s0_min(model, goal);
        let s1 = crate::qualitative_reachability::s1_min(model, goal, &s0);
        SInfinity::new(s1, goal_flags(model, goal))
    }

    fn compute_probability_mecs<
        M: ReadStateSpace
            + ReadPredecessors<
                StateIdx = M::StateIndex,
                ChoiceIdx = M::ChoiceIndex,
                BranchIdx = M::BranchIndex,
            >,
    >(
        model: &M,
        s0_s1: &S0S1<M::StateIndex>,
        collapse_mecs: CollapseMecs,
    ) -> Mecs<M::StateIndex, M::ChoiceIndex> {
        match collapse_mecs {
            CollapseMecs::WhenNecessary => Mecs::empty(),
            CollapseMecs::WheneverPossible => {
                let excluded_choices = To1::with_entries(vec![false; model.choices().len()]);
                Mecs::compute(
                    model,
                    ExcludeStatesAndChoices::new(non_maybe_states(model, s0_s1), excluded_choices),
                )
            }
        }
    }

    fn compute_reward_mecs<
        M: ReadStateSpace
            + ReadPredecessors<
                StateIdx = M::StateIndex,
                ChoiceIdx = M::ChoiceIndex,
                BranchIdx = M::BranchIndex,
            >,
        Rew: RewardsSource<M::StateIndex, M::ChoiceIndex>,
    >(
        model: &M,
        s_inf: &SInfinity<M::StateIndex>,
        rewards: &Rew,
    ) -> Mecs<M::StateIndex, M::ChoiceIndex> {
        let _ = (model, s_inf, rewards);
        Mecs::empty()
    }

    fn neutral_value() -> f64 {
        0.0
    }

    fn is_better(before: f64, new: f64) -> bool {
        new >= before
    }
}

pub struct Minimise {}

impl NonDeterminism for Minimise {
    fn compute_s0_s1<
        M: ReadStateSpace
            + ReadAtomicPropositions<StateIdx = M::StateIndex>
            + ReadPredecessors<
                StateIdx = M::StateIndex,
                ChoiceIdx = M::ChoiceIndex,
                BranchIdx = M::BranchIndex,
            >,
    >(
        model: &M,
        goal: &StateDescription<M>,
    ) -> S0S1<M::StateIndex> {
        let s0 = crate::qualitative_reachability::s0_min(model, goal);
        let s1 = crate::qualitative_reachability::s1_min(model, goal, &s0);
        S0S1::new(s0, s1)
    }

    fn compute_s_inf<
        M: ReadStateSpace
            + ReadAtomicPropositions<StateIdx = M::StateIndex>
            + ReadPredecessors<
                StateIdx = M::StateIndex,
                ChoiceIdx = M::ChoiceIndex,
                BranchIdx = M::BranchIndex,
            >,
    >(
        model: &M,
        goal: &StateDescription<M>,
    ) -> SInfinity<M::StateIndex> {
        let s1 = crate::qualitative_reachability::s1_max(model, goal);
        SInfinity::new(s1, goal_flags(model, goal))
    }

    fn compute_probability_mecs<
        M: ReadStateSpace
            + ReadPredecessors<
                StateIdx = M::StateIndex,
                ChoiceIdx = M::ChoiceIndex,
                BranchIdx = M::BranchIndex,
            >,
    >(
        model: &M,
        s0_s1: &S0S1<M::StateIndex>,
        collapse_mecs: CollapseMecs,
    ) -> Mecs<M::StateIndex, M::ChoiceIndex> {
        // All ECs among maybe states have been removed by the qualitative precomputation
        let _ = (model, s0_s1, collapse_mecs);
        Mecs::empty()
    }

    fn compute_reward_mecs<
        M: ReadStateSpace
            + ReadPredecessors<
                StateIdx = M::StateIndex,
                ChoiceIdx = M::ChoiceIndex,
                BranchIdx = M::BranchIndex,
            >,
        Rew: RewardsSource<M::StateIndex, M::ChoiceIndex>,
    >(
        model: &M,
        s_inf: &SInfinity<M::StateIndex>,
        rewards: &Rew,
    ) -> Mecs<M::StateIndex, M::ChoiceIndex> {
        Mecs::compute(
            model,
            ExcludeStatesAndChoices::new(
                // TODO: We could avoid allocating here by instead internally storing that the value
                //  needs to be reversed
                non_maybe_states(model, s_inf),
                // TODO: This currently builds a vector, but we could just read the information
                //  from the model on the fly and avoid allocating.
                rewarded_choices(model, rewards),
            ),
        )
    }

    fn neutral_value() -> f64 {
        f64::INFINITY
    }

    fn is_better(before: f64, new: f64) -> bool {
        new <= before
    }
}

fn goal_flags<M: ReadStateSpace + ReadAtomicPropositions<StateIdx = M::StateIndex>>(
    model: &M,
    goal: &StateDescription<M>,
) -> To1<M::StateIndex, bool> {
    let mut flags = To1::with_entries(vec![false; model.states().len()]);
    goal.write_flags(&mut flags);
    flags
}

fn non_maybe_states<M: ReadStateSpace>(
    model: &M,
    precomputed_states: &impl PrecomputedStates<StateIdx = M::StateIndex>,
) -> To1<M::StateIndex, bool> {
    let mut excluded = To1::with_capacity(model.states().len());
    for state in model.states() {
        excluded.add_checked(state, !precomputed_states.is_maybe_state(state));
    }
    excluded
}

fn rewarded_choices<M: ReadStateSpace, Rew: RewardsSource<M::StateIndex, M::ChoiceIndex>>(
    model: &M,
    rewards: &Rew,
) -> To1<M::ChoiceIndex, bool> {
    let mut rewarded = To1::with_capacity(model.choices().len());
    for state in model.states() {
        let state_reward = if rewards.has_state_rewards() {
            rewards.state_reward(state)
        } else {
            0.0
        };
        for choice in model.choices_of_state(state) {
            let choice_reward = if rewards.has_choice_rewards() {
                rewards.choice_reward(choice)
            } else {
                0.0
            };
            rewarded.add_checked(choice, state_reward + choice_reward != 0.0);
        }
    }
    rewarded
}
