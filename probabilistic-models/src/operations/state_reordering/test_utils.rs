use crate::operations::state_reordering::StateOrdering;
use crate::{StateIndex, Uniform};
use typed_index_collections::{Index, RawIndex, To1};

pub fn state<R: RawIndex>(index: R) -> StateIndex<Uniform<R>> {
    StateIndex::from_raw(index)
}

pub fn ordering<R: RawIndex>(
    entries: Vec<R>,
) -> To1<StateIndex<Uniform<R>>, StateIndex<Uniform<R>>> {
    To1::with_entries(entries.into_iter().map(|i| state(i)).collect::<Vec<_>>())
}

pub fn state_ordering(old_to_new: Vec<usize>) -> StateOrdering<StateIndex<Uniform<usize>>> {
    StateOrdering::new(ordering(old_to_new))
}
