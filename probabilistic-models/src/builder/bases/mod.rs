mod mdp;
pub use mdp::MdpBuilder;

mod valuation_builder;
use valuation_builder::ValuationBuilder;

use crate::valuations::{
    BareStandaloneValuation, GetValuationClassIndex, GetValuationData, StandaloneValuation,
    Valuations,
};
use crate::{BaseModel, BranchIndex, ChoiceIndex, RawIndex, StateIndex};

pub trait BaseModelBuilder {
    type BaseModel: BaseModel<Self::Index>;
    type Index: RawIndex;

    fn state_by_valuation<V: GetValuationClassIndex<Self::Index> + GetValuationData<Self::Index>>(
        &self,
        valuation: &V,
    ) -> Option<StateIndex<Self::Index>>;
    fn add_state<Val: GetValuationData<Self::Index> + GetValuationClassIndex<Self::Index>>(
        &mut self,
        valuation: Val,
    ) -> StateIndex<Self::Index>;
    fn state_valuations(&self) -> &Valuations<Self::Index, StateIndex<Self::Index>>;
    fn state_valuations_mut(&mut self) -> &mut Valuations<Self::Index, StateIndex<Self::Index>>;

    fn add_choice(&mut self) -> ChoiceIndex<Self::Index>; // TODO: Do we need this or is finish_choice sufficient?
    fn add_branch(
        &mut self,
        rate_or_probability: f64,
        target: StateIndex<Self::Index>,
    ) -> BranchIndex<Self::Index>;
    fn finish_choice(&mut self);
    fn finish_branch(&mut self); // TODO: I don't think this method is ever useful, as branches are atomically created, i.e. every check performed here could be performed in add_branch itself

    fn into_base_and_valuations(
        self,
    ) -> (
        Self::BaseModel,
        Valuations<Self::Index, StateIndex<Self::Index>>,
    );
}
