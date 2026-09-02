use crate::base_model::{Mdp, StateChoiceBranchTriples, StateChoicePairs};
use crate::owners::TwoPlayer;
use crate::traits::{ReadOwners, derive_read_state_space};
use crate::{BranchIndex, ChoiceIndex, StateIndex};
use typed_index_collections::{Index, To1};
// TODO: There is quite a lot of duplication with the MDP class. Perhaps some of this can be handled
//  by macros instead (e.g. something of the form `derive_read_state_space!(...)`?)

#[derive(Default)]
pub struct TwoPlayerTurnBasedGame<StateIdx: Index, ChoiceIdx: Index, BranchIdx: Index> {
    pub base_mdp: Mdp<StateIdx, ChoiceIdx, BranchIdx>,
    pub owners: To1<StateIdx, TwoPlayer>,
}

impl<StateIdx: Index, ChoiceIdx: Index, BranchIdx: Index> super::ReadStateSpace
    for TwoPlayerTurnBasedGame<StateIdx, ChoiceIdx, BranchIdx>
{
    type StateIdx = StateIdx;
    type ChoiceIdx = ChoiceIdx;
    type BranchIdx = BranchIdx;

    derive_read_state_space!(base_mdp);
}

impl<StateIdx: Index, ChoiceIdx: Index, BranchIdx: Index> super::BaseModel
    for TwoPlayerTurnBasedGame<StateIdx, ChoiceIdx, BranchIdx>
{
    type StateIndex = StateIdx;
    type ChoiceIndex = ChoiceIdx;
    type BranchIndex = BranchIdx;
}

impl TwoPlayerTurnBasedGame<StateIndex<usize>, ChoiceIndex<usize>, BranchIndex<usize>> {
    pub fn with_default_types() -> Self {
        Self::default()
    }
}

impl<StateIdx: Index, ChoiceIdx: Index, BranchIdx: Index>
    TwoPlayerTurnBasedGame<StateIdx, ChoiceIdx, BranchIdx>
{
    pub fn add_state(&mut self, state_index: StateIdx, owner: TwoPlayer) {
        self.base_mdp.add_state(state_index);
        self.owners.add_checked(state_index, owner);
    }
    pub fn add_choice(&mut self) -> ChoiceIdx {
        self.base_mdp.add_choice()
    }

    pub fn add_branch(&mut self, probability: f64, target: StateIdx) -> BranchIdx {
        self.base_mdp.add_branch(probability, target)
    }

    pub fn add_choice_from_slice(&mut self, branches: &[(f64, StateIdx)]) -> ChoiceIdx {
        self.base_mdp.add_choice_from_slice(branches)
    }

    pub fn state_choice_pairs(&self) -> StateChoicePairs<'_, StateIdx, ChoiceIdx> {
        self.base_mdp.state_choice_pairs()
    }

    pub fn state_choice_branch_triples(
        &self,
    ) -> StateChoiceBranchTriples<'_, StateIdx, ChoiceIdx, BranchIdx> {
        self.base_mdp.state_choice_branch_triples()
    }
}

impl<SI: Index, CI: Index, BI: Index> ReadOwners for TwoPlayerTurnBasedGame<SI, CI, BI> {
    type OwnerType = TwoPlayer;
    type StateIdx = SI;

    fn state_owner(&self, state: Self::StateIdx) -> Self::OwnerType {
        self.owners[state]
    }
}
