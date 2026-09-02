mod reachability;
pub use reachability::{BackwardReachability, Reachability};

mod predecessors;
pub use predecessors::ReadPredecessors;

mod branch_labels;
pub use branch_labels::ReadBranchLabels;

mod choice_labels;
pub use choice_labels::ReadChoiceLabels;

mod initial_state;
pub use initial_state::ReadInitialStates;

mod state_specifier;
pub use state_specifier::StateSet;

mod atomic_propositions;
pub use atomic_propositions::ReadAtomicPropositions;

mod valuations;
pub use valuations::ReadValuations;

mod owners;
pub use owners::ReadOwners;

use typed_index_collections::{Index, IndexRange, SemiboundedIndexRange};

pub trait ReadStateSpace {
    type StateIdx: Index;
    type ChoiceIdx: Index;
    type BranchIdx: Index;

    fn states(&self) -> SemiboundedIndexRange<Self::StateIdx>;
    fn choices(&self) -> SemiboundedIndexRange<Self::ChoiceIdx>;
    fn branches(&self) -> SemiboundedIndexRange<Self::BranchIdx>;

    fn choices_of_state(&self, state: Self::StateIdx) -> IndexRange<Self::ChoiceIdx>;
    fn branches_of_choice(&self, choice: Self::ChoiceIdx) -> IndexRange<Self::BranchIdx>;

    fn branch_probability(&self, branch: Self::BranchIdx) -> f64;
    fn branch_destination(&self, branch: Self::BranchIdx) -> Self::StateIdx;
}

macro_rules! derive_read_state_space {
    ($subcomponent:ident) => {
        fn states(&self) -> typed_index_collections::SemiboundedIndexRange<Self::StateIdx> {
            self.$subcomponent.states()
        }

        fn choices(&self) -> typed_index_collections::SemiboundedIndexRange<Self::ChoiceIdx> {
            self.$subcomponent.choices()
        }

        fn branches(&self) -> typed_index_collections::SemiboundedIndexRange<Self::BranchIdx> {
            self.$subcomponent.branches()
        }

        fn choices_of_state(
            &self,
            state: Self::StateIdx,
        ) -> typed_index_collections::IndexRange<Self::ChoiceIdx> {
            self.$subcomponent.choices_of_state(state)
        }

        fn branches_of_choice(
            &self,
            choice: Self::ChoiceIdx,
        ) -> typed_index_collections::IndexRange<Self::BranchIdx> {
            self.$subcomponent.branches_of_choice(choice)
        }

        fn branch_probability(&self, branch: Self::BranchIdx) -> f64 {
            self.$subcomponent.branch_probability(branch)
        }

        fn branch_destination(&self, branch: Self::BranchIdx) -> Self::StateIdx {
            self.$subcomponent.branch_destination(branch)
        }
    };
}
pub(crate) use derive_read_state_space;

impl<M: ReadStateSpace, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds> ReadStateSpace
    for crate::Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    type StateIdx = <M as ReadStateSpace>::StateIdx;
    type ChoiceIdx = <M as ReadStateSpace>::ChoiceIdx;
    type BranchIdx = <M as ReadStateSpace>::BranchIdx;

    derive_read_state_space!(base);
}
