use crate::sccs::ExclusionCriterion;
use typed_index_collections::{Index, To1};

// This trait is used by value iteration and SCC computation to exclude states for which we have
// qualitatively precomputed the answer. As its behaviour differs for reachability probabilities and
// expected rewards, there are separate implementations (S0S1 for the former, SInfinity for the
// latter)
pub trait PrecomputedStates {
    type StateIdx: Index;

    fn is_maybe_state(&self, state: Self::StateIdx) -> bool;
    fn initial_value(&self, state: Self::StateIdx) -> f64;
}

impl<StateIdx: Index, P: PrecomputedStates<StateIdx = StateIdx>> ExclusionCriterion<StateIdx>
    for P
{
    fn is_excluded(&self, state: StateIdx) -> bool {
        !self.is_maybe_state(state)
    }
}

pub struct S0S1<StateIdx: Index> {
    s0: To1<StateIdx, bool>,
    s1: To1<StateIdx, bool>,
}

impl<StateIdx: Index> S0S1<StateIdx> {
    pub fn new(s0: To1<StateIdx, bool>, s1: To1<StateIdx, bool>) -> Self {
        Self { s0, s1 }
    }
}

impl<StateIdx: Index> PrecomputedStates for S0S1<StateIdx> {
    type StateIdx = StateIdx;

    fn is_maybe_state(&self, state: StateIdx) -> bool {
        !self.s0[state] && !self.s1[state]
    }

    fn initial_value(&self, state: StateIdx) -> f64 {
        if self.s1[state] { 1.0 } else { 0.0 }
    }
}

pub struct SInfinity<StateIdx: Index> {
    s1: To1<StateIdx, bool>,
    goal: To1<StateIdx, bool>,
}

impl<StateIdx: Index> SInfinity<StateIdx> {
    pub fn new(s1: To1<StateIdx, bool>, goal: To1<StateIdx, bool>) -> Self {
        Self { s1, goal }
    }
}

impl<StateIdx: Index> PrecomputedStates for SInfinity<StateIdx> {
    type StateIdx = StateIdx;

    fn is_maybe_state(&self, state: StateIdx) -> bool {
        self.s1[state] && !self.goal[state]
    }

    fn initial_value(&self, state: StateIdx) -> f64 {
        if self.s1[state] { 0.0 } else { f64::INFINITY }
    }
}
