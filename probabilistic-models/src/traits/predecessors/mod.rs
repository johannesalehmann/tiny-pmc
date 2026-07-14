use crate::Model;
use crate::predecessors::Predecessors;
use typed_index_collections::{CsrRanges, Index, IndexRange};

pub trait ReadPredecessors {
    type StateIdx: Index;
    type ChoiceIdx: Index;
    type BranchIdx: Index;
    type PredecessorIdx: Index;
    fn predecessors_of_state(&self, state: Self::StateIdx) -> IndexRange<Self::PredecessorIdx>;
    fn predecessor_source(&self, predecessor: Self::PredecessorIdx) -> Self::BranchIdx;
    fn choice_of_branch(&self, branch: Self::BranchIdx) -> Self::ChoiceIdx;
    fn state_of_choice(&self, choice: Self::ChoiceIdx) -> Self::StateIdx;
}

impl<
    SI: Index,
    CI: Index,
    BI: Index,
    PI: Index,
    M,
    Ini,
    ChLabel,
    BrLabel,
    Obs,
    APs,
    Rew,
    Ann,
    StateVals,
> ReadPredecessors
    for Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Predecessors<SI, CI, BI, PI>>
{
    type StateIdx = SI;
    type ChoiceIdx = CI;
    type BranchIdx = BI;
    type PredecessorIdx = PI;

    fn predecessors_of_state(&self, state: Self::StateIdx) -> IndexRange<Self::PredecessorIdx> {
        self.predecessors.state_to_predecessor.index(state)
    }

    fn predecessor_source(&self, predecessor: Self::PredecessorIdx) -> Self::BranchIdx {
        self.predecessors.predecessor_to_branch[predecessor]
    }

    fn choice_of_branch(&self, branch: Self::BranchIdx) -> Self::ChoiceIdx {
        self.predecessors.branch_to_choice[branch]
    }

    fn state_of_choice(&self, choice: Self::ChoiceIdx) -> Self::StateIdx {
        self.predecessors.choice_to_state[choice]
    }
}
