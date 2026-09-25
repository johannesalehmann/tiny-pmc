use crate::StateIndex;
use crate::operations::state_reordering::StateOrdering;
use typed_index_collections::{Index, RawIndex, To1};

pub fn state<I: RawIndex>(index: I) -> StateIndex<I> {
    StateIndex::from_raw(index)
}

pub fn ordering<I: RawIndex>(entries: Vec<I>) -> To1<StateIndex<I>, StateIndex<I>> {
    To1::with_entries(entries.into_iter().map(|i| state(i)).collect::<Vec<_>>())
}

pub fn state_ordering(old_to_new: Vec<usize>) -> StateOrdering<StateIndex<usize>> {
    StateOrdering::new(ordering(old_to_new))
}
