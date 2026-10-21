use crate::ModelBuilder;
use crate::bases::{BaseModelBuilder, ValuationBuilder};
use crate::choice_labels::ActionNameChoiceLabels;
use crate::owners::{NoOwners, OwnersBuilder, TurnBasedOwnersBuilder};
use prism_model::{Expression, Identifier, Span, VariableReference};
use probabilistic_models::base_model::{BaseModel, ChoiceBranchGraph, Mdp};
use probabilistic_models::traits::{BuildChoiceBranchGraph, BuildStates};
use probabilistic_models::valuations::{GetValuationClassIndex, GetValuationData, Valuations};
use probabilistic_models::{
    BranchIndex, ChoiceIndex, HasIx, Ix, StateIndex, Uniform, ValuationClassIndex, ValuationIndex,
};
use typed_index_collections::RawIndex;

#[derive(Default)]
pub struct MdpBuilder<I: Ix, Own: OwnersBuilder<Ix = I> = NoOwners<I>> {
    mdp: Mdp<I>,
    valuation: ValuationBuilder<I>,
    next_state_index: StateIndex<I>,
    owners: Own,
}

impl<I: Ix, Own: OwnersBuilder<Ix = I>> HasIx for MdpBuilder<I, Own> {
    type Ix = I;
}

impl<I: Ix, Own: OwnersBuilder<Ix = I>> MdpBuilder<I, Own> {
    pub(crate) fn with_owners<Own2: OwnersBuilder<Ix = I>>(
        self,
        owners: Own2,
    ) -> MdpBuilder<I, Own2> {
        MdpBuilder {
            mdp: self.mdp,
            valuation: self.valuation,
            next_state_index: self.next_state_index,
            owners,
        }
    }
}

impl<I: Ix, Own: OwnersBuilder<Ix = I>> BaseModelBuilder for MdpBuilder<I, Own> {
    type BaseModel = BaseModel<ChoiceBranchGraph<I>, Own::Owners>;
    type Valuation = Valuations<I>;
    type Owners = Own;

    fn state_by_valuation<
        Val: GetValuationClassIndex<ValuationClassIndex<I>> + GetValuationData<ValuationIndex<I>>,
    >(
        &self,
        valuation: &Val,
    ) -> Option<StateIndex<I>> {
        self.valuation.state_by_valuation(valuation)
    }

    fn add_valuation<
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
    fn add_state(&mut self, state_index: StateIndex<I>) {
        let added_index = self.mdp.add_state();
        assert_eq!(added_index, state_index);
    }

    fn valuation_builder(&self) -> &ValuationBuilder<I> {
        &self.valuation
    }

    fn valuation_builder_mut(&mut self) -> &mut ValuationBuilder<I> {
        &mut self.valuation
    }

    fn owners_mut(&mut self) -> &mut Own {
        &mut self.owners
    }

    fn start_choice(&mut self) -> ChoiceIndex<I> {
        self.mdp.add_choice()
    }

    fn add_branch(&mut self, probability: f64, target: StateIndex<I>) -> BranchIndex<I> {
        self.mdp.add_branch(probability, target)
    }

    fn finish_choice(&mut self) {
        // TODO: Verify probabilities add up to one!
    }

    fn into_base_and_valuations(self) -> (Self::BaseModel, Valuations<I>) {
        let base = BaseModel {
            graph: self.mdp.graph,
            players: self.owners.into_owners(),
        };
        (base, self.valuation.into_state_valuations())
    }
}

impl<'a, S: Span>
    ModelBuilder<
        'a,
        S,
        crate::queries::ModelOnly<S>,
        crate::labels::OnlyNecessary,
        crate::initial_states_source::StartFromInitialStates,
        MdpBuilder<Uniform<usize>>,
        crate::initial_states_builder::SingleInitialStatesBuilder<Uniform<usize>>,
        crate::atomic_propositions_builder::AtomicPropositionVectorsBuilder<Uniform<usize>>,
        ActionNameChoiceLabels<Uniform<usize>>,
        crate::rewards_builder::RewardVectorsBuilder<Uniform<usize>>,
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
            rewards: Default::default(),
            state_space_restriction: Default::default(),
        }
    }
}

impl<
    'a,
    S: Span,
    Q: crate::queries::QueryCollection,
    L: crate::labels::LabelSource,
    IS: crate::initial_states_source::InitialStateSource,
    I: Ix,
    IB: crate::initial_states_builder::InitialStatesBuilder<Ix = I>,
    APs: crate::atomic_propositions_builder::AtomicPropositionBuilder<Ix = I>,
    CL: crate::choice_labels::ChoiceLabelBuilder<Ix = I>,
    Rew: crate::rewards_builder::RewardsBuilder<Ix = I>,
> ModelBuilder<'a, S, Q, L, IS, MdpBuilder<I, NoOwners<I>>, IB, APs, CL, Rew>
{
    /// Tracks state ownership according to the players of the model, producing a turn-based game.
    pub fn with_owners(
        self,
    ) -> ModelBuilder<'a, S, Q, L, IS, MdpBuilder<I, TurnBasedOwnersBuilder<I>>, IB, APs, CL, Rew>
    {
        self.map_base(|base| base.with_owners(TurnBasedOwnersBuilder::new()))
    }
}

impl<
    'a,
    S: Span,
    Q: crate::queries::QueryCollection,
    L: crate::labels::LabelSource,
    IS: crate::initial_states_source::InitialStateSource,
    I: Ix,
    IB: crate::initial_states_builder::InitialStatesBuilder<Ix = I>,
    APs: crate::atomic_propositions_builder::AtomicPropositionBuilder<Ix = I>,
    CL: crate::choice_labels::ChoiceLabelBuilder<Ix = I>,
    Rew: crate::rewards_builder::RewardsBuilder<Ix = I>,
> ModelBuilder<'a, S, Q, L, IS, MdpBuilder<I, TurnBasedOwnersBuilder<I>>, IB, APs, CL, Rew>
{
    pub fn without_owners(
        self,
    ) -> ModelBuilder<'a, S, Q, L, IS, MdpBuilder<I, NoOwners<I>>, IB, APs, CL, Rew> {
        self.map_base(|base| base.with_owners(NoOwners::new()))
    }
}
