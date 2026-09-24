use typed_index_collections::{Index, To1};

pub trait ExclusionCriterion<StateIdx: Index, ChoiceIdx: Index> {
    fn is_state_excluded(&self, state: StateIdx) -> bool;
    fn is_choice_excluded(&self, choice: ChoiceIdx) -> bool;
}

impl<StateIdx: Index, ChoiceIdx: Index> ExclusionCriterion<StateIdx, ChoiceIdx> for () {
    fn is_state_excluded(&self, _state: StateIdx) -> bool {
        false
    }

    fn is_choice_excluded(&self, _choice: ChoiceIdx) -> bool {
        false
    }
}

pub struct ExcludeStatesAndChoices<StateIdx: Index, ChoiceIdx: Index> {
    pub excluded_states: To1<StateIdx, bool>,
    pub excluded_choices: To1<ChoiceIdx, bool>,
}

impl<StateIdx: Index, ChoiceIdx: Index> ExcludeStatesAndChoices<StateIdx, ChoiceIdx> {
    pub fn new(
        excluded_states: To1<StateIdx, bool>,
        excluded_choices: To1<ChoiceIdx, bool>,
    ) -> Self {
        Self {
            excluded_states,
            excluded_choices,
        }
    }
}

impl<StateIdx: Index, ChoiceIdx: Index> ExclusionCriterion<StateIdx, ChoiceIdx>
    for ExcludeStatesAndChoices<StateIdx, ChoiceIdx>
{
    fn is_state_excluded(&self, state: StateIdx) -> bool {
        self.excluded_states[state]
    }

    fn is_choice_excluded(&self, choice: ChoiceIdx) -> bool {
        self.excluded_choices[choice]
    }
}
