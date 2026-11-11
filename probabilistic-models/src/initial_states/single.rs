use crate::initial_states::InitialStates;
use crate::{HasIx, Ix, StateIndex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SingleInitialState<I: Ix> {
    pub index: StateIndex<I>,
}

impl<I: Ix> crate::Component for SingleInitialState<I> {}

impl<I: Ix> HasIx for SingleInitialState<I> {
    type Ix = I;
}

impl<I: Ix> SingleInitialState<I> {
    pub fn new(index: StateIndex<I>) -> Self {
        Self { index }
    }

    /// Returns an [`InitialStates`] vector with `num_states` entries in total, where only the state
    /// `self.index` is marked as initial.
    pub fn to_initial_states(&self, num_states: usize) -> InitialStates<I> {
        InitialStates::with_true_values([&self.index], num_states)
    }

    /// Converts the given [`InitialStates`] into a [`SingleInitialState`], if it contains exactly
    /// one initial state.
    pub fn try_from_initial_states(
        initial: &InitialStates<I>,
    ) -> Result<Self, NotSingleInitialStateError> {
        let mut true_values = initial.true_values().into_iter();
        let index = true_values
            .next()
            .ok_or(NotSingleInitialStateError::NoInitialState)?;
        if true_values.next().is_some() {
            return Err(NotSingleInitialStateError::MultipleInitialStates);
        }
        Ok(Self { index })
    }
}

impl<I: Ix> TryFrom<&InitialStates<I>> for SingleInitialState<I> {
    type Error = NotSingleInitialStateError;

    fn try_from(value: &InitialStates<I>) -> Result<Self, Self::Error> {
        Self::try_from_initial_states(value)
    }
}

/// The reason why a set of initial states could not be converted into a [`SingleInitialState`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotSingleInitialStateError {
    NoInitialState,
    MultipleInitialStates,
}

impl std::fmt::Display for NotSingleInitialStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotSingleInitialStateError::NoInitialState => write!(f, "there is no initial state"),
            NotSingleInitialStateError::MultipleInitialStates => {
                write!(f, "there are multiple initial states")
            }
        }
    }
}

impl std::error::Error for NotSingleInitialStateError {}
