mod valuation_to_state;
use valuation_to_state::ValuationToEntity;

use crate::valuations::{
    BareStandaloneValuation, GetValuationClassIndex, GetValuationData, StandaloneValuation,
    Valuations,
};
use crate::{RawIndex, StateIndex, ValuationClassIndex};
use typed_index_collections::{Index, To1};

pub struct ValuationBuilder<
    StateIdx: Index,
    ClassIdx: Index,
    ClassEntryIdx: Index,
    ValuationIdx: Index,
> {
    state_valuations: Valuations<StateIdx, ClassIdx, ClassEntryIdx, ValuationIdx>,
    valuation_to_state: To1<ClassIdx, ValuationToEntity<StateIdx>>,
}

impl<StateIdx: Index, ClassIdx: Index, ClassEntryIdx: Index, ValuationIdx: Index>
    ValuationBuilder<StateIdx, ClassIdx, ClassEntryIdx, ValuationIdx>
{
    pub fn state_by_valuation<
        Val: GetValuationClassIndex<ClassIdx> + GetValuationData<ValuationIdx>,
    >(
        &self,
        valuation: &Val,
    ) -> Option<StateIdx> {
        self.valuation_to_state[valuation.valuation_class_index()].get(valuation)
    }

    pub fn add_state_valuation<
        Val: GetValuationClassIndex<ClassIdx> + GetValuationData<ValuationIdx>,
    >(
        &mut self,
        valuation: &Val,
        state_index: StateIdx,
    ) {
        self.valuation_to_state[valuation.valuation_class_index()].add(valuation, state_index);
    }

    pub fn state_valuations(&self) -> &Valuations<StateIdx, ClassIdx, ClassEntryIdx, ValuationIdx> {
        &self.state_valuations
    }

    pub fn state_valuations_mut(
        &mut self,
    ) -> &mut Valuations<StateIdx, ClassIdx, ClassEntryIdx, ValuationIdx> {
        &mut self.state_valuations
    }

    pub fn into_state_valuations(
        self,
    ) -> Valuations<StateIdx, ClassIdx, ClassEntryIdx, ValuationIdx> {
        self.state_valuations
    }
}
