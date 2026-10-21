mod mdp;
pub use mdp::MdpBuilder;
use probabilistic_models::{BranchIndex, ChoiceIndex, StateIndex};
use probabilistic_models::{HasIx, ValuationClassIndex, ValuationIndex};

use crate::owners::OwnersBuilder;

mod valuation_builder;
pub use valuation_builder::ValuationBuilder;

use probabilistic_models::valuations::{
    GetValuationClassIndex, GetValuationData, ModelStandaloneValuation, StandaloneValuation,
    Valuations,
};

pub trait BaseModelBuilder: HasIx {
    type BaseModel;
    type Valuation;
    type Owners: OwnersBuilder<Ix = Self::Ix>;

    fn state_by_valuation<
        Val: GetValuationClassIndex<ValuationClassIndex<Self::Ix>>
            + GetValuationData<ValuationIndex<Self::Ix>>,
    >(
        &self,
        valuation: &Val,
    ) -> Option<StateIndex<Self::Ix>>;
    fn add_valuation<
        Val: GetValuationClassIndex<ValuationClassIndex<Self::Ix>>
            + GetValuationData<ValuationIndex<Self::Ix>>,
    >(
        &mut self,
        valuation: Val,
    ) -> StateIndex<Self::Ix>;
    fn add_state(&mut self, state_index: StateIndex<Self::Ix>);
    fn state_valuations(&self) -> &Valuations<Self::Ix> {
        self.valuation_builder().state_valuations()
    }
    fn state_valuations_mut(&mut self) -> &mut Valuations<Self::Ix> {
        self.valuation_builder_mut().state_valuations_mut()
    }
    fn valuation_builder(&self) -> &ValuationBuilder<Self::Ix>;
    fn valuation_builder_mut(&mut self) -> &mut ValuationBuilder<Self::Ix>;
    fn owners_mut(&mut self) -> &mut Self::Owners;
    fn create_valuation(
        &self,
        class_index: ValuationClassIndex<Self::Ix>,
    ) -> ModelStandaloneValuation<'_, Self::Ix> {
        StandaloneValuation::new(class_index, self.state_valuations().class(class_index))
    }

    fn start_choice(&mut self) -> ChoiceIndex<Self::Ix>; // TODO: Do we need this or is finish_choice sufficient?
    fn add_branch(
        &mut self,
        rate_or_probability: f64,
        target: StateIndex<Self::Ix>,
    ) -> BranchIndex<Self::Ix>;

    // TODO: Instead of requiring an explicit call to this, start_choice
    //  could return a struct through which the branches are added. This struct, on being dropped,
    //  could then perform choice-finishing operations. The same could be done for add_state. This
    //  also more closely models that actual structure and ensures choices and branches are added
    //  correctly.
    fn finish_choice(&mut self);

    fn into_base_and_valuations(self) -> (Self::BaseModel, Self::Valuation);
}
