use crate::{HasIx, Ix, StateIndex};
use typed_index_collections::To1;

pub type InitialStates<I> = To1<StateIndex<I>, bool>;
// TODO: Conversion to SingleInitialState
// TODO: Constructors from a list of initial states

impl<I: Ix> crate::Component for InitialStates<I> {}

impl<I: Ix> HasIx for InitialStates<I> {
    type Ix = I;
}
