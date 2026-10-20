use crate::traits::{
    BuildBranchGraph, BuildChoiceBranchGraph, BuildChoiceGraph, BuildStates, ClearModel,
    ReadChoiceDestinations, ReadStateBranches, ReadStateSpace,
};
use crate::{BranchIndex, ChoiceIndex, HasIx, Ix, StateIndex};
use typed_index_collections::{Csr, Index, IndexRange, RawIndex, SemiboundedIndexRange, To1};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ChoiceBranchGraph<I: Ix> {
    pub state_to_choice: Csr<StateIndex<I>, ChoiceIndex<I>>,
    pub choice_to_branch: Csr<ChoiceIndex<I>, BranchIndex<I>>,
    pub branch_probabilities: To1<BranchIndex<I>, f64>,
    pub branch_destinations: To1<BranchIndex<I>, StateIndex<I>>,
}

impl<I: Ix> HasIx for ChoiceBranchGraph<I> {
    type Ix = I;
}

impl<I: Ix> ReadStateSpace for ChoiceBranchGraph<I> {
    fn states(&self) -> SemiboundedIndexRange<StateIndex<I>> {
        self.state_to_choice.keys()
    }

    fn choices(&self) -> SemiboundedIndexRange<ChoiceIndex<I>> {
        self.choice_to_branch.keys()
    }

    fn branches(&self) -> SemiboundedIndexRange<BranchIndex<I>> {
        self.choice_to_branch.values()
    }

    fn choices_of_state(&self, state: StateIndex<I>) -> IndexRange<ChoiceIndex<I>> {
        self.state_to_choice.index(state)
    }

    fn branches_of_choice(&self, choice: ChoiceIndex<I>) -> IndexRange<BranchIndex<I>> {
        self.choice_to_branch.index(choice)
    }

    fn branch_probability(&self, branch: BranchIndex<I>) -> f64 {
        self.branch_probabilities[branch]
    }

    fn branch_destination(&self, branch: BranchIndex<I>) -> StateIndex<I> {
        self.branch_destinations[branch]
    }
}

impl<I: Ix> BuildStates for ChoiceBranchGraph<I> {
    fn add_state(&mut self) -> StateIndex<I> {
        self.state_to_choice.add_empty_entry()
    }
}

impl<I: Ix> BuildChoiceBranchGraph for ChoiceBranchGraph<I> {
    fn add_choice(&mut self) -> ChoiceIndex<I> {
        let choice = self.choice_to_branch.add_empty_entry();
        self.state_to_choice
            .extend_last_entry(choice + I::Choice::one());
        choice
    }

    fn add_branch(&mut self, probability: f64, destination: StateIndex<I>) -> BranchIndex<I> {
        let branch = self.branch_destinations.add(destination);
        self.branch_probabilities.add(probability);
        self.choice_to_branch
            .extend_last_entry(branch + I::Branch::one());
        branch
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ChoiceGraph<I: Ix> {
    pub state_to_choice: Csr<StateIndex<I>, ChoiceIndex<I>>,
    pub choice_destinations: To1<ChoiceIndex<I>, StateIndex<I>>,
}

impl<I: Ix> HasIx for ChoiceGraph<I> {
    type Ix = I;
}

// Every choice has exactly one branch, which has the same index as the choice.
impl<I: Ix> ReadStateSpace for ChoiceGraph<I> {
    fn states(&self) -> SemiboundedIndexRange<StateIndex<I>> {
        self.state_to_choice.keys()
    }

    fn choices(&self) -> SemiboundedIndexRange<ChoiceIndex<I>> {
        self.state_to_choice.values()
    }

    fn branches(&self) -> SemiboundedIndexRange<BranchIndex<I>> {
        SemiboundedIndexRange::new(self.state_to_choice.values().end().reinterpret_as())
    }

    fn choices_of_state(&self, state: StateIndex<I>) -> IndexRange<ChoiceIndex<I>> {
        self.state_to_choice.index(state)
    }

    fn branches_of_choice(&self, choice: ChoiceIndex<I>) -> IndexRange<BranchIndex<I>> {
        IndexRange::with_single_index(choice.reinterpret_as())
    }

    fn branch_probability(&self, _branch: BranchIndex<I>) -> f64 {
        1.0
    }

    fn branch_destination(&self, branch: BranchIndex<I>) -> StateIndex<I> {
        self.choice_destinations[branch.reinterpret_as::<ChoiceIndex<I>>()]
    }
}

impl<I: Ix> ReadChoiceDestinations for ChoiceGraph<I> {
    fn choice_destination(&self, choice: ChoiceIndex<I>) -> StateIndex<I> {
        self.choice_destinations[choice]
    }
}

impl<I: Ix> BuildStates for ChoiceGraph<I> {
    fn add_state(&mut self) -> StateIndex<I> {
        self.state_to_choice.add_empty_entry()
    }
}

impl<I: Ix> BuildChoiceGraph for ChoiceGraph<I> {
    fn add_choice(&mut self, destination: StateIndex<I>) -> ChoiceIndex<I> {
        let choice = self.choice_destinations.add(destination);
        self.state_to_choice
            .extend_last_entry(choice + I::Choice::one());
        choice
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct BranchGraph<I: Ix> {
    pub state_to_branch: Csr<StateIndex<I>, BranchIndex<I>>,
    pub branch_probabilities: To1<BranchIndex<I>, f64>,
    pub branch_destinations: To1<BranchIndex<I>, StateIndex<I>>,
}

impl<I: Ix> HasIx for BranchGraph<I> {
    type Ix = I;
}

// Each state has exactly one choice, which has the same index as the state.
impl<I: Ix> ReadStateSpace for BranchGraph<I> {
    fn states(&self) -> SemiboundedIndexRange<StateIndex<I>> {
        self.state_to_branch.keys()
    }

    fn choices(&self) -> SemiboundedIndexRange<ChoiceIndex<I>> {
        SemiboundedIndexRange::new(self.state_to_branch.keys().end().reinterpret_as())
    }

    fn branches(&self) -> SemiboundedIndexRange<BranchIndex<I>> {
        self.state_to_branch.values()
    }

    fn choices_of_state(&self, state: StateIndex<I>) -> IndexRange<ChoiceIndex<I>> {
        IndexRange::with_single_index(state.reinterpret_as())
    }

    fn branches_of_choice(&self, choice: ChoiceIndex<I>) -> IndexRange<BranchIndex<I>> {
        self.state_to_branch
            .index(choice.reinterpret_as::<StateIndex<I>>())
    }

    fn branch_probability(&self, branch: BranchIndex<I>) -> f64 {
        self.branch_probabilities[branch]
    }

    fn branch_destination(&self, branch: BranchIndex<I>) -> StateIndex<I> {
        self.branch_destinations[branch]
    }
}

impl<I: Ix> ReadStateBranches for BranchGraph<I> {
    fn branches_of_state(&self, state: StateIndex<I>) -> IndexRange<BranchIndex<I>> {
        self.state_to_branch.index(state)
    }
}

impl<I: Ix> BuildStates for BranchGraph<I> {
    fn add_state(&mut self) -> StateIndex<I> {
        self.state_to_branch.add_empty_entry()
    }
}

impl<I: Ix> BuildBranchGraph for BranchGraph<I> {
    fn add_branch(&mut self, probability: f64, destination: StateIndex<I>) -> BranchIndex<I> {
        let branch = self.branch_destinations.add(destination);
        self.branch_probabilities.add(probability);
        self.state_to_branch
            .extend_last_entry(branch + I::Branch::one());
        branch
    }
}

impl<I: Ix> ClearModel for ChoiceBranchGraph<I> {
    fn clear(&mut self) {
        self.state_to_choice.clear();
        self.choice_to_branch.clear();
        self.branch_probabilities.clear();
        self.branch_destinations.clear();
    }
}

impl<I: Ix> ClearModel for ChoiceGraph<I> {
    fn clear(&mut self) {
        self.state_to_choice.clear();
        self.choice_destinations.clear();
    }
}

impl<I: Ix> ClearModel for BranchGraph<I> {
    fn clear(&mut self) {
        self.state_to_branch.clear();
        self.branch_probabilities.clear();
        self.branch_destinations.clear();
    }
}
