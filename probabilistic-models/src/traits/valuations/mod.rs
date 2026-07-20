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

    fn state_valuation(
        &self,
        state: Self::StateIdx,
    ) -> ValuationEntry<'_, Self::ClassIdx, Self::ClassEntryIdx, Self::ValuationIdx> {
        self.state_valuations.state_valuation(state)
    }
}
