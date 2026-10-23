use super::{PermuteStates, StateOrdering};
use crate::base_model::{BranchGraph, ChoiceBranchGraph, ChoiceGraph};
use crate::traits::{
    BuildBranchGraph, BuildChoiceBranchGraph, BuildChoiceGraph, BuildStates,
    ReadChoiceDestinations, ReadStateBranches, ReadStateSpace,
};
use crate::{Ix, StateIndex};

fn assert_ordering_covers_states<I: Ix>(n: usize, ordering: &StateOrdering<StateIndex<I>>) {
    let ordering_n = ordering.len();
    assert_eq!(
        n, ordering_n,
        "The model has {n} states, but the state ordering covers {ordering_n} states."
    );
}

impl<I: Ix> PermuteStates for ChoiceBranchGraph<I> {
    fn permute_states(&self, ordering: &StateOrdering<StateIndex<I>>) -> Self {
        assert_ordering_covers_states(self.states().len(), ordering);
        let mut new_graph = ChoiceBranchGraph::default();
        for &old_state in &ordering.new_to_old {
            new_graph.add_state();
            for choice in self.choices_of_state(old_state) {
                new_graph.add_choice();
                for branch in self.branches_of_choice(choice) {
                    let p = self.branch_probability(branch);
                    let dest = self.branch_destination(branch);
                    new_graph.add_branch(p, ordering.old_to_new[dest]);
                }
            }
        }
        new_graph
    }
}

impl<I: Ix> PermuteStates for ChoiceGraph<I> {
    fn permute_states(&self, ordering: &StateOrdering<StateIndex<I>>) -> Self {
        assert_ordering_covers_states(self.states().len(), ordering);
        let mut new_graph = ChoiceGraph::default();
        for &old_state in &ordering.new_to_old {
            new_graph.add_state();
            for choice in self.choices_of_state(old_state) {
                let dest = self.choice_destination(choice);
                new_graph.add_choice(ordering.old_to_new[dest]);
            }
        }
        new_graph
    }
}

impl<I: Ix> PermuteStates for BranchGraph<I> {
    fn permute_states(&self, ordering: &StateOrdering<StateIndex<I>>) -> Self {
        assert_ordering_covers_states(self.states().len(), ordering);
        let mut new_graph = BranchGraph::default();
        for &old_state in &ordering.new_to_old {
            new_graph.add_state();
            for branch in self.branches_of_state(old_state) {
                let p = self.branch_probability(branch);
                let dest = self.branch_destination(branch);
                new_graph.add_branch(p, ordering.old_to_new[dest]);
            }
        }
        new_graph
    }
}
