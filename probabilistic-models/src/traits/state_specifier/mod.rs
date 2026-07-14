use typed_index_collections::{
    Index, MappedIndices, To1, To1BoolValuesIterator, ValuePerIndexSource,
};

pub trait StateSet<StateIdx: Index> {
    type IntoIterator: Iterator<Item = StateIdx>;
    fn is_in_set(self, index: StateIdx) -> bool;
    fn iter(self) -> Self::IntoIterator;
}

impl<StateIdx: Index> StateSet<StateIdx> for &StateIdx {
    type IntoIterator = std::iter::Once<StateIdx>;

    fn is_in_set(self, index: StateIdx) -> bool {
        *self == index
    }

    fn iter(self) -> Self::IntoIterator {
        std::iter::once(*self)
    }
}
impl<StateIdx: Index> StateSet<StateIdx> for StateIdx {
    type IntoIterator = std::iter::Once<StateIdx>;

    fn is_in_set(self, index: StateIdx) -> bool {
        self == index
    }

    fn iter(self) -> Self::IntoIterator {
        std::iter::once(self)
    }
}

impl<'a, StateIdx: Index> StateSet<StateIdx> for &'a To1<StateIdx, bool> {
    type IntoIterator = To1BoolValuesIterator<StateIdx, &'a To1<StateIdx, bool>>;

    fn is_in_set(self, index: StateIdx) -> bool {
        self[index]
    }

    fn iter(self) -> Self::IntoIterator {
        self.true_values().into_iter()
    }
}

// TODO: It would be nicer to generically implement this trait for all operations that can be
//  applied to To1
impl<'a, OtherIdx: Index, StateIdx: Index> StateSet<StateIdx>
    for MappedIndices<'a, OtherIdx, StateIdx, bool>
{
    type IntoIterator =
        To1BoolValuesIterator<StateIdx, MappedIndices<'a, OtherIdx, StateIdx, bool>>;

    fn is_in_set(self, index: StateIdx) -> bool {
        *self.get(index)
    }

    fn iter(self) -> Self::IntoIterator {
        self.true_values().into_iter()
    }
}

impl<'a, StateIdx: Index> StateSet<StateIdx> for &'a [StateIdx] {
    type IntoIterator = std::iter::Cloned<std::slice::Iter<'a, StateIdx>>;

    fn is_in_set(self, index: StateIdx) -> bool {
        self.contains(&index)
    }

    fn iter(self) -> Self::IntoIterator {
        self.into_iter().cloned()
    }
}
