mod valuation_to_state;
use probabilistic_models::{
    Ix, StateIndex, ValuationClassEntryIndex, ValuationClassIndex, ValuationIndex,
};
use valuation_to_state::ValuationToEntity;

use probabilistic_models::valuations::{
    GetValuationClassIndex, GetValuationData, ValuationClass, Valuations,
};
use typed_index_collections::To1;

#[derive(Default)]
pub struct ValuationBuilder<I: Ix> {
    state_valuations: Valuations<I>,
    valuation_to_state: To1<ValuationClassIndex<I>, ValuationToEntity<StateIndex<I>>>,
}

impl<I: Ix> ValuationBuilder<I> {
    pub fn add_class(
        &mut self,
        class: ValuationClass<ValuationClassEntryIndex<I>>,
    ) -> ValuationClassIndex<I> {
        let width = class.size_in_bits();
        let index = self.state_valuations.add_class(class);
        self.valuation_to_state
            .add_checked(index, ValuationToEntity::new(width));
        index
    }

    pub fn state_by_valuation<
        Val: GetValuationClassIndex<ValuationClassIndex<I>> + GetValuationData<ValuationIndex<I>>,
    >(
        &self,
        valuation: &Val,
    ) -> Option<StateIndex<I>> {
        self.valuation_to_state[valuation.valuation_class_index()].get(valuation)
    }

    pub fn add_state_valuation<
        Val: GetValuationClassIndex<ValuationClassIndex<I>> + GetValuationData<ValuationIndex<I>>,
    >(
        &mut self,
        valuation: &Val,
        state_index: StateIndex<I>,
    ) {
        self.valuation_to_state[valuation.valuation_class_index()].add(valuation, state_index);
        self.state_valuations.add_valuation(state_index, valuation);
    }

    pub fn state_valuations(&self) -> &Valuations<I> {
        &self.state_valuations
    }

    pub fn state_valuations_mut(&mut self) -> &mut Valuations<I> {
        &mut self.state_valuations
    }

    pub fn into_state_valuations(self) -> Valuations<I> {
        self.state_valuations
    }
}
