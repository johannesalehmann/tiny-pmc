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

    fn successors_of_state(&self, state: Self::StateIdx) -> impl Iterator<Item = Self::StateIdx>
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
    choices_iterator: IndexRangeIterator<M::ChoiceIdx>,
    branches_iterator: IndexRangeIterator<M::BranchIdx>,
    model: &'a M,
}

impl<'a, M: ReadStateSpace> Iterator for SuccessorIterator<'a, M> {
    type Item = M::StateIdx;

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

#[cfg(test)]
mod tests {
    use crate::StateIndex;
    use crate::base_model::Mdp;
    use crate::traits::ReadStateSpace;
    use typed_index_collections::Index;

    #[test]
    fn successors_of_state() {
        let mut mdp = Mdp::with_default_types();
        let s0 = StateIndex::from_raw(0);
        let s1 = StateIndex::from_raw(1);
        let s2 = StateIndex::from_raw(2);
        let s3 = StateIndex::from_raw(3);
        let s4 = StateIndex::from_raw(4);

        mdp.add_state(s0);
        mdp.add_choice_from_slice(&[(0.3, s0), (0.7, s1)]);

        mdp.add_state(s1);
        mdp.add_choice_from_slice(&[(1.0, s3)]);
        mdp.add_choice_from_slice(&[(0.6, s1), (0.4, s2)]);
        mdp.add_choice_from_slice(&[(1.0, s4)]);

        mdp.add_state(s2);
        mdp.add_choice_from_slice(&[(0.1, s3), (0.3, s2), (0.3, s1), (0.3, s0)]);

        mdp.add_state(s3);

        mdp.add_state(s4);
        mdp.add_choice_from_slice(&[(1.0, s1)]);

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
