use crate::Model;
use crate::annotations::AtomicPropositions;
use crate::{AtomicPropositionIndex, HasIx, Ix, StateIndex};

pub trait ReadAtomicPropositions: HasIx {
    fn is_atomic_proposition_set(
        &self,
        state: StateIndex<Self::Ix>,
        atomic_proposition: AtomicPropositionIndex<Self::Ix>,
    ) -> bool;

    fn atomic_proposition_by_name(&self, name: &str) -> Option<AtomicPropositionIndex<Self::Ix>>;
}

macro_rules! derive_read_atomic_propositions {
    ($subcomponent:ident) => {
        fn is_atomic_proposition_set(
            &self,
            state: $crate::StateIndex<Self::Ix>,
            atomic_proposition: $crate::AtomicPropositionIndex<Self::Ix>,
        ) -> bool {
            self.$subcomponent
                .is_atomic_proposition_set(state, atomic_proposition)
        }
        fn atomic_proposition_by_name(
            &self,
            name: &str,
        ) -> Option<$crate::AtomicPropositionIndex<Self::Ix>> {
            self.$subcomponent.atomic_proposition_by_name(name)
        }
    };
}

pub(crate) use derive_read_atomic_propositions;

impl<I: Ix> ReadAtomicPropositions for AtomicPropositions<I> {
    fn is_atomic_proposition_set(
        &self,
        state: StateIndex<Self::Ix>,
        atomic_proposition: AtomicPropositionIndex<Self::Ix>,
    ) -> bool {
        self.entries()[atomic_proposition][state]
    }

    fn atomic_proposition_by_name(&self, name: &str) -> Option<AtomicPropositionIndex<Self::Ix>> {
        self.index_by_name(name)
    }
}

impl<
    M: HasIx,
    Ini,
    ChLabel,
    BrLabel,
    Obs,
    APs: ReadAtomicPropositions<Ix = M::Ix>,
    Rew,
    Ann,
    StateVals,
    Preds,
> ReadAtomicPropositions for Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    derive_read_atomic_propositions!(atomic_propositions);
}

impl<T: ReadAtomicPropositions> ReadAtomicPropositions for &T {
    fn is_atomic_proposition_set(
        &self,
        state: StateIndex<Self::Ix>,
        atomic_proposition: AtomicPropositionIndex<Self::Ix>,
    ) -> bool {
        (**self).is_atomic_proposition_set(state, atomic_proposition)
    }
    fn atomic_proposition_by_name(&self, name: &str) -> Option<AtomicPropositionIndex<Self::Ix>> {
        (**self).atomic_proposition_by_name(name)
    }
}

impl<T: ReadAtomicPropositions> ReadAtomicPropositions for &mut T {
    fn is_atomic_proposition_set(
        &self,
        state: StateIndex<Self::Ix>,
        atomic_proposition: AtomicPropositionIndex<Self::Ix>,
    ) -> bool {
        (**self).is_atomic_proposition_set(state, atomic_proposition)
    }
    fn atomic_proposition_by_name(&self, name: &str) -> Option<AtomicPropositionIndex<Self::Ix>> {
        (**self).atomic_proposition_by_name(name)
    }
}

pub trait ReadAtomicPropositionsMaybe: HasIx {
    type WithAtomicPropositions: ReadAtomicPropositions<Ix = Self::Ix>;

    fn has_atomic_propositions(&self) -> bool;
    fn try_with_atomic_propositions(self) -> Option<Self::WithAtomicPropositions>;
}

impl<T: ReadAtomicPropositions> ReadAtomicPropositionsMaybe for T {
    type WithAtomicPropositions = T;

    fn has_atomic_propositions(&self) -> bool {
        true
    }

    fn try_with_atomic_propositions(self) -> Option<Self::WithAtomicPropositions> {
        Some(self)
    }
}

impl<T: ReadAtomicPropositions> ReadAtomicPropositionsMaybe for Option<T> {
    type WithAtomicPropositions = T;

    fn has_atomic_propositions(&self) -> bool {
        self.is_some()
    }

    fn try_with_atomic_propositions(self) -> Option<Self::WithAtomicPropositions> {
        self
    }
}

impl<
    M: HasIx,
    Ini,
    ChLabel,
    BrLabel,
    Obs,
    APs: ReadAtomicPropositions<Ix = M::Ix>,
    Rew,
    Ann,
    StateVals,
    Preds,
> ReadAtomicPropositionsMaybe
    for Model<M, Ini, ChLabel, BrLabel, Obs, Option<APs>, Rew, Ann, StateVals, Preds>
{
    type WithAtomicPropositions =
        Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>;

    fn has_atomic_propositions(&self) -> bool {
        self.atomic_propositions.is_some()
    }

    fn try_with_atomic_propositions(self) -> Option<Self::WithAtomicPropositions> {
        Some(Model {
            base: self.base,
            initial: self.initial,
            choice_labels: self.choice_labels,
            branch_labels: self.branch_labels,
            observations: self.observations,
            atomic_propositions: self.atomic_propositions?,
            rewards: self.rewards,
            annotations: self.annotations,
            state_valuations: self.state_valuations,
            predecessors: self.predecessors,
        })
    }
}
