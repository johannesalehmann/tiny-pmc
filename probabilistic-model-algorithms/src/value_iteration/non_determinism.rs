use crate::state_description::StateDescription;
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
    ) -> (To1<M::StateIndex, bool>, To1<M::StateIndex, bool>);

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
    ) -> (To1<M::StateIndex, bool>, To1<M::StateIndex, bool>) {
        let s0 = crate::qualitative_reachability::s0_max(model, goal);
        let s1 = crate::qualitative_reachability::s1_max(model, goal);
        (s0, s1)
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
    ) -> (To1<M::StateIndex, bool>, To1<M::StateIndex, bool>) {
        let s0 = crate::qualitative_reachability::s0_min(model, goal);
        let s1 = crate::qualitative_reachability::s1_min(model, goal, &s0);
        (s0, s1)
    }

    fn neutral_value() -> f64 {
        1.0
    }

    fn is_better(before: f64, new: f64) -> bool {
        new <= before
    }
}
