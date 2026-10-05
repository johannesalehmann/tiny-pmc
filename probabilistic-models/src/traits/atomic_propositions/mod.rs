use crate::Model;
use crate::annotations::AtomicPropositions;
use typed_index_collections::Index;

pub trait ReadAtomicPropositions {
    type StateIdx: Index;
    type APIdx: Index;

    fn is_atomic_proposition_set(
        &self,
        state: Self::StateIdx,
        atomic_proposition: Self::APIdx,
    ) -> bool;

    fn atomic_proposition_by_name(&self, name: &str) -> Option<Self::APIdx>;
}

macro_rules! derive_read_atomic_propositions {
    ($subcomponent:ident) => {
        fn is_atomic_proposition_set(
            &self,
            state: Self::StateIdx,
            atomic_proposition: Self::APIdx,
        ) -> bool {
            self.$subcomponent
                .is_atomic_proposition_set(state, atomic_proposition)
        }
        fn atomic_proposition_by_name(&self, name: &str) -> Option<Self::APIdx> {
            self.$subcomponent.atomic_proposition_by_name(name)
        }
    };
}

pub(crate) use derive_read_atomic_propositions;

impl<AI: Index, SI: Index, AEI: Index> ReadAtomicPropositions for AtomicPropositions<AI, SI, AEI> {
    type StateIdx = SI;
    type APIdx = AI;

    fn is_atomic_proposition_set(
        &self,
        state: Self::StateIdx,
        atomic_proposition: Self::APIdx,
    ) -> bool {
        self.entries()[atomic_proposition][state]
    }

    fn atomic_proposition_by_name(&self, name: &str) -> Option<Self::APIdx> {
        self.index_by_name(name)
    }
}

impl<M, Ini, ChLabel, BrLabel, Obs, APs: ReadAtomicPropositions, Rew, Ann, StateVals, Preds>
    ReadAtomicPropositions
    for Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    type StateIdx = APs::StateIdx;
    type APIdx = APs::APIdx;

    derive_read_atomic_propositions!(atomic_propositions);
}

impl<T: ReadAtomicPropositions> ReadAtomicPropositions for &T {
    type StateIdx = T::StateIdx;
    type APIdx = T::APIdx;

    fn is_atomic_proposition_set(
        &self,
        state: Self::StateIdx,
        atomic_proposition: Self::APIdx,
    ) -> bool {
        (**self).is_atomic_proposition_set(state, atomic_proposition)
    }
    fn atomic_proposition_by_name(&self, name: &str) -> Option<Self::APIdx> {
        (**self).atomic_proposition_by_name(name)
    }
}

impl<T: ReadAtomicPropositions> ReadAtomicPropositions for &mut T {
    type StateIdx = T::StateIdx;
    type APIdx = T::APIdx;

    fn is_atomic_proposition_set(
        &self,
        state: Self::StateIdx,
        atomic_proposition: Self::APIdx,
    ) -> bool {
        (**self).is_atomic_proposition_set(state, atomic_proposition)
    }
    fn atomic_proposition_by_name(&self, name: &str) -> Option<Self::APIdx> {
        (**self).atomic_proposition_by_name(name)
    }
}

pub trait ReadAtomicPropositionsMaybe {
    type StateIdx: Index;
    type APIdx: Index;
    type WithAtomicPropositions: ReadAtomicPropositions<StateIdx = Self::StateIdx, APIdx = Self::APIdx>;

    fn has_atomic_propositions(&self) -> bool;
    fn try_with_atomic_propositions(self) -> Option<Self::WithAtomicPropositions>;
}

impl<T: ReadAtomicPropositions> ReadAtomicPropositionsMaybe for T {
    type StateIdx = T::StateIdx;
    type APIdx = T::APIdx;
    type WithAtomicPropositions = T;

    fn has_atomic_propositions(&self) -> bool {
        true
    }

    fn try_with_atomic_propositions(self) -> Option<Self::WithAtomicPropositions> {
        Some(self)
    }
}

impl<T: ReadAtomicPropositions> ReadAtomicPropositionsMaybe for Option<T> {
    type StateIdx = T::StateIdx;
    type APIdx = T::APIdx;
    type WithAtomicPropositions = T;

    fn has_atomic_propositions(&self) -> bool {
        self.is_some()
    }

    fn try_with_atomic_propositions(self) -> Option<Self::WithAtomicPropositions> {
        self
    }
}

impl<M, Ini, ChLabel, BrLabel, Obs, APs: ReadAtomicPropositions, Rew, Ann, StateVals, Preds>
    ReadAtomicPropositionsMaybe
    for Model<M, Ini, ChLabel, BrLabel, Obs, Option<APs>, Rew, Ann, StateVals, Preds>
{
    type StateIdx = APs::StateIdx;
    type APIdx = APs::APIdx;
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
