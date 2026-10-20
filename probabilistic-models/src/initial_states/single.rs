use crate::{HasIx, Ix, StateIndex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SingleInitialState<I: Ix> {
    pub index: StateIndex<I>,
}

impl<I: Ix> crate::Component for SingleInitialState<I> {}

impl<I: Ix> HasIx for SingleInitialState<I> {
    type Ix = I;
}

// TODO: Constructor
// TODO: Conversion to InitialStates
