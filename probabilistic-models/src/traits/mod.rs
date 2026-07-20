mod base_implementation;

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

mod valuations;
pub use valuations::ReadValuations;

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

pub trait ReadAtomicPropositions {
    type StateIdx: Index;
    type AnnotationIdx: Index;

    fn is_atomic_proposition_set(
        &self,
        state: Self::StateIdx,
        atomic_proposition: Self::AnnotationIdx,
    ) -> bool;
}
