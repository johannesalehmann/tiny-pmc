use crate::Model;
use crate::valuations::{ValuationEntry, Valuations};
use typed_index_collections::Index;

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

macro_rules! derive_read_valuations {
    ($subcomponent:ident) => {
        fn state_valuation(
            &self,
            state: Self::StateIdx,
        ) -> $crate::valuations::ValuationEntry<
            '_,
            Self::ClassIdx,
            Self::ClassEntryIdx,
            Self::ValuationIdx,
        > {
            self.$subcomponent.state_valuation(state)
        }
    };
}
pub(crate) use derive_read_valuations;

impl<EntityIdx: Index, ClassIdx: Index, ClassEntryIdx: Index, ValuationIdx: Index> ReadValuations
    for Valuations<EntityIdx, ClassIdx, ClassEntryIdx, ValuationIdx>
{
    type StateIdx = EntityIdx;
    type ClassIdx = ClassIdx;
    type ClassEntryIdx = ClassEntryIdx;
    type ValuationIdx = ValuationIdx;

    fn state_valuation(
        &self,
        state: Self::StateIdx,
    ) -> ValuationEntry<'_, Self::ClassIdx, Self::ClassEntryIdx, Self::ValuationIdx> {
        self.entry(state)
    }
}

impl<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals: ReadValuations, Preds> ReadValuations
    for Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>
{
    type StateIdx = StateVals::StateIdx;
    type ClassIdx = StateVals::ClassIdx;
    type ClassEntryIdx = StateVals::ClassEntryIdx;
    type ValuationIdx = StateVals::ValuationIdx;

    derive_read_valuations!(state_valuations);
}

impl<T: ReadValuations> ReadValuations for &T {
    type StateIdx = T::StateIdx;
    type ClassIdx = T::ClassIdx;
    type ClassEntryIdx = T::ClassEntryIdx;
    type ValuationIdx = T::ValuationIdx;

    fn state_valuation(
        &self,
        state: Self::StateIdx,
    ) -> ValuationEntry<'_, Self::ClassIdx, Self::ClassEntryIdx, Self::ValuationIdx> {
        (**self).state_valuation(state)
    }
}

impl<T: ReadValuations> ReadValuations for &mut T {
    type StateIdx = T::StateIdx;
    type ClassIdx = T::ClassIdx;
    type ClassEntryIdx = T::ClassEntryIdx;
    type ValuationIdx = T::ValuationIdx;

    fn state_valuation(
        &self,
        state: Self::StateIdx,
    ) -> ValuationEntry<'_, Self::ClassIdx, Self::ClassEntryIdx, Self::ValuationIdx> {
        (**self).state_valuation(state)
    }
}

pub trait ReadValuationsMaybe {
    type StateIdx: Index;
    type ClassIdx: Index;
    type ClassEntryIdx: Index;
    type ValuationIdx: Index;
    type WithValuations: ReadValuations<
            StateIdx = Self::StateIdx,
            ClassIdx = Self::ClassIdx,
            ClassEntryIdx = Self::ClassEntryIdx,
            ValuationIdx = Self::ValuationIdx,
        >;

    fn has_valuations(&self) -> bool;
    fn try_with_valuations(self) -> Option<Self::WithValuations>;
}

impl<T: ReadValuations> ReadValuationsMaybe for T {
    type StateIdx = T::StateIdx;
    type ClassIdx = T::ClassIdx;
    type ClassEntryIdx = T::ClassEntryIdx;
    type ValuationIdx = T::ValuationIdx;
    type WithValuations = T;

    fn has_valuations(&self) -> bool {
        true
    }

    fn try_with_valuations(self) -> Option<Self::WithValuations> {
        Some(self)
    }
}

impl<T: ReadValuations> ReadValuationsMaybe for Option<T> {
    type StateIdx = T::StateIdx;
    type ClassIdx = T::ClassIdx;
    type ClassEntryIdx = T::ClassEntryIdx;
    type ValuationIdx = T::ValuationIdx;
    type WithValuations = T;

    fn has_valuations(&self) -> bool {
        self.is_some()
    }

    fn try_with_valuations(self) -> Option<Self::WithValuations> {
        self
    }
}

impl<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals: ReadValuations, Preds>
    ReadValuationsMaybe
    for Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, Option<StateVals>, Preds>
{
    type StateIdx = StateVals::StateIdx;
    type ClassIdx = StateVals::ClassIdx;
    type ClassEntryIdx = StateVals::ClassEntryIdx;
    type ValuationIdx = StateVals::ValuationIdx;
    type WithValuations = Model<M, Ini, ChLabel, BrLabel, Obs, APs, Rew, Ann, StateVals, Preds>;

    fn has_valuations(&self) -> bool {
        self.state_valuations.is_some()
    }

    fn try_with_valuations(self) -> Option<Self::WithValuations> {
        Some(Model {
            base: self.base,
            initial: self.initial,
            choice_labels: self.choice_labels,
            branch_labels: self.branch_labels,
            observations: self.observations,
            atomic_propositions: self.atomic_propositions,
            rewards: self.rewards,
            annotations: self.annotations,
            state_valuations: self.state_valuations?,
            predecessors: self.predecessors,
        })
    }
}
