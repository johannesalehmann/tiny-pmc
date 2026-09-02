use crate::base_model::BaseModel;
use crate::traits::ReadStateSpace;
use crate::{ChoiceIndex, StateIndex};
use typed_index_collections::{
    Csr, Index, IndexRange, RawIndex, SemiboundedIndexRange, To1, ValuePerIndexSource,
};

#[derive(Default)]
pub struct TransitionSystem<StateIdx: Index, ChoiceIdx: Index> {
    pub state_to_choice: Csr<StateIdx, ChoiceIdx>,
    pub choice_destination: To1<ChoiceIdx, StateIdx>,
}

impl<SI: Index, CI: Index> ReadStateSpace for TransitionSystem<SI, CI> {
    type StateIdx = SI;
    type ChoiceIdx = CI;
    type BranchIdx = CI; // As there is exactly one branch per choice, we can re-use the index here

    fn states(&self) -> SemiboundedIndexRange<Self::StateIdx> {
        self.state_to_choice.keys()
    }

    fn choices(&self) -> SemiboundedIndexRange<Self::ChoiceIdx> {
        self.state_to_choice.values()
    }

    fn branches(&self) -> SemiboundedIndexRange<Self::BranchIdx> {
        self.state_to_choice.values()
    }

    fn choices_of_state(&self, state: Self::StateIdx) -> IndexRange<Self::ChoiceIdx> {
        self.state_to_choice.index(state)
    }

    fn branches_of_choice(&self, choice: Self::ChoiceIdx) -> IndexRange<Self::BranchIdx> {
        IndexRange::with_single_index(choice)
    }

    fn branch_probability(&self, _branch: Self::BranchIdx) -> f64 {
        1.0
    }

    fn branch_destination(&self, branch: Self::BranchIdx) -> Self::StateIdx {
        self.choice_destination[branch]
    }
}

impl<SI: Index, CI: Index> BaseModel for TransitionSystem<SI, CI> {
    type StateIndex = SI;
    type ChoiceIndex = CI;
    type BranchIndex = CI;
}

impl TransitionSystem<StateIndex<usize>, ChoiceIndex<usize>> {
    pub fn with_default_types() -> Self {
        Self::default()
    }
}

impl<SI: Index, CI: Index> TransitionSystem<SI, CI> {
    pub fn add_state(&mut self) -> SI {
        self.state_to_choice
            .add_entry_unchecked(self.choice_destination.keys().end())
    }
    pub fn add_transition(&mut self, destination: SI) -> CI {
        let choice_index = self.choice_destination.add(destination);
        self.state_to_choice
            .extend_last_entry(choice_index + CI::RawType::one());
        choice_index
    }
}
