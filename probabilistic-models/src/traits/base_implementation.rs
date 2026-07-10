use crate::traits::{BranchRange, Branches, ChoiceRange, Choices, ReadStateSpace, States};
use crate::{BaseModel, Mdp};
use typed_index_collections::Index;

impl<StateIdx: Index, ChoiceIdx: Index, BranchIdx: Index> super::ReadStateSpace
    for Mdp<StateIdx, ChoiceIdx, BranchIdx>
{
    type StateIdx = StateIdx;
    type ChoiceIdx = ChoiceIdx;
    type BranchIdx = BranchIdx;

    fn states(&self) -> States<Self::StateIdx> {
        States::from_usize(self.state_to_choice.count_entries())
    }

    fn choices(&self) -> Choices<Self::ChoiceIdx> {
        Choices::from_usize(self.choice_to_branch.count_entries())
    }

    fn branches(&self) -> Branches<Self::BranchIdx> {
        Branches::from_usize(self.branch_probabilities.len())
    }

    fn choices_of_state(&self, state: Self::StateIdx) -> ChoiceRange<Self::ChoiceIdx> {
        let range = self.state_to_choice.index(state);
        ChoiceRange::new(range.start(), range.end())
    }

    fn branches_of_choice(&self, choice: Self::ChoiceIdx) -> BranchRange<Self::BranchIdx> {
        let range = self.choice_to_branch.index(choice);
        BranchRange::new(range.start(), range.end())
    }

    fn branch_probability(&self, state: Self::BranchIdx) -> f64 {
        self.branch_probabilities[state]
    }

    fn branch_destination(&self, state: Self::BranchIdx) -> Self::StateIdx {
        self.branch_destinations[state]
    }
}

impl<M: ReadStateSpace, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals> ReadStateSpace
    for crate::Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals>
{
    type StateIdx = <M as ReadStateSpace>::StateIdx;
    type ChoiceIdx = <M as ReadStateSpace>::ChoiceIdx;
    type BranchIdx = <M as ReadStateSpace>::BranchIdx;

    fn states(&self) -> States<Self::StateIdx> {
        self.base.states()
    }

    fn choices(&self) -> Choices<Self::ChoiceIdx> {
        self.base.choices()
    }

    fn branches(&self) -> Branches<Self::BranchIdx> {
        self.base.branches()
    }

    fn choices_of_state(&self, state: Self::StateIdx) -> ChoiceRange<Self::ChoiceIdx> {
        self.base.choices_of_state(state)
    }

    fn branches_of_choice(&self, choice: Self::ChoiceIdx) -> BranchRange<Self::BranchIdx> {
        self.base.branches_of_choice(choice)
    }

    fn branch_probability(&self, branch: Self::BranchIdx) -> f64 {
        self.base.branch_probability(branch)
    }

    fn branch_destination(&self, branch: Self::BranchIdx) -> Self::StateIdx {
        self.branch_destination(branch)
    }
}
