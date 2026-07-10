mod mdp;
pub use mdp::MdpBuilder;
use typed_index_collections::Index;

mod valuation_builder;
use valuation_builder::ValuationBuilder;

use crate::RawIndex;
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
    fn add_state<
        Val: GetValuationClassIndex<Self::ClassIdx> + GetValuationData<Self::ValuationIdx>,
    >(
        &mut self,
        valuation: Val,
    ) -> Self::StateIdx;
    fn state_valuations(
        &self,
    ) -> &Valuations<Self::StateIdx, Self::ClassIdx, Self::ClassEntryIdx, Self::ValuationIdx>;
    fn state_valuations_mut(
        &mut self,
    ) -> &mut Valuations<Self::StateIdx, Self::ClassIdx, Self::ClassEntryIdx, Self::ValuationIdx>;

    fn add_choice(&mut self) -> Self::ChoiceIdx; // TODO: Do we need this or is finish_choice sufficient?
    fn add_branch(&mut self, rate_or_probability: f64, target: Self::StateIdx) -> Self::BranchIdx;
    fn finish_choice(&mut self);
    fn finish_branch(&mut self); // TODO: I don't think this method is ever useful, as branches are atomically created, i.e. every check performed here could be performed in add_branch itself

    fn into_base_and_valuations(self) -> (Self::BaseModel, Self::Valuation);
}
