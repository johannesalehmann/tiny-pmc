use crate::ModelBuilder;
use crate::bases::{BaseModelBuilder, ValuationBuilder};
use crate::choice_labels::ActionNameChoiceLabels;
use prism_model::{Expression, Identifier, Span, VariableReference};
use probabilistic_models::base_model::Mdp;
use probabilistic_models::valuations::{GetValuationClassIndex, GetValuationData, Valuations};
use probabilistic_models::{
    AnnotationEntryIndex, AtomicPropositionIndex, BranchIndex, ChoiceIndex, ChoiceLabelIndex,
    StateIndex, ValuationClassEntryIndex, ValuationClassIndex, ValuationIndex,
};
use typed_index_collections::{Index, RawIndex};

#[derive(Default)]
pub struct MdpBuilder<
    StateIdx: Index,
    ChoiceIdx: Index,
    BranchIdx: Index,
    ClassIdx: Index,
    ClassEntryIdx: Index,
    ValuationIdx: Index,
> {
    mdp: Mdp<StateIdx, ChoiceIdx, BranchIdx>,
    valuation: ValuationBuilder<StateIdx, ClassIdx, ClassEntryIdx, ValuationIdx>,
    next_state_index: StateIdx,
}

// TODO: This function exists to get existing unit tests working without specifying the full set of
//  types in every test. Consider moving it to probabilistic-models, where the tests reside, or
//  devising some other way of constructing an MdpBuilder.
impl
    MdpBuilder<
        StateIndex<usize>,
        ChoiceIndex<usize>,
        BranchIndex<usize>,
        ValuationClassIndex<usize>,
        ValuationClassEntryIndex<usize>,
        ValuationIndex<usize>,
    >
{
    pub fn with_default_index_types() -> Self {
        Self::default()
    }
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

    fn add_valuation<Val: GetValuationClassIndex<ClassIdx> + GetValuationData<ValuationIdx>>(
        &mut self,
        valuation: Val,
    ) -> StateIdx {
        let index = self.next_state_index;
        self.next_state_index += StateIdx::RawType::one();
        self.valuation.add_state_valuation(&valuation, index);
        index
    }

    // Before calling this, call add_valuation to get the next state index
    fn add_state(&mut self, state_index: StateIdx) {
        self.mdp.add_state(state_index);
    }

    fn valuation_builder(
        &self,
    ) -> &ValuationBuilder<StateIdx, ClassIdx, ClassEntryIdx, ValuationIdx> {
        &self.valuation
    }

    fn valuation_builder_mut(
        &mut self,
    ) -> &mut ValuationBuilder<StateIdx, ClassIdx, ClassEntryIdx, ValuationIdx> {
        &mut self.valuation
    }

    fn start_choice(&mut self) -> ChoiceIdx {
        self.mdp.add_choice()
    }

    fn add_branch(&mut self, probability: f64, target: StateIdx) -> BranchIdx {
        self.mdp.add_branch(probability, target)
    }

    fn finish_choice(&mut self) {
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

impl<'a, S: Span>
    ModelBuilder<
        'a,
        S,
        crate::queries::ModelOnly<S>,
        crate::labels::OnlyNecessary,
        crate::initial_states_source::StartFromInitialStates,
        MdpBuilder<
            StateIndex<u32>,
            ChoiceIndex<u32>,
            BranchIndex<u32>,
            ValuationClassIndex<u16>,
            ValuationClassEntryIndex<u16>,
            ValuationIndex<usize>,
        >,
        crate::initial_states_builder::SingleInitialStatesBuilder<StateIndex<u32>>,
        crate::atomic_propositions_builder::AtomicPropositionVectorsBuilder<
            AtomicPropositionIndex<usize>,
            StateIndex<u32>,
            AnnotationEntryIndex<usize>,
        >,
        ActionNameChoiceLabels<ChoiceIndex<u32>, ChoiceLabelIndex<usize>>,
    >
{
    pub fn new_mdp_builder(
        model: &'a mut prism_model::Model<
            VariableReference,
            S,
            Expression<VariableReference, S>,
            Identifier<S>,
        >,
    ) -> Self {
        Self {
            model,
            base: Default::default(),
            initial_state_source: Default::default(),
            initial_states_builder: Default::default(),
            atomic_propositions: Default::default(),
            queries: Default::default(),
            constants: Default::default(),
            labels: Default::default(),
            choice_labels: Default::default(),
        }
    }
}
