use super::{PermuteStates, StateOrdering};
use crate::annotations::{AtomicPropositions, TypedAnnotation};
use typed_index_collections::{Index, To1};

impl<SI: Index, AI: Index, AEI: Index> PermuteStates for AtomicPropositions<AI, SI, AEI> {
    type StateIndex = SI;

    fn permute_states(&self, ordering: &StateOrdering<SI>) -> Self {
        let mut atomic_propositions = AtomicPropositions::new();
        for (name, old_annotations) in self {
            let n = old_annotations.values().len();
            let ordering_n = ordering.len();
            assert_eq!(
                n, ordering_n,
                "Atomic proposition `{name}` has a value for {n} states, but the state ordering covers {ordering_n} states."
            );
            let mut new_annotations =
                TypedAnnotation::with_identity_distribution_and_entries(To1::with_entries(vec![
                    false;
                    n
                ]));
            for (old, new) in ordering.old_to_new.enumerate() {
                new_annotations[*new] = old_annotations[old];
            }

            atomic_propositions.add_entry(name.to_string(), new_annotations);
        }
        atomic_propositions
    }
}

#[cfg(test)]
mod tests {
    use super::PermuteStates;
    use crate::annotations::{AtomicPropositions, TypedAnnotation};
    use crate::operations::state_reordering::StateOrdering;
    use crate::operations::state_reordering::test_utils::state_ordering;
    use crate::{AnnotationEntryIndex, AnnotationIndex, StateIndex};
    use typed_index_collections::To1;

    type Aps =
        AtomicPropositions<AnnotationIndex<usize>, StateIndex<usize>, AnnotationEntryIndex<usize>>;

    fn aps(entries: &[(&str, &[bool])]) -> Aps {
        let mut aps = AtomicPropositions::new();
        for &(name, values) in entries {
            aps.add_entry(
                name.to_string(),
                TypedAnnotation::with_identity_distribution_and_entries(To1::with_entries(
                    values.to_vec(),
                )),
            );
        }
        aps
    }

    #[test]
    fn identity() {
        let input = aps(&[("a", &[true, false, true, true, false])]);
        let permuted = input.permute_states(&state_ordering(vec![0, 1, 2, 3, 4]));
        assert_eq!(permuted, input);
    }

    #[test]
    fn swap() {
        let input = aps(&[("a", &[true, false])]);
        let permuted = input.permute_states(&state_ordering(vec![1, 0]));
        assert_eq!(permuted, aps(&[("a", &[false, true])]));
    }

    #[test]
    fn cycle() {
        let input = aps(&[("a", &[true, false, false])]);
        let permuted = input.permute_states(&state_ordering(vec![1, 2, 0]));
        assert_eq!(permuted, aps(&[("a", &[false, true, false])]));
    }

    #[test]
    fn multiple_propositions() {
        let input = aps(&[
            ("a", &[true, false, false, false]),
            ("b", &[false, true, true, false]),
            ("c", &[false, false, false, true]),
        ]);
        let permuted = input.permute_states(&state_ordering(vec![2, 0, 3, 1]));
        let expected = aps(&[
            ("a", &[false, false, true, false]),
            ("b", &[true, false, false, true]),
            ("c", &[false, true, false, false]),
        ]);
        assert_eq!(permuted, expected);
        // Check that the order of the names was not changed:
        assert_eq!(permuted.names(), input.names());
    }

    #[test]
    fn constant_propositions() {
        let input = aps(&[("all", &[true; 4]), ("none", &[false; 4])]);
        let permuted = input.permute_states(&state_ordering(vec![3, 1, 0, 2]));
        assert_eq!(permuted, input);
    }

    #[test]
    fn round_trip() {
        let input = aps(&[
            ("a", &[true, false, false, true, false]),
            ("b", &[false, false, true, true, true]),
        ]);
        let forward = state_ordering(vec![2, 4, 0, 1, 3]);
        let backward = StateOrdering::new(forward.new_to_old.clone());
        let permuted = input.permute_states(&forward);
        assert_ne!(permuted, input);
        assert_eq!(permuted.permute_states(&backward), input);
    }

    #[test]
    fn no_propositions() {
        let input = aps(&[]);
        let permuted = input.permute_states(&state_ordering(vec![2, 0, 1]));
        assert_eq!(permuted, aps(&[]));
    }

    #[test]
    #[should_panic(expected = "Atomic proposition `b` has a value for 2 states")]
    fn too_few_values() {
        let input = aps(&[("a", &[true, false, true]), ("b", &[true, false])]);
        input.permute_states(&state_ordering(vec![1, 2, 0]));
    }

    #[test]
    #[should_panic(expected = "Atomic proposition `b` has a value for 4 states")]
    fn too_many_values() {
        let input = aps(&[
            ("a", &[true, false, true]),
            ("b", &[true, false, true, false]),
        ]);
        input.permute_states(&state_ordering(vec![1, 2, 0]));
    }

    #[test]
    fn no_states() {
        let input = aps(&[("a", &[]), ("b", &[])]);
        let permuted = input.permute_states(&state_ordering(vec![]));
        assert_eq!(permuted, aps(&[("a", &[]), ("b", &[])]));
    }
}
