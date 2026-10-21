use crate::bases::ValuationBuilder;
use probabilistic_models::base_model::{BaseModel, ChoiceBranchGraph, Mdp};
use probabilistic_models::traits::{BuildChoiceBranchGraph, BuildStates};
use probabilistic_models::valuations::{
    GetValuationClassIndex, GetValuationData, ModelStandaloneValuation, StandaloneValuation,
    Valuations,
};
use probabilistic_models::{
    BranchIndex, ChoiceIndex, HasIx, Ix, StateIndex, ValuationClassIndex, ValuationIndex,
};
use typed_index_collections::RawIndex;

#[derive(Default)]
pub(crate) struct MdpBuilder<I: Ix> {
    mdp: Mdp<I>,
    valuation: ValuationBuilder<I>,
    next_state_index: StateIndex<I>,
}

impl<I: Ix> HasIx for MdpBuilder<I> {
    type Ix = I;
}

impl<I: Ix> MdpBuilder<I> {
    pub fn state_by_valuation<
        Val: GetValuationClassIndex<ValuationClassIndex<I>> + GetValuationData<ValuationIndex<I>>,
    >(
        &self,
        valuation: &Val,
    ) -> Option<StateIndex<I>> {
        self.valuation.state_by_valuation(valuation)
    }

    pub fn add_valuation<
        Val: GetValuationClassIndex<ValuationClassIndex<I>> + GetValuationData<ValuationIndex<I>>,
    >(
        &mut self,
        valuation: Val,
    ) -> StateIndex<I> {
        let index = self.next_state_index;
        self.next_state_index += <I as Ix>::State::one();
        self.valuation.add_state_valuation(&valuation, index);
        index
    }

    // Before calling this, call add_valuation to get the next state index
    pub fn add_state(&mut self, state_index: StateIndex<I>) {
        let added_index = self.mdp.add_state();
        assert_eq!(added_index, state_index);
    }

    pub fn state_valuations(&self) -> &Valuations<I> {
        self.valuation.state_valuations()
    }

    pub fn valuation_builder_mut(&mut self) -> &mut ValuationBuilder<I> {
        &mut self.valuation
    }

    pub fn create_valuation(
        &self,
        class_index: ValuationClassIndex<I>,
    ) -> ModelStandaloneValuation<'_, I> {
        StandaloneValuation::new(class_index, self.state_valuations().class(class_index))
    }

    // TODO: Do we need this or is finish_choice sufficient?
    pub fn start_choice(&mut self) -> ChoiceIndex<I> {
        self.mdp.add_choice()
    }

    pub fn add_branch(&mut self, probability: f64, target: StateIndex<I>) -> BranchIndex<I> {
        self.mdp.add_branch(probability, target)
    }

    // TODO: Instead of requiring an explicit call to this, start_choice
    //  could return a struct through which the branches are added. This struct, on being dropped,
    //  could then perform choice-finishing operations. The same could be done for add_state. This
    //  also more closely models that actual structure and ensures choices and branches are added
    //  correctly.
    pub fn finish_choice(&mut self) {
        // TODO: Verify probabilities add up to one!
    }

    pub fn into_graph_and_valuations(self) -> (ChoiceBranchGraph<I>, Valuations<I>) {
        let BaseModel { graph, .. } = self.mdp;
        (graph, self.valuation.into_state_valuations())
    }
}
