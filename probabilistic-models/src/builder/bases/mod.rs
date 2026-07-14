mod mdp;
pub use mdp::MdpBuilder;
use typed_index_collections::Index;

mod valuation_builder;
pub use valuation_builder::ValuationBuilder;

use crate::valuations::{GetValuationClassIndex, GetValuationData, Valuations};

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

    fn start_choice(&mut self) -> Self::ChoiceIdx; // TODO: Do we need this or is finish_choice sufficient?
    fn add_branch(&mut self, rate_or_probability: f64, target: Self::StateIdx) -> Self::BranchIdx;
    fn finish_choice(&mut self);

    fn add_choice_from_slice(&mut self, branches: &[(f64, Self::StateIdx)]) -> Self::ChoiceIdx {
        let index = self.start_choice();
        for &(rate_or_probability, target) in branches {
            self.add_branch(rate_or_probability, target);
        }
        self.finish_choice();
        index
    }

    fn into_base_and_valuations(self) -> (Self::BaseModel, Self::Valuation);
}
