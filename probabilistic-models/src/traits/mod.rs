mod base_implementation;

mod reachability;
pub use reachability::{BackwardReachability, Reachability};

mod predecessors;
pub use predecessors::ReadPredecessors;

mod branch_labels;
mod choice_labels;
mod state_specifier;

pub use state_specifier::StateSet;

use crate::valuations::ValuationEntry;
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

pub trait ReadValuations {
    type StateIdx: Index;
    type ClassIdx: Index;
    type ClassEntryIdx: Index;
    type ValuationIdx: Index;
    fn state_valuation(
        &self,
        state: Self::StateIdx,
    ) -> ValuationEntry<'_, Self::ClassIdx, Self::ClassEntryIdx, Self::ValuationIdx>;
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
