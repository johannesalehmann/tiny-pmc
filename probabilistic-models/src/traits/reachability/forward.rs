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
    use crate::mdp;
    use crate::traits::reachability::Reachability;
    use crate::{BranchIndex, ChoiceIndex, StateIndex};
    use typed_index_collections::To1;

    fn create_mdp() -> (
        Mdp<StateIndex<usize>, ChoiceIndex<usize>, BranchIndex<usize>>,
        StateIndex<usize>,
        StateIndex<usize>,
        StateIndex<usize>,
        StateIndex<usize>,
        StateIndex<usize>,
        StateIndex<usize>,
        StateIndex<usize>,
        StateIndex<usize>,
    ) {
        mdp!(mdp = {
            s0 -> 0.3: s1 & 0.7: s5,
            s1 ->,
            s2 -> 1.0: s4,
            s2 -> 1.0: s5,
            s3 -> 1.0: s3,
            s4 -> 1.0: s2,
            s4 -> 1.0: s4,
            s5 -> 1.0: s0,
            s5 -> 1.0: s6,
            s5 -> 1.0: s5,
            s6 -> 0.3: s3 & 0.7: s6,
            s7 ->
        });
        (mdp, s0, s1, s2, s3, s4, s5, s6, s7)
    }

    #[test]
    fn single_state() {
        let (mdp, s0, s1, s2, s3, s4, s5, s6, _s7) = create_mdp();
        let reachable_states = mdp.reachable_states(s0);
        assert_eq!(true, reachable_states[s0]);
        assert_eq!(true, reachable_states[s1]);
        assert_eq!(false, reachable_states[s2]);
        assert_eq!(true, reachable_states[s3]);
        assert_eq!(false, reachable_states[s4]);
        assert_eq!(true, reachable_states[s5]);
        assert_eq!(true, reachable_states[s6]);
    }

    #[test]
    fn single_state_non_zero_start() {
        let (mdp, s0, s1, s2, s3, s4, s5, s6, s7) = create_mdp();
        let reachable_states = mdp.reachable_states(s6);
        assert_eq!(false, reachable_states[s0]);
        assert_eq!(false, reachable_states[s1]);
        assert_eq!(false, reachable_states[s2]);
        assert_eq!(true, reachable_states[s3]);
        assert_eq!(false, reachable_states[s4]);
        assert_eq!(false, reachable_states[s5]);
        assert_eq!(true, reachable_states[s6]);
        assert_eq!(false, reachable_states[s7]);
    }

    #[test]
    fn multiple_states() {
        let (mdp, s0, s1, s2, s3, s4, s5, s6, s7) = create_mdp();
        let reachable_states = mdp.reachable_states(&[s6, s2][..]);
        assert_eq!(true, reachable_states[s0]);
        assert_eq!(true, reachable_states[s1]);
        assert_eq!(true, reachable_states[s2]);
        assert_eq!(true, reachable_states[s3]);
        assert_eq!(true, reachable_states[s4]);
        assert_eq!(true, reachable_states[s5]);
        assert_eq!(true, reachable_states[s6]);
        assert_eq!(false, reachable_states[s7]);
    }
    #[test]
    fn input_buffer() {
        let (mdp, s0, s1, s2, s3, s4, s5, s6, s7) = create_mdp();
        let mut origin = To1::with_entries(vec![false; 8]);
        origin[s2] = true;
        origin[s6] = true;
        let reachable_states = mdp.reachable_states(&origin);
        assert_eq!(true, reachable_states[s0]);
        assert_eq!(true, reachable_states[s1]);
        assert_eq!(true, reachable_states[s2]);
        assert_eq!(true, reachable_states[s3]);
        assert_eq!(true, reachable_states[s4]);
        assert_eq!(true, reachable_states[s5]);
        assert_eq!(true, reachable_states[s6]);
        assert_eq!(false, reachable_states[s7]);
    }
}
