use probabilistic_models::{Ix, StateIndex};
use std::fmt::{Display, Formatter};

/// The result of checking a query, i.e. a value for each state of interest (usually the initial
/// states).
#[derive(Clone, Debug, PartialEq)]
pub enum CheckingResult<I: Ix> {
    Bool(TypedCheckingResult<I, bool>),
    Float(TypedCheckingResult<I, f64>),
}

impl<I: Ix> CheckingResult<I> {
    pub fn as_bool(&self) -> Option<&TypedCheckingResult<I, bool>> {
        match self {
            CheckingResult::Bool(result) => Some(result),
            CheckingResult::Float(_) => None,
        }
    }

    pub fn as_float(&self) -> Option<&TypedCheckingResult<I, f64>> {
        match self {
            CheckingResult::Bool(_) => None,
            CheckingResult::Float(result) => Some(result),
        }
    }

    pub fn iter(&self) -> Box<dyn Iterator<Item = (StateIndex<I>, StateValue)> + '_> {
        match self {
            CheckingResult::Bool(vals) => {
                Box::new(vals.iter().map(|(state, &value)| (state, value.into())))
            }
            CheckingResult::Float(vals) => {
                Box::new(vals.iter().map(|(state, &value)| (state, value.into())))
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StateValue {
    Bool(bool),
    Float(f64),
}

impl From<bool> for StateValue {
    fn from(value: bool) -> Self {
        StateValue::Bool(value)
    }
}

impl From<f64> for StateValue {
    fn from(value: f64) -> Self {
        StateValue::Float(value)
    }
}

impl Display for StateValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StateValue::Bool(value) => write!(f, "{value}"),
            StateValue::Float(value) => write!(f, "{value}"),
        }
    }
}

/// Maps a set of states to values of type `T`. The entries are sorted by state index, which makes
/// the iteration order deterministic and allows lookups via binary search.
#[derive(Clone, Debug, PartialEq)]
pub struct TypedCheckingResult<I: Ix, T> {
    entries: Vec<(StateIndex<I>, T)>,
}

impl<I: Ix, T> TypedCheckingResult<I, T> {
    pub fn for_single_state(state: StateIndex<I>, value: T) -> Self {
        Self {
            entries: vec![(state, value)],
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn get(&self, state: StateIndex<I>) -> Option<&T> {
        self.entries
            .binary_search_by_key(&state, |(entry_state, _)| *entry_state)
            .ok()
            .map(|position| &self.entries[position].1)
    }

    pub fn iter(&self) -> TypedCheckingResultIter<'_, I, T> {
        self.into_iter()
    }
}

pub type TypedCheckingResultIter<'a, I, T> = std::iter::Map<
    std::slice::Iter<'a, (StateIndex<I>, T)>,
    fn(&'a (StateIndex<I>, T)) -> (StateIndex<I>, &'a T),
>;

impl<'a, I: Ix, T> IntoIterator for &'a TypedCheckingResult<I, T> {
    type Item = (StateIndex<I>, &'a T);
    type IntoIter = TypedCheckingResultIter<'a, I, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter().map(|(state, value)| (*state, value))
    }
}
