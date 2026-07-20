mod valuation_to_state;
use valuation_to_state::ValuationToEntity;

use probabilistic_models::valuations::{
    GetValuationClassIndex, GetValuationData, ValuationClass, Valuations,
};
use typed_index_collections::{Index, To1};

#[derive(Default)]
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
    pub fn add_class(&mut self, class: ValuationClass<ClassEntryIdx>) -> ClassIdx {
        let width = class.size_in_bits();
        let index = self.state_valuations.add_class(class);
        self.valuation_to_state
            .add_checked(index, ValuationToEntity::new(width));
        index
    }

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
        self.state_valuations.add_valuation(state_index, valuation);
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
