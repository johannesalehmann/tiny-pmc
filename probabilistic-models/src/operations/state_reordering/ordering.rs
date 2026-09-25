use num_traits::Bounded;
use typed_index_collections::{Index, RawIndex, To1};

pub struct StateOrdering<SI: Index> {
    pub old_to_new: To1<SI, SI>,
    pub new_to_old: To1<SI, SI>,
}

impl<SI: Index> StateOrdering<SI> {
    pub fn new(old_to_new: To1<SI, SI>) -> Self {
        assert!(
            old_to_new.len() <= SI::RawType::max_value().as_usize(),
            "StateOrdering::new can only be called if `old_to_new.len() <= SI::RawType::max_value()`"
        );
        let unassigned = SI::from_raw(SI::RawType::max_value());

        let mut new_to_old = To1::with_entries(vec![unassigned; old_to_new.len()]);
        for (old, &new) in old_to_new.enumerate() {
            if new.raw().as_usize() >= old_to_new.len() {
                panic!(
                    "Invalid `old_to_new` assignment. {old:?} maps to {new:?}, but there are only {} states",
                    old_to_new.len()
                );
            }
            if new_to_old[new] != unassigned {
                panic!(
                    "Invalid `old_to_new` assignment. Both {:?} and {old:?} map to {new:?}.",
                    new_to_old[new]
                );
            }
            new_to_old[new] = old;
        }
        Self {
            old_to_new,
            new_to_old,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::StateIndex;
    use crate::operations::state_reordering::StateOrdering;
    use typed_index_collections::{Index, RawIndex, To1};

    fn state<I: RawIndex>(index: I) -> StateIndex<I> {
        StateIndex::from_raw(index)
    }
    fn ordering<I: RawIndex>(entries: Vec<I>) -> To1<StateIndex<I>, StateIndex<I>> {
        To1::with_entries(entries.into_iter().map(|i| state(i)).collect::<Vec<_>>())
    }
    #[test]
    fn identity() {
        for i in 0..10 {
            let old_to_new = ordering((0..i).collect::<Vec<usize>>());
            let ordering = StateOrdering::new(old_to_new.clone());
            assert_eq!(ordering.old_to_new, old_to_new);
            assert_eq!(ordering.new_to_old, old_to_new);
        }
    }
    #[test]
    fn new_valid() {
        let orderings = [
            (vec![1usize, 0], vec![1, 0]),
            (vec![3, 2, 1, 0], vec![3, 2, 1, 0]),
            (vec![1, 0, 3, 2], vec![1, 0, 3, 2]),
            (vec![1, 2, 3, 0], vec![3, 0, 1, 2]),
            (vec![0, 2, 3, 4, 5, 1], vec![0, 5, 1, 2, 3, 4]),
        ];
        for (old_to_new, new_to_old) in orderings {
            let old_to_new = ordering(old_to_new);
            let new_to_old = ordering(new_to_old);
            let ordering = StateOrdering::new(old_to_new.clone());
            assert_eq!(ordering.old_to_new, old_to_new);
            assert_eq!(ordering.new_to_old, new_to_old);
        }
    }

    #[test]
    #[should_panic(expected = "but there are only 4 states")]
    fn invalid_index() {
        let old_to_new = ordering(vec![0usize, 1, 2, 4]);
        StateOrdering::new(old_to_new);
    }

    #[test]
    #[should_panic(expected = "but there are only 4 states")]
    fn invalid_index_sentinel() {
        // 255 is also the internal "unassigned" marker for `u8` indices
        let old_to_new = ordering(vec![0u8, 1, 2, 255]);
        StateOrdering::new(old_to_new);
    }

    #[test]
    fn maximal_index() {
        // Indices 0..=254 are valid for `u8`, so 255 states is the maximum.
        let old_to_new = To1::<StateIndex<u8>, StateIndex<u8>>::with_entries(
            (0..u8::MAX).rev().map(state).collect::<Vec<_>>(),
        );
        assert_eq!(old_to_new.len(), 255);
        let ordering = StateOrdering::new(old_to_new.clone());
        assert_eq!(ordering.old_to_new, old_to_new);
        // reversal is its own inverse
        assert_eq!(ordering.new_to_old, old_to_new);
    }

    #[test]
    #[should_panic(expected = "StateOrdering::new can only be called")]
    fn exhaustive_index() {
        let old_to_new = To1::<StateIndex<u8>, StateIndex<u8>>::with_entries(
            (0..=u8::MAX).map(state).collect::<Vec<_>>(),
        );
        assert_eq!(old_to_new.len(), 256);
        StateOrdering::new(old_to_new);
    }

    #[test]
    #[should_panic(expected = "Both")]
    fn double_assignment() {
        let old_to_new = ordering(vec![0usize, 0, 2, 3]);
        StateOrdering::new(old_to_new);
    }
    #[test]
    #[should_panic(expected = "Both")]
    fn double_assignment_2() {
        let old_to_new = ordering(vec![0usize, 3, 2, 1, 2, 4]);
        StateOrdering::new(old_to_new);
    }
}
