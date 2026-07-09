mod mdp;
pub use mdp::MdpBuilder;

mod valuation_builder;
use valuation_builder::ValuationBuilder;

use crate::valuations::{StandaloneValuation, Valuations};
use crate::{BranchIndex, ChoiceIndex, RawIndex, StateIndex};

pub trait BaseModelBuilder {
    type BaseModel;
    type Index: RawIndex;

    fn state_by_valuation(
        &self,
        valuation: &StandaloneValuation<Self::Index>,
    ) -> Option<StateIndex<Self::Index>>;
    fn add_state(&mut self, valuation: StandaloneValuation<Self::Index>)
    -> StateIndex<Self::Index>;
    fn state_valuations(&self) -> &Valuations<Self::Index, StateIndex<Self::Index>>;

    fn add_choice(&mut self) -> ChoiceIndex<Self::Index>;
    fn add_branch(
        &mut self,
        rate_or_probability: f64,
        target: StateIndex<Self::Index>,
    ) -> BranchIndex<Self::Index>;
    fn finish_choice(&mut self);
    fn finish_branch(&mut self);
}
