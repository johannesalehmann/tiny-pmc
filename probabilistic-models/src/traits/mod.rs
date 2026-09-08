mod reachability;
pub use reachability::{BackwardReachability, Reachability};

mod predecessors;
pub use predecessors::ReadPredecessors;
#[allow(unused)]
pub(crate) use predecessors::derive_read_predecessors;

mod branch_labels;
pub use branch_labels::ReadBranchLabels;
#[allow(unused)]
pub(crate) use branch_labels::derive_read_branch_labels;

mod choice_labels;
pub use choice_labels::ReadChoiceLabels;
#[allow(unused)]
pub(crate) use choice_labels::derive_read_choice_labels;

mod initial_state;
pub use initial_state::ReadInitialStates;
#[allow(unused)]
pub(crate) use initial_state::derive_read_initial_states;

mod state_specifier;
pub use state_specifier::StateSet;

mod atomic_propositions;
pub use atomic_propositions::ReadAtomicPropositions;
#[allow(unused)]
pub(crate) use atomic_propositions::derive_read_atomic_propositions;

mod valuations;
pub use valuations::ReadValuations;
#[allow(unused)]
pub(crate) use valuations::derive_read_valuations;

mod owners;
pub use owners::ReadOwners;
#[allow(unused)]
pub(crate) use owners::derive_read_owners;

use typed_index_collections::{Index, IndexRange, IndexRangeIterator, SemiboundedIndexRange};

pub trait ReadStateSpace {
    // The Index types in this trait have suffix `Index`, whereas those in the other Read traits
    // have suffix `Idx`. This is intentional to allow notation of the form:
    //     M: ReadStateSpace + ReadAtomicPropositions<StateIdx=M::StateIndex>
    // instead of
    //     M: ReadStateSpace + ReadAtomicPropositions<StateIdx=<M as ReadStateSpace>::StateIdx>
    type StateIndex: Index;
    type ChoiceIndex: Index;
    type BranchIndex: Index;

    fn states(&self) -> SemiboundedIndexRange<Self::StateIndex>;
    fn choices(&self) -> SemiboundedIndexRange<Self::ChoiceIndex>;
    fn branches(&self) -> SemiboundedIndexRange<Self::BranchIndex>;

    fn choices_of_state(&self, state: Self::StateIndex) -> IndexRange<Self::ChoiceIndex>;
    fn branches_of_choice(&self, choice: Self::ChoiceIndex) -> IndexRange<Self::BranchIndex>;

    fn branch_probability(&self, branch: Self::BranchIndex) -> f64;
    fn branch_destination(&self, branch: Self::BranchIndex) -> Self::StateIndex;

    fn successors_of_state(&self, state: Self::StateIndex) -> impl Iterator<Item = Self::StateIndex>
    where
        Self: Sized,
    {
        SuccessorIterator {
            choices_iterator: self.choices_of_state(state).into_iter(),
            branches_iterator: IndexRangeIterator::empty(), // The branch iterator will be initialised on the first iteration
            model: self,
        }
    }
}

struct SuccessorIterator<'a, M: ReadStateSpace> {
    choices_iterator: IndexRangeIterator<M::ChoiceIndex>,
    branches_iterator: IndexRangeIterator<M::BranchIndex>,
    model: &'a M,
}

impl<'a, M: ReadStateSpace> Iterator for SuccessorIterator<'a, M> {
    type Item = M::StateIndex;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(branch) = self.branches_iterator.next() {
                return Some(self.model.branch_destination(branch));
            } else {
                if let Some(choice) = self.choices_iterator.next() {
                    self.branches_iterator = self.model.branches_of_choice(choice).into_iter();
                } else {
                    return None;
                }
            }
        }
    }
}

macro_rules! derive_read_state_space {
    ($subcomponent:ident) => {
        fn states(&self) -> typed_index_collections::SemiboundedIndexRange<Self::StateIndex> {
            self.$subcomponent.states()
        }

        fn choices(&self) -> typed_index_collections::SemiboundedIndexRange<Self::ChoiceIndex> {
            self.$subcomponent.choices()
        }

        fn branches(&self) -> typed_index_collections::SemiboundedIndexRange<Self::BranchIndex> {
            self.$subcomponent.branches()
        }

        fn choices_of_state(
            &self,
            state: Self::StateIndex,
        ) -> typed_index_collections::IndexRange<Self::ChoiceIndex> {
            self.$subcomponent.choices_of_state(state)
        }

        fn branches_of_choice(
            &self,
            choice: Self::ChoiceIndex,
        ) -> typed_index_collections::IndexRange<Self::BranchIndex> {
            self.$subcomponent.branches_of_choice(choice)
        }

        fn branch_probability(&self, branch: Self::BranchIndex) -> f64 {
            self.$subcomponent.branch_probability(branch)
        }

        fn branch_destination(&self, branch: Self::BranchIndex) -> Self::StateIndex {
            self.$subcomponent.branch_destination(branch)
        }
    };
}
pub(crate) use derive_read_state_space;

impl<M: ReadStateSpace, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds> ReadStateSpace
    for crate::Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    type StateIndex = M::StateIndex;
    type ChoiceIndex = M::ChoiceIndex;
    type BranchIndex = M::BranchIndex;

    derive_read_state_space!(base);
}

#[cfg(test)]
mod tests {
    use crate::mdp;
    use crate::traits::ReadStateSpace;

    #[test]
    fn successors_of_state() {
        mdp!(mdp = {
            s0 -> 0.3: s0 & 0.7: s1,
            s1 -> 1.0: s3,
            s1 -> 0.6: s1 & 0.4: s2,
            s1 -> 1.0: s4,
            s2 -> 0.1: s3 & 0.3: s2 & 0.3: s1 & 0.3: s0,
            s3 ->,
            s4 -> 1.0: s1
        });

        assert_eq!(
            mdp.successors_of_state(s0).collect::<Vec<_>>(),
            vec![s0, s1]
        );
        assert_eq!(
            mdp.successors_of_state(s1).collect::<Vec<_>>(),
            vec![s3, s1, s2, s4]
        );
        assert_eq!(
            mdp.successors_of_state(s2).collect::<Vec<_>>(),
            vec![s3, s2, s1, s0]
        );
        assert_eq!(mdp.successors_of_state(s3).collect::<Vec<_>>(), vec![]);
        assert_eq!(mdp.successors_of_state(s4).collect::<Vec<_>>(), vec![s1]);
    }
}
