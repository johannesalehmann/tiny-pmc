mod reachability;
pub use reachability::{BackwardReachability, Reachability};

mod predecessors;
#[allow(unused)]
pub(crate) use predecessors::derive_read_predecessors;
pub use predecessors::{ReadPredecessors, ReadPredecessorsMaybe};

mod branch_labels;
#[allow(unused)]
pub(crate) use branch_labels::derive_read_branch_labels;
pub use branch_labels::{ReadBranchLabels, ReadBranchLabelsMaybe};

mod choice_labels;
#[allow(unused)]
pub(crate) use choice_labels::derive_read_choice_labels;
pub use choice_labels::{ReadChoiceLabels, ReadChoiceLabelsMaybe};

mod initial_state;
#[allow(unused)]
pub(crate) use initial_state::derive_read_initial_states;
pub use initial_state::{ReadInitialStates, ReadInitialStatesMaybe};

mod state_specifier;
pub use state_specifier::{AsStateSet, BoxedStateSet, SingleState, StateSet};

mod atomic_propositions;
#[allow(unused)]
pub(crate) use atomic_propositions::derive_read_atomic_propositions;
pub use atomic_propositions::{ReadAtomicPropositions, ReadAtomicPropositionsMaybe};

mod valuations;
#[allow(unused)]
pub(crate) use valuations::derive_read_valuations;
pub use valuations::{ReadValuations, ReadValuationsMaybe};

mod rewards;
#[allow(unused)]
pub(crate) use rewards::derive_read_rewards;
pub use rewards::{ReadRewards, ReadRewardsMaybe};

mod build_state_space;
pub use build_state_space::{
    BuildBranchGraph, BuildChoiceBranchGraph, BuildChoiceGraph, BuildStates, ClearModel,
};
#[allow(unused)]
pub(crate) use build_state_space::{
    derive_build_branch_graph, derive_build_choice_branch_graph, derive_build_choice_graph,
    derive_build_states,
};

use crate::{BranchIndex, ChoiceIndex, HasIx, StateIndex};
use typed_index_collections::{IndexRange, IndexRangeIterator, SemiboundedIndexRange};

/// A branch together with the choice and state it belongs to.
pub type StateChoiceBranch<I> = (StateIndex<I>, ChoiceIndex<I>, BranchIndex<I>);

/// A branch (probability and destintation) together with the choice and state it belongs to
pub type Transition<I> = (StateIndex<I>, ChoiceIndex<I>, f64, StateIndex<I>);

pub trait ReadStateSpace: HasIx {
    fn states(&self) -> SemiboundedIndexRange<StateIndex<Self::Ix>>;
    fn choices(&self) -> SemiboundedIndexRange<ChoiceIndex<Self::Ix>>;
    fn branches(&self) -> SemiboundedIndexRange<BranchIndex<Self::Ix>>;

    fn choices_of_state(&self, state: StateIndex<Self::Ix>) -> IndexRange<ChoiceIndex<Self::Ix>>;
    fn branches_of_choice(
        &self,
        choice: ChoiceIndex<Self::Ix>,
    ) -> IndexRange<BranchIndex<Self::Ix>>;

    fn branch_probability(&self, branch: BranchIndex<Self::Ix>) -> f64;
    fn branch_destination(&self, branch: BranchIndex<Self::Ix>) -> StateIndex<Self::Ix>;
    // TODO: a branch(BI) -> (f64, SI) might be a nice quality-of-life improvement.

    fn successors_of_state(
        &self,
        state: StateIndex<Self::Ix>,
    ) -> impl Iterator<Item = StateIndex<Self::Ix>>
    where
        Self: Sized,
    {
        SuccessorIterator {
            choices_iterator: self.choices_of_state(state).into_iter(),
            branches_iterator: IndexRangeIterator::empty(), // The branch iterator will be initialised on the first iteration
            model: self,
        }
    }

    /// All (state, choice, branch) triples, ordered by state, then choice, then branch.
    fn state_choice_branch_triples(&self) -> impl Iterator<Item = StateChoiceBranch<Self::Ix>>
    where
        Self: Sized,
    {
        self.states().into_iter().flat_map(move |state| {
            self.choices_of_state(state)
                .into_iter()
                .flat_map(move |choice| {
                    self.branches_of_choice(choice)
                        .into_iter()
                        .map(move |branch| (state, choice, branch))
                })
        })
    }

    /// All (state, choice, probability, destination) tuples, ordered by state, then choice, then
    /// branch.
    fn transitions(&self) -> impl Iterator<Item = Transition<Self::Ix>>
    where
        Self: Sized,
    {
        self.state_choice_branch_triples()
            .map(move |(state, choice, branch)| {
                (
                    state,
                    choice,
                    self.branch_probability(branch),
                    self.branch_destination(branch),
                )
            })
    }
}

struct SuccessorIterator<'a, M: ReadStateSpace> {
    choices_iterator: IndexRangeIterator<ChoiceIndex<M::Ix>>,
    branches_iterator: IndexRangeIterator<BranchIndex<M::Ix>>,
    model: &'a M,
}

impl<'a, M: ReadStateSpace> Iterator for SuccessorIterator<'a, M> {
    type Item = StateIndex<M::Ix>;

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
        fn states(
            &self,
        ) -> typed_index_collections::SemiboundedIndexRange<$crate::StateIndex<Self::Ix>> {
            self.$subcomponent.states()
        }

        fn choices(
            &self,
        ) -> typed_index_collections::SemiboundedIndexRange<$crate::ChoiceIndex<Self::Ix>> {
            self.$subcomponent.choices()
        }

        fn branches(
            &self,
        ) -> typed_index_collections::SemiboundedIndexRange<$crate::BranchIndex<Self::Ix>> {
            self.$subcomponent.branches()
        }

        fn choices_of_state(
            &self,
            state: $crate::StateIndex<Self::Ix>,
        ) -> typed_index_collections::IndexRange<$crate::ChoiceIndex<Self::Ix>> {
            self.$subcomponent.choices_of_state(state)
        }

        fn branches_of_choice(
            &self,
            choice: $crate::ChoiceIndex<Self::Ix>,
        ) -> typed_index_collections::IndexRange<$crate::BranchIndex<Self::Ix>> {
            self.$subcomponent.branches_of_choice(choice)
        }

        fn branch_probability(&self, branch: $crate::BranchIndex<Self::Ix>) -> f64 {
            self.$subcomponent.branch_probability(branch)
        }

        fn branch_destination(
            &self,
            branch: $crate::BranchIndex<Self::Ix>,
        ) -> $crate::StateIndex<Self::Ix> {
            self.$subcomponent.branch_destination(branch)
        }

        fn successors_of_state(
            &self,
            state: $crate::StateIndex<Self::Ix>,
        ) -> impl Iterator<Item = $crate::StateIndex<Self::Ix>>
        where
            Self: Sized,
        {
            self.$subcomponent.successors_of_state(state)
        }

        fn state_choice_branch_triples(
            &self,
        ) -> impl Iterator<Item = $crate::traits::StateChoiceBranch<Self::Ix>>
        where
            Self: Sized,
        {
            self.$subcomponent.state_choice_branch_triples()
        }

        fn transitions(&self) -> impl Iterator<Item = $crate::traits::Transition<Self::Ix>>
        where
            Self: Sized,
        {
            self.$subcomponent.transitions()
        }
    };
}
pub(crate) use derive_read_state_space;

impl<M: ReadStateSpace, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds> ReadStateSpace
    for crate::Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    derive_read_state_space!(base);
}

pub trait ReadStateBranches: ReadStateSpace {
    fn branches_of_state(&self, state: StateIndex<Self::Ix>) -> IndexRange<BranchIndex<Self::Ix>>;
}

macro_rules! derive_read_state_branches {
    ($subcomponent:ident) => {
        fn branches_of_state(
            &self,
            state: $crate::StateIndex<Self::Ix>,
        ) -> typed_index_collections::IndexRange<$crate::BranchIndex<Self::Ix>> {
            self.$subcomponent.branches_of_state(state)
        }
    };
}
pub(crate) use derive_read_state_branches;

impl<M: ReadStateBranches, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
    ReadStateBranches
    for crate::Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    derive_read_state_branches!(base);
}

impl<T: ReadStateBranches> ReadStateBranches for &T {
    fn branches_of_state(&self, state: StateIndex<Self::Ix>) -> IndexRange<BranchIndex<Self::Ix>> {
        (**self).branches_of_state(state)
    }
}

impl<T: ReadStateBranches> ReadStateBranches for &mut T {
    fn branches_of_state(&self, state: StateIndex<Self::Ix>) -> IndexRange<BranchIndex<Self::Ix>> {
        (**self).branches_of_state(state)
    }
}

pub trait ReadChoiceDestinations: ReadStateSpace {
    fn choice_destination(&self, choice: ChoiceIndex<Self::Ix>) -> StateIndex<Self::Ix>;
}

macro_rules! derive_read_choice_destinations {
    ($subcomponent:ident) => {
        fn choice_destination(
            &self,
            choice: $crate::ChoiceIndex<Self::Ix>,
        ) -> $crate::StateIndex<Self::Ix> {
            self.$subcomponent.choice_destination(choice)
        }
    };
}
pub(crate) use derive_read_choice_destinations;

impl<M: ReadChoiceDestinations, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
    ReadChoiceDestinations
    for crate::Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    derive_read_choice_destinations!(base);
}

impl<T: ReadChoiceDestinations> ReadChoiceDestinations for &T {
    fn choice_destination(&self, choice: ChoiceIndex<Self::Ix>) -> StateIndex<Self::Ix> {
        (**self).choice_destination(choice)
    }
}

impl<T: ReadChoiceDestinations> ReadChoiceDestinations for &mut T {
    fn choice_destination(&self, choice: ChoiceIndex<Self::Ix>) -> StateIndex<Self::Ix> {
        (**self).choice_destination(choice)
    }
}

impl<T: ReadStateSpace> ReadStateSpace for &T {
    fn states(&self) -> SemiboundedIndexRange<StateIndex<Self::Ix>> {
        (**self).states()
    }
    fn choices(&self) -> SemiboundedIndexRange<ChoiceIndex<Self::Ix>> {
        (**self).choices()
    }
    fn branches(&self) -> SemiboundedIndexRange<BranchIndex<Self::Ix>> {
        (**self).branches()
    }
    fn choices_of_state(&self, state: StateIndex<Self::Ix>) -> IndexRange<ChoiceIndex<Self::Ix>> {
        (**self).choices_of_state(state)
    }
    fn branches_of_choice(
        &self,
        choice: ChoiceIndex<Self::Ix>,
    ) -> IndexRange<BranchIndex<Self::Ix>> {
        (**self).branches_of_choice(choice)
    }
    fn branch_probability(&self, branch: BranchIndex<Self::Ix>) -> f64 {
        (**self).branch_probability(branch)
    }
    fn branch_destination(&self, branch: BranchIndex<Self::Ix>) -> StateIndex<Self::Ix> {
        (**self).branch_destination(branch)
    }
    fn successors_of_state(
        &self,
        state: StateIndex<Self::Ix>,
    ) -> impl Iterator<Item = StateIndex<Self::Ix>>
    where
        Self: Sized,
    {
        (**self).successors_of_state(state)
    }
    fn state_choice_branch_triples(&self) -> impl Iterator<Item = StateChoiceBranch<Self::Ix>>
    where
        Self: Sized,
    {
        (**self).state_choice_branch_triples()
    }
    fn transitions(&self) -> impl Iterator<Item = Transition<Self::Ix>>
    where
        Self: Sized,
    {
        (**self).transitions()
    }
}

impl<T: ReadStateSpace> ReadStateSpace for &mut T {
    fn states(&self) -> SemiboundedIndexRange<StateIndex<Self::Ix>> {
        (**self).states()
    }
    fn choices(&self) -> SemiboundedIndexRange<ChoiceIndex<Self::Ix>> {
        (**self).choices()
    }
    fn branches(&self) -> SemiboundedIndexRange<BranchIndex<Self::Ix>> {
        (**self).branches()
    }
    fn choices_of_state(&self, state: StateIndex<Self::Ix>) -> IndexRange<ChoiceIndex<Self::Ix>> {
        (**self).choices_of_state(state)
    }
    fn branches_of_choice(
        &self,
        choice: ChoiceIndex<Self::Ix>,
    ) -> IndexRange<BranchIndex<Self::Ix>> {
        (**self).branches_of_choice(choice)
    }
    fn branch_probability(&self, branch: BranchIndex<Self::Ix>) -> f64 {
        (**self).branch_probability(branch)
    }
    fn branch_destination(&self, branch: BranchIndex<Self::Ix>) -> StateIndex<Self::Ix> {
        (**self).branch_destination(branch)
    }
    fn successors_of_state(
        &self,
        state: StateIndex<Self::Ix>,
    ) -> impl Iterator<Item = StateIndex<Self::Ix>>
    where
        Self: Sized,
    {
        (**self).successors_of_state(state)
    }
    fn state_choice_branch_triples(&self) -> impl Iterator<Item = StateChoiceBranch<Self::Ix>>
    where
        Self: Sized,
    {
        (**self).state_choice_branch_triples()
    }
    fn transitions(&self) -> impl Iterator<Item = Transition<Self::Ix>>
    where
        Self: Sized,
    {
        (**self).transitions()
    }
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
