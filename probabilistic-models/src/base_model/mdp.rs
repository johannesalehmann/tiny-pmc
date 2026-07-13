use crate::choices::{ChoiceToBranch, StateToChoice};
use typed_index_collections::{ChainedCsrIter, Csr, CsrIterator, Index, To1};

#[derive(Default)]
pub struct Mdp<StateIdx: Index, ChoiceIdx: Index, BranchIdx: Index> {
    pub state_to_choice: StateToChoice<StateIdx, ChoiceIdx>,
    pub choice_to_branch: ChoiceToBranch<ChoiceIdx, BranchIdx>,
    pub branch_probabilities: To1<BranchIdx, f64>,
    pub branch_destinations: To1<BranchIdx, StateIdx>,
}

impl<StateIdx: Index, ChoiceIdx: Index, BranchIdx: Index> super::BaseModel
    for Mdp<StateIdx, ChoiceIdx, BranchIdx>
{
    type StateIndex = StateIdx;
    type ChoiceIndex = ChoiceIdx;
    type BranchIndex = BranchIdx;
}

impl<StateIdx: Index, ChoiceIdx: Index, BranchIdx: Index> Mdp<StateIdx, ChoiceIdx, BranchIdx> {
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
