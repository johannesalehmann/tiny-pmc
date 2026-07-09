use crate::builder::bases::{BaseModelBuilder, ValuationBuilder};
use crate::valuations::{
    BareStandaloneValuation, GetValuationClassIndex, GetValuationData, StandaloneValuation,
    Valuations,
};
use crate::{BranchIndex, ChoiceIndex, Mdp, RawIndex, StateIndex};

pub struct MdpBuilder<I: RawIndex> {
    mdp: Mdp<I>,
    next_state: StateIndex<I>,
    next_choice: ChoiceIndex<I>,
    next_branch: BranchIndex<I>,
    valuation: ValuationBuilder<I>,
}

impl<I: RawIndex> BaseModelBuilder for MdpBuilder<I> {
    type BaseModel = Mdp<I>;
    type Index = I;

    fn state_by_valuation<V: GetValuationClassIndex<I> + GetValuationData<I>>(
        &self,
        valuation: &V,
    ) -> Option<StateIndex<Self::Index>> {
        self.valuation.state_by_valuation(valuation)
    }

    fn add_state<Val: GetValuationData<I> + GetValuationClassIndex<I>>(
        &mut self,
        valuation: Val,
    ) -> StateIndex<Self::Index> {
        let index = self.next_state;

        self.mdp
            .state_to_choice
            .add_entry(self.next_state, self.next_choice, self.next_choice);
        self.valuation
            .add_state_valuation(&valuation, self.next_state);
        self.next_state += Self::Index::one();
        index
    }

    fn state_valuations(&self) -> &Valuations<Self::Index, StateIndex<Self::Index>> {
        self.valuation.state_valuations()
    }

    fn state_valuations_mut(&mut self) -> &mut Valuations<Self::Index, StateIndex<Self::Index>> {
        self.valuation.state_valuations_mut()
    }

    fn add_choice(&mut self) -> ChoiceIndex<Self::Index> {
        let index = self.next_choice;
        self.next_choice += I::one();

        self.mdp
            .choice_to_branch
            .add_entry(self.next_choice, self.next_branch, self.next_branch);
        self.mdp.state_to_choice.extend_last_entry(self.next_choice);
        index
    }

    fn add_branch(&mut self, probability: f64, target: StateIndex<I>) -> BranchIndex<Self::Index> {
        let index = self.next_branch;
        self.next_branch += I::one();

        self.mdp.branch_targets.add_unchecked(target);
        self.mdp.branch_probabilities.add_unchecked(probability);
        self.mdp
            .choice_to_branch
            .extend_last_entry(self.next_branch);
        index
    }

    fn finish_choice(&mut self) {
        // Nothing to do, MDPs can have any number of choices
    }

    fn finish_branch(&mut self) {
        // TODO: Verify probabilities add up to one!
    }

    fn into_base_and_valuations(
        self,
    ) -> (
        Self::BaseModel,
        Valuations<Self::Index, StateIndex<Self::Index>>,
    ) {
        (self.mdp, self.valuation.into_state_valuations())
    }
}
