mod valuation_to_state;
use valuation_to_state::ValuationToEntity;

use crate::valuations::{
    BareStandaloneValuation, GetValuationClassIndex, GetValuationData, StandaloneValuation,
    Valuations,
};
use crate::{RawIndex, StateIndex, ValuationClassIndex};
use typed_index_collections::To1;

pub struct ValuationBuilder<I: RawIndex> {
    state_valuations: Valuations<I, StateIndex<I>>,
    valuation_to_state: To1<ValuationClassIndex<I>, ValuationToEntity<StateIndex<I>>>,
}

impl<I: RawIndex> ValuationBuilder<I> {
    pub fn state_by_valuation<V: GetValuationClassIndex<I> + GetValuationData<I>>(
        &self,
        valuation: &V,
    ) -> Option<StateIndex<I>> {
        self.valuation_to_state[valuation.valuation_class_index()].get(valuation)
    }

    pub fn add_state_valuation<Val: GetValuationData<I> + GetValuationClassIndex<I>>(
        &mut self,
        valuation: &Val,
        state_index: StateIndex<I>,
    ) {
        self.valuation_to_state[valuation.valuation_class_index()].add(valuation, state_index);
    }

    pub fn state_valuations(&self) -> &Valuations<I, StateIndex<I>> {
        &self.state_valuations
    }

    pub fn state_valuations_mut(&mut self) -> &mut Valuations<I, StateIndex<I>> {
        &mut self.state_valuations
    }

    pub fn into_state_valuations(self) -> Valuations<I, StateIndex<I>> {
        self.state_valuations
    }
}
