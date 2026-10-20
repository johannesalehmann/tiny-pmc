use probabilistic_models::{ChoiceIndex, Ix, StateIndex};
use typed_index_collections::To1;

pub trait ExclusionCriterion<I: Ix> {
    fn is_state_excluded(&self, state: StateIndex<I>) -> bool;
    fn is_choice_excluded(&self, choice: ChoiceIndex<I>) -> bool;
}

impl<I: Ix> ExclusionCriterion<I> for () {
    fn is_state_excluded(&self, _state: StateIndex<I>) -> bool {
        false
    }

    fn is_choice_excluded(&self, _choice: ChoiceIndex<I>) -> bool {
        false
    }
}

pub struct ExcludeStatesAndChoices<I: Ix> {
    pub excluded_states: To1<StateIndex<I>, bool>,
    pub excluded_choices: To1<ChoiceIndex<I>, bool>,
}

impl<I: Ix> ExcludeStatesAndChoices<I> {
    pub fn new(
        excluded_states: To1<StateIndex<I>, bool>,
        excluded_choices: To1<ChoiceIndex<I>, bool>,
    ) -> Self {
        Self {
            excluded_states,
            excluded_choices,
        }
    }
}

impl<I: Ix> ExclusionCriterion<I> for ExcludeStatesAndChoices<I> {
    fn is_state_excluded(&self, state: StateIndex<I>) -> bool {
        self.excluded_states[state]
    }

    fn is_choice_excluded(&self, choice: ChoiceIndex<I>) -> bool {
        self.excluded_choices[choice]
    }
}
