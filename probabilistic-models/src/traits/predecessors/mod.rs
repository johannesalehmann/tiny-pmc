use crate::Model;
use crate::predecessors::Predecessors;
use typed_index_collections::{Index, IndexRange, SemiboundedIndexRange};

pub trait ReadPredecessors {
    type StateIdx: Index;
    type ChoiceIdx: Index;
    type BranchIdx: Index;
    type PredecessorIdx: Index;

    // TODO: This function is a bit of a hack to access the number of states for any
    //  M: ReadPredecessors. However, if M also implements e.g. ReadStateSpace, this information is
    //  duplicated. It might be cleaner to add predecessors_of_states(range<StateIdx>), which
    //  returns a CsrSlice, which has keys() (which then correspond the state space if the above
    //  function is called with an unbounded range. (But this idea still needs some refinement,
    //  and Csr does not allow slicing a the time of writing.)
    fn predecessor_states(&self) -> SemiboundedIndexRange<Self::StateIdx>;
    fn predecessors(&self) -> SemiboundedIndexRange<Self::PredecessorIdx>;
    fn predecessors_of_state(&self, state: Self::StateIdx) -> IndexRange<Self::PredecessorIdx>;
    fn branch_of_predecessor(&self, predecessor: Self::PredecessorIdx) -> Self::BranchIdx;
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

    fn predecessor_states(&self) -> SemiboundedIndexRange<Self::StateIdx> {
        self.predecessors.state_to_predecessor.keys()
    }

    fn predecessors(&self) -> SemiboundedIndexRange<Self::PredecessorIdx> {
        self.predecessors.predecessor_to_branch.keys()
    }

    fn predecessors_of_state(&self, state: Self::StateIdx) -> IndexRange<Self::PredecessorIdx> {
        self.predecessors.state_to_predecessor.index(state)
    }

    fn branch_of_predecessor(&self, predecessor: Self::PredecessorIdx) -> Self::BranchIdx {
        self.predecessors.predecessor_to_branch[predecessor]
    }

    fn choice_of_branch(&self, branch: Self::BranchIdx) -> Self::ChoiceIdx {
        self.predecessors.branch_to_choice[branch]
    }

    fn state_of_choice(&self, choice: Self::ChoiceIdx) -> Self::StateIdx {
        self.predecessors.choice_to_state[choice]
    }
}
