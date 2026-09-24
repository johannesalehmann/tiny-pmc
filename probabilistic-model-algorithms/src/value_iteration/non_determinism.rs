use crate::state_description::StateDescription;
use crate::value_iteration::precomputed_states::{S0S1, SInfinity};
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
