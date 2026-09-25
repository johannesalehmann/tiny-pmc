mod atomic_propositions;
mod dfs;
mod initial_states;
mod mdp;
mod ordering;
mod rewards;

pub use ordering::StateOrdering;

use crate::traits::ReadStateSpace;
use typed_index_collections::Index;

pub trait PermuteStates {
    type StateIndex: Index;

    fn permute_states(&self, ordering: &StateOrdering<Self::StateIndex>) -> Self;
}

pub trait PermuteStatesWithContext<SI: Index, CI: Index> {
    fn permute_states_with_context<Base: ReadStateSpace<StateIndex = SI, ChoiceIndex = CI>>(
        &self,
        ordering: &StateOrdering<SI>,
        old_base: &Base,
        new_base: &Base,
    ) -> Self;
}

impl<CI: Index, T: PermuteStates> PermuteStatesWithContext<T::StateIndex, CI> for T {
    fn permute_states_with_context<
        Base: ReadStateSpace<StateIndex = T::StateIndex, ChoiceIndex = CI>,
    >(
        &self,
        ordering: &StateOrdering<T::StateIndex>,
        _old_base: &Base,
        _new_base: &Base,
    ) -> Self {
        self.permute_states(ordering)
    }
}

impl<SI: Index, CI: Index> PermuteStatesWithContext<SI, CI> for () {
    fn permute_states_with_context<Base: ReadStateSpace<StateIndex = SI, ChoiceIndex = CI>>(
        &self,
        _ordering: &StateOrdering<SI>,
        _old_base: &Base,
        _new_base: &Base,
    ) -> Self {
        ()
    }
}
