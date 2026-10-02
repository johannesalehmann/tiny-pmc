use typed_index_collections::Index;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SingleInitialState<StateIdx: Index> {
    pub index: StateIdx,
}

impl<SI: Index> crate::Component for SingleInitialState<SI> {}

// TODO: Constructor
// TODO: Conversion to InitialStates
