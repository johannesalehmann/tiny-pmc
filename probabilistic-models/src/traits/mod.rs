mod base_implementation;
mod index_ranges;

pub use index_ranges::*;

use crate::valuations::ValuationEntry;
use typed_index_collections::Index;

pub trait ReadStateSpace {
    type StateIdx: Index;
    type ChoiceIdx: Index;
    type BranchIdx: Index;

    fn states(&self) -> States<Self::StateIdx>;
    fn choices(&self) -> Choices<Self::ChoiceIdx>;
    fn branches(&self) -> Branches<Self::BranchIdx>;

    fn choices_of_state(&self, state: Self::StateIdx) -> ChoiceRange<Self::ChoiceIdx>;
    fn branches_of_choice(&self, choice: Self::ChoiceIdx) -> BranchRange<Self::BranchIdx>;

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
