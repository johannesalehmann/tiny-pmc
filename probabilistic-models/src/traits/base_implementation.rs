use crate::base_model::Mdp;
use crate::traits::ReadStateSpace;
use typed_index_collections::{Index, IndexRange, SemiboundedIndexRange};

impl<StateIdx: Index, ChoiceIdx: Index, BranchIdx: Index> super::ReadStateSpace
    for Mdp<StateIdx, ChoiceIdx, BranchIdx>
{
    type StateIdx = StateIdx;
    type ChoiceIdx = ChoiceIdx;
    type BranchIdx = BranchIdx;

    fn states(&self) -> SemiboundedIndexRange<Self::StateIdx> {
        self.state_to_choice.keys()
    }

    fn choices(&self) -> SemiboundedIndexRange<Self::ChoiceIdx> {
        self.choice_to_branch.keys()
    }

    fn branches(&self) -> SemiboundedIndexRange<Self::BranchIdx> {
        self.choice_to_branch.values()
    }

    fn choices_of_state(&self, state: Self::StateIdx) -> IndexRange<Self::ChoiceIdx> {
        self.state_to_choice.index(state)
    }

    fn branches_of_choice(&self, choice: Self::ChoiceIdx) -> IndexRange<Self::BranchIdx> {
        self.choice_to_branch.index(choice)
    }

    fn branch_probability(&self, state: Self::BranchIdx) -> f64 {
        self.branch_probabilities[state]
    }

    fn branch_destination(&self, state: Self::BranchIdx) -> Self::StateIdx {
        self.branch_destinations[state]
    }
}

impl<M: ReadStateSpace, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds> ReadStateSpace
    for crate::Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    type StateIdx = <M as ReadStateSpace>::StateIdx;
    type ChoiceIdx = <M as ReadStateSpace>::ChoiceIdx;
    type BranchIdx = <M as ReadStateSpace>::BranchIdx;

    fn states(&self) -> SemiboundedIndexRange<Self::StateIdx> {
        self.base.states()
    }

    fn choices(&self) -> SemiboundedIndexRange<Self::ChoiceIdx> {
        self.base.choices()
    }

    fn branches(&self) -> SemiboundedIndexRange<Self::BranchIdx> {
        self.base.branches()
    }

    fn choices_of_state(&self, state: Self::StateIdx) -> IndexRange<Self::ChoiceIdx> {
        self.base.choices_of_state(state)
    }

    fn branches_of_choice(&self, choice: Self::ChoiceIdx) -> IndexRange<Self::BranchIdx> {
        self.base.branches_of_choice(choice)
    }

    fn branch_probability(&self, branch: Self::BranchIdx) -> f64 {
        self.base.branch_probability(branch)
    }

    fn branch_destination(&self, branch: Self::BranchIdx) -> Self::StateIdx {
        self.base.branch_destination(branch)
    }
}
