mod valuation_to_state;
use valuation_to_state::ValuationToEntity;

use crate::to1::To1;
use crate::valuations::{StandaloneValuation, Valuations};
use crate::{RawIndex, StateIndex, ValuationClassIndex};

pub struct ValuationBuilder<I: RawIndex> {
    state_valuations: Valuations<I, StateIndex<I>>,
    valuation_to_state: To1<ValuationClassIndex<I>, ValuationToEntity<StateIndex<I>>>,
}

impl<I: RawIndex> ValuationBuilder<I> {
    pub fn state_by_valuation(&self, valuation: &StandaloneValuation<I>) -> Option<StateIndex<I>> {
        self.valuation_to_state[valuation.class_index].get(valuation)
    }

    pub fn add_state_valuation(
        &mut self,
        valuation: &StandaloneValuation<I>,
        state_index: StateIndex<I>,
    ) {
        self.valuation_to_state[valuation.class_index].add(&valuation, state_index);
    }

    pub fn state_valuations(&self) -> &Valuations<I, StateIndex<I>> {
        &self.state_valuations
    }

    pub fn into_state_valuations(self) -> Valuations<I, StateIndex<I>> {
        self.state_valuations
    }
}
