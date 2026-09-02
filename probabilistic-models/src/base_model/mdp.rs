use crate::choices::{ChoiceToBranch, StateToChoice};
use crate::{BranchIndex, ChoiceIndex, StateIndex};
use typed_index_collections::{
    ChainedCsrIter, Csr, CsrIterator, Index, IndexRange, RawIndex, SemiboundedIndexRange, To1,
};

#[derive(Default)]
pub struct Mdp<StateIdx: Index, ChoiceIdx: Index, BranchIdx: Index> {
    pub state_to_choice: StateToChoice<StateIdx, ChoiceIdx>,
    pub choice_to_branch: ChoiceToBranch<ChoiceIdx, BranchIdx>,
    pub branch_probabilities: To1<BranchIdx, f64>,
    pub branch_destinations: To1<BranchIdx, StateIdx>,
}

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

    fn branch_probability(&self, branch: Self::BranchIdx) -> f64 {
        self.branch_probabilities[branch]
    }

    fn branch_destination(&self, branch: Self::BranchIdx) -> Self::StateIdx {
        self.branch_destinations[branch]
    }
}

impl<StateIdx: Index, ChoiceIdx: Index, BranchIdx: Index> super::BaseModel
    for Mdp<StateIdx, ChoiceIdx, BranchIdx>
{
}

impl Mdp<StateIndex<usize>, ChoiceIndex<usize>, BranchIndex<usize>> {
    pub fn with_default_types() -> Self {
        Self::default()
    }
}

impl<StateIdx: Index, ChoiceIdx: Index, BranchIdx: Index> Mdp<StateIdx, ChoiceIdx, BranchIdx> {
    pub fn add_state(&mut self, state_index: StateIdx) {
        let next_choice = self.state_to_choice.end();
        self.state_to_choice
            .add_entry(state_index, next_choice, next_choice);
    }
    pub fn add_choice(&mut self) -> ChoiceIdx {
        let choice_index = self.choice_to_branch.keys().end();
        let branch_index = self.choice_to_branch.end();
        self.choice_to_branch
            .add_entry(choice_index, branch_index, branch_index);
        self.state_to_choice
            .extend_last_entry(choice_index + ChoiceIdx::RawType::one());
        choice_index
    }

    pub fn add_branch(&mut self, probability: f64, target: StateIdx) -> BranchIdx {
        let branch_index = self.branch_destinations.add(target);
        self.branch_probabilities.add(probability);
        self.choice_to_branch
            .extend_last_entry(branch_index + BranchIdx::RawType::one());
        branch_index
    }

    pub fn add_choice_from_slice(&mut self, branches: &[(f64, StateIdx)]) -> ChoiceIdx {
        let index = self.add_choice();
        for &(rate_or_probability, target) in branches {
            self.add_branch(rate_or_probability, target);
        }
        index
    }

    pub fn state_choice_pairs(&self) -> StateChoicePairs<'_, StateIdx, ChoiceIdx> {
        StateChoicePairs {
            state_to_choice: &self.state_to_choice,
        }
    }

    pub fn state_choice_branch_triples(
        &self,
    ) -> StateChoiceBranchTriples<'_, StateIdx, ChoiceIdx, BranchIdx> {
        StateChoiceBranchTriples {
            state_to_choice: &self.state_to_choice,
            choice_to_branch: &self.choice_to_branch,
        }
    }
}

pub struct StateChoicePairs<'a, StateIdx: Index, ChoiceIdx: Index> {
    state_to_choice: &'a Csr<StateIdx, ChoiceIdx>,
}
impl<'a, StateIdx: Index, ChoiceIdx: Index> IntoIterator
    for StateChoicePairs<'a, StateIdx, ChoiceIdx>
{
    type Item = (StateIdx, ChoiceIdx);
    type IntoIter = CsrIterator<'a, StateIdx, ChoiceIdx>;

    fn into_iter(self) -> Self::IntoIter {
        self.state_to_choice.into_iter()
    }
}

pub struct StateChoiceBranchTriples<'a, StateIdx: Index, ChoiceIdx: Index, BranchIdx: Index> {
    state_to_choice: &'a Csr<StateIdx, ChoiceIdx>,
    choice_to_branch: &'a Csr<ChoiceIdx, BranchIdx>,
}

impl<'a, StateIdx: Index, ChoiceIdx: Index, BranchIdx: Index> IntoIterator
    for StateChoiceBranchTriples<'a, StateIdx, ChoiceIdx, BranchIdx>
{
    type Item = (StateIdx, ChoiceIdx, BranchIdx);
    type IntoIter = StateChoiceBranchTriplesIterator<'a, StateIdx, ChoiceIdx, BranchIdx>;

    fn into_iter(self) -> Self::IntoIter {
        StateChoiceBranchTriplesIterator {
            iterator: self
                .state_to_choice
                .chain(self.choice_to_branch)
                .into_iter(),
        }
    }
}

pub struct StateChoiceBranchTriplesIterator<'a, StateIdx: Index, ChoiceIdx: Index, BranchIdx: Index>
{
    iterator: ChainedCsrIter<
        StateIdx,
        ChoiceIdx,
        BranchIdx,
        CsrIterator<'a, StateIdx, ChoiceIdx>,
        CsrIterator<'a, ChoiceIdx, BranchIdx>,
    >,
}

impl<'a, StateIdx: Index, ChoiceIdx: Index, BranchIdx: Index> Iterator
    for StateChoiceBranchTriplesIterator<'a, StateIdx, ChoiceIdx, BranchIdx>
{
    type Item = (StateIdx, ChoiceIdx, BranchIdx);

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator
            .next()
            .map(|((state, choice), branch)| (state, choice, branch))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iterator.size_hint()
    }
}
