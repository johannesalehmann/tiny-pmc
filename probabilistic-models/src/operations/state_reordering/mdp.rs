use super::{PermuteStates, StateOrdering};
use crate::base_model::Mdp;
use crate::traits::ReadStateSpace;
use typed_index_collections::Index;

impl<SI: Index, CI: Index, BI: Index> PermuteStates for Mdp<SI, CI, BI> {
    type StateIndex = SI;

    fn permute_states(&self, ordering: &StateOrdering<SI>) -> Self {
        let (n, ordering_n) = (self.states().len(), ordering.len());
        assert_eq!(
            n, ordering_n,
            "The MDP has {n} states, but the state ordering covers {ordering_n} states."
        );
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

#[cfg(test)]
mod tests {
    use super::PermuteStates;
    use crate::base_model::Mdp;
    use crate::mdp;
    use crate::operations::state_reordering::StateOrdering;
    use crate::operations::state_reordering::test_utils::{state, state_ordering};
    use crate::traits::ReadStateSpace;
    use crate::{BranchIndex, ChoiceIndex, StateIndex};
    use typed_index_collections::Index;

    type TestMdp = Mdp<StateIndex<usize>, ChoiceIndex<usize>, BranchIndex<usize>>;

    /// Builds an MDP from `states[state][choice] = [(probability, destination)]`.
    fn mdp_from(states: &[&[&[(f64, usize)]]]) -> TestMdp {
        let mut mdp = Mdp::with_default_types();
        for (index, choices) in states.iter().enumerate() {
            mdp.add_state(state(index));
            for branches in *choices {
                let branches: Vec<_> = branches.iter().map(|&(p, d)| (p, state(d))).collect();
                mdp.add_choice_from_slice(&branches);
            }
        }
        mdp
    }

    /// The choices of the state with the given index as a list of branches `(probability, destination)`.
    fn choices_of(mdp: &TestMdp, index: usize) -> Vec<Vec<(f64, usize)>> {
        mdp.choices_of_state(state(index))
            .into_iter()
            .map(|choice| {
                mdp.branches_of_choice(choice)
                    .into_iter()
                    .map(|branch| {
                        let destination = mdp.branch_destination(branch).raw();
                        (mdp.branch_probability(branch), destination)
                    })
                    .collect()
            })
            .collect()
    }

    /// All permutations of `0..n`.
    fn permutations(n: usize) -> Vec<Vec<usize>> {
        if n == 0 {
            return vec![vec![]];
        }
        permutations(n - 1)
            .into_iter()
            .flat_map(|permutation| {
                (0..n).map(move |position| {
                    let mut extended = permutation.clone();
                    extended.insert(position, n - 1);
                    extended
                })
            })
            .collect()
    }

    #[test]
    fn identity() {
        mdp!(input = {
            s0 -> 0.5: s1 & 0.5: s2,
            s1 -> 1.0: s1,
            s1 -> 0.3: s0 & 0.7: s2,
            s2 -> 1.0: s0
        });
        let permuted = input.permute_states(&state_ordering(vec![0, 1, 2]));
        assert_eq!(permuted, input);
    }

    #[test]
    fn swap() {
        mdp!(input = {
            a -> 0.4: a & 0.6: b,
            b -> 1.0: a
        });
        mdp!(expected = {
            b -> 1.0: a,
            a -> 0.4: a & 0.6: b
        });
        let permuted = input.permute_states(&state_ordering(vec![1, 0]));
        assert_eq!(permuted, expected);
    }

    #[test]
    fn cycle() {
        mdp!(input = {
            s0 -> 0.5: s1 & 0.5: s2,
            s1 -> 1.0: s2,
            s2 -> 1.0: s0
        });
        mdp!(expected = {
            s2 -> 1.0: s0,
            s0 -> 0.5: s1 & 0.5: s2,
            s1 -> 1.0: s2
        });
        let permuted = input.permute_states(&state_ordering(vec![1, 2, 0]));
        assert_eq!(permuted, expected);
    }

    #[test]
    fn multiple_choices_and_branches() {
        mdp!(input = {
            s0 -> 0.1: s0 & 0.2: s1 & 0.7: s2,
            s0 -> 1.0: s2,
            s1 -> 0.5: s0 & 0.5: s1,
            s2 -> 0.3: s1 & 0.7: s2,
            s2 -> 1.0: s0,
            s2 -> 0.6: s0 & 0.4: s1
        });
        mdp!(expected = {
            s1 -> 0.5: s0 & 0.5: s1,
            s2 -> 0.3: s1 & 0.7: s2,
            s2 -> 1.0: s0,
            s2 -> 0.6: s0 & 0.4: s1,
            s0 -> 0.1: s0 & 0.2: s1 & 0.7: s2,
            s0 -> 1.0: s2
        });
        let permuted = input.permute_states(&state_ordering(vec![2, 0, 1]));
        assert_eq!(permuted, expected);
    }

    #[test]
    fn state_without_choices() {
        let input = mdp_from(&[
            &[&[(0.5, 1), (0.5, 2)], &[(1.0, 2)]], // s0
            &[],                                   // s1
            &[&[(1.0, 0)]],                        // s2
        ]);
        let expected = mdp_from(&[
            &[],                                   // s1
            &[&[(1.0, 2)]],                        // s2
            &[&[(0.5, 0), (0.5, 1)], &[(1.0, 1)]], // s0
        ]);
        let permuted = input.permute_states(&state_ordering(vec![2, 0, 1]));
        assert_eq!(permuted, expected);
    }

    #[test]
    fn empty_choice() {
        // Empty choices are not a valid MDP, but may occur in sub-MDPs with exit values. Let's test
        //  for this just in case
        mdp!(input = {
            s0 -> 1.0: s1,
            s1 ->,
            s2 -> 1.0: s0
        });
        mdp!(expected = {
            s2 -> 1.0: s0,
            s0 -> 1.0: s1,
            s1 ->
        });
        let permuted = input.permute_states(&state_ordering(vec![1, 2, 0]));
        assert_eq!(permuted, expected);
    }

    #[test]
    fn round_trip() {
        mdp!(input = {
            s0 -> 0.1: s0 & 0.2: s1 & 0.7: s2,
            s0 -> 1.0: s2,
            s1 -> 0.5: s0 & 0.5: s1,
            s2 -> 0.3: s1 & 0.7: s2,
            s2 -> 1.0: s0
        });
        let forward = state_ordering(vec![2, 0, 1]);
        let backward = StateOrdering::new(forward.new_to_old.clone());
        let permuted = input.permute_states(&forward);
        assert_ne!(permuted, input);
        assert_eq!(permuted.permute_states(&backward), input);
    }

    #[test]
    fn all_orderings_preserve_structure() {
        mdp!(input = {
            s0 -> 0.5: s1 & 0.5: s3,
            s0 -> 1.0: s0,
            s1 -> 0.25: s0 & 0.25: s2 & 0.5: s3,
            s2 -> 1.0: s2,
            s3 -> 0.9: s1 & 0.1: s2,
            s3 -> 1.0: s0
        });
        for old_to_new in permutations(4) {
            let ordering = state_ordering(old_to_new.clone());
            let permuted = input.permute_states(&ordering);
            assert_eq!(permuted.states().len(), input.states().len());
            assert_eq!(permuted.choices().len(), input.choices().len());
            assert_eq!(permuted.branches().len(), input.branches().len());
            for old in 0..4 {
                let expected: Vec<Vec<(f64, usize)>> = choices_of(&input, old)
                    .into_iter()
                    .map(|branches| {
                        branches
                            .into_iter()
                            .map(|(p, destination)| (p, old_to_new[destination]))
                            .collect()
                    })
                    .collect();
                assert_eq!(
                    choices_of(&permuted, old_to_new[old]),
                    expected,
                    "wrong choices for state {old} under the ordering {old_to_new:?}"
                );
            }
            let backward = StateOrdering::new(ordering.new_to_old.clone());
            assert_eq!(permuted.permute_states(&backward), input);
        }
    }

    #[test]
    #[should_panic(expected = "The MDP has 3 states, but the state ordering covers 2")]
    fn ordering_too_short() {
        mdp!(input = {
            s0 -> 1.0: s1,
            s1 -> 1.0: s2,
            s2 -> 1.0: s0
        });
        input.permute_states(&state_ordering(vec![1, 0]));
    }

    #[test]
    #[should_panic(expected = "The MDP has 3 states, but the state ordering covers 4")]
    fn ordering_too_long() {
        mdp!(input = {
            s0 -> 1.0: s1,
            s1 -> 1.0: s2,
            s2 -> 1.0: s0
        });
        input.permute_states(&state_ordering(vec![1, 2, 0, 3]));
    }

    #[test]
    fn no_states() {
        let permuted = mdp_from(&[]).permute_states(&state_ordering(vec![]));
        assert_eq!(permuted, mdp_from(&[]));
    }
}
