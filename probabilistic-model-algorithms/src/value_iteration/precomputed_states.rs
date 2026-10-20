use crate::sccs::ExclusionCriterion;
use probabilistic_models::{ChoiceIndex, HasIx, Ix, StateIndex};
use typed_index_collections::To1;

// This trait is used by value iteration and SCC computation to exclude states for which we have
// qualitatively precomputed the answer. As its behaviour differs for reachability probabilities and
// expected rewards, there are separate implementations (S0S1 for the former, SInfinity for the
// latter)
pub trait PrecomputedStates: HasIx {
    fn is_maybe_state(&self, state: StateIndex<Self::Ix>) -> bool;
    fn initial_value(&self, state: StateIndex<Self::Ix>) -> f64;
    // The maximum that values can take during value iteration. This is 1 for
    // reachability probabilities and infinity for rewards. It's placed here as
    // this location already distinguishes between rewards and probabilities.
    // Perhaps renaming the trait would be a good idea.
    fn max_value(&self) -> f64;
}

impl<P: PrecomputedStates> ExclusionCriterion<P::Ix> for P {
    fn is_state_excluded(&self, state: StateIndex<P::Ix>) -> bool {
        !self.is_maybe_state(state)
    }

    fn is_choice_excluded(&self, _choice: ChoiceIndex<P::Ix>) -> bool {
        false
    }
}

pub struct S0S1<I: Ix> {
    s0: To1<StateIndex<I>, bool>,
    s1: To1<StateIndex<I>, bool>,
}

impl<I: Ix> S0S1<I> {
    pub fn new(s0: To1<StateIndex<I>, bool>, s1: To1<StateIndex<I>, bool>) -> Self {
        Self { s0, s1 }
    }
}

impl<I: Ix> HasIx for S0S1<I> {
    type Ix = I;
}

impl<I: Ix> PrecomputedStates for S0S1<I> {
    fn is_maybe_state(&self, state: StateIndex<I>) -> bool {
        !self.s0[state] && !self.s1[state]
    }

    fn initial_value(&self, state: StateIndex<I>) -> f64 {
        if self.s1[state] { 1.0 } else { 0.0 }
    }

    fn max_value(&self) -> f64 {
        1.0
    }
}

pub struct SInfinity<I: Ix> {
    s1: To1<StateIndex<I>, bool>,
    goal: To1<StateIndex<I>, bool>,
}

impl<I: Ix> SInfinity<I> {
    pub fn new(s1: To1<StateIndex<I>, bool>, goal: To1<StateIndex<I>, bool>) -> Self {
        Self { s1, goal }
    }
}

impl<I: Ix> HasIx for SInfinity<I> {
    type Ix = I;
}

impl<I: Ix> PrecomputedStates for SInfinity<I> {
    fn is_maybe_state(&self, state: StateIndex<I>) -> bool {
        self.s1[state] && !self.goal[state]
    }

    fn initial_value(&self, state: StateIndex<I>) -> f64 {
        if self.s1[state] { 0.0 } else { f64::INFINITY }
    }

    fn max_value(&self) -> f64 {
        f64::INFINITY
    }
}
