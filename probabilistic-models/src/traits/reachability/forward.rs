use crate::traits::ReadStateSpace;
use crate::traits::state_specifier::StateSet;
use typed_index_collections::{Index, To1};

pub trait Reachability {
    type StateIdx: Index;
    // TODO: Allow user to provide reusable buffer and open state list to reduce allocations (and
    //  do the same for backward reachability
    fn reachable_states<S: StateSet<Self::StateIdx>>(&self, from: S) -> To1<Self::StateIdx, bool>;
}

impl<M: ReadStateSpace> Reachability for M {
    type StateIdx = M::StateIdx;

    fn reachable_states<S: StateSet<Self::StateIdx>>(&self, from: S) -> To1<Self::StateIdx, bool> {
        let mut open_states = from.iter().collect::<Vec<_>>();
        let mut buffer = To1::with_entries(vec![false; self.states().len()]);

        for &state in &open_states {
            buffer[state] = true;
        }

        while let Some(open) = open_states.pop() {
            for choice in self.choices_of_state(open) {
                for branch in self.branches_of_choice(choice) {
                    if self.branch_probability(branch) > 0.0 {
                        let destination = self.branch_destination(branch);
                        if !buffer[destination] {
                            buffer[destination] = true;
                            open_states.push(destination);
                        }
                    }
                }
            }
        }

        buffer
    }
}

#[cfg(test)]
mod tests {
    use crate::base_model::Mdp;
    use crate::traits::reachability::Reachability;
    use crate::{BranchIndex, ChoiceIndex, StateIndex};
    use typed_index_collections::{Index, To1};

    fn create_mdp() -> Mdp<StateIndex<usize>, ChoiceIndex<usize>, BranchIndex<usize>> {
        let mut mdp = Mdp::with_default_types();
        mdp.add_state(StateIndex::from_raw(0));
        mdp.add_choice_from_slice(&[
            (0.3, StateIndex::from_raw(1)),
            (0.7, StateIndex::from_raw(5)),
        ]);

        mdp.add_state(StateIndex::from_raw(1));

        mdp.add_state(StateIndex::from_raw(2));
        mdp.add_choice_from_slice(&[(1.0, StateIndex::from_raw(4))]);
        mdp.add_choice_from_slice(&[(1.0, StateIndex::from_raw(5))]);

        mdp.add_state(StateIndex::from_raw(3));
        mdp.add_choice_from_slice(&[(1.0, StateIndex::from_raw(3))]);

        mdp.add_state(StateIndex::from_raw(4));
        mdp.add_choice_from_slice(&[(1.0, StateIndex::from_raw(2))]);
        mdp.add_choice_from_slice(&[(1.0, StateIndex::from_raw(4))]);

        mdp.add_state(StateIndex::from_raw(5));
        mdp.add_choice_from_slice(&[(1.0, StateIndex::from_raw(0))]);
        mdp.add_choice_from_slice(&[(1.0, StateIndex::from_raw(6))]);
        mdp.add_choice_from_slice(&[(1.0, StateIndex::from_raw(5))]);

        mdp.add_state(StateIndex::from_raw(6));
        mdp.add_choice_from_slice(&[
            (0.3, StateIndex::from_raw(3)),
            (0.7, StateIndex::from_raw(6)),
        ]);

        mdp.add_state(StateIndex::from_raw(7));

        mdp
    }

    #[test]
    fn single_state() {
        let mdp = create_mdp();
        let reachable_states = mdp.reachable_states(StateIndex::from_raw(0));
        assert_eq!(true, reachable_states[StateIndex::from_raw(0)]);
        assert_eq!(true, reachable_states[StateIndex::from_raw(1)]);
        assert_eq!(false, reachable_states[StateIndex::from_raw(2)]);
        assert_eq!(true, reachable_states[StateIndex::from_raw(3)]);
        assert_eq!(false, reachable_states[StateIndex::from_raw(4)]);
        assert_eq!(true, reachable_states[StateIndex::from_raw(5)]);
        assert_eq!(true, reachable_states[StateIndex::from_raw(6)]);
    }

    #[test]
    fn single_state_non_zero_start() {
        let mdp = create_mdp();
        let reachable_states = mdp.reachable_states(StateIndex::from_raw(6));
        assert_eq!(false, reachable_states[StateIndex::from_raw(0)]);
        assert_eq!(false, reachable_states[StateIndex::from_raw(1)]);
        assert_eq!(false, reachable_states[StateIndex::from_raw(2)]);
        assert_eq!(true, reachable_states[StateIndex::from_raw(3)]);
        assert_eq!(false, reachable_states[StateIndex::from_raw(4)]);
        assert_eq!(false, reachable_states[StateIndex::from_raw(5)]);
        assert_eq!(true, reachable_states[StateIndex::from_raw(6)]);
        assert_eq!(false, reachable_states[StateIndex::from_raw(7)]);
    }

    #[test]
    fn multiple_states() {
        let mdp = create_mdp();
        let reachable_states =
            mdp.reachable_states(&[StateIndex::from_raw(6), StateIndex::from_raw(2)][..]);
        assert_eq!(true, reachable_states[StateIndex::from_raw(0)]);
        assert_eq!(true, reachable_states[StateIndex::from_raw(1)]);
        assert_eq!(true, reachable_states[StateIndex::from_raw(2)]);
        assert_eq!(true, reachable_states[StateIndex::from_raw(3)]);
        assert_eq!(true, reachable_states[StateIndex::from_raw(4)]);
        assert_eq!(true, reachable_states[StateIndex::from_raw(5)]);
        assert_eq!(true, reachable_states[StateIndex::from_raw(6)]);
        assert_eq!(false, reachable_states[StateIndex::from_raw(7)]);
    }
    #[test]
    fn input_buffer() {
        let mdp = create_mdp();
        let origin = To1::with_entries(vec![false, false, true, false, false, false, true, false]);
        let reachable_states = mdp.reachable_states(&origin);
        assert_eq!(true, reachable_states[StateIndex::from_raw(0)]);
        assert_eq!(true, reachable_states[StateIndex::from_raw(1)]);
        assert_eq!(true, reachable_states[StateIndex::from_raw(2)]);
        assert_eq!(true, reachable_states[StateIndex::from_raw(3)]);
        assert_eq!(true, reachable_states[StateIndex::from_raw(4)]);
        assert_eq!(true, reachable_states[StateIndex::from_raw(5)]);
        assert_eq!(true, reachable_states[StateIndex::from_raw(6)]);
        assert_eq!(false, reachable_states[StateIndex::from_raw(7)]);
    }
}
