use crate::initial_states::{InitialStates, SingleInitialState};
use crate::traits::{ReadInitialStates, StateSet};
use crate::{HasIx, Ix};

// TODO: Can we get rid of the ReadInitialStates bounds here?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitialStatesEnum<
    I: Ix,
    Single: ReadInitialStates<Ix = I> = SingleInitialState<I>,
    Multiple: ReadInitialStates<Ix = I> = InitialStates<I>,
> {
    Single(Single),
    Multiple(Multiple),
}
impl<I: Ix, S: ReadInitialStates<Ix = I>, M: ReadInitialStates<Ix = I>> crate::Component
    for InitialStatesEnum<I, S, M>
{
}

impl<I: Ix, S: ReadInitialStates<Ix = I>, M: ReadInitialStates<Ix = I>> HasIx
    for InitialStatesEnum<I, S, M>
{
    type Ix = I;
}

impl<I: Ix, Single: ReadInitialStates<Ix = I>, Multiple: ReadInitialStates<Ix = I>>
    InitialStatesEnum<I, Single, Multiple>
{
    pub fn is_single(&self) -> bool {
        match self {
            InitialStatesEnum::Single(_) => true,
            InitialStatesEnum::Multiple(_) => false,
        }
    }
    pub fn is_multiple(&self) -> bool {
        match self {
            InitialStatesEnum::Single(_) => false,
            InitialStatesEnum::Multiple(_) => true,
        }
    }
    pub fn into_single(self) -> Option<Single> {
        match self {
            InitialStatesEnum::Single(i) => Some(i),
            InitialStatesEnum::Multiple(_) => None,
        }
    }
    pub fn into_multiple(self) -> Option<Multiple> {
        match self {
            InitialStatesEnum::Single(_) => None,
            InitialStatesEnum::Multiple(i) => Some(i),
        }
    }

    // TODO: Distinguish between the "unwrapping" into_multiple and another function that always
    //  returns Multiple by potentially constructing a new Multiple from the current Single

    pub fn as_ref(&self) -> InitialStatesEnum<I, &Single, &Multiple> {
        match self {
            InitialStatesEnum::Single(i) => InitialStatesEnum::Single(i),
            InitialStatesEnum::Multiple(i) => InitialStatesEnum::Multiple(i),
        }
    }

    pub fn as_mut(&mut self) -> InitialStatesEnum<I, &mut Single, &mut Multiple> {
        match self {
            InitialStatesEnum::Single(i) => InitialStatesEnum::Single(i),
            InitialStatesEnum::Multiple(i) => InitialStatesEnum::Multiple(i),
        }
    }
}

impl<I: Ix, Single: ReadInitialStates<Ix = I>, Multiple: ReadInitialStates<Ix = I>>
    ReadInitialStates for InitialStatesEnum<I, Single, Multiple>
{
    fn is_initial(&self, state: crate::StateIndex<I>) -> bool {
        match self {
            InitialStatesEnum::Single(i) => i.is_initial(state),
            InitialStatesEnum::Multiple(i) => i.is_initial(state),
        }
    }

    fn initial_states(&self) -> impl StateSet<StateIdx = crate::StateIndex<I>> {
        match self {
            InitialStatesEnum::Single(i) => i.initial_states().boxed(),
            InitialStatesEnum::Multiple(i) => i.initial_states().boxed(),
        }
    }
}

// From impls for `Single`

impl<I: Ix> From<SingleInitialState<I>>
    for InitialStatesEnum<I, SingleInitialState<I>, InitialStates<I>>
{
    fn from(value: SingleInitialState<I>) -> Self {
        Self::Single(value)
    }
}

impl<'a, I: Ix> From<&'a SingleInitialState<I>>
    for InitialStatesEnum<I, &'a SingleInitialState<I>, &'a InitialStates<I>>
{
    fn from(value: &'a SingleInitialState<I>) -> Self {
        Self::Single(value)
    }
}
impl<'a, I: Ix> From<&'a mut SingleInitialState<I>>
    for InitialStatesEnum<I, &'a mut SingleInitialState<I>, &'a mut InitialStates<I>>
{
    fn from(value: &'a mut SingleInitialState<I>) -> Self {
        Self::Single(value)
    }
}

// From impls for `Multiple`

impl<I: Ix> From<InitialStates<I>>
    for InitialStatesEnum<I, SingleInitialState<I>, InitialStates<I>>
{
    fn from(value: InitialStates<I>) -> Self {
        Self::Multiple(value)
    }
}

impl<'a, I: Ix> From<&'a InitialStates<I>>
    for InitialStatesEnum<I, &'a SingleInitialState<I>, &'a InitialStates<I>>
{
    fn from(value: &'a InitialStates<I>) -> Self {
        Self::Multiple(value)
    }
}
impl<'a, I: Ix> From<&'a mut InitialStates<I>>
    for InitialStatesEnum<I, &'a mut SingleInitialState<I>, &'a mut InitialStates<I>>
{
    fn from(value: &'a mut InitialStates<I>) -> Self {
        Self::Multiple(value)
    }
}
