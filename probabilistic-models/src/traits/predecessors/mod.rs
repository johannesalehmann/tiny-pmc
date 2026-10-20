use crate::Model;
use crate::predecessors::Predecessors;
use crate::{BranchIndex, ChoiceIndex, HasIx, Ix, PredecessorIndex, StateIndex};
use typed_index_collections::{IndexRange, SemiboundedIndexRange, ValuePerIndexSource};

pub trait ReadPredecessors: HasIx {
    // TODO: This function is a bit of a hack to access the number of states for any
    //  M: ReadPredecessors. However, if M also implements e.g. ReadStateSpace, this information is
    //  duplicated. It might be cleaner to add predecessors_of_states(range<StateIdx>), which
    //  returns a CsrSlice, which has keys() (which then correspond the state space if the above
    //  function is called with an unbounded range. (But this idea still needs some refinement,
    //  and Csr does not allow slicing a the time of writing.)
    fn predecessor_states(&self) -> SemiboundedIndexRange<StateIndex<Self::Ix>>;
    fn predecessors(&self) -> SemiboundedIndexRange<PredecessorIndex<Self::Ix>>;
    fn predecessors_of_state(
        &self,
        state: StateIndex<Self::Ix>,
    ) -> IndexRange<PredecessorIndex<Self::Ix>>;
    fn branch_of_predecessor(
        &self,
        predecessor: PredecessorIndex<Self::Ix>,
    ) -> BranchIndex<Self::Ix>;
    fn choice_of_branch(&self, branch: BranchIndex<Self::Ix>) -> ChoiceIndex<Self::Ix>;
    fn state_of_choice(&self, choice: ChoiceIndex<Self::Ix>) -> StateIndex<Self::Ix>;
    // TODO: There are probably quite a few locations in the algorithms that were written before
    //  these helper functions existed. Find them and use the helper functions to declutter them.
    fn choice_of_predecessor(
        &self,
        predecessor: PredecessorIndex<Self::Ix>,
    ) -> ChoiceIndex<Self::Ix> {
        self.choice_of_branch(self.branch_of_predecessor(predecessor))
    }
    fn source_state_of_predecessor(
        &self,
        predecessor: PredecessorIndex<Self::Ix>,
    ) -> StateIndex<Self::Ix> {
        self.state_of_choice(self.choice_of_branch(self.branch_of_predecessor(predecessor)))
    }
}

macro_rules! derive_read_predecessors {
    ($subcomponent:ident) => {
        fn predecessor_states(
            &self,
        ) -> typed_index_collections::SemiboundedIndexRange<$crate::StateIndex<Self::Ix>> {
            self.$subcomponent.predecessor_states()
        }

        fn predecessors(
            &self,
        ) -> typed_index_collections::SemiboundedIndexRange<$crate::PredecessorIndex<Self::Ix>> {
            self.$subcomponent.predecessors()
        }

        fn predecessors_of_state(
            &self,
            state: $crate::StateIndex<Self::Ix>,
        ) -> typed_index_collections::IndexRange<$crate::PredecessorIndex<Self::Ix>> {
            self.$subcomponent.predecessors_of_state(state)
        }

        fn branch_of_predecessor(
            &self,
            predecessor: $crate::PredecessorIndex<Self::Ix>,
        ) -> $crate::BranchIndex<Self::Ix> {
            self.$subcomponent.branch_of_predecessor(predecessor)
        }

        fn choice_of_branch(
            &self,
            branch: $crate::BranchIndex<Self::Ix>,
        ) -> $crate::ChoiceIndex<Self::Ix> {
            self.$subcomponent.choice_of_branch(branch)
        }

        fn state_of_choice(
            &self,
            choice: $crate::ChoiceIndex<Self::Ix>,
        ) -> $crate::StateIndex<Self::Ix> {
            self.$subcomponent.state_of_choice(choice)
        }
    };
}
pub(crate) use derive_read_predecessors;

impl<I: Ix> ReadPredecessors for Predecessors<I> {
    fn predecessor_states(&self) -> SemiboundedIndexRange<StateIndex<Self::Ix>> {
        self.state_to_predecessor.keys()
    }

    fn predecessors(&self) -> SemiboundedIndexRange<PredecessorIndex<Self::Ix>> {
        self.predecessor_to_branch.keys()
    }

    fn predecessors_of_state(
        &self,
        state: StateIndex<Self::Ix>,
    ) -> IndexRange<PredecessorIndex<Self::Ix>> {
        self.state_to_predecessor.index(state)
    }

    fn branch_of_predecessor(
        &self,
        predecessor: PredecessorIndex<Self::Ix>,
    ) -> BranchIndex<Self::Ix> {
        self.predecessor_to_branch[predecessor]
    }

    fn choice_of_branch(&self, branch: BranchIndex<Self::Ix>) -> ChoiceIndex<Self::Ix> {
        self.branch_to_choice[branch]
    }

    fn state_of_choice(&self, choice: ChoiceIndex<Self::Ix>) -> StateIndex<Self::Ix> {
        self.choice_to_state[choice]
    }
}

impl<
    M: HasIx,
    Ini,
    ChLabel,
    BrLabel,
    Obs,
    APs,
    Rew,
    Ann,
    StateVals,
    Preds: ReadPredecessors<Ix = M::Ix>,
> ReadPredecessors for Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    derive_read_predecessors!(predecessors);
}

impl<T: ReadPredecessors> ReadPredecessors for &T {
    fn predecessor_states(&self) -> SemiboundedIndexRange<StateIndex<Self::Ix>> {
        (**self).predecessor_states()
    }
    fn predecessors(&self) -> SemiboundedIndexRange<PredecessorIndex<Self::Ix>> {
        (**self).predecessors()
    }
    fn predecessors_of_state(
        &self,
        state: StateIndex<Self::Ix>,
    ) -> IndexRange<PredecessorIndex<Self::Ix>> {
        (**self).predecessors_of_state(state)
    }
    fn branch_of_predecessor(
        &self,
        predecessor: PredecessorIndex<Self::Ix>,
    ) -> BranchIndex<Self::Ix> {
        (**self).branch_of_predecessor(predecessor)
    }
    fn choice_of_branch(&self, branch: BranchIndex<Self::Ix>) -> ChoiceIndex<Self::Ix> {
        (**self).choice_of_branch(branch)
    }
    fn state_of_choice(&self, choice: ChoiceIndex<Self::Ix>) -> StateIndex<Self::Ix> {
        (**self).state_of_choice(choice)
    }
    fn choice_of_predecessor(
        &self,
        predecessor: PredecessorIndex<Self::Ix>,
    ) -> ChoiceIndex<Self::Ix> {
        (**self).choice_of_predecessor(predecessor)
    }
    fn source_state_of_predecessor(
        &self,
        predecessor: PredecessorIndex<Self::Ix>,
    ) -> StateIndex<Self::Ix> {
        (**self).source_state_of_predecessor(predecessor)
    }
}

impl<T: ReadPredecessors> ReadPredecessors for &mut T {
    fn predecessor_states(&self) -> SemiboundedIndexRange<StateIndex<Self::Ix>> {
        (**self).predecessor_states()
    }
    fn predecessors(&self) -> SemiboundedIndexRange<PredecessorIndex<Self::Ix>> {
        (**self).predecessors()
    }
    fn predecessors_of_state(
        &self,
        state: StateIndex<Self::Ix>,
    ) -> IndexRange<PredecessorIndex<Self::Ix>> {
        (**self).predecessors_of_state(state)
    }
    fn branch_of_predecessor(
        &self,
        predecessor: PredecessorIndex<Self::Ix>,
    ) -> BranchIndex<Self::Ix> {
        (**self).branch_of_predecessor(predecessor)
    }
    fn choice_of_branch(&self, branch: BranchIndex<Self::Ix>) -> ChoiceIndex<Self::Ix> {
        (**self).choice_of_branch(branch)
    }
    fn state_of_choice(&self, choice: ChoiceIndex<Self::Ix>) -> StateIndex<Self::Ix> {
        (**self).state_of_choice(choice)
    }
    fn choice_of_predecessor(
        &self,
        predecessor: PredecessorIndex<Self::Ix>,
    ) -> ChoiceIndex<Self::Ix> {
        (**self).choice_of_predecessor(predecessor)
    }
    fn source_state_of_predecessor(
        &self,
        predecessor: PredecessorIndex<Self::Ix>,
    ) -> StateIndex<Self::Ix> {
        (**self).source_state_of_predecessor(predecessor)
    }
}

pub trait ReadPredecessorsMaybe: HasIx {
    type WithPredecessors: ReadPredecessors<Ix = Self::Ix>;

    fn has_predecessors(&self) -> bool;
    fn try_with_predecessors(self) -> Option<Self::WithPredecessors>;
}

impl<T: ReadPredecessors> ReadPredecessorsMaybe for T {
    type WithPredecessors = T;

    fn has_predecessors(&self) -> bool {
        true
    }

    fn try_with_predecessors(self) -> Option<Self::WithPredecessors> {
        Some(self)
    }
}

impl<T: ReadPredecessors> ReadPredecessorsMaybe for Option<T> {
    type WithPredecessors = T;

    fn has_predecessors(&self) -> bool {
        self.is_some()
    }

    fn try_with_predecessors(self) -> Option<Self::WithPredecessors> {
        self
    }
}

impl<
    M: HasIx,
    Ini,
    ChLabel,
    BrLabel,
    Obs,
    APs,
    Rew,
    Ann,
    StateVals,
    Preds: ReadPredecessors<Ix = M::Ix>,
> ReadPredecessorsMaybe
    for Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Option<Preds>>
{
    type WithPredecessors = Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>;

    fn has_predecessors(&self) -> bool {
        self.predecessors.is_some()
    }

    fn try_with_predecessors(self) -> Option<Self::WithPredecessors> {
        Some(Model {
            base: self.base,
            initial: self.initial,
            choice_labels: self.choice_labels,
            branch_labels: self.branch_labels,
            observations: self.observations,
            atomic_propositions: self.atomic_propositions,
            rewards: self.rewards,
            annotations: self.annotations,
            state_valuations: self.state_valuations,
            predecessors: self.predecessors?,
        })
    }
}
