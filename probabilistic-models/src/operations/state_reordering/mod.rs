mod atomic_propositions;
mod dfs;
mod graph;
mod initial_states;
mod mdp;
mod models;
mod ordering;
mod rewards;
#[cfg(test)]
mod test_utils;

pub use dfs::SuccessorOrder;
pub use ordering::StateOrdering;

use crate::traits::{ReadInitialStates, ReadStateSpace};
use crate::{HasIx, Ix, Model, StateIndex};
use num_traits::Bounded;
use typed_index_collections::RawIndex;

pub trait PermuteStates: HasIx {
    fn permute_states(&self, ordering: &StateOrdering<StateIndex<Self::Ix>>) -> Self;
}

pub trait PermuteStatesWithContext<I: Ix> {
    fn permute_states_with_context<Base: ReadStateSpace<Ix = I>>(
        &self,
        ordering: &StateOrdering<StateIndex<I>>,
        old_base: &Base,
        new_base: &Base,
    ) -> Self;
}

impl<T: PermuteStates> PermuteStatesWithContext<T::Ix> for T {
    fn permute_states_with_context<Base: ReadStateSpace<Ix = T::Ix>>(
        &self,
        ordering: &StateOrdering<StateIndex<T::Ix>>,
        _old_base: &Base,
        _new_base: &Base,
    ) -> Self {
        self.permute_states(ordering)
    }
}

impl<I: Ix> PermuteStatesWithContext<I> for () {
    fn permute_states_with_context<Base: ReadStateSpace<Ix = I>>(
        &self,
        _ordering: &StateOrdering<StateIndex<I>>,
        _old_base: &Base,
        _new_base: &Base,
    ) -> Self {
        ()
    }
}

impl<I: Ix> PermuteStatesWithContext<I> for std::convert::Infallible {
    fn permute_states_with_context<Base: ReadStateSpace<Ix = I>>(
        &self,
        _ordering: &StateOrdering<StateIndex<I>>,
        _old_base: &Base,
        _new_base: &Base,
    ) -> Self {
        match *self {}
    }
}

impl<I: Ix, T: PermuteStatesWithContext<I>> PermuteStatesWithContext<I> for Option<T> {
    fn permute_states_with_context<Base: ReadStateSpace<Ix = I>>(
        &self,
        ordering: &StateOrdering<StateIndex<I>>,
        old_base: &Base,
        new_base: &Base,
    ) -> Self {
        self.as_ref()
            .map(|t| t.permute_states_with_context(ordering, old_base, new_base))
    }
}

impl<
    M: PermuteStates + ReadStateSpace,
    Ini: PermuteStatesWithContext<M::Ix>,
    ChLabel: PermuteStatesWithContext<M::Ix>,
    BrLabel: PermuteStatesWithContext<M::Ix>,
    Obs: PermuteStatesWithContext<M::Ix>,
    APs: PermuteStatesWithContext<M::Ix>,
    Rew: PermuteStatesWithContext<M::Ix>,
    Ann: PermuteStatesWithContext<M::Ix>,
    StateVals: PermuteStatesWithContext<M::Ix>,
    Preds: PermuteStatesWithContext<M::Ix>,
> Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    #[must_use]
    pub fn reorder_dfs(&self, successor_order: SuccessorOrder) -> Self
    where
        Self: ReadInitialStates<Ix = M::Ix>,
    {
        assert!(
            self.states().len() < <M::Ix as Ix>::State::max_value().as_usize(),
            "dfs reordering needs the maximal value of the state index type to be larger than the number of states of the model"
        );
        let state_ordering = dfs::compute_dfs_state_ordering(self, successor_order);
        self.reorder(state_ordering)
    }

    pub fn reorder(&self, state_ordering: StateOrdering<StateIndex<M::Ix>>) -> Self {
        let base_model = self.base.permute_states(&state_ordering);
        Model {
            initial: self.initial.permute_states_with_context(
                &state_ordering,
                &self.base,
                &base_model,
            ),
            choice_labels: self.choice_labels.permute_states_with_context(
                &state_ordering,
                &self.base,
                &base_model,
            ),
            branch_labels: self.branch_labels.permute_states_with_context(
                &state_ordering,
                &self.base,
                &base_model,
            ),
            observations: self.observations.permute_states_with_context(
                &state_ordering,
                &self.base,
                &base_model,
            ),
            atomic_propositions: self.atomic_propositions.permute_states_with_context(
                &state_ordering,
                &self.base,
                &base_model,
            ),
            rewards: self.rewards.permute_states_with_context(
                &state_ordering,
                &self.base,
                &base_model,
            ),
            annotations: self.annotations.permute_states_with_context(
                &state_ordering,
                &self.base,
                &base_model,
            ),
            state_valuations: self.state_valuations.permute_states_with_context(
                &state_ordering,
                &self.base,
                &base_model,
            ),
            predecessors: self.predecessors.permute_states_with_context(
                &state_ordering,
                &self.base,
                &base_model,
            ),
            base: base_model,
        }
    }
}
