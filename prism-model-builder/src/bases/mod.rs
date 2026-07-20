mod mdp;
pub use mdp::MdpBuilder;
use typed_index_collections::Index;

mod valuation_builder;
pub use valuation_builder::ValuationBuilder;

use probabilistic_models::valuations::{
    GetValuationClassIndex, GetValuationData, StandaloneValuation, Valuations,
};

pub trait BaseModelBuilder {
    type BaseModel;
    type Valuation;
    type StateIdx: Index;
    type ChoiceIdx: Index;
    type BranchIdx: Index;
    type ClassIdx: Index;
    type ClassEntryIdx: Index;
    type ValuationIdx: Index;

    fn state_by_valuation<
        Val: GetValuationClassIndex<Self::ClassIdx> + GetValuationData<Self::ValuationIdx>,
    >(
        &self,
        valuation: &Val,
    ) -> Option<Self::StateIdx>;
    fn add_valuation<
        Val: GetValuationClassIndex<Self::ClassIdx> + GetValuationData<Self::ValuationIdx>,
    >(
        &mut self,
        valuation: Val,
    ) -> Self::StateIdx;
    fn add_state(&mut self, state_index: Self::StateIdx);
    fn state_valuations(
        &self,
    ) -> &Valuations<Self::StateIdx, Self::ClassIdx, Self::ClassEntryIdx, Self::ValuationIdx> {
        self.valuation_builder().state_valuations()
    }
    fn state_valuations_mut(
        &mut self,
    ) -> &mut Valuations<Self::StateIdx, Self::ClassIdx, Self::ClassEntryIdx, Self::ValuationIdx>
    {
        self.valuation_builder_mut().state_valuations_mut()
    }
    fn valuation_builder(
        &self,
    ) -> &ValuationBuilder<Self::StateIdx, Self::ClassIdx, Self::ClassEntryIdx, Self::ValuationIdx>;
    fn valuation_builder_mut(
        &mut self,
    ) -> &mut ValuationBuilder<
        Self::StateIdx,
        Self::ClassIdx,
        Self::ClassEntryIdx,
        Self::ValuationIdx,
    >;
    fn create_valuation(
        &self,
        class_index: Self::ClassIdx,
    ) -> StandaloneValuation<Self::ClassIdx, Self::ClassEntryIdx, Self::ValuationIdx> {
        StandaloneValuation::new(class_index, self.state_valuations().class(class_index))
    }

    fn start_choice(&mut self) -> Self::ChoiceIdx; // TODO: Do we need this or is finish_choice sufficient?
    fn add_branch(&mut self, rate_or_probability: f64, target: Self::StateIdx) -> Self::BranchIdx;

    // TODO: Instead of requiring an explicit call to this, start_choice
    //  could return a struct through which the branches are added. This struct, on being dropped,
    //  could then perform choice-finishing operations. The same could be done for add_state. This
    //  also more closely models that actual structure and ensures choices and branches are added
    //  correctly.
    fn finish_choice(&mut self);

    fn into_base_and_valuations(self) -> (Self::BaseModel, Self::Valuation);
}
