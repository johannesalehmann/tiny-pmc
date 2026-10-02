use crate::initial_states::{InitialStates, SingleInitialState};
use crate::traits::{ReadInitialStates, StateSet};
use typed_index_collections::Index;

// TODO: Can we get rid of the ReadInitialStates bounds here?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitialStatesEnum<
    StateIdx: Index,
    Single: ReadInitialStates<StateIdx = StateIdx>,
    Multiple: ReadInitialStates<StateIdx = StateIdx>,
> {
    Single(Single),
    Multiple(Multiple),
}

impl<
    StateIdx: Index,
    Single: ReadInitialStates<StateIdx = StateIdx>,
    Multiple: ReadInitialStates<StateIdx = StateIdx>,
> InitialStatesEnum<StateIdx, Single, Multiple>
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

    pub fn as_ref(&self) -> InitialStatesEnum<StateIdx, &Single, &Multiple> {
        match self {
            InitialStatesEnum::Single(i) => InitialStatesEnum::Single(i),
            InitialStatesEnum::Multiple(i) => InitialStatesEnum::Multiple(i),
        }
    }

    pub fn as_mut(&mut self) -> InitialStatesEnum<StateIdx, &mut Single, &mut Multiple> {
        match self {
            InitialStatesEnum::Single(i) => InitialStatesEnum::Single(i),
            InitialStatesEnum::Multiple(i) => InitialStatesEnum::Multiple(i),
        }
    }
}

impl<
    StateIdx: Index,
    Single: ReadInitialStates<StateIdx = StateIdx>,
    Multiple: ReadInitialStates<StateIdx = StateIdx>,
> ReadInitialStates for InitialStatesEnum<StateIdx, Single, Multiple>
{
    type StateIdx = StateIdx;

    fn is_initial(&self, state: Self::StateIdx) -> bool {
        match self {
            InitialStatesEnum::Single(i) => i.is_initial(state),
            InitialStatesEnum::Multiple(i) => i.is_initial(state),
        }
    }

    fn initial_states(&self) -> impl StateSet<StateIdx = StateIdx> {
        match self {
            InitialStatesEnum::Single(i) => i.initial_states().boxed(),
            InitialStatesEnum::Multiple(i) => i.initial_states().boxed(),
        }
    }
}

// From impls for `Single`

impl<SI: Index> From<SingleInitialState<SI>>
    for InitialStatesEnum<SI, SingleInitialState<SI>, InitialStates<SI>>
{
    fn from(value: SingleInitialState<SI>) -> Self {
        Self::Single(value)
    }
}

impl<'a, SI: Index> From<&'a SingleInitialState<SI>>
    for InitialStatesEnum<SI, &'a SingleInitialState<SI>, &'a InitialStates<SI>>
{
    fn from(value: &'a SingleInitialState<SI>) -> Self {
        Self::Single(value)
    }
}
impl<'a, SI: Index> From<&'a mut SingleInitialState<SI>>
    for InitialStatesEnum<SI, &'a mut SingleInitialState<SI>, &'a mut InitialStates<SI>>
{
    fn from(value: &'a mut SingleInitialState<SI>) -> Self {
        Self::Single(value)
    }
}

// From impls for `Multiple`

impl<SI: Index> From<InitialStates<SI>>
    for InitialStatesEnum<SI, SingleInitialState<SI>, InitialStates<SI>>
{
    fn from(value: InitialStates<SI>) -> Self {
        Self::Multiple(value)
    }
}

impl<'a, SI: Index> From<&'a InitialStates<SI>>
    for InitialStatesEnum<SI, &'a SingleInitialState<SI>, &'a InitialStates<SI>>
{
    fn from(value: &'a InitialStates<SI>) -> Self {
        Self::Multiple(value)
    }
}
impl<'a, SI: Index> From<&'a mut InitialStates<SI>>
    for InitialStatesEnum<SI, &'a mut SingleInitialState<SI>, &'a mut InitialStates<SI>>
{
    fn from(value: &'a mut InitialStates<SI>) -> Self {
        Self::Multiple(value)
    }
}
