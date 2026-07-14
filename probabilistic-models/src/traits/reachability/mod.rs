use super::ReadStateSpace;
use crate::traits::state_specifier::StateSet;
use typed_index_collections::{Index, To1};

pub trait Reachability {
    type StateIdx: Index;
    fn reachable_states<S: StateSet<Self::StateIdx>>(&self, from: S) -> To1<Self::StateIdx, bool>;
}

impl<M: ReadStateSpace> Reachability for M {
    type StateIdx = M::StateIdx;

    fn reachable_states<S: StateSet<Self::StateIdx>>(&self, from: S) -> To1<Self::StateIdx, bool> {
        let mut open_states = from.iter().collect::<Vec<_>>();
        let mut buffer = To1::with_entries(vec![false; self.states().len()]);

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
    use crate::builder::{BaseModelBuilder, MdpBuilder};
    use crate::traits::reachability::Reachability;
    use crate::{
        BranchIndex, ChoiceIndex, StateIndex, ValuationClassEntryIndex, ValuationClassIndex,
        ValuationIndex,
    };
    use typed_index_collections::{Index, To1};

    fn create_mdp() -> Mdp<StateIndex<u32>, ChoiceIndex<u32>, BranchIndex<u32>> {
        let mut builder: MdpBuilder<
            StateIndex<u32>,
            ChoiceIndex<u32>,
            BranchIndex<u32>,
            ValuationClassIndex<u32>,
            ValuationClassEntryIndex<u32>,
            ValuationIndex<u32>,
        > = MdpBuilder::default();
        builder.add_state(StateIndex::from_raw(0));
        builder.add_choice_from_slice(&[
            (0.3, StateIndex::from_raw(1)),
            (0.7, StateIndex::from_raw(5)),
        ]);

        builder.add_state(StateIndex::from_raw(1));

        builder.add_state(StateIndex::from_raw(2));
        builder.add_choice_from_slice(&[(1.0, StateIndex::from_raw(4))]);
        builder.add_choice_from_slice(&[(1.0, StateIndex::from_raw(5))]);

        builder.add_state(StateIndex::from_raw(3));
        builder.add_choice_from_slice(&[(1.0, StateIndex::from_raw(3))]);

        builder.add_state(StateIndex::from_raw(4));
        builder.add_choice_from_slice(&[(1.0, StateIndex::from_raw(2))]);
        builder.add_choice_from_slice(&[(1.0, StateIndex::from_raw(4))]);

        builder.add_state(StateIndex::from_raw(5));
        builder.add_choice_from_slice(&[(1.0, StateIndex::from_raw(0))]);
        builder.add_choice_from_slice(&[(1.0, StateIndex::from_raw(6))]);
        builder.add_choice_from_slice(&[(1.0, StateIndex::from_raw(5))]);

        builder.add_state(StateIndex::from_raw(6));
        builder.add_choice_from_slice(&[
            (0.3, StateIndex::from_raw(3)),
            (0.7, StateIndex::from_raw(6)),
        ]);

        builder.add_state(StateIndex::from_raw(7));

        builder.into_base_and_valuations().0
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
