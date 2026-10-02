use typed_index_collections::{
    Index, MappedIndices, To1, To1BoolValuesIterator, ValuePerIndexSource,
};

pub trait StateSet {
    type StateIdx: Index;
    type IntoIterator: Iterator<Item = Self::StateIdx>;
    fn is_in_set(self, index: Self::StateIdx) -> bool;
    fn iter(self) -> Self::IntoIterator;
    fn boxed<'a>(self) -> BoxedStateSet<'a, Self::StateIdx>
    where
        Self: Sized + 'a,
    {
        BoxedStateSet {
            base: Box::new(self),
        }
    }
}

trait DynStateSet<'a, StateIdx> {
    fn is_in_set_boxed(self: Box<Self>, index: StateIdx) -> bool;
    fn iter_boxed(self: Box<Self>) -> Box<dyn Iterator<Item = StateIdx> + 'a>;
}

impl<'a, S: StateSet + 'a> DynStateSet<'a, S::StateIdx> for S {
    fn is_in_set_boxed(self: Box<Self>, index: S::StateIdx) -> bool {
        (*self).is_in_set(index)
    }

    fn iter_boxed(self: Box<Self>) -> Box<dyn Iterator<Item = S::StateIdx> + 'a> {
        Box::new((*self).iter())
    }
}

pub struct BoxedStateSet<'a, StateIdx: Index> {
    base: Box<dyn DynStateSet<'a, StateIdx> + 'a>,
}

impl<'a, StateIdx: Index> StateSet for BoxedStateSet<'a, StateIdx> {
    type StateIdx = StateIdx;
    type IntoIterator = Box<dyn Iterator<Item = StateIdx> + 'a>;

    fn is_in_set(self, index: StateIdx) -> bool {
        self.base.is_in_set_boxed(index)
    }

    fn iter(self) -> Self::IntoIterator {
        self.base.iter_boxed()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SingleState<StateIdx: Index>(pub StateIdx);

impl<StateIdx: Index> StateSet for SingleState<StateIdx> {
    type StateIdx = StateIdx;
    type IntoIterator = std::iter::Once<StateIdx>;

    fn is_in_set(self, index: StateIdx) -> bool {
        self.0 == index
    }

    fn iter(self) -> Self::IntoIterator {
        std::iter::once(self.0)
    }
}

pub trait AsStateSet: Index {
    fn as_state_set(self) -> SingleState<Self>;
}

impl<StateIdx: Index> AsStateSet for StateIdx {
    fn as_state_set(self) -> SingleState<Self> {
        SingleState(self)
    }
}

impl<'a, StateIdx: Index> StateSet for &'a To1<StateIdx, bool> {
    type StateIdx = StateIdx;
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
impl<'a, OtherIdx: Index, StateIdx: Index> StateSet
    for MappedIndices<'a, OtherIdx, StateIdx, bool>
{
    type StateIdx = StateIdx;
    type IntoIterator =
        To1BoolValuesIterator<StateIdx, MappedIndices<'a, OtherIdx, StateIdx, bool>>;

    fn is_in_set(self, index: StateIdx) -> bool {
        *self.get(index)
    }

    fn iter(self) -> Self::IntoIterator {
        self.true_values().into_iter()
    }
}

impl<'a, StateIdx: Index> StateSet for &'a [StateIdx] {
    type StateIdx = StateIdx;
    type IntoIterator = std::iter::Cloned<std::slice::Iter<'a, StateIdx>>;

    fn is_in_set(self, index: StateIdx) -> bool {
        self.contains(&index)
    }

    fn iter(self) -> Self::IntoIterator {
        self.into_iter().cloned()
    }
}
