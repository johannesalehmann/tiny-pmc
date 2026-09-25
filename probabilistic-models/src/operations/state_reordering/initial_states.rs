use super::{PermuteStates, StateOrdering};
use crate::InitialStates;
use crate::initial_states::SingleInitialState;
use typed_index_collections::Index;

impl<SI: Index> PermuteStates for SingleInitialState<SI> {
    type StateIndex = SI;

    fn permute_states(&self, ordering: &StateOrdering<SI>) -> Self {
        SingleInitialState {
            index: ordering.old_to_new[self.index],
        }
    }
}
impl<SI: Index> PermuteStates for InitialStates<SI> {
    type StateIndex = SI;

    fn permute_states(&self, ordering: &StateOrdering<SI>) -> Self {
        let mut new_states = InitialStates::with_entries(vec![false; self.len()]);
        for old_initial in self.true_values() {
            new_states[ordering.old_to_new[old_initial]] = true;
        }
        new_states
    }
}
