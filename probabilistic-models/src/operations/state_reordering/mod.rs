mod atomic_propositions;
mod dfs;
mod initial_states;
mod mdp;
mod ordering;
mod rewards;
#[cfg(test)]
mod test_utils;

pub use dfs::SuccessorOrder;
pub use ordering::StateOrdering;

use crate::Model;
use crate::traits::{ReadInitialStates, ReadStateSpace};
use num_traits::Bounded;
use typed_index_collections::{Index, RawIndex};

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

impl<
    SI: Index,
    CI: Index,
    M: PermuteStates<StateIndex = SI> + ReadStateSpace<StateIndex = SI, ChoiceIndex = CI>,
    Ini: PermuteStatesWithContext<SI, CI>,
    ChLabel: PermuteStatesWithContext<SI, CI>,
    BrLabel: PermuteStatesWithContext<SI, CI>,
    Obs: PermuteStatesWithContext<SI, CI>,
    APs: PermuteStatesWithContext<SI, CI>,
    Rew: PermuteStatesWithContext<SI, CI>,
    Ann: PermuteStatesWithContext<SI, CI>,
    StateVals: PermuteStatesWithContext<SI, CI>,
    Preds: PermuteStatesWithContext<SI, CI>,
> Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    #[must_use]
    pub fn reorder_dfs(&self, successor_order: SuccessorOrder) -> Self
    where
        Self: ReadStateSpace<StateIndex = SI> + ReadInitialStates<StateIdx = SI>,
    {
        assert!(
            self.states().len() < SI::RawType::max_value().as_usize(),
            "dfs reordering needs the maximal value of the state index type to be larger than the number of states of the model"
        );
        let state_ordering = dfs::compute_dfs_state_ordering(self, successor_order);
        self.reorder(state_ordering)
    }

    pub fn reorder(&self, state_ordering: StateOrdering<SI>) -> Self {
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
