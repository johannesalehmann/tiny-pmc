use typed_index_collections::Index;

pub trait ExclusionCriterion<StateIdx: Index> {
    fn is_excluded(&self, state: StateIdx) -> bool;
}

impl<StateIdx: Index> ExclusionCriterion<StateIdx> for () {
    fn is_excluded(&self, _state: StateIdx) -> bool {
        false
    }
}
