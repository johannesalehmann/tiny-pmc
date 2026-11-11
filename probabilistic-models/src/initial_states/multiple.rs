use crate::{HasIx, Ix, StateIndex};
use typed_index_collections::To1;

/// To convert to [`SingleInitialState`](crate::initial_states::SingleInitialState), use
/// [`SingleInitialState::try_from_initial_states`](crate::initial_states::SingleInitialState::try_from_initial_states)
pub type InitialStates<I> = To1<StateIndex<I>, bool>;
// TODO: Perhaps `InitialStates` should be a separate type instead of an alias to `To1`? This would
//  allow:
//  - constructors (e.g. from a list or from a single initial state)
//  - better-named methods
//  - implementing IntoIterator in a sensible way (i.e. iterating the initial states)
// TODO: Constructors from a list of initial states

impl<I: Ix> crate::Component for InitialStates<I> {}

impl<I: Ix> HasIx for InitialStates<I> {
    type Ix = I;
}
