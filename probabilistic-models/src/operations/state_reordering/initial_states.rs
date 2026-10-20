use super::{PermuteStates, StateOrdering};
use crate::initial_states::{InitialStates, InitialStatesEnum, SingleInitialState};
use crate::traits::ReadInitialStates;
use crate::{Ix, StateIndex};

impl<I: Ix> PermuteStates for SingleInitialState<I> {
    fn permute_states(&self, ordering: &StateOrdering<StateIndex<I>>) -> Self {
        SingleInitialState {
            index: ordering.old_to_new[self.index],
        }
    }
}
impl<I: Ix> PermuteStates for InitialStates<I> {
    fn permute_states(&self, ordering: &StateOrdering<StateIndex<I>>) -> Self {
        let (n, ordering_n) = (self.len(), ordering.len());
        assert_eq!(
            n, ordering_n,
            "The initial states cover {n} states, but the state ordering covers {ordering_n} states."
        );
        let mut new_states = InitialStates::with_entries(vec![false; n]);
        for old_initial in self.true_values() {
            new_states[ordering.old_to_new[old_initial]] = true;
        }
        new_states
    }
}

impl<
    I: Ix,
    Single: PermuteStates<Ix = I> + ReadInitialStates<Ix = I>,
    Multiple: PermuteStates<Ix = I> + ReadInitialStates<Ix = I>,
> PermuteStates for InitialStatesEnum<I, Single, Multiple>
{
    fn permute_states(&self, ordering: &StateOrdering<StateIndex<Self::Ix>>) -> Self {
        match self {
            InitialStatesEnum::Single(i) => InitialStatesEnum::Single(i.permute_states(ordering)),
            InitialStatesEnum::Multiple(i) => {
                InitialStatesEnum::Multiple(i.permute_states(ordering))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PermuteStates;
    use crate::Uniform;

    use crate::initial_states::{InitialStates, SingleInitialState};
    use crate::operations::state_reordering::StateOrdering;
    use crate::operations::state_reordering::test_utils::{state, state_ordering};
    use typed_index_collections::To1;

    fn initial_states(values: &[bool]) -> InitialStates<Uniform<usize>> {
        To1::with_entries(values.to_vec())
    }

    fn single(index: usize) -> SingleInitialState<Uniform<usize>> {
        SingleInitialState {
            index: state(index),
        }
    }

    #[test]
    fn single_identity() {
        let ordering = state_ordering(vec![0, 1, 2]);
        for index in 0..3 {
            assert_eq!(single(index).permute_states(&ordering), single(index));
        }
    }

    #[test]
    fn single_cycle() {
        let ordering = state_ordering(vec![1, 2, 0]);
        assert_eq!(single(0).permute_states(&ordering), single(1));
        assert_eq!(single(1).permute_states(&ordering), single(2));
        assert_eq!(single(2).permute_states(&ordering), single(0));
    }

    #[test]
    fn identity() {
        let input = initial_states(&[true, false, true, false]);
        let permuted = input.permute_states(&state_ordering(vec![0, 1, 2, 3]));
        assert_eq!(permuted, input);
    }

    #[test]
    fn one_initial_state() {
        let input = initial_states(&[true, false, false]);
        let permuted = input.permute_states(&state_ordering(vec![1, 2, 0]));
        assert_eq!(permuted, initial_states(&[false, true, false]));
    }

    #[test]
    fn multiple_initial_states() {
        let input = initial_states(&[true, false, true, true, false]);
        let permuted = input.permute_states(&state_ordering(vec![2, 4, 0, 1, 3]));
        assert_eq!(permuted, initial_states(&[true, true, true, false, false]));
    }

    #[test]
    fn constant_initial_states() {
        let ordering = state_ordering(vec![3, 1, 0, 2]);
        for values in [[true; 4], [false; 4]] {
            let input = initial_states(&values);
            assert_eq!(input.permute_states(&ordering), input);
        }
    }

    #[test]
    fn round_trip() {
        let input = initial_states(&[true, false, false, true, false]);
        let forward = state_ordering(vec![2, 4, 0, 1, 3]);
        let backward = StateOrdering::new(forward.new_to_old.clone());
        let permuted = input.permute_states(&forward);
        assert_ne!(permuted, input);
        assert_eq!(permuted.permute_states(&backward), input);
    }

    #[test]
    fn no_states() {
        let permuted = initial_states(&[]).permute_states(&state_ordering(vec![]));
        assert_eq!(permuted, initial_states(&[]));
    }

    #[test]
    #[should_panic(expected = "The initial states cover 2 states, but the state ordering covers 3")]
    fn too_few_values() {
        initial_states(&[true, false]).permute_states(&state_ordering(vec![1, 2, 0]));
    }

    #[test]
    #[should_panic(expected = "The initial states cover 4 states, but the state ordering covers 3")]
    fn too_many_values() {
        initial_states(&[true, false, false, false]).permute_states(&state_ordering(vec![1, 2, 0]));
    }
}
