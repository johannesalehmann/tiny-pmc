use super::StateOrdering;
use crate::traits::{ReadInitialStates, ReadStateSpace, StateSet};
use num_traits::Bounded;
use typed_index_collections::{Index, RawIndex, To1};

// In which order the successors of a state are expanded
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SuccessorOrder {
    // The successors are expanded according to the order in the input model, i.e. choice by choice
    // and branch by branch within a choice.
    Preserve,
    // The states that are reached by actions with the highest probability are expanded first. For
    // performance reasons, this does not merge probabilities across multiple branches, i.e. a
    // transition of the form `s0 -> 0.4: s1 & 0.3: s2 & 0.3: s2` would expand s1 first.
    HighestProbabilityFirst,
}

pub fn compute_dfs_state_ordering<
    SI: Index,
    M: ReadStateSpace<StateIndex = SI> + ReadInitialStates<StateIdx = SI>,
>(
    model: &M,
    successor_order: SuccessorOrder,
) -> StateOrdering<SI> {
    let unassigned = SI::from_raw(SI::RawType::max_value());

    let mut old_to_new: To1<SI, SI> = To1::with_entries(vec![unassigned; model.states().len()]);
    let mut new_to_old: To1<SI, SI> = To1::with_capacity(model.states().len());
    let mut state_counter = SI::from_raw(SI::RawType::zero());

    let initial_states: Vec<SI> = model.initial_states().iter().collect();
    let roots = initial_states.into_iter().chain(model.states());

    let mut open_states = Vec::new();
    // Allocate a vector here for the successors of the state that is currently expanded
    let mut successor_buffer = Vec::new();
    for root in roots {
        if old_to_new[root] != unassigned {
            continue;
        }
        open_states.push(root);
        while let Some(state) = open_states.pop() {
            // A state can be on the stack multiple times, if it is reached again before it is
            // expanded. Only its most recent occurrence, which is the first one to be popped, counts.
            if old_to_new[state] != unassigned {
                continue;
            }
            old_to_new[state] = state_counter;
            new_to_old.add_checked(state_counter, state);
            state_counter += SI::RawType::one();

            successor_buffer.clear();
            // TODO: For SuccessorOrder::Preserve, adding the states to successor_buffer could be
            //  avoided by instead iterating choices and branches in reverse. This requires
            //  implementing a double-ended iterator for IndexRange
            for choice in model.choices_of_state(state) {
                for branch in model.branches_of_choice(choice) {
                    let destination = model.branch_destination(branch);
                    if old_to_new[destination] == unassigned {
                        successor_buffer.push((model.branch_probability(branch), destination));
                    }
                }
            }
            if successor_order == SuccessorOrder::HighestProbabilityFirst {
                successor_buffer.sort_by(|(p1, _), (p2, _)| p2.partial_cmp(p1).unwrap());
            }
            // The stack pops the last element first, so the successors are pushed in reverse.
            open_states.extend(successor_buffer.iter().rev().map(|&(_, dest)| dest));
        }
    }

    StateOrdering {
        old_to_new,
        new_to_old,
    }
}

#[cfg(test)]
mod tests {
    use super::{SuccessorOrder, compute_dfs_state_ordering};
    use crate::initial_states::SingleInitialState;
    use crate::operations::state_reordering::StateOrdering;
    use crate::operations::state_reordering::test_utils::state;
    use crate::traits::{ReadInitialStates, ReadStateSpace};
    use crate::{Model, StateIndex, mdp};
    use SuccessorOrder::{HighestProbabilityFirst, Preserve};
    use typed_index_collections::{Index, To1};

    // Computes the DFS ordering, checks that it is a valid ordering via `StateOrdering` and finally
    // returns the raw indices of the ordering.
    fn dfs<
        M: ReadStateSpace<StateIndex = StateIndex<usize>>
            + ReadInitialStates<StateIdx = StateIndex<usize>>,
    >(
        model: &M,
        successor_order: SuccessorOrder,
    ) -> Vec<usize> {
        let ordering = compute_dfs_state_ordering(model, successor_order);
        let valid = StateOrdering::new(ordering.old_to_new.clone());
        assert_eq!(valid.new_to_old, ordering.new_to_old);
        ordering.old_to_new.iter().map(|new| new.raw()).collect()
    }

    #[test]
    fn single_state() {
        mdp!(m = { s0 -> deadlock });
        let model = Model::new(m).with_initial_state(s0);
        assert_eq!(dfs(&model, Preserve), vec![0]);
        assert_eq!(dfs(&model, HighestProbabilityFirst), vec![0]);
    }

    #[test]
    fn chain() {
        mdp!(m = {
            s0 -> 1.0: s1,
            s1 -> 1.0: s2,
            s2 -> deadlock
        });
        let model = Model::new(m).with_initial_state(s0);
        assert_eq!(dfs(&model, Preserve), vec![0, 1, 2]);
    }

    #[test]
    fn preserve_is_depth_first() {
        // (a breadth-first search would reach s2 before s3)
        mdp!(m = {
            s0 -> 0.5: s1 & 0.5: s2,
            s1 -> 1.0: s3,
            s2 -> deadlock,
            s3 -> deadlock
        });
        let model = Model::new(m).with_initial_state(s0);
        assert_eq!(dfs(&model, Preserve), vec![0, 1, 3, 2]);
    }

    #[test]
    fn preserve_uses_listed_order() {
        mdp!(m = {
            s0 -> 0.5: s2 & 0.5: s1,
            s1 -> deadlock,
            s2 -> deadlock
        });
        let model = Model::new(m).with_initial_state(s0);
        assert_eq!(dfs(&model, Preserve), vec![0, 2, 1]);
    }

    #[test]
    fn preserve_multiple_choices() {
        mdp!(m = {
            s0 -> 1.0: s3,
            s0 -> 0.5: s1 & 0.5: s2,
            s1 -> deadlock,
            s2 -> deadlock,
            s3 -> deadlock
        });
        let model = Model::new(m).with_initial_state(s0);
        assert_eq!(dfs(&model, Preserve), vec![0, 2, 3, 1]);
    }

    #[test]
    fn cycle() {
        mdp!(m = {
            s0 -> 1.0: s1,
            s1 -> 1.0: s0
        });
        let model = Model::new(m).with_initial_state(s0);
        assert_eq!(dfs(&model, Preserve), vec![0, 1]);
    }

    #[test]
    fn diamond() {
        mdp!(m = {
            s0 -> 0.5: s1 & 0.5: s2,
            s1 -> 1.0: s3,
            s2 -> 1.0: s3,
            s3 -> deadlock
        });
        let model = Model::new(m).with_initial_state(s0);
        assert_eq!(dfs(&model, Preserve), vec![0, 1, 3, 2]);
    }

    #[test]
    fn state_reached_again_while_on_the_stack() {
        mdp!(m = {
            s0 -> 0.5: s1 & 0.5: s2,
            s1 -> 1.0: s2,
            s2 -> deadlock
        });
        let model = Model::new(m).with_initial_state(s0);
        assert_eq!(dfs(&model, Preserve), vec![0, 1, 2]);
    }

    #[test]
    fn same_destination_multiple_times() {
        mdp!(m = {
            s0 -> 0.4: s2 & 0.3: s1 & 0.3: s2,
            s1 -> deadlock,
            s2 -> deadlock
        });
        let model = Model::new(m).with_initial_state(s0);
        assert_eq!(dfs(&model, Preserve), vec![0, 2, 1]);

        mdp!(m = {
            s0 -> 1.0: s1,
            s0 -> 1.0: s1,
            s1 -> deadlock
        });
        let model = Model::new(m).with_initial_state(s0);
        assert_eq!(dfs(&model, Preserve), vec![0, 1]);
    }

    #[test]
    fn initial_state_gets_index_zero() {
        mdp!(m = {
            s0 -> 1.0: s1,
            s1 -> 1.0: s2,
            s2 -> deadlock
        });
        let model = Model::new(m).with_initial_state(s2);
        assert_eq!(dfs(&model, Preserve), vec![1, 2, 0]);
    }

    #[test]
    fn unreachable_states() {
        mdp!(m = {
            s0 -> deadlock,
            s1 -> 1.0: s3,
            s2 -> 1.0: s4,
            s3 -> deadlock,
            s4 -> deadlock
        });
        let model = Model::new(m).with_initial_state(s0);
        assert_eq!(dfs(&model, Preserve), vec![0, 1, 3, 2, 4]);
    }

    #[test]
    fn multiple_initial_states() {
        mdp!(m = {
            s0 -> deadlock,
            s1 -> 1.0: s3,
            s2 -> deadlock,
            s3 -> deadlock
        });
        let model =
            Model::new(m).with_initial_states(To1::with_entries(vec![false, true, true, false]));
        assert_eq!(dfs(&model, Preserve), vec![3, 0, 2, 1]);
    }

    #[test]
    fn initial_state_reachable_from_another_initial_state() {
        mdp!(m = {
            s0 -> 1.0: s2,
            s1 -> deadlock,
            s2 -> 1.0: s1
        });
        let model = Model::new(m).with_initial_states(To1::with_entries(vec![true, false, true]));
        assert_eq!(dfs(&model, Preserve), vec![0, 2, 1]);
    }

    #[test]
    fn no_initial_states() {
        mdp!(m = {
            s0 -> 1.0: s2,
            s1 -> deadlock,
            s2 -> deadlock
        });
        let model = Model::new(m).with_initial_states(To1::with_entries(vec![false; 3]));
        assert_eq!(dfs(&model, Preserve), vec![0, 2, 1]);
    }

    #[test]
    fn highest_probability_first() {
        mdp!(m = {
            s0 -> 0.2: s1 & 0.8: s2,
            s1 -> deadlock,
            s2 -> deadlock
        });
        let model = Model::new(m).with_initial_state(s0);
        assert_eq!(dfs(&model, HighestProbabilityFirst), vec![0, 2, 1]);
    }

    #[test]
    fn highest_probability_first_three_successors() {
        mdp!(m = {
            s0 -> 0.2: s1 & 0.5: s2 & 0.3: s3,
            s1 -> deadlock,
            s2 -> deadlock,
            s3 -> deadlock
        });
        let model = Model::new(m).with_initial_state(s0);
        assert_eq!(dfs(&model, HighestProbabilityFirst), vec![0, 3, 1, 2]);
    }

    #[test]
    fn highest_probability_first_compares_across_choices() {
        mdp!(m = {
            s0 -> 0.3: s1 & 0.7: s2,
            s0 -> 0.4: s3 & 0.6: s4,
            s1 -> deadlock,
            s2 -> deadlock,
            s3 -> deadlock,
            s4 -> deadlock
        });
        let model = Model::new(m).with_initial_state(s0);
        assert_eq!(dfs(&model, HighestProbabilityFirst), vec![0, 4, 1, 3, 2]);
    }

    #[test]
    fn highest_probability_first_at_every_level() {
        mdp!(m = {
            s0 -> 0.4: s1 & 0.6: s2,
            s1 -> 0.3: s3 & 0.7: s4,
            s2 -> deadlock,
            s3 -> deadlock,
            s4 -> deadlock
        });
        let model = Model::new(m).with_initial_state(s0);
        assert_eq!(dfs(&model, HighestProbabilityFirst), vec![0, 2, 1, 4, 3]);
    }

    #[test]
    fn multiple_branches_to_same_destination() {
        mdp!(m = {
            s0 -> 0.3: s1 & 0.2: s2 & 0.5: s2,
            s1 -> deadlock,
            s2 -> deadlock
        });
        let model = Model::new(m).with_initial_state(s0);
        assert_eq!(dfs(&model, HighestProbabilityFirst), vec![0, 2, 1]);
        assert_eq!(dfs(&model, Preserve), vec![0, 1, 2]);
    }

    #[test]
    fn highest_probability_first_skips_visited_states() {
        mdp!(m = {
            s0 -> 0.4: s1 & 0.6: s2,
            s1 -> 1.0: s3,
            s2 -> 1.0: s3,
            s3 -> deadlock
        });
        let model = Model::new(m).with_initial_state(s0);
        assert_eq!(dfs(&model, HighestProbabilityFirst), vec![0, 3, 1, 2]);
    }

    #[test]
    fn reorder_dfs_reorders_the_model() {
        mdp!(m = {
            s0 -> 1.0: s1,
            s1 -> 1.0: s2,
            s2 -> 1.0: s0
        });
        let model = Model::new(m).with_initial_state(s1);
        let reordered = model.reorder_dfs(Preserve);
        mdp!(expected = {
            s1 -> 1.0: s2,
            s2 -> 1.0: s0,
            s0 -> 1.0: s1
        });
        assert_eq!(reordered.base, expected);
        assert_eq!(reordered.initial, SingleInitialState { index: state(0) });
    }
}
