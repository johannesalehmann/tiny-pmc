use super::{PermuteStates, StateOrdering};
use crate::base_model::Mdp;
use crate::traits::ReadStateSpace;
use typed_index_collections::Index;

impl<SI: Index, CI: Index, BI: Index> PermuteStates for Mdp<SI, CI, BI> {
    type StateIndex = SI;

    fn permute_states(&self, ordering: &StateOrdering<SI>) -> Self {
        let mut new_mdp = Mdp::default();
        for (new_state, &old_state) in ordering.new_to_old.enumerate() {
            new_mdp.add_state(new_state);
            for choice in self.choices_of_state(old_state) {
                new_mdp.add_choice();
                for branch in self.branches_of_choice(choice) {
                    let p = self.branch_probability(branch);
                    let dest = self.branch_destination(branch);
                    new_mdp.add_branch(p, ordering.old_to_new[dest]);
                }
            }
        }
        new_mdp
    }
}
