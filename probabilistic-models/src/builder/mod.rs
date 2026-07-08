mod bases;

use crate::index::RawIndex;
use crate::valuations::{StandaloneValuation, Valuations};
use crate::{BranchIndex, ChoiceIndex, Model, StateIndex};

pub struct ModelBuilderBuilder<
    Base: BaseModelBuilder,
    Ini: InitialStatesBuilder,
    APs: AtomicPropositionBuilder,
> {
    base: Base,
    initial_states: Ini,
    atomic_propositions: APs,
}

impl<Base: BaseModelBuilder, Ini: InitialStatesBuilder, APs: AtomicPropositionBuilder>
    ModelBuilderBuilder<Base, Ini, APs>
{
    pub fn new(base: Base) -> ModelBuilderBuilder<Base, (), ()> {
        Self {
            base,
            initial_states: (),
            atomic_propositions: (),
        }
    }

    // TODO: Functions to add and remove initial states

    pub fn finish(self) -> ModelBuilder<BaseModelBuilder> {
        ModelBuilder {
            base: self.base,
            initial_states: self.initial_states,
            atomic_propositions: self.atomic_propositions,
        }
    }
}

pub struct ModelBuilder<
    Base: BaseModelBuilder,
    Ini: InitialStatesBuilder,
    APs: AtomicPropositionBuilder,
> {
    base: Base,
    initial_states: Ini,
    atomic_propositions: APs,
}

impl<Base: BaseModelBuilder, Ini: InitialStatesBuilder, APs: AtomicPropositionBuilder>
    ModelBuilder<Base, Ini, APs>
{
    pub fn finish() -> Model<
        Base::Index,
        Base::BaseModel,
        Ini::InitialStates,
        (),
        (),
        (),
        APs::AtomicPropositions,
        (),
        (),
        Valuations<Base::Index, StateIndex<Base::Index>>,
    > {
        todo!()
    }
}

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
        target: StateIndex<I>,
    ) -> BranchIndex<Self::Index>;
    fn finish_choice(&mut self);
    fn finish_branch(&mut self);
}

pub trait InitialStatesBuilder {
    type InitialStates;
    type Index: RawIndex;

    fn stores_initial_states() -> bool;
    fn mark_state(&mut self, state: StateIndex<Self::Index>);
}

pub trait AtomicPropositionBuilder {
    type AtomicPropositions;
    type Index: RawIndex;

    fn stores_atomic_propositions() -> bool;
    fn set_value(&mut self, id: String, state: StateIndex<Self::Index>, value: bool);
}
