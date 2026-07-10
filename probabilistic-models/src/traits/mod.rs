mod base_implementation;
mod index_ranges;

pub use index_ranges::*;

use crate::valuations::ValuationEntry;
use typed_index_collections::Index;

pub trait ReadStateSpace {
    type StateIndex: Index;
    type ChoiceIndex: Index;
    type BranchIndex: Index;

    fn states(&self) -> States<Self::StateIndex>;
    fn choices(&self) -> Choices<Self::ChoiceIndex>;
    fn branches(&self) -> Branches<Self::BranchIndex>;

    fn choices_of_state(&self, state: Self::StateIndex) -> ChoiceRange<Self::ChoiceIndex>;
    fn branches_of_choice(&self, state: Self::BranchIndex) -> BranchRange<Self::ChoiceIndex>;

    fn branch_probability(&self, state: Self::BranchIndex) -> f64;
    fn branch_destination(&self, state: Self::BranchIndex) -> Self::StateIndex;
}

pub trait ReadValuations {
    type StateIndex: Index;
    // TODO: Fix once raw indices have been removed from the interface
    // fn state_valuation(&self, state: Self::StateIndex) -> ValuationEntry<'_, I>
}

pub trait ReadAtomicPropositions {
    type StateIndex: Index;
    // TODO
}
