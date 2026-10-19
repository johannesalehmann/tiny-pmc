use crate::traits::{
    BuildBranchGraph, BuildChoiceBranchGraph, BuildChoiceGraph, BuildStates, ClearModel,
    ReadChoiceDestinations, ReadStateBranches, ReadStateSpace,
};
use typed_index_collections::{Csr, Index, IndexRange, RawIndex, SemiboundedIndexRange, To1};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ChoiceBranchGraph<StateIdx: Index, ChoiceIdx: Index, BranchIdx: Index> {
    pub state_to_choice: Csr<StateIdx, ChoiceIdx>,
    pub choice_to_branch: Csr<ChoiceIdx, BranchIdx>,
    pub branch_probabilities: To1<BranchIdx, f64>,
    pub branch_destinations: To1<BranchIdx, StateIdx>,
}

impl<StateIdx: Index, ChoiceIdx: Index, BranchIdx: Index> ReadStateSpace
    for ChoiceBranchGraph<StateIdx, ChoiceIdx, BranchIdx>
{
    type StateIndex = StateIdx;
    type ChoiceIndex = ChoiceIdx;
    type BranchIndex = BranchIdx;

    fn states(&self) -> SemiboundedIndexRange<Self::StateIndex> {
        self.state_to_choice.keys()
    }

    fn choices(&self) -> SemiboundedIndexRange<Self::ChoiceIndex> {
        self.choice_to_branch.keys()
    }

    fn branches(&self) -> SemiboundedIndexRange<Self::BranchIndex> {
        self.choice_to_branch.values()
    }

    fn choices_of_state(&self, state: Self::StateIndex) -> IndexRange<Self::ChoiceIndex> {
        self.state_to_choice.index(state)
    }

    fn branches_of_choice(&self, choice: Self::ChoiceIndex) -> IndexRange<Self::BranchIndex> {
        self.choice_to_branch.index(choice)
    }

    fn branch_probability(&self, branch: Self::BranchIndex) -> f64 {
        self.branch_probabilities[branch]
    }

    fn branch_destination(&self, branch: Self::BranchIndex) -> Self::StateIndex {
        self.branch_destinations[branch]
    }
}

impl<StateIdx: Index, ChoiceIdx: Index, BranchIdx: Index> BuildStates
    for ChoiceBranchGraph<StateIdx, ChoiceIdx, BranchIdx>
{
    fn add_state(&mut self) -> Self::StateIndex {
        self.state_to_choice.add_empty_entry()
    }
}

impl<StateIdx: Index, ChoiceIdx: Index, BranchIdx: Index> BuildChoiceBranchGraph
    for ChoiceBranchGraph<StateIdx, ChoiceIdx, BranchIdx>
{
    fn add_choice(&mut self) -> Self::ChoiceIndex {
        let choice = self.choice_to_branch.add_empty_entry();
        self.state_to_choice
            .extend_last_entry(choice + ChoiceIdx::RawType::one());
        choice
    }

    fn add_branch(&mut self, probability: f64, destination: Self::StateIndex) -> Self::BranchIndex {
        let branch = self.branch_destinations.add(destination);
        self.branch_probabilities.add(probability);
        self.choice_to_branch
            .extend_last_entry(branch + BranchIdx::RawType::one());
        branch
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ChoiceGraph<StateIdx: Index, ChoiceIdx: Index> {
    pub state_to_choice: Csr<StateIdx, ChoiceIdx>,
    pub choice_destinations: To1<ChoiceIdx, StateIdx>,
}

// Every choice has exactly one branch, so choices double as branches.
impl<StateIdx: Index, ChoiceIdx: Index> ReadStateSpace for ChoiceGraph<StateIdx, ChoiceIdx> {
    type StateIndex = StateIdx;
    type ChoiceIndex = ChoiceIdx;
    type BranchIndex = ChoiceIdx;

    fn states(&self) -> SemiboundedIndexRange<Self::StateIndex> {
        self.state_to_choice.keys()
    }

    fn choices(&self) -> SemiboundedIndexRange<Self::ChoiceIndex> {
        self.state_to_choice.values()
    }

    fn branches(&self) -> SemiboundedIndexRange<Self::BranchIndex> {
        self.state_to_choice.values()
    }

    fn choices_of_state(&self, state: Self::StateIndex) -> IndexRange<Self::ChoiceIndex> {
        self.state_to_choice.index(state)
    }

    fn branches_of_choice(&self, choice: Self::ChoiceIndex) -> IndexRange<Self::BranchIndex> {
        IndexRange::with_single_index(choice)
    }

    fn branch_probability(&self, _branch: Self::BranchIndex) -> f64 {
        1.0
    }

    fn branch_destination(&self, branch: Self::BranchIndex) -> Self::StateIndex {
        self.choice_destinations[branch]
    }
}

impl<StateIdx: Index, ChoiceIdx: Index> ReadChoiceDestinations
    for ChoiceGraph<StateIdx, ChoiceIdx>
{
    fn choice_destination(&self, choice: Self::ChoiceIndex) -> Self::StateIndex {
        self.choice_destinations[choice]
    }
}

impl<StateIdx: Index, ChoiceIdx: Index> BuildStates for ChoiceGraph<StateIdx, ChoiceIdx> {
    fn add_state(&mut self) -> Self::StateIndex {
        self.state_to_choice.add_empty_entry()
    }
}

impl<StateIdx: Index, ChoiceIdx: Index> BuildChoiceGraph for ChoiceGraph<StateIdx, ChoiceIdx> {
    fn add_choice(&mut self, destination: Self::StateIndex) -> Self::ChoiceIndex {
        let choice = self.choice_destinations.add(destination);
        self.state_to_choice
            .extend_last_entry(choice + ChoiceIdx::RawType::one());
        choice
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct BranchGraph<StateIdx: Index, BranchIdx: Index> {
    pub state_to_branch: Csr<StateIdx, BranchIdx>,
    pub branch_probabilities: To1<BranchIdx, f64>,
    pub branch_destinations: To1<BranchIdx, StateIdx>,
}

// Set ChoiceIndex = StateIdx, because each state has exactly one choice
impl<StateIdx: Index, BranchIdx: Index> ReadStateSpace for BranchGraph<StateIdx, BranchIdx> {
    type StateIndex = StateIdx;
    type ChoiceIndex = StateIdx;
    type BranchIndex = BranchIdx;

    fn states(&self) -> SemiboundedIndexRange<Self::StateIndex> {
        self.state_to_branch.keys()
    }

    fn choices(&self) -> SemiboundedIndexRange<Self::ChoiceIndex> {
        self.state_to_branch.keys()
    }

    fn branches(&self) -> SemiboundedIndexRange<Self::BranchIndex> {
        self.state_to_branch.values()
    }

    fn choices_of_state(&self, state: Self::StateIndex) -> IndexRange<Self::ChoiceIndex> {
        IndexRange::with_single_index(state)
    }

    fn branches_of_choice(&self, choice: Self::ChoiceIndex) -> IndexRange<Self::BranchIndex> {
        self.state_to_branch.index(choice)
    }

    fn branch_probability(&self, branch: Self::BranchIndex) -> f64 {
        self.branch_probabilities[branch]
    }

    fn branch_destination(&self, branch: Self::BranchIndex) -> Self::StateIndex {
        self.branch_destinations[branch]
    }
}

impl<StateIdx: Index, BranchIdx: Index> ReadStateBranches for BranchGraph<StateIdx, BranchIdx> {
    fn branches_of_state(&self, state: Self::StateIndex) -> IndexRange<Self::BranchIndex> {
        self.state_to_branch.index(state)
    }
}

impl<StateIdx: Index, BranchIdx: Index> BuildStates for BranchGraph<StateIdx, BranchIdx> {
    fn add_state(&mut self) -> Self::StateIndex {
        self.state_to_branch.add_empty_entry()
    }
}

impl<StateIdx: Index, BranchIdx: Index> BuildBranchGraph for BranchGraph<StateIdx, BranchIdx> {
    fn add_branch(&mut self, probability: f64, destination: Self::StateIndex) -> Self::BranchIndex {
        let branch = self.branch_destinations.add(destination);
        self.branch_probabilities.add(probability);
        self.state_to_branch
            .extend_last_entry(branch + BranchIdx::RawType::one());
        branch
    }
}

impl<StateIdx: Index, ChoiceIdx: Index, BranchIdx: Index> ClearModel
    for ChoiceBranchGraph<StateIdx, ChoiceIdx, BranchIdx>
{
    fn clear(&mut self) {
        self.state_to_choice.clear();
        self.choice_to_branch.clear();
        self.branch_probabilities.clear();
        self.branch_destinations.clear();
    }
}

impl<StateIdx: Index, ChoiceIdx: Index> ClearModel for ChoiceGraph<StateIdx, ChoiceIdx> {
    fn clear(&mut self) {
        self.state_to_choice.clear();
        self.choice_destinations.clear();
    }
}

impl<StateIdx: Index, BranchIdx: Index> ClearModel for BranchGraph<StateIdx, BranchIdx> {
    fn clear(&mut self) {
        self.state_to_branch.clear();
        self.branch_probabilities.clear();
        self.branch_destinations.clear();
    }
}
