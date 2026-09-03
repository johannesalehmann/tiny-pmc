use crate::Model;
use crate::predecessors::Predecessors;
use typed_index_collections::{Index, IndexRange, SemiboundedIndexRange, ValuePerIndexSource};

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

macro_rules! derive_read_predecessors {
    ($subcomponent:ident) => {
        fn predecessor_states(
            &self,
        ) -> typed_index_collections::SemiboundedIndexRange<Self::StateIdx> {
            self.$subcomponent.predecessor_states()
        }

        fn predecessors(
            &self,
        ) -> typed_index_collections::SemiboundedIndexRange<Self::PredecessorIdx> {
            self.$subcomponent.predecessors()
        }

        fn predecessors_of_state(
            &self,
            state: Self::StateIdx,
        ) -> typed_index_collections::IndexRange<Self::PredecessorIdx> {
            self.$subcomponent.predecessors_of_state(state)
        }

        fn branch_of_predecessor(&self, predecessor: Self::PredecessorIdx) -> Self::BranchIdx {
            self.$subcomponent.branch_of_predecessor(predecessor)
        }

        fn choice_of_branch(&self, branch: Self::BranchIdx) -> Self::ChoiceIdx {
            self.$subcomponent.choice_of_branch(branch)
        }

        fn state_of_choice(&self, choice: Self::ChoiceIdx) -> Self::StateIdx {
            self.$subcomponent.state_of_choice(choice)
        }
    };
}
pub(crate) use derive_read_predecessors;

impl<SI: Index, CI: Index, BI: Index, PI: Index> ReadPredecessors for Predecessors<SI, CI, BI, PI> {
    type StateIdx = SI;
    type ChoiceIdx = CI;
    type BranchIdx = BI;
    type PredecessorIdx = PI;

    fn predecessor_states(&self) -> SemiboundedIndexRange<Self::StateIdx> {
        self.state_to_predecessor.keys()
    }

    fn predecessors(&self) -> SemiboundedIndexRange<Self::PredecessorIdx> {
        self.predecessor_to_branch.keys()
    }

    fn predecessors_of_state(&self, state: Self::StateIdx) -> IndexRange<Self::PredecessorIdx> {
        self.state_to_predecessor.index(state)
    }

    fn branch_of_predecessor(&self, predecessor: Self::PredecessorIdx) -> Self::BranchIdx {
        self.predecessor_to_branch[predecessor]
    }

    fn choice_of_branch(&self, branch: Self::BranchIdx) -> Self::ChoiceIdx {
        self.branch_to_choice[branch]
    }

    fn state_of_choice(&self, choice: Self::ChoiceIdx) -> Self::StateIdx {
        self.choice_to_state[choice]
    }
}

impl<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds: ReadPredecessors>
    ReadPredecessors for Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    type StateIdx = Preds::StateIdx;
    type ChoiceIdx = Preds::ChoiceIdx;
    type BranchIdx = Preds::BranchIdx;
    type PredecessorIdx = Preds::PredecessorIdx;

    derive_read_predecessors!(predecessors);
}
