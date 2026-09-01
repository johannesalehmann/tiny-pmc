use std::marker::PhantomData;
use typed_index_collections::{Index, RawIndex};

pub trait ExclusionCriterion {
    type StateIdx: Index;

    type Iterator<'a>: Iterator<Item = Self::StateIdx>
    where
        Self: 'a;

    fn iter_states<'a>(&'a self) -> Self::Iterator<'a>;

    // This function iterates excluded states by repeatedly calling is_state_excluded. This is
    // useful for quickly implementing iter_states(...). However, if is_state_excluded is expensive
    // and the underlying data structure permits more efficient iteration (e.g. if the excluded
    // states are stored in a list), then it is recommended to manually implement iter_states(...)
    // Due to the lack of associated type defaults, it is not possible to provide a default
    // implementation for iter_states(...).
    fn automatic_iter_states(
        &self,
        model_size: usize,
    ) -> ExclusionCriterionIterator<'_, Self, Self::StateIdx> {
        ExclusionCriterionIterator {
            next_index: Self::StateIdx::default(),
            model_size: Self::StateIdx::from_raw(<Self::StateIdx as Index>::RawType::from_usize(
                model_size,
            )),
            exclusion_criterion: &self,
        }
    }

    fn is_state_excluded(&self, index: Self::StateIdx) -> bool;
}

pub struct ExclusionCriterionIterator<
    'a,
    Ex: ExclusionCriterion<StateIdx = Idx> + ?Sized,
    Idx: Index,
> {
    next_index: Idx,
    model_size: Idx,
    exclusion_criterion: &'a Ex,
}

impl<'a, Ex: ExclusionCriterion<StateIdx = Idx>, Idx: Index> Iterator
    for ExclusionCriterionIterator<'a, Ex, Idx>
{
    type Item = Idx;

    fn next(&mut self) -> Option<Self::Item> {
        while self.next_index < self.model_size {
            if self.exclusion_criterion.is_state_excluded(self.next_index) {
                let result = Some(self.next_index);
                self.next_index += Idx::RawType::one();
                return result;
            }
            self.next_index += Idx::RawType::one();
        }
        None
    }
}

pub struct NoExclusion<StateIdx: Index> {
    phantom_data: PhantomData<StateIdx>,
}

impl<StateIdx: Index> NoExclusion<StateIdx> {
    pub fn new() -> Self {
        Self {
            phantom_data: PhantomData,
        }
    }
}

impl<StateIdx: Index> ExclusionCriterion for NoExclusion<StateIdx> {
    type StateIdx = StateIdx;
    type Iterator<'a>
        = std::iter::Empty<StateIdx>
    where
        StateIdx: 'a;

    fn iter_states(&self) -> Self::Iterator<'_> {
        std::iter::empty()
    }

    fn is_state_excluded(&self, index: StateIdx) -> bool {
        let _ = index;
        false
    }
}

pub struct ExclusionList<'a, StateIdx: Index> {
    excluded_states: &'a [StateIdx],
}

impl<'a, StateIdx: Index> ExclusionList<'a, StateIdx> {
    pub fn new(excluded_states: &'a [StateIdx]) -> Self {
        Self { excluded_states }
    }
}

impl<'a, StateIdx: Index> ExclusionCriterion for ExclusionList<'a, StateIdx> {
    type StateIdx = StateIdx;
    type Iterator<'b>
        = std::iter::Cloned<std::slice::Iter<'b, StateIdx>>
    where
        Self: 'b;

    fn iter_states(&self) -> Self::Iterator<'_> {
        self.excluded_states.iter().cloned()
    }

    fn is_state_excluded(&self, index: StateIdx) -> bool {
        self.excluded_states.contains(&index)
    }
}
