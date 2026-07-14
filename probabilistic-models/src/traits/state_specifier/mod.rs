use typed_index_collections::{Index, To1, To1BoolValuesIterator};

pub trait StateSet<StateIdx: Index> {
    type IntoIterator<'a>: Iterator<Item = StateIdx>
    where
        StateIdx: 'a,
        Self: 'a;
    fn is_in_set(&self, index: StateIdx) -> bool;
    fn iter<'a>(&'a self) -> Self::IntoIterator<'a>;
}

impl<StateIdx: Index> StateSet<StateIdx> for StateIdx {
    type IntoIterator<'a>
        = std::iter::Once<StateIdx>
    where
        StateIdx: 'a;

    fn is_in_set(&self, index: StateIdx) -> bool {
        *self == index
    }

    fn iter<'a>(&'a self) -> Self::IntoIterator<'a> {
        std::iter::once(*self)
    }
}

impl<StateIdx: Index> StateSet<StateIdx> for To1<StateIdx, bool> {
    type IntoIterator<'a>
        = To1BoolValuesIterator<'a, StateIdx>
    where
        StateIdx: 'a;

    fn is_in_set(&self, index: StateIdx) -> bool {
        self[index]
    }

    fn iter<'a>(&'a self) -> Self::IntoIterator<'a> {
        self.true_values().into_iter()
    }
}

impl<StateIdx: Index> StateSet<StateIdx> for &[StateIdx] {
    type IntoIterator<'a>
        = std::iter::Cloned<std::slice::Iter<'a, StateIdx>>
    where
        StateIdx: 'a,
        Self: 'a;

    fn is_in_set(&self, index: StateIdx) -> bool {
        self.contains(&index)
    }

    fn iter<'a>(&'a self) -> Self::IntoIterator<'a> {
        self.into_iter().cloned()
    }
}
