use crate::builder::bases::{BaseModelBuilder, ValuationBuilder};
use crate::valuations::{
    BareStandaloneValuation, GetValuationClassIndex, GetValuationData, StandaloneValuation,
    Valuations,
};
use crate::{BranchIndex, ChoiceIndex, Mdp, RawIndex, StateIndex};
use num_traits::One;
use typed_index_collections::Index;

pub struct MdpBuilder<
    StateIdx: Index,
    ChoiceIdx: Index,
    BranchIdx: Index,
    ClassIdx: Index,
    ClassEntryIdx: Index,
    ValuationIdx: Index,
> {
    mdp: Mdp<StateIdx, ChoiceIdx, BranchIdx>,
    next_state: StateIdx,
    next_choice: ChoiceIdx,
    next_branch: BranchIdx,
    valuation: ValuationBuilder<StateIdx, ClassIdx, ClassEntryIdx, ValuationIdx>,
}

impl<
    StateIdx: Index,
    ChoiceIdx: Index,
    BranchIdx: Index,
    ClassIdx: Index,
    ClassEntryIdx: Index,
    ValuationIdx: Index,
> BaseModelBuilder
    for MdpBuilder<StateIdx, ChoiceIdx, BranchIdx, ClassIdx, ClassEntryIdx, ValuationIdx>
{
    type BaseModel = Mdp<StateIdx, ChoiceIdx, BranchIdx>;
    type Valuation = Valuations<StateIdx, ClassIdx, ClassEntryIdx, ValuationIdx>;

    type StateIdx = StateIdx;
    type ChoiceIdx = ChoiceIdx;
    type BranchIdx = BranchIdx;
    type ClassIdx = ClassIdx;
    type ClassEntryIdx = ClassEntryIdx;
    type ValuationIdx = ValuationIdx;

    fn state_by_valuation<
        Val: GetValuationClassIndex<ClassIdx> + GetValuationData<ValuationIdx>,
    >(
        &self,
        valuation: &Val,
    ) -> Option<StateIdx> {
        self.valuation.state_by_valuation(valuation)
    }

    fn add_state<Val: GetValuationClassIndex<ClassIdx> + GetValuationData<ValuationIdx>>(
        &mut self,
        valuation: Val,
    ) -> StateIdx {
        let index = self.next_state;

        self.mdp
            .state_to_choice
            .add_entry(self.next_state, self.next_choice, self.next_choice);
        self.valuation
            .add_state_valuation(&valuation, self.next_state);
        self.next_state += StateIdx::RawType::one();
        index
    }

    fn state_valuations(&self) -> &Valuations<StateIdx, ClassIdx, ClassEntryIdx, ValuationIdx> {
        self.valuation.state_valuations()
    }

    fn state_valuations_mut(
        &mut self,
    ) -> &mut Valuations<StateIdx, ClassIdx, ClassEntryIdx, ValuationIdx> {
        self.valuation.state_valuations_mut()
    }

    fn add_choice(&mut self) -> ChoiceIdx {
        let index = self.next_choice;
        self.next_choice += ChoiceIdx::RawType::one();

        self.mdp
            .choice_to_branch
            .add_entry(self.next_choice, self.next_branch, self.next_branch);
        self.mdp.state_to_choice.extend_last_entry(self.next_choice);
        index
    }

    fn add_branch(&mut self, probability: f64, target: StateIdx) -> BranchIdx {
        let index = self.next_branch;
        self.next_branch += BranchIdx::RawType::one();

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
        Valuations<StateIdx, ClassIdx, ClassEntryIdx, ValuationIdx>,
    ) {
        (self.mdp, self.valuation.into_state_valuations())
    }
}
