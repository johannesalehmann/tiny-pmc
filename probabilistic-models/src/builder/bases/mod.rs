mod valuation_to_state;
use valuation_to_state::ValuationToEntity;

use super::BaseModelBuilder;
use crate::to1::To1;
use crate::valuations::{StandaloneValuation, Valuations};
use crate::{BranchIndex, ChoiceIndex, Mdp, RawIndex, StateIndex, ValuationClassIndex};
use std::collections::HashMap;
use std::ops::Index;

pub struct ValuationBuilder<I: RawIndex> {
    state_valuations: Valuations<I, StateIndex<I>>,
    valuation_to_state: To1<ValuationClassIndex<I>, ValuationToEntity<StateIndex<I>>>,
}

impl<I: RawIndex> ValuationBuilder<I> {
    fn state_by_valuation(&self, valuation: &StandaloneValuation<I>) -> Option<StateIndex<I>> {
        self.valuation_to_state[valuation.class_index].get(valuation)
    }

    fn add_state_valuation(
        &mut self,
        valuation: &StandaloneValuation<I>,
        state_index: StateIndex<I>,
    ) {
        self.valuation_to_state[valuation.class_index].add(&valuation, state_index);
    }
}

pub struct MdpBuilder<I: RawIndex> {
    mdp: Mdp<I>,
    next_state: StateIndex<I>,
    next_choice: ChoiceIndex<I>,
    next_branch: BranchIndex<I>,
    valuation: ValuationBuilder<I>,
}

impl<I: RawIndex> BaseModelBuilder for MdpBuilder<I> {
    type BaseModel = Mdp;
    type Index = I;

    fn state_by_valuation(
        &self,
        valuation: &StandaloneValuation<Self::Index>,
    ) -> Option<StateIndex<Self::Index>> {
        self.valuation.state_by_valuation(valuation)
    }

    fn add_state(
        &mut self,
        valuation: StandaloneValuation<Self::Index>,
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
        &self.valuation.state_valuations
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

        self.mdp.branch_targets.add(target);
        self.mdp.branch_probabilities.add(probability);
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
}
